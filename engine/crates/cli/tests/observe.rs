// SPDX-License-Identifier: Apache-2.0
//! The decisive fixture for `headwater observe` (#2024).
//!
//! # The defect this target exists for
//!
//! `headwater check` reads a `kind: verification` entry of
//! `.headwater/observations.yml` and compares its `criterion_digest` with the
//! census digest of the criterion the verification proves. No verb wrote such
//! an entry, so an adopter wrote the digest by hand, and a digest the reader
//! does not accept reads as `suspect` on the first run.
//!
//! # Why these cases drive the binary
//!
//! The claim is a round trip through two verbs: what `observe` writes, `check`
//! reads as `observed`. A case that called the writer and the reader as
//! functions would prove that they agree inside one process, and not that the
//! file between them carries the agreement. So each case copies one
//! verification and the criterion it proves into a scratch repository, runs
//! the built binary to write the snapshot and runs it again to check it.

mod common;

use common::{repository, Root};
use std::path::Path;

const VERIFICATION: &str = "HW-VER-0002";
const CRITERION: &str = "HW-AC-0003";
const COMMIT: &str = "788885a9";
const SNAPSHOT: &str = ".headwater/observations.yml";

/// The path, under the repository, of each document a case copies.
fn source(id: &str) -> &'static str {
    match id {
        "HW-VER-0002" => "docs/verifications/0002-a-fixture-tree-of-document-paths-symlinks-and-identifiers-proves-the-document-path-finding.md",
        "HW-VER-0003" => "docs/verifications/0003-a-fixture-tree-of-three-list-anchors-proves-that-the-export-carries-list-members.md",
        "HW-AC-0003" => "docs/acceptance-criteria/0003-check-reports-a-document-path-written-as-a-relation-target-once-in-every-spelling-and-through-a-symlink.md",
        "HW-AC-0004" => "docs/acceptance-criteria/0004-each-list-anchor-in-the-native-export-carries-its-sorted-members-and-a-single-pattern-carries-none.md",
        other => panic!("no source for {other}"),
    }
}

/// Copy one document of this repository into the scratch root, at its own
/// path, with `edit` applied to its text.
fn place(root: &Root, id: &str, edit: impl FnOnce(String) -> String) {
    let path = source(id);
    let text = std::fs::read_to_string(repository().join(path)).expect("the document reads");
    let target = root.at.join(path);
    std::fs::create_dir_all(target.parent().expect("a parent")).expect("the shelf is made");
    std::fs::write(&target, edit(text)).expect("the document writes");
}

/// A scratch root that holds HW-VER-0002 and HW-AC-0003, the one criterion it
/// proves.
fn pair(label: &str) -> Root {
    let root = Root::new(label);
    place(&root, VERIFICATION, |text| text);
    place(&root, CRITERION, |text| text);
    root
}

fn snapshot(root: &Path) -> String {
    std::fs::read_to_string(root.join(SNAPSHOT)).expect("the snapshot reads")
}

/// The line of the verification block that names `id`, from the text report.
fn state(out: &str, id: &str) -> String {
    let prefix = format!("{id} ");
    out.lines()
        .map(str::trim)
        .find(|line| line.starts_with(&prefix))
        .unwrap_or_else(|| panic!("the report names {id}:\n{out}"))
        .to_string()
}

/// The round trip the issue exists for: the written digest is the one the
/// reader accepts, an edit to the criterion makes it suspect, and a second run
/// of the writer makes it observed again.
#[test]
fn what_observe_writes_check_reads_as_observed_and_an_edited_criterion_reads_suspect() {
    let root = pair("round-trip");

    let before = root.run(&["check"]);
    assert_eq!(
        state(&before.out, VERIFICATION),
        format!("{VERIFICATION} declared"),
        "{before:?}"
    );

    let wrote = root.run(&["observe", VERIFICATION, "--commit", COMMIT]);
    assert_eq!(wrote.code, Some(0), "{wrote:?}");

    let observed = root.run(&["check"]);
    assert_eq!(
        state(&observed.out, VERIFICATION),
        format!("{VERIFICATION} observed at {COMMIT}"),
        "{observed:?}"
    );
    assert!(
        !observed
            .out
            .contains("relation.target.verification.suspect"),
        "an observed verification is no finding: {}",
        observed.out
    );

    // One byte of the criterion moves.
    let criterion = root.at.join(source(CRITERION));
    let text = std::fs::read_to_string(&criterion).expect("the criterion reads");
    std::fs::write(
        &criterion,
        text.replacen("# Check reports", "# Check  reports", 1),
    )
    .expect("the criterion writes");

    let suspect = root.run(&["check"]);
    assert_eq!(
        state(&suspect.out, VERIFICATION),
        format!("{VERIFICATION} suspect since {COMMIT}, {CRITERION} changed"),
        "{suspect:?}"
    );
    assert!(
        suspect.out.contains("relation.target.verification.suspect"),
        "{}",
        suspect.out
    );

    let again = root.run(&["observe", VERIFICATION, "--commit", COMMIT]);
    assert_eq!(again.code, Some(0), "{again:?}");
    let restored = root.run(&["check"]);
    assert_eq!(
        state(&restored.out, VERIFICATION),
        format!("{VERIFICATION} observed at {COMMIT}"),
        "{restored:?}"
    );
}

/// Done-when 2: a second run with the same input changes nothing.
#[test]
fn a_second_run_with_the_same_input_leaves_the_snapshot_byte_identical() {
    let root = pair("idempotent");
    let first = root.run(&["observe", VERIFICATION, "--commit", COMMIT]);
    assert_eq!(first.code, Some(0), "{first:?}");
    let once = snapshot(&root.at);
    let second = root.run(&["observe", VERIFICATION, "--commit", COMMIT]);
    assert_eq!(second.code, Some(0), "{second:?}");
    assert_eq!(snapshot(&root.at), once);
}

