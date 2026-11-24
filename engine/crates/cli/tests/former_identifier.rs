// SPDX-License-Identifier: Apache-2.0
//! [#2025](https://github.com/headwater-ai/headwater/issues/2025): a former
//! identifier resolves to the document that carried it.
//!
//! # What this target holds
//!
//! The base package declares `former_ids` in the `former_identifier` role, and
//! the CLI finds that facet by its role. So over this repository's own
//! taxonomy, `headwater explain` given a former identifier answers the
//! document that lists it, under its live identifier, and `headwater check`
//! reports a former identifier that another document also claims as
//! `identifier.claimed_twice`. A corpus that lists a former identifier once
//! draws no finding about it, so the gate is held in both directions.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
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

/// Repoint `contents.bundles` in a scratch copy of the authored manifest, as
/// `classify.rs` does and for the reason it states.
fn repoint_bundles(package: &Path) {
    let up = "../".repeat(headwater_resolve::package::PACKAGES.split('/').count() + 1);
    let manifest = package.join(headwater_resolve::package::MANIFEST);
    let text = std::fs::read_to_string(&manifest).expect("the scratch manifest reads");
    let from = "  bundles: ../../docs/taxonomies";
    assert!(text.contains(from), "the authored manifest states `{from}`");
    let to = format!("  bundles: {up}docs/taxonomies");
    std::fs::write(&manifest, text.replace(from, &to)).expect("the scratch manifest writes");
}

/// The renamed decision. It lists the identifier it carried before.
const RENAMED: &str = "docs/decisions/0001-a-renamed-decision.md";

fn decision(id: &str, title: &str, former: &str) -> String {
    format!(
        "---\nid: {id}\ntitle: {title}\nstatus: current\nstatus_since: 2026-11-24\nlast_verified: 2026-11-24\nsummary: A fixture decision for former identifiers.\n{former}---\n\n# {title}\n\n## Context\n\nA fixture.\n\n## Decision\n\nA fixture.\n"
    )
}

/// A scratch corpus over this repository's own taxonomy source, with the
/// decisions `files` names written in.
struct Root {
    at: PathBuf,
}

impl Root {
    fn new(label: &str, files: &[(&str, String)]) -> Root {
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-former-{}-{label}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).expect("the root is made");

        let repository = repository();
        copy(
            &repository.join("taxonomy-source/headwater-standard"),
            &at.join(".headwater/packages/headwater-standard"),
        );
        repoint_bundles(&at.join(".headwater/packages/headwater-standard"));
        copy(
            &repository.join("docs/taxonomies"),
            &at.join("docs/taxonomies"),
        );
        for name in ["taxonomy.yml", "overlay.yml"] {
            let to = at.join(".headwater").join(name);
            std::fs::create_dir_all(to.parent().expect("it has a parent"))
                .expect("the declaration directory is there");
            std::fs::copy(repository.join(".headwater").join(name), to)
                .expect("the declaration copies");
        }
        for (path, text) in files {
            let path = at.join(path);
            std::fs::create_dir_all(path.parent().expect("it has a parent"))
                .expect("the shelf is made");
            std::fs::write(&path, text).expect("the document writes");
        }

        let root = Root { at };
        let resolved = root.run(&["taxonomy", "resolve"]);
        assert_eq!(
            resolved.status.code(),
            Some(0),
            "the fixture resolves\n{}",
            String::from_utf8_lossy(&resolved.stderr)
        );
        root
    }

    fn run(&self, arguments: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_headwater"))
            .args(arguments)
            .arg("--root")
            .arg(&self.at)
            .output()
            .expect("the binary runs")
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.at);
    }
}

