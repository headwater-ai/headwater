// SPDX-License-Identifier: Apache-2.0
//! The publish order in `.github/workflows/publish-crates.yml`, held against
//! the workspace it claims to enumerate.
//!
//! # What it costs to be wrong here, and where it shows
//!
//! `publish-crates.yml` walks a hand-written list of crate names and publishes
//! each one to crates.io in that order. The list was walked once, when
//! [#735](https://github.com/headwater-ai/headwater/issues/735) was filed, and
//! it is deliberately not recomputed on every run — a topological order that
//! moves underneath a release is worse than one a diff shows. What nothing did
//! was compare the list against `engine/Cargo.toml`'s `members`.
//!
//! A member missing from the list is green in every job this repository runs.
//! It fails at a release, on a tag, after the version bump has landed and the
//! tag is cut, at the point where the next crate in the order fails to resolve
//! a dependency that was never published. [#479](https://github.com/headwater-ai/headwater/issues/479)
//! added `headwater-paint` and had to remember this file by hand, which is the
//! shape [HW-OBL-0172](../../../../docs/obligations/0172-nine-hand-kept-constants-enumerate-an-enum-and-nothing-holds-one-against-the-variants.md)
//! and "enumerate from the declaration" both name.
//!
//! # Why a set and an ordering, rather than the whole order
//!
//! Two different claims live in that string. The first is which crates it
//! names, and the workspace decides that: a member is publishable unless its
//! own manifest says otherwise. The second is the order, and the dependency
//! graph decides that. This file holds the set exactly, and holds the order
//! only where a member's manifest declares a dependency on another member —
//! which is the part a wrong order actually breaks.

use std::path::{Path, PathBuf};

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

/// Every crate name the workflow's `order=` string lists, in its own order.
///
/// The string is a shell assignment continued over several lines with a
/// trailing backslash, so the parse takes everything between the opening quote
/// and the closing one and then splits on whitespace.
fn published_order() -> Vec<String> {
    let text =
        std::fs::read_to_string(repository_root().join(".github/workflows/publish-crates.yml"))
            .expect("the publish workflow is on disk");
    let start = text
        .find("order=\"")
        .expect("publish-crates.yml assigns the publish order to `order`");
    let rest = &text[start + "order=\"".len()..];
    let end = rest
        .find('"')
        .expect("the `order` assignment closes its quote");
    rest[..end]
        .split_whitespace()
        .filter(|word| *word != "\\")
        .map(str::to_string)
        .collect()
}

/// Every member of the engine workspace, as `(crate name, manifest text)`.
///
/// The name comes from each member's own `[package] name`, because a directory
/// name and a crate name are two different things and only one of them is what
/// `cargo publish` takes.
fn workspace_members() -> Vec<(String, String)> {
    let root = repository_root().join("engine");
    let text = std::fs::read_to_string(root.join("Cargo.toml")).expect("the workspace manifest");
    let start = text
        .find("members = [")
        .expect("the workspace declares members");
    let rest = &text[start..];
    let end = rest.find(']').expect("the members list closes");
    rest[..end]
        .lines()
        .filter_map(|line| line.trim().strip_prefix('"'))
        .filter_map(|line| line.split('"').next())
        .map(|relative| {
            let manifest = root.join(relative).join("Cargo.toml");
            let member = std::fs::read_to_string(&manifest)
                .unwrap_or_else(|why| panic!("{}: {why}", manifest.display()));
            let name = member
                .lines()
                .find_map(|line| line.trim().strip_prefix("name = \""))
                .and_then(|rest| rest.split('"').next())
                .unwrap_or_else(|| panic!("{} names no package", manifest.display()))
                .to_string();
            (name, member)
        })
        .collect()
}