/// Every entry the verb does not write keeps its bytes, comments and a control
/// entry included, and the entry it replaces is the only one that moves.
#[test]
fn every_other_entry_keeps_its_bytes_and_the_named_entry_is_replaced() {
    let root = pair("keeps");
    let before = format!(
        "# written by hand\nCTL-ONE:\n  commit: abc1234   # the control ran here\n\n{VERIFICATION}:\n  kind: verification\n  commit: 0000000\n  criterion_digest: \"sha256:old\"\n# a trailing note\nCTL-TWO:\n  kind: control\n  commit: def5678\n"
    );
    std::fs::write(root.at.join(SNAPSHOT), &before).expect("the snapshot writes");

    let wrote = root.run(&["observe", VERIFICATION, "--commit", COMMIT]);
    assert_eq!(wrote.code, Some(0), "{wrote:?}");
    let after = snapshot(&root.at);

    let head = "# written by hand\nCTL-ONE:\n  commit: abc1234   # the control ran here\n\n";
    let tail = "# a trailing note\nCTL-TWO:\n  kind: control\n  commit: def5678\n";
    assert!(after.starts_with(head), "{after}");
    assert!(after.ends_with(tail), "{after}");
    assert!(!after.contains("sha256:old"), "{after}");
    assert!(
        after.contains(&format!("  commit: \"{COMMIT}\"\n")),
        "{after}"
    );

    let observed = root.run(&["check"]);
    assert_eq!(
        state(&observed.out, VERIFICATION),
        format!("{VERIFICATION} observed at {COMMIT}"),
        "{observed:?}"
    );
}

/// A new entry is appended after every entry already there.
#[test]
fn a_new_entry_is_appended_after_the_entries_already_there() {
    let root = pair("appends");
    let before = "CTL-ONE:\n  commit: abc1234";
    std::fs::write(root.at.join(SNAPSHOT), before).expect("the snapshot writes");
    let wrote = root.run(&["observe", VERIFICATION, "--commit", COMMIT]);
    assert_eq!(wrote.code, Some(0), "{wrote:?}");
    let after = snapshot(&root.at);
    assert!(
        after.starts_with("CTL-ONE:\n  commit: abc1234\n"),
        "{after}"
    );
    assert!(
        after.contains(&format!("\n{VERIFICATION}:\n  kind: verification\n")),
        "{after}"
    );
}

/// The four refusals that write nothing: a commit the reader would report
/// `unknown`, a snapshot that does not read, an identifier that names no
/// verification, and an identifier that names a document of another kind.
#[test]
fn a_commit_the_reader_rejects_writes_nothing() {
    let root = pair("bad-commit");
    let ran = root.run(&["observe", VERIFICATION, "--commit", "nonexistent"]);
    assert_eq!(ran.code, Some(1), "{ran:?}");
    assert!(ran.err.contains("nonexistent"), "{ran:?}");
    assert!(!root.at.join(SNAPSHOT).exists(), "nothing is written");
}

#[test]
fn a_snapshot_that_does_not_read_is_not_written_over() {
    let root = pair("unread");
    let before = "CTL-ONE: [not, a, mapping\n";
    std::fs::write(root.at.join(SNAPSHOT), before).expect("the snapshot writes");
    let ran = root.run(&["observe", VERIFICATION, "--commit", COMMIT]);
    assert_eq!(ran.code, Some(1), "{ran:?}");
    assert!(ran.err.contains(SNAPSHOT), "{ran:?}");
    assert_eq!(snapshot(&root.at), before);
}

#[test]
fn an_identifier_that_names_no_verification_is_refused_and_the_others_are_written() {
    let root = pair("unknown-id");
    let ran = root.run(&[
        "observe",
        "HW-VER-9999",
        CRITERION,
        VERIFICATION,
        "--commit",
        COMMIT,
    ]);
    assert_eq!(ran.code, Some(1), "{ran:?}");
    assert!(ran.err.contains("HW-VER-9999"), "{ran:?}");
    assert!(ran.err.contains(CRITERION), "{ran:?}");
    let after = snapshot(&root.at);
    assert!(after.contains(&format!("{VERIFICATION}:\n")), "{after}");
    assert!(!after.contains("HW-VER-9999"), "{after}");
    assert!(!after.contains(&format!("{CRITERION}:")), "{after}");
}

/// The snapshot holds one `criterion_digest` an entry, so a verification that
/// two criteria reach has no entry that states it, and one that no criterion
/// reaches has no digest to record.
#[test]
fn a_verification_two_criteria_reach_or_none_reaches_is_refused_with_the_criteria() {
    let root = pair("not-one");
    // HW-AC-0004 is made to name HW-VER-0002 as well, so two criteria reach it.
    place(&root, "HW-AC-0004", |text| {
        text.replace("HW-VER-0003", VERIFICATION)
    });
    // HW-VER-0003 is copied without the criterion it proves.
    place(&root, "HW-VER-0003", |text| text);

    let ran = root.run(&["observe", VERIFICATION, "HW-VER-0003", "--commit", COMMIT]);
    assert_eq!(ran.code, Some(1), "{ran:?}");
    assert!(
        ran.err.contains("HW-AC-0003") && ran.err.contains("HW-AC-0004"),
        "{ran:?}"
    );
    assert!(ran.err.contains("HW-VER-0003"), "{ran:?}");
    assert!(
        !root.at.join(SNAPSHOT).exists(),
        "nothing is written: {ran:?}"
    );
}