/// The decisive case at the CLI. `explain` given the former identifier exits 0
/// and names the document under its live identifier, in text and in JSON.
#[test]
fn explain_resolves_a_former_identifier_to_its_document() {
    let root = Root::new(
        "explain",
        &[(
            RENAMED,
            decision(
                "HW-DR-0001",
                "A renamed decision",
                "former_ids:\n  - HW-DR-0900\n",
            ),
        )],
    );

    for args in [
        &["explain", "HW-DR-0900"][..],
        &["explain", "HW-DR-0900", "--json"],
    ] {
        let explained = root.run(args);
        let stdout = String::from_utf8_lossy(&explained.stdout);
        assert_eq!(
            explained.status.code(),
            Some(0),
            "{args:?}\n{stdout}\n{}",
            String::from_utf8_lossy(&explained.stderr)
        );
        assert!(stdout.contains(RENAMED), "{args:?}\n{stdout}");
        assert!(stdout.contains("HW-DR-0001"), "{args:?}\n{stdout}");
    }

    // A value no document lists is still refused, so the resolution above is
    // the former identifier's and not a fallback that answers anything.
    let refused = root.run(&["explain", "HW-DR-0901"]);
    assert_ne!(refused.status.code(), Some(0));
}

/// The gate, in its three directions over one rule. A former identifier listed
/// once draws no `identifier.claimed_twice`. One that a second document also
/// lists, or that equals a live identifier, draws it in both files.
#[test]
fn check_reports_a_former_identifier_claimed_twice() {
    // A finding opens its line with the rule and its obligation in brackets.
    // The bare name also stands in the run's list of rules, which is no finding.
    const RULE: &str = "identifier.claimed_twice (";
    let once = Root::new(
        "once",
        &[(
            RENAMED,
            decision(
                "HW-DR-0001",
                "A renamed decision",
                "former_ids: [HW-DR-0900]\n",
            ),
        )],
    );
    let checked = once.run(&["check"]);
    let stdout = String::from_utf8_lossy(&checked.stdout);
    assert!(
        !stdout
            .lines()
            .any(|line| line.trim_start().starts_with(RULE)),
        "{stdout}"
    );
    assert!(
        !stdout.contains("former_ids"),
        "a declared facet draws no finding of its own\n{stdout}"
    );

    for (label, former) in [("former", "HW-DR-0900"), ("live", "HW-DR-0001")] {
        let twice = Root::new(
            label,
            &[
                (
                    RENAMED,
                    decision(
                        "HW-DR-0001",
                        "A renamed decision",
                        "former_ids: [HW-DR-0900]\n",
                    ),
                ),
                (
                    "docs/decisions/0002-a-second-decision.md",
                    decision(
                        "HW-DR-0002",
                        "A second decision",
                        &format!("former_ids: [{former}]\n"),
                    ),
                ),
            ],
        );
        let checked = twice.run(&["check"]);
        let stdout = String::from_utf8_lossy(&checked.stdout);
        let hits: Vec<&str> = stdout
            .lines()
            .filter(|line| line.trim_start().starts_with(RULE))
            .collect();
        assert_eq!(hits.len(), 2, "{label}: one finding in each file\n{stdout}");
        // The report folds at eighty columns, so the words are compared and
        // not the lines.
        let words = stdout.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(
            words.contains(&format!("remove `{former}` from the `former_ids`")),
            "{label}: the repair names the list\n{stdout}"
        );
    }
}

/// The pointer object of a route's JSON whose path is `path`: the text from its
/// `path` member to the next pointer's, or to the end of the pointers.
fn pointer_at<'a>(json: &'a str, path: &str) -> Option<&'a str> {
    let start = json.find(&format!("\"path\": \"{path}\""))?;
    let rest = &json[start..];
    let end = rest[1..]
        .find("\"path\": \"")
        .map(|at| at + 1)
        .or_else(|| rest.find("\"withheld\""))
        .unwrap_or(rest.len());
    Some(&rest[..end])
}