/// The workflow names every member of the workspace, and no name that is not
/// one.
///
/// Both directions, because they fail differently: a member the list omits
/// breaks the release at the first crate that depends on it, and a name the
/// workspace no longer has makes `cargo publish` refuse a package it cannot
/// find, which reads like a registry fault.
#[test]
fn the_publish_order_names_exactly_the_members_of_the_workspace() {
    let order = published_order();
    let members: Vec<String> = workspace_members()
        .into_iter()
        .map(|(name, _)| name)
        .collect();

    let missing: Vec<&String> = members
        .iter()
        .filter(|name| !order.contains(name))
        .collect();
    let extra: Vec<&String> = order
        .iter()
        .filter(|name| !members.contains(name))
        .collect();

    assert!(
        missing.is_empty() && extra.is_empty(),
        "publish-crates.yml and engine/Cargo.toml disagree about the workspace.\n\
         members the publish order omits, which fail at a release and nowhere else: {missing:?}\n\
         names the publish order carries and the workspace does not: {extra:?}"
    );
}

/// No crate is published before a member it depends on.
///
/// This is the claim the order itself makes, held against each member's own
/// `[dependencies]` rather than against a second hand-written graph. A
/// dependency on a crate outside this workspace is not read: crates.io already
/// has it.
#[test]
fn no_crate_is_published_before_a_member_it_depends_on() {
    let order = published_order();
    let position = |name: &str| order.iter().position(|one| one == name);

    let mut wrong = Vec::new();
    for (name, manifest) in workspace_members() {
        let Some(mine) = position(&name) else {
            continue; // the case above is what reports this
        };
        for line in manifest.lines() {
            let line = line.trim();
            let Some(dependency) = line.strip_prefix("headwater-") else {
                continue;
            };
            let Some(dependency) = dependency.split([' ', '.', '=']).next() else {
                continue;
            };
            let dependency = format!("headwater-{dependency}");
            if dependency == name {
                continue;
            }
            if let Some(theirs) = position(&dependency) {
                if theirs > mine {
                    wrong.push(format!(
                        "{name} is published at {mine} and depends on {dependency} at {theirs}"
                    ));
                }
            }
        }
    }

    assert!(
        wrong.is_empty(),
        "the publish order puts a crate before something it depends on:\n{}",
        wrong.join("\n")
    );
}

/// The body of the loop that publishes each crate, from `for crate in $order`
/// to its `done`.
fn publish_loop() -> String {
    let text =
        std::fs::read_to_string(repository_root().join(".github/workflows/publish-crates.yml"))
            .expect("the publish workflow is on disk");
    let start = text
        .find("for crate in $order; do")
        .expect("publish-crates.yml walks the order with `for crate in $order`");
    let rest = &text[start..];
    let end = rest
        .find("\n          done")
        .expect("the publish loop closes with `done`");
    rest[..end].to_string()
}

/// Runs the workflow's own publish loop under `sh -c`, over the crates `alpha`
/// and `beta` at version 9.9.9, with every command it calls replaced by a
/// stub that appends one line to standard output.
///
/// `curl` answers 404, so no crate is skipped. `publish_with_retry` prints
/// `publish <crate>`. `sh`, which is how the loop runs the gate, prints
/// `gate <arguments>`, and exits 1 when its arguments match `refuse`. The loop
/// runs under `set -e`, as the workflow step does.
fn run_publish_loop(refuse: &str) -> (bool, Vec<String>) {
    let script = format!(
        "set -e\n\
         order=\"alpha beta\"\n\
         version=9.9.9\n\
         curl() {{ echo 404; }}\n\
         publish_with_retry() {{ echo \"publish $1\"; }}\n\
         sh() {{\n\
             echo \"gate $*\"\n\
             case \"$*\" in {refuse}) return 1 ;; esac\n\
         }}\n\
         {}\n\
         done\n",
        publish_loop()
    );
    let out = std::process::Command::new("sh")
        .arg("-c")
        .arg(&script)
        .output()
        .expect("sh runs the publish loop");
    let lines = String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|line| line.starts_with("gate ") || line.starts_with("publish "))
        .map(str::to_string)
        .collect();
    (out.status.success(), lines)
}

