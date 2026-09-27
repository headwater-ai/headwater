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

/// The loop waits on the index, and not on a clock.
///
/// A fixed sleep is what let v0.3.0 and v0.4.0 publish a crate before
/// crates.io listed its dependency (#1197). This case reads the shape only; the
/// behavioral cases below hold what the gate does.
#[test]
fn the_publish_loop_gates_each_crate_on_the_index_and_not_on_a_fixed_sleep() {
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
    let gate = body
        .find("crates-index-wait.sh \"$crate\"")
        .expect("the publish loop calls tools/repo/crates-index-wait.sh on each crate");
    let publish = body
        .find("publish_with_retry \"$crate\"")
        .expect("the publish loop publishes with publish_with_retry");
    assert!(
        gate < publish,
        "the publish loop publishes a crate before it waits for the crate's dependencies"
    );
    let published = body
        .find("crates-index-wait.sh --published \"$crate\"")
        .expect("the publish loop waits for the index to list the crate it just published");
    assert!(
        publish < published,
        "the publish loop waits for the crate itself before it publishes it"
    );
}

/// A sparse index on disk that lists every workspace member at `version`,
/// except the members `behind` names, which it lists at an older version only.
fn sparse_index(case: &str, version: &str, behind: &[&str]) -> PathBuf {
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
    let version = env!("CARGO_PKG_VERSION");
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
    let version = env!("CARGO_PKG_VERSION");
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
    let version = env!("CARGO_PKG_VERSION");
    let index = sparse_index("unlisted", version, &["headwater-import"]);
    let out = run_gate(&index, &["--published", "headwater-import", version]);
    let said = String::from_utf8_lossy(&out.stdout).into_owned();
    let _ = std::fs::remove_dir_all(&index);
    assert!(
        !out.status.success() && said.contains("::error::headwater-import:"),
        "the gate passed a published crate the index does not list: {said}"
    );
}
