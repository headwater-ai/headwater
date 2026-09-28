// SPDX-License-Identifier: Apache-2.0
//! `check --fix` over a decision that a live successor supersedes and that
//! still reads `current` (#1198).
//!
//! The check crate holds the rule and its patch against a fixture tree, and
//! the scaffold crate holds the writer against a string. This file holds the
//! two together over this repository's own taxonomy: the verb reports the
//! target, and the fix writes both the state and the successor's own
//! state-entry date into the file.

use std::path::{Path, PathBuf};
use std::process::Command;

const RULE: &str = "lifecycle.state.not_set_by_edge";

/// The obligation `CT-LIFE-6` of headwater/standard binds the rule to (#1212).
/// A finding names it beside the rule, so a control that stopped naming the
/// rule changes the line both cases read.
const OBLIGATION: &str = "OB-LIFE-6";

/// The pinned date every run here takes, so the fix cannot read a clock.
const NOW: &str = "2026-09-27";

/// The output with each run of whitespace folded to one space, so a finding
/// reads as one sentence wherever the report wrapped it.
fn flat(out: &str) -> String {
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

struct Root {
    at: PathBuf,
}

#[derive(Debug)]
struct Ran {
    code: Option<i32>,
    out: String,
    err: String,
}

impl Root {
    /// The label keeps two cases apart: cargo runs them as threads of one
    /// process, so the pid alone names one directory for both.
    fn new(label: &str) -> Root {
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-superseded-left-current-{}-{label}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).expect("the root is made");
        let repository = repository();
        copy(
            &repository.join(headwater_resolve::package::PACKAGES),
            &at.join(headwater_resolve::package::PACKAGES),
        );
        copy(
            &repository.join("docs/taxonomies"),
            &at.join("docs/taxonomies"),
        );
        for file in [".headwater/taxonomy.yml", ".headwater/overlay.yml"] {
            std::fs::copy(repository.join(file), at.join(file)).expect("the declaration copies");
        }
        let root = Root { at };
        let resolved = root.run(&["taxonomy", "resolve"]);
        assert_eq!(resolved.code, Some(0), "the fixture resolves\n{resolved:?}");
        root
    }

    /// Scaffold one decision and return its path and its identifier.
    fn decision(&self, title: &str) -> (PathBuf, String) {
        let before = self.decisions();
        let made = self.run(&["new", "decision", "--title", title]);
        assert_eq!(made.code, Some(0), "the decision scaffolds\n{made:?}");
        let path = self
            .decisions()
            .into_iter()
            .find(|path| !before.contains(path))
            .expect("the scaffolder wrote a document");
        let text = std::fs::read_to_string(&path).expect("the document reads");
        let id = text
            .lines()
            .find_map(|line| line.strip_prefix("id: "))
            .expect("the scaffolder wrote an identifier")
            .trim()
            .to_string();
        (path, id)
    }

    fn decisions(&self) -> Vec<PathBuf> {
        match std::fs::read_dir(self.at.join("docs/decisions")) {
            Ok(entries) => entries
                .map(|entry| entry.expect("the entry reads").path())
                .filter(|path| path.extension().is_some_and(|kind| kind == "md"))
                .collect(),
            Err(_) => Vec::new(),
        }
    }

    fn run(&self, arguments: &[&str]) -> Ran {
        let output = Command::new(env!("CARGO_BIN_EXE_headwater"))
            .args(arguments)
            .arg("--root")
            .arg(&self.at)
            .output()
            .expect("the binary runs");
        Ran {
            code: output.status.code(),
            out: String::from_utf8_lossy(&output.stdout).into_owned(),
            err: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.at);
    }
}

fn copy(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("the directory is there");
    for entry in std::fs::read_dir(from).expect("the fixture directory reads") {
        let entry = entry.expect("the entry reads");
        let target = to.join(entry.file_name());
        match entry.file_type().expect("the file type reads").is_dir() {
            true => copy(&entry.path(), &target),
            false => {
                std::fs::copy(entry.path(), &target).expect("the fixture copies");
            }
        }
    }
}

/// Write `key: value` into the front matter of `path`, replacing the line
/// that opens with `key:` or adding one before the block closes.
fn set_facet(path: &Path, key: &str, value: &str) {
    let text = std::fs::read_to_string(path).expect("the document reads");
    let body = text
        .strip_prefix("---\n")
        .expect("the document opens a block");
    let (block, rest) = body.split_once("\n---\n").expect("the block closes");
    let prefix = format!("{key}:");
    let mut lines: Vec<String> = block.lines().map(str::to_string).collect();
    match lines.iter().position(|line| line.starts_with(&prefix)) {
        Some(at) => lines[at] = format!("{key}: {value}"),
        None => lines.push(format!("{key}: {value}")),
    }
    std::fs::write(path, format!("---\n{}\n---\n{rest}", lines.join("\n")))
        .expect("the document writes");
}