/// The loop runs the gate on a crate's dependencies before it publishes the
/// crate, and on the crate itself after, for every crate.
///
/// The loop is executed, not read, so a gate call that is commented out, or
/// that stands after the publish, fails here (verify of #1197 at c065b76d).
#[test]
fn the_publish_loop_runs_the_index_gate_before_and_after_each_publish() {
    let (ok, lines) = run_publish_loop("nothing-matches");
    let gate = "gate ../tools/repo/crates-index-wait.sh";
    let expected = vec![
        format!("{gate} alpha 9.9.9"),
        "publish alpha".to_string(),
        format!("{gate} --published alpha 9.9.9"),
        format!("{gate} beta 9.9.9"),
        "publish beta".to_string(),
        format!("{gate} --published beta 9.9.9"),
    ];
    assert!(
        ok,
        "the publish loop failed with every gate passing: {lines:?}"
    );
    assert_eq!(
        lines, expected,
        "the publish loop does not run the index gate around each publish"
    );
}

/// A gate that refuses stops the loop before the crate publishes.
#[test]
fn the_publish_loop_stops_before_a_crate_the_gate_refuses() {
    let (ok, lines) = run_publish_loop("*crates-index-wait.sh\\ beta\\ *");
    assert!(
        !ok,
        "the publish loop succeeded though the gate refused beta: {lines:?}"
    );
    assert!(
        !lines.iter().any(|line| line == "publish beta"),
        "the publish loop published beta after the gate refused it: {lines:?}"
    );
}

/// The loop waits on the index, and not on a clock.
///
/// A fixed sleep is what let v0.3.0 and v0.4.0 publish a crate before
/// crates.io listed its dependency (#1197).
#[test]
fn the_publish_loop_has_no_fixed_sleep() {
    let body = publish_loop();
    let bare_sleep: Vec<&str> = body
        .lines()
        .map(str::trim)
        .filter(|line| {
            line.strip_prefix("sleep ")
                .is_some_and(|rest| rest.chars().all(|c| c.is_ascii_digit()))
        })
        .collect();
    assert!(
        bare_sleep.is_empty(),
        "the publish loop waits a fixed time: {bare_sleep:?}"
    );
}

/// A sparse index on disk that lists every workspace member at `version`,
/// except the members `behind` names, which it lists at 0.0.1 only.
fn sparse_index(case: &str, version: &str, behind: &[&str]) -> PathBuf {
    sparse_index_with(case, version, behind, "")
}

/// As [`sparse_index`], and each member `behind` names also gets the line
/// `behind_line`, where `{n}` stands for the name and `{v}` for `version`.
fn sparse_index_with(case: &str, version: &str, behind: &[&str], behind_line: &str) -> PathBuf {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "hw-index-{case}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = std::fs::remove_dir_all(&dir);
    for (name, _) in workspace_members() {
        let path = dir.join(&name[0..2]).join(&name[2..4]).join(&name);
        std::fs::create_dir_all(path.parent().expect("a parent")).expect("index directory");
        let mut lines = format!("{{\"name\":\"{name}\",\"vers\":\"0.0.1\",\"deps\":[]}}\n");
        if !behind.contains(&name.as_str()) {
            lines.push_str(&format!(
                "{{\"name\":\"{name}\",\"vers\":\"{version}\",\"deps\":[]}}\n"
            ));
        } else if !behind_line.is_empty() {
            lines.push_str(&behind_line.replace("{n}", &name).replace("{v}", version));
            lines.push('\n');
        }
        std::fs::write(&path, lines).expect("index file");
    }
    dir
}

/// Runs the gate against `index` with a one-second deadline.
fn run_gate(index: &Path, args: &[&str]) -> std::process::Output {
    std::process::Command::new("sh")
        .arg(repository_root().join("tools/repo/crates-index-wait.sh"))
        .args(args)
        .env("HW_CRATES_INDEX", format!("file://{}", index.display()))
        .env("HW_CRATES_INDEX_DEADLINE", "1")
        .output()
        .expect("sh runs the gate")
}