/// The decisive case for `headwater route`. A task that writes a document's
/// former identifier, or its live one at the end of a sentence, is offered that
/// document by its identifier, and not through a ranked hit on the terms `hw`
/// and `dr`. A value no document lists offers nothing by identifier.
#[test]
fn route_names_a_document_by_its_live_or_former_identifier() {
    let root = Root::new(
        "route",
        &[(
            RENAMED,
            decision(
                "HW-DR-0001",
                "A renamed decision",
                "former_ids:\n  - HW-DR-0900\n",
            ),
        )],
    );

    for (task, written) in [
        ("what did HW-DR-0900 rule", "HW-DR-0900"),
        ("what did we rule in HW-DR-0001.", "HW-DR-0001"),
    ] {
        let routed = root.run(&["route", task, "--json"]);
        let stdout = String::from_utf8_lossy(&routed.stdout);
        assert_eq!(
            routed.status.code(),
            Some(0),
            "{task}\n{stdout}\n{}",
            String::from_utf8_lossy(&routed.stderr)
        );
        assert!(stdout.contains("\"version\": \"1.3\""), "{task}\n{stdout}");
        let pointer = pointer_at(&stdout, RENAMED)
            .unwrap_or_else(|| panic!("{task}: the renamed decision is offered\n{stdout}"));
        assert!(
            pointer.contains("\"by\": \"identifier\""),
            "{task}: offered by its identifier\n{pointer}"
        );
        let identifiers = pointer
            .split("\"identifiers\":")
            .nth(1)
            .unwrap_or_else(|| panic!("{task}: the evidence lists identifiers\n{pointer}"));
        assert!(
            identifiers.contains(&format!("\"{written}\"")),
            "{task}: the identifier as the task wrote it\n{pointer}"
        );
    }

    // The text report tells a former identifier from a live one, on a line
    // with no em dash, which marks a pointer in this report (HW-OBL-0149).
    let text = root.run(&["route", "what did HW-DR-0900 rule"]);
    let stdout = String::from_utf8_lossy(&text.stdout);
    let line = stdout
        .lines()
        .find(|line| line.contains("names its former identifier, HW-DR-0900 (now HW-DR-0001)"))
        .unwrap_or_else(|| panic!("the former identifier is named\n{stdout}"));
    assert!(!line.contains('\u{2014}'), "{line}");

    // A document both identified and named is offered once, as identified.
    let both = root.run(&["route", "HW-DR-0900 is a renamed decision", "--json"]);
    let stdout = String::from_utf8_lossy(&both.stdout);
    assert_eq!(
        stdout.matches(&format!("\"path\": \"{RENAMED}\"")).count(),
        1,
        "{stdout}"
    );
    let pointer = pointer_at(&stdout, RENAMED).expect("the renamed decision is offered");
    assert!(pointer.contains("\"by\": \"identifier\""), "{pointer}");

    // A value no document lists is offered by no identifier.
    let missed = root.run(&["route", "what did HW-DR-0901 rule", "--json"]);
    let stdout = String::from_utf8_lossy(&missed.stdout);
    assert_eq!(missed.status.code(), Some(0), "{stdout}");
    assert!(!stdout.contains("\"by\": \"identifier\""), "{stdout}");
}

/// Folded clause A of #2025. A former identifier is spent, so `headwater new`
/// mints past it, and never mints a value that the index then resolves to the
/// renamed document and refuses as taken.
#[test]
fn new_mints_past_a_former_identifier() {
    let root = Root::new(
        "mint",
        &[(
            RENAMED,
            decision(
                "HW-DR-0001",
                "A renamed decision",
                "former_ids: [HW-DR-0002]\n",
            ),
        )],
    );
    let made = root.run(&["new", "decision", "--title", "A scratch ruling"]);
    let stderr = String::from_utf8_lossy(&made.stderr);
    assert_eq!(
        made.status.code(),
        Some(0),
        "{}\n{stderr}",
        String::from_utf8_lossy(&made.stdout)
    );
    let renamed = root.at.join(RENAMED);
    let minted = std::fs::read_dir(root.at.join("docs/decisions"))
        .expect("the decisions shelf is there")
        .map(|entry| entry.expect("the entry reads").path())
        .find(|path| *path != renamed)
        .expect("the scaffolder wrote a document");
    let text = std::fs::read_to_string(&minted).expect("the minted document reads");
    assert!(
        text.contains("id: HW-DR-0003\n"),
        "{}\n{text}",
        minted.display()
    );
}