/// Put `lines` into the front matter of `path`, before the block closes.
fn into_front_matter(path: &Path, lines: &str) {
    let text = std::fs::read_to_string(path).expect("the document reads");
    let body = text
        .strip_prefix("---\n")
        .expect("the document opens a block");
    let (block, rest) = body.split_once("\n---\n").expect("the block closes");
    std::fs::write(path, format!("---\n{block}\n{lines}---\n{rest}")).expect("the document writes");
}

fn facet<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    let prefix = format!("{key}: ");
    text.lines()
        .find_map(|line| line.strip_prefix(prefix.as_str()))
        .map(str::trim)
}

/// A live successor declares `supersedes` at a decision that still reads
/// `current`. `check` reports the target and exits non-zero under `--strict`,
/// and `check --fix` writes `superseded` and the successor's own date.
#[test]
fn check_fix_writes_the_state_and_the_successor_s_date_onto_the_target() {
    let root = Root::new("fix");
    let (target, target_id) = root.decision("A ruling that a later one replaces");
    let (successor, _) = root.decision("The ruling that replaces it");
    set_facet(&target, "status", "current");
    set_facet(&target, "status_since", "2026-07-01");
    set_facet(&successor, "status", "current");
    set_facet(&successor, "status_since", "2026-08-05");
    into_front_matter(
        &successor,
        &format!("relations:\n  supersedes:\n    - {target_id}\n"),
    );
    // A scaffold may already carry a relations block; either way the
    // successor now declares the edge.
    let successor_text = std::fs::read_to_string(&successor).expect("the successor reads");
    assert!(
        successor_text.contains(&format!("- {target_id}")),
        "{successor_text}"
    );

    let checked = root.run(&["check", "--strict", "--no-cache", "--now", NOW]);
    // The report wraps a finding across lines, and where the break falls
    // moves with the obligation the rule names, so the match reads the
    // output with each run of whitespace folded to one space.
    let named = format!("{RULE} ({OBLIGATION}): `{target_id}` stands at `current`");
    let reported = flat(&checked.out).contains(&named);
    assert!(
        reported,
        "the target is reported\n{}{}",
        checked.out, checked.err
    );
    assert_ne!(checked.code, Some(0), "an error fails a strict run");

    let fixed = root.run(&["check", "--fix", "--no-cache", "--now", NOW]);
    let text = std::fs::read_to_string(&target).expect("the target reads");
    assert_eq!(
        facet(&text, "status"),
        Some("superseded"),
        "{text}\n{}{}",
        fixed.out,
        fixed.err
    );
    assert_eq!(facet(&text, "status_since"), Some("2026-08-05"), "{text}");
    let still = flat(&fixed.out).contains(&named);
    assert!(!still, "the fixed run no longer reports it\n{}", fixed.out);
}

/// A live successor over a target that stands at a terminal state other than
/// the one the edge sets is reported and does not fail a strict run.
///
/// Which terminal state the target ends at is a judgment between two, so the
/// remedy is not mechanical and total, and the finding is advisory with no
/// patch. The scratch root carries the four files the overlay's language
/// regime lists outside the corpus root, so that nothing else fails the run
/// and the exit status is this rule's to decide.
#[test]
fn a_target_deprecated_before_its_successor_is_advisory_and_strict_passes() {
    let root = Root::new("deprecated");
    let repository = repository();
    for file in [
        "README.md",
        ".github/CONTRIBUTING.md",
        ".github/SECURITY.md",
        ".github/ISSUE_TEMPLATE/issue.md",
    ] {
        let to = root.at.join(file);
        std::fs::create_dir_all(to.parent().expect("a parent")).expect("the directory is made");
        std::fs::copy(repository.join(file), &to).expect("the outside-root file copies");
    }
    let (target, target_id) = root.decision("A ruling that was deprecated first");
    let (successor, successor_id) = root.decision("The ruling that later replaces it");
    set_facet(&target, "status", "deprecated");
    set_facet(&target, "status_since", "2026-07-01");
    into_front_matter(
        &target,
        &format!("relations:\n  superseded_by:\n    - {successor_id}\n"),
    );
    set_facet(&successor, "status", "current");
    set_facet(&successor, "status_since", "2026-08-05");
    into_front_matter(
        &successor,
        &format!("relations:\n  supersedes:\n    - {target_id}\n"),
    );

    let checked = root.run(&["check", "--strict", "--no-cache", "--now", NOW]);
    let named = format!("{RULE} ({OBLIGATION}): `{target_id}` stands at `deprecated`");
    assert!(
        flat(&checked.out).contains(&named),
        "the target is still reported\n{}{}",
        checked.out,
        checked.err
    );
    assert_eq!(
        checked.code,
        Some(0),
        "an advisory finding does not fail a strict run\n{}{}",
        checked.out,
        checked.err
    );

    let fixed = root.run(&["check", "--fix", "--no-cache", "--now", NOW]);
    let text = std::fs::read_to_string(&target).expect("the target reads");
    assert_eq!(
        facet(&text, "status"),
        Some("deprecated"),
        "no fix is offered\n{text}\n{}",
        fixed.err
    );
}