/// The gate refuses a crate whose dependency the index does not list at the
/// release version, and names the dependency.
///
/// This is the v0.4.0 failure: `headwater-import` stopped because the index
/// did not list `headwater-scaffold` 0.4.0.
#[test]
fn the_index_gate_refuses_a_crate_whose_dependency_the_index_does_not_list() {
    let version = headwater_resolve::release::ENGINE;
    let index = sparse_index("refuses", version, &["headwater-scaffold"]);
    let out = run_gate(&index, &["headwater-import", version]);
    let said = String::from_utf8_lossy(&out.stdout).into_owned();
    let _ = std::fs::remove_dir_all(&index);
    assert!(
        !out.status.success(),
        "the gate passed headwater-import while the index did not list headwater-scaffold {version}"
    );
    assert!(
        said.contains("::error::headwater-import:") && said.contains("headwater-scaffold"),
        "the gate's refusal does not name the dependency it waited for: {said}"
    );
}

/// The gate passes a crate once the index lists every dependency, and passes a
/// published crate once the index lists the crate itself.
#[test]
fn the_index_gate_passes_a_crate_whose_dependencies_the_index_lists() {
    let version = headwater_resolve::release::ENGINE;
    let index = sparse_index("passes", version, &[]);
    let deps = run_gate(&index, &["headwater-import", version]);
    let published = run_gate(&index, &["--published", "headwater-import", version]);
    let _ = std::fs::remove_dir_all(&index);
    assert!(
        deps.status.success(),
        "the gate refused headwater-import with every dependency listed: {}{}",
        String::from_utf8_lossy(&deps.stdout),
        String::from_utf8_lossy(&deps.stderr)
    );
    assert!(
        published.status.success(),
        "the gate refused a published headwater-import that the index lists: {}{}",
        String::from_utf8_lossy(&published.stdout),
        String::from_utf8_lossy(&published.stderr)
    );
}

/// `--published` refuses while the index does not list the crate itself, so a
/// cargo availability timeout cannot count as success for the next crate.
#[test]
fn the_index_gate_refuses_a_published_crate_the_index_does_not_list() {
    let version = headwater_resolve::release::ENGINE;
    let index = sparse_index("unlisted", version, &["headwater-import"]);
    let out = run_gate(&index, &["--published", "headwater-import", version]);
    let said = String::from_utf8_lossy(&out.stdout).into_owned();
    let _ = std::fs::remove_dir_all(&index);
    assert!(
        !out.status.success() && said.contains("::error::headwater-import:"),
        "the gate passed a published crate the index does not list: {said}"
    );
}

/// The gate matches the whole version, so a pre-release whose text begins
/// with the release version is not the release.
#[test]
fn the_index_gate_refuses_a_dependency_listed_only_at_a_pre_release_of_the_version() {
    let version = headwater_resolve::release::ENGINE;
    let index = sparse_index_with(
        "prerelease",
        version,
        &["headwater-scaffold"],
        "{\"name\":\"{n}\",\"vers\":\"{v}-rc1\",\"deps\":[]}",
    );
    let out = run_gate(&index, &["headwater-import", version]);
    let said = String::from_utf8_lossy(&out.stdout).into_owned();
    let _ = std::fs::remove_dir_all(&index);
    assert!(
        !out.status.success() && said.contains("headwater-scaffold"),
        "the gate read headwater-scaffold {version}-rc1 as {version}: {said}"
    );
}

/// A dependency whose release version is yanked cannot be resolved by a new
/// publish, so the gate does not count it as listed.
#[test]
fn the_index_gate_refuses_a_dependency_whose_release_version_is_yanked() {
    let version = headwater_resolve::release::ENGINE;
    let index = sparse_index_with(
        "yanked",
        version,
        &["headwater-scaffold"],
        "{\"name\":\"{n}\",\"vers\":\"{v}\",\"deps\":[],\"yanked\":true}",
    );
    let out = run_gate(&index, &["headwater-import", version]);
    let said = String::from_utf8_lossy(&out.stdout).into_owned();
    let _ = std::fs::remove_dir_all(&index);
    assert!(
        !out.status.success() && said.contains("headwater-scaffold"),
        "the gate passed headwater-scaffold {version} though the index marks it yanked: {said}"
    );
}
