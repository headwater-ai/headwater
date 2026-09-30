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
//!
//! # The version each internal dependency names
//!
//! The last cases hold a second hand-kept list in the same manifest: the
//! `version` of every `[workspace.dependencies]` entry on a member, against
//! `[workspace.package] version`. A release bump that misses one entry is green
//! in every job until the publish ([#1315](https://github.com/headwater-ai/headwater/issues/1315)).
//! The members come from the same `members = [...]` parse as above, never from
//! a list written here, which is what HW-OBL-0172 asks for.

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
    let text = workspace_manifest();
    member_paths(&text)
        .into_iter()
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

/// The text of `engine/Cargo.toml`, the workspace manifest.
fn workspace_manifest() -> String {
    std::fs::read_to_string(repository_root().join("engine/Cargo.toml"))
        .expect("the workspace manifest")
}

/// The relative path of every member that a workspace manifest's
/// `members = [...]` declares, in its own order.
///
/// It takes the manifest text rather than reading the file, so that a planted
/// manifest goes through the same parse as the real one. The parse is TOML's
/// own, so a form cargo accepts is a form this reads.
fn member_paths(manifest: &str) -> Vec<String> {
    workspace_table(manifest)
        .get("members")
        .and_then(toml_edit::Item::as_array)
        .expect("the workspace declares members as an array")
        .iter()
        .map(|member| {
            member
                .as_str()
                .expect("each workspace member is a string")
                .to_string()
        })
        .collect()
}