/// Folded clause B of #2025. A former identifier that equals a live one is
/// reported as claimed twice, and the claim file of the value still has one
/// claimant: the document that holds it as its live identifier. So the claim
/// check still runs for that document.
#[test]
fn a_live_claimant_keeps_its_claim_check_when_a_former_identifier_collides() {
    const SECOND: &str = "docs/decisions/0002-a-second-decision.md";
    let root = Root::new(
        "claimant",
        &[
            (RENAMED, decision("HW-DR-0001", "A renamed decision", "")),
            (
                SECOND,
                decision(
                    "HW-DR-0002",
                    "A second decision",
                    "former_ids: [HW-DR-0001]\n",
                ),
            ),
            (
                ".headwater/ids/decision_id/HW-DR-0002",
                format!("{SECOND}\n"),
            ),
        ],
    );
    let checked = root.run(&["check"]);
    let stdout = String::from_utf8_lossy(&checked.stdout);
    let words = stdout.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(
        words.contains("identifier.claim.missing (")
            && words.contains(".headwater/ids/decision_id/HW-DR-0001"),
        "the live claimant's claim is owed\n{stdout}"
    );
}

/// The other direction of clause B. A value live on two documents is still
/// contended when a third, earlier in path order, lists it as former. The
/// index compares each claim with the first one only, so both live pairs are
/// reported as former pairs, and a claim check that read the report alone
/// would owe one claim file to two claimants.
#[test]
fn two_live_claimants_stay_contended_when_a_former_identifier_comes_first() {
    let root = Root::new(
        "contended",
        &[
            (
                RENAMED,
                decision(
                    "HW-DR-0001",
                    "A renamed decision",
                    "former_ids: [HW-DR-0005]\n",
                ),
            ),
            (
                "docs/decisions/0002-a-second-decision.md",
                decision("HW-DR-0005", "A second decision", ""),
            ),
            (
                "docs/decisions/0003-a-third-decision.md",
                decision("HW-DR-0005", "A third decision", ""),
            ),
            (
                ".headwater/ids/decision_id/HW-DR-0001",
                format!("{RENAMED}\n"),
            ),
        ],
    );
    let checked = root.run(&["check"]);
    let stdout = String::from_utf8_lossy(&checked.stdout);
    let words = stdout.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(
        words.contains("identifier.claimed_twice ("),
        "the collision is reported\n{stdout}"
    );
    assert!(
        !words.contains(".headwater/ids/decision_id/HW-DR-0005"),
        "no claim is owed for a value two documents hold live\n{stdout}"
    );
}

/// A live holder on the untyped shelf counts as well. The duplicate report
/// compares both shelves, and one claim file cannot name two claimants, so a
/// value that a typed and an untyped document both hold live is contended and
/// owes no claim.
#[test]
fn a_typed_and_an_untyped_live_holder_stay_contended() {
    const UNTYPED: &str = "docs/notes/an-untyped-note.md";
    let root = Root::new(
        "untyped",
        &[
            (
                "docs/decisions/0005-a-typed-decision.md",
                decision("HW-DR-0005", "A typed decision", ""),
            ),
            (
                UNTYPED,
                "---\nid: HW-DR-0005\ntitle: An untyped note\n---\n\n# An untyped note\n\nA fixture.\n"
                    .to_string(),
            ),
        ],
    );
    let checked = root.run(&["check"]);
    let stdout = String::from_utf8_lossy(&checked.stdout);
    let words = stdout.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(
        words.contains("identifier.claimed_twice (") && words.contains(UNTYPED),
        "the untyped holder is indexed and the collision is reported\n{stdout}"
    );
    assert!(
        !words.contains(".headwater/ids/decision_id/HW-DR-0005"),
        "no claim is owed for a value a typed and an untyped document hold live\n{stdout}"
    );
}