/// The `[workspace]` table of a manifest.
fn workspace_table(manifest: &str) -> toml_edit::Item {
    let document: toml_edit::DocumentMut = manifest
        .parse()
        .unwrap_or_else(|why| panic!("the workspace manifest is not TOML: {why}"));
    document
        .get("workspace")
        .cloned()
        .expect("the manifest has a [workspace] table")
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

// The version every internal dependency names.
//
// `[workspace.dependencies]` in `engine/Cargo.toml` carries one entry for each
// member, with a path and a version, and the version is a literal: nothing in
// cargo derives it from `[workspace.package] version`. A release bump that moves
// the package version and misses one entry builds and tests green, because a
// path dependency resolves by path. It fails only at the publish, on a tag, when
// the crate that names the stale entry asks crates.io for a version that was
// published a release ago (#1315). This is the same shape as the publish order
// above, a hand-kept list that nothing held against the declaration, and it is
// recorded against HW-OBL-0172 for the same reason.

/// Every `[workspace.dependencies]` entry on a workspace member whose `version`
/// differs from `[workspace.package] version` or is missing, as one line per
/// entry that names it.
///
/// It takes the manifest text, so that the real manifest and a planted one go
/// through the same judge. An entry whose `path` names no member is a
/// dependency from outside this workspace, and it is not read.
fn stale_member_dependency_versions(manifest: &str) -> Vec<String> {
    member_dependency_versions(manifest)
        .into_iter()
        .filter_map(|MemberEntry { name, path, version, package }| match version {
            Some(version) if version == package => None,
            Some(version) => Some(format!(
                "{name} (path {path}) names version {version}, and [workspace.package] version is {package}"
            )),
            None => Some(format!(
                "{name} (path {path}) names no version, so it cannot be published; [workspace.package] version is {package}"
            )),
        })
        .collect()
}

/// One `[workspace.dependencies]` entry on a member, with the
/// `[workspace.package] version` it is held against.
struct MemberEntry {
    name: String,
    path: String,
    version: Option<String>,
    package: String,
}

/// Every `[workspace.dependencies]` entry whose `path` names a member,
/// whatever TOML form the entry is
/// written in: an inline table with or without spaces, or a
/// `[workspace.dependencies.<name>]` sub-table. A version that is not a string
/// reads as `None`, the same as no version.
fn member_dependency_versions(manifest: &str) -> Vec<MemberEntry> {
    let members = member_paths(manifest);
    let workspace = workspace_table(manifest);
    let package = workspace
        .get("package")
        .and_then(|table| table.get("version"))
        .and_then(toml_edit::Item::as_str)
        .expect("[workspace.package] declares a version")
        .to_string();
    let Some(dependencies) = workspace
        .get("dependencies")
        .and_then(toml_edit::Item::as_table_like)
    else {
        return Vec::new();
    };
    dependencies
        .iter()
        .filter_map(|(name, entry)| {
            let entry = entry.as_table_like()?;
            let path = entry.get("path").and_then(toml_edit::Item::as_str)?;
            if !members.iter().any(|member| member == path) {
                return None;
            }
            let version = entry
                .get("version")
                .and_then(toml_edit::Item::as_str)
                .map(str::to_string);
            Some(MemberEntry {
                name: name.to_string(),
                path: path.to_string(),
                version,
                package: package.clone(),
            })
        })
        .collect()
}

/// The real manifest names the workspace version on every internal
/// dependency.
#[test]
fn every_workspace_dependency_on_a_member_names_the_workspace_version() {
    let stale = stale_member_dependency_versions(&workspace_manifest());
    assert!(
        stale.is_empty(),
        "engine/Cargo.toml: a [workspace.dependencies] entry on a member does not name \
         [workspace.package] version. A bump moves both; a stale entry builds green and \
         fails at the crates.io publish.\n{}",
        stale.join("\n")
    );
}

/// A bump that moves `[workspace.package] version` and misses one entry is
/// caught, and the finding names the entry it missed.
#[test]
fn a_bump_that_misses_one_workspace_dependency_names_that_dependency() {
    let version = headwater_resolve::release::ENGINE;
    // Move the package version and every entry but `headwater-yaml` to a new
    // version, which is the bump that missed one line.
    let next = "99.0.1";
    let planted = planted_bump(next, &["headwater-yaml"]);
    let stale = stale_member_dependency_versions(&planted);
    assert!(
        stale.len() == 1 && stale[0].contains("headwater-yaml"),
        "the planted bump left headwater-yaml at {version} and moved the rest to {next}; \
         the judge should name headwater-yaml alone, and it said: {stale:?}"
    );
}

/// One way to write the `headwater-yaml` entry, given the version it names.
type Rewrite = fn(&str) -> String;

/// The version one patch release after `version`: `0.5.0` gives `0.5.1`.
fn next_patch(version: &str) -> String {
    let (head, patch) = version
        .rsplit_once('.')
        .expect("the engine version has a patch component");
    let patch: u64 = patch.parse().expect("the patch component is a number");
    format!("{head}.{}", patch + 1)
}

/// The real manifest with `[workspace.package] version` moved to `next`, and
/// the version of every `[workspace.dependencies]` entry on a member moved to
/// `next` as well, except the entries named in `missed`.
///
/// It finds each version through the parsed document and replaces the bytes
/// of that value alone. A replacement of text such as `version = "<v>" }`
/// matches one spelling of an entry, so an ordinary edit to another entry,
/// such as `default-features = false` after its version or its keys in another
/// order, left that entry out of the bump. Each case that plants on it then
/// named that entry beside the one it missed, and blamed the judge for the
/// plant.
fn planted_bump(next: &str, missed: &[&str]) -> String {
    let real = workspace_manifest();
    let members = member_paths(&real);
    let document = toml_edit::Document::parse(real.as_str()).expect("engine/Cargo.toml is TOML");
    let workspace = document
        .get("workspace")
        .and_then(toml_edit::Item::as_table_like)
        .expect("engine/Cargo.toml has a [workspace] table");
    let mut spans: Vec<std::ops::Range<usize>> = Vec::new();
    spans.push(
        workspace
            .get("package")
            .and_then(|package| package.get("version"))
            .and_then(toml_edit::Item::span)
            .expect("[workspace.package] declares a version, and the parse keeps its span"),
    );
    let dependencies = workspace
        .get("dependencies")
        .and_then(toml_edit::Item::as_table_like)
        .expect("engine/Cargo.toml has a [workspace.dependencies] table");
    for (name, entry) in dependencies.iter() {
        if missed.contains(&name) {
            continue;
        }
        let Some(entry) = entry.as_table_like() else {
            continue;
        };
        let on_member = entry
            .get("path")
            .and_then(toml_edit::Item::as_str)
            .is_some_and(|path| members.iter().any(|member| member == path));
        if let (true, Some(version)) = (on_member, entry.get("version")) {
            spans.push(
                version
                    .span()
                    .expect("the parse keeps the span of each version"),
            );
        }
    }
    spans.sort_by_key(|span| std::cmp::Reverse(span.start));
    let mut planted = real.clone();
    for span in spans {
        planted.replace_range(span, &format!("\"{next}\""));
    }
    planted
}

/// The manifest planted with a bump to `next` that missed `headwater-yaml`,
/// with the `headwater-yaml` entry then rewritten by `rewrite`. The bump moves
/// the package and every other entry to `next`.
fn planted_bump_missing_yaml(next: &str, rewrite: impl Fn(&str) -> String) -> String {
    let version = headwater_resolve::release::ENGINE;
    let bumped = planted_bump(next, &["headwater-yaml"]);
    let entry = format!("headwater-yaml = {{ path = \"crates/yaml\", version = \"{version}\" }}");
    assert!(
        bumped.contains(&entry),
        "engine/Cargo.toml has no `{entry}` line, so this plant cannot rewrite the \
         headwater-yaml entry into another form; the plant is out of date, not the judge"
    );
    bumped.replace(&entry, &rewrite(version))
}

/// The missed entry is caught whatever TOML form it takes. Cargo reads an
/// inline table without spaces, and a `[workspace.dependencies.<name>]`
/// sub-table, the same as the form this manifest uses, so a judge that read one
/// spelling would pass a stale version in the others.
#[test]
fn a_missed_workspace_dependency_is_named_in_every_toml_form_cargo_reads() {
    let forms: [(&str, Rewrite); 3] = [
        ("an inline table with no spaces", |v| {
            format!("headwater-yaml={{path=\"crates/yaml\",version=\"{v}\"}}")
        }),
        ("an inline table with the keys reversed", |v| {
            format!("headwater-yaml = {{ version = \"{v}\", path = \"crates/yaml\" }}")
        }),
        ("a sub-table at the end of the manifest", |_| String::new()),
    ];
    let version = headwater_resolve::release::ENGINE;
    for (form, rewrite) in forms {
        let mut planted = planted_bump_missing_yaml("99.0.1", rewrite);
        if form.starts_with("a sub-table") {
            planted.push_str(&format!(
                "\n\n[workspace.dependencies.headwater-yaml]\npath = \"crates/yaml\"\nversion = \"{version}\"\n"
            ));
        }
        let stale = stale_member_dependency_versions(&planted);
        // The finding names the stale version itself, so a judge that read
        // this form's version as missing ("names no version") does not pass.
        let stale_version = format!("names version {version},");
        assert!(
            stale.len() == 1
                && stale[0].contains("headwater-yaml")
                && stale[0].contains(&stale_version),
            "headwater-yaml was left at {version}, written as {form}; the judge should say it \
             `{stale_version}`, and it said: {stale:?}"
        );
    }
}

/// A patch bump that misses one entry is caught. This is the bump that builds
/// green: the caret requirement `^0.5.0` accepts the `0.5.1` path crate, so
/// only the crates.io publish fails. A judge that compared major and minor
/// alone would pass every planted case at `99.0.1` and pass this one too.
#[test]
fn a_patch_bump_that_misses_one_workspace_dependency_names_it() {
    let version = headwater_resolve::release::ENGINE;
    let next = next_patch(version);
    let planted = planted_bump_missing_yaml(&next, |v| {
        format!("headwater-yaml = {{ path = \"crates/yaml\", version = \"{v}\" }}")
    });
    assert!(
        planted.contains(&format!("\nversion = \"{next}\"\n")),
        "the plant did not move [workspace.package] version to {next}"
    );
    let stale = stale_member_dependency_versions(&planted);
    let stale_version = format!("names version {version},");
    assert!(
        stale.len() == 1
            && stale[0].contains("headwater-yaml")
            && stale[0].contains(&stale_version),
        "the patch bump moved the package and every other entry to {next} and left \
         headwater-yaml at {version}; the judge should name headwater-yaml alone and say it \
         `{stale_version}`, and it said: {stale:?}"
    );
}

/// A bump that moves only `[workspace.package] version` leaves every member
/// entry behind, and the judge names each one. So the judge holds each entry
/// against the package version, and not against another entry: a judge that
/// compared the entries with each other would find them all equal here.
#[test]
fn a_bump_of_the_package_version_alone_names_every_member_entry() {
    let real = workspace_manifest();
    let version = headwater_resolve::release::ENGINE;
    let entries: Vec<String> = member_dependency_versions(&real)
        .into_iter()
        .map(|entry| entry.name)
        .collect();
    assert!(
        !entries.is_empty(),
        "engine/Cargo.toml has no [workspace.dependencies] entry on a member to hold"
    );
    let missed: Vec<&str> = entries.iter().map(String::as_str).collect();
    let next = next_patch(version);
    let stale = stale_member_dependency_versions(&planted_bump(&next, &missed));
    let stale_version = format!("names version {version},");
    let unnamed: Vec<&String> = entries
        .iter()
        .filter(|name| {
            !stale
                .iter()
                .any(|line| line.starts_with(&format!("{name} ")) && line.contains(&stale_version))
        })
        .collect();
    assert!(
        stale.len() == entries.len() && unnamed.is_empty(),
        "the package moved to {next} and all {} member entries stayed at {version}; the judge \
         should name each of them, and it named {} and missed {unnamed:?}",
        entries.len(),
        stale.len()
    );
}

/// The judge reads every member that another member depends on through
/// `workspace = true`. A form the judge cannot read would drop an entry from
/// this set, and a case that holds only the entries it read cannot see that.
#[test]
fn the_judge_reads_every_member_entry_that_a_member_depends_on() {
    let manifest = workspace_manifest();
    let read: Vec<String> = member_dependency_versions(&manifest)
        .into_iter()
        .map(|entry| entry.name)
        .collect();
    let members = workspace_members();
    let names: Vec<&String> = members.iter().map(|(name, _)| name).collect();
    let mut depended_on: Vec<String> = Vec::new();
    for (_, text) in &members {
        let document: toml_edit::DocumentMut = text.parse().expect("a member manifest is TOML");
        for table in ["dependencies", "dev-dependencies", "build-dependencies"] {
            let Some(deps) = document.get(table).and_then(toml_edit::Item::as_table_like) else {
                continue;
            };
            for (name, entry) in deps.iter() {
                let inherits = entry
                    .as_table_like()
                    .and_then(|entry| entry.get("workspace"))
                    .and_then(toml_edit::Item::as_bool)
                    == Some(true);
                if inherits
                    && names.iter().any(|member| *member == name)
                    && !depended_on.iter().any(|seen| seen == name)
                {
                    depended_on.push(name.to_string());
                }
            }
        }
    }
    assert!(
        !depended_on.is_empty(),
        "no member depends on another through `workspace = true`, so this case measured nothing"
    );
    let unread: Vec<&String> = depended_on
        .iter()
        .filter(|name| !read.contains(name))
        .collect();
    assert!(
        unread.is_empty(),
        "members depend on these through `workspace = true`, and the judge read no \
         [workspace.dependencies] entry for them: {unread:?} (read {} entries)",
        read.len()
    );
}

/// An internal dependency with a path and no version is named too: cargo
/// refuses to publish a crate that depends on one.
#[test]
fn a_workspace_dependency_on_a_member_with_no_version_is_named() {
    let real = workspace_manifest();
    let version = headwater_resolve::release::ENGINE;
    let planted = real.replace(
        &format!("headwater-yaml = {{ path = \"crates/yaml\", version = \"{version}\" }}"),
        "headwater-yaml = { path = \"crates/yaml\" }",
    );
    assert_ne!(
        planted, real,
        "engine/Cargo.toml has no headwater-yaml entry to plant on"
    );
    let stale = stale_member_dependency_versions(&planted);
    assert!(
        stale.len() == 1 && stale[0].contains("headwater-yaml") && stale[0].contains("no version"),
        "headwater-yaml was planted with no version; the judge said: {stale:?}"
    );
}

/// An entry whose path names no member is not this workspace's to hold, so a
/// different version on it is not a finding.
#[test]
fn a_workspace_dependency_outside_the_members_is_not_read() {
    let real = workspace_manifest();
    let planted = real.replacen(
        "[workspace.dependencies]\n",
        "[workspace.dependencies]\nelsewhere = { path = \"../elsewhere\", version = \"0.0.1\" }\n",
        1,
    );
    assert_ne!(
        planted, real,
        "engine/Cargo.toml has no [workspace.dependencies] table"
    );
    let stale = stale_member_dependency_versions(&planted);
    assert!(
        stale.is_empty(),
        "the judge read a non-member entry: {stale:?}"
    );
}

/// Every member inherits its version from `[workspace.package]`, so the one
/// number the dependency entries are held against is the number each crate
/// publishes under.
#[test]
fn every_member_takes_its_version_from_the_workspace() {
    let own: Vec<String> = workspace_members()
        .into_iter()
        .filter(|(_, manifest)| {
            !manifest
                .lines()
                .any(|line| line.trim() == "version.workspace = true")
        })
        .map(|(name, _)| name)
        .collect();
    assert!(
        own.is_empty(),
        "these members do not write `version.workspace = true`, so a workspace bump \
         does not move them: {own:?}"
    );
}
