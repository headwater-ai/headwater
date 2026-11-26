// SPDX-License-Identifier: Apache-2.0
//! `headwater gate` reads back a read set that lists a file name with a space
//! or a control character in it, and compares the file itself (#2070).
//!
//! The artifact wrote each input as `input <path> <digest>` and the gate split
//! the line at every space, so a path with a space refused the whole artifact
//! as "does not read as a read set". A path with a control character was
//! written as backslash text and read back as that text, so the gate hashed a
//! file that is not in the tree and voided the verdict.

#![cfg(unix)]

mod common;
use common::Root;

/// A decision file name that holds a space.
const SPACED: &str = "docs/decisions/0002-a spaced name.md";

/// A decision file name that holds a bell character.
const BELL: &str = "docs/decisions/0003-a\u{7}bell.md";

/// One decision, `HW-DR-<number>`, that a check reads like any other.
fn decision(number: &str) -> String {
    format!(
        "---\nid: HW-DR-{number}\ntitle: A decision with an awkward file name\nstatus: \
         current\nstatus_since: 2026-08-01\nlast_verified: 2026-08-01\nsummary: One \
         decision whose file name holds a character a read set must spell.\nprovenance:\n  \
         warrant: asserted\n  agency: human\n  evidence_basis: unevidenced\n---\n\n# A \
         decision with an awkward file name\n\n## Context\n\nA fixture.\n\n## Decision\n\nIt \
         is read.\n\n## Consequences\n\nThe gate reads it back.\n"
    )
}

fn root(label: &str) -> Root {
    Root::shaped(label, |at| {
        std::fs::write(at.join(SPACED), decision("0002")).expect("the spaced decision writes");
        std::fs::write(at.join(BELL), decision("0003")).expect("the bell decision writes");
    })
}

/// The read set lists both files, a gate over the unchanged tree reads it and
/// names neither, and one edited byte of the spaced file is named as moved.
#[test]
fn a_gate_reads_back_a_path_with_a_space_or_a_control_character() {
    let root = root("gate-paths");
    let read_set = root.at.join("clean.readset");
    let read_set = read_set.to_str().expect("the read set path is UTF-8");
    let ran = root.run(&["check", "--read-set", read_set]);
    assert!(ran.code.is_some(), "{ran:?}");
    let recorded = std::fs::read_to_string(read_set).expect("the read set reads");
    assert!(recorded.contains(SPACED), "the read set lists the spaced file: {recorded}");
    assert!(
        recorded.contains("0003-a\\x07bell.md"),
        "the read set spells the bell: {recorded}"
    );

    let unmoved = root.run(&["gate", "--read-set", read_set]);
    // The repository's barrier rules never carry, so the gate exits 1 on any
    // tree; what it must not do is read a listed file as gone or moved.
    assert!(
        !unmoved.out.contains("is not in this tree") && !unmoved.out.contains(" reads sha256:"),
        "the gate finds every listed file as it was: {unmoved:?}"
    );
    assert!(
        !unmoved.err.contains("does not read as a read set"),
        "the gate reads the artifact: {unmoved:?}"
    );
    for listed in [SPACED, "0003-a\\x07bell.md"] {
        assert!(
            !unmoved.out.contains(listed),
            "an unchanged file is not named: {unmoved:?}"
        );
    }

    let edited = std::fs::read_to_string(root.at.join(SPACED)).expect("the decision reads");
    std::fs::write(root.at.join(SPACED), edited.replace("It is read.", "It is read again."))
        .expect("the decision writes");
    let moved = root.run(&["gate", "--read-set", read_set]);
    assert_eq!(moved.code, Some(1), "{moved:?}");
    assert!(
        moved.out.contains(&format!("{SPACED} reads")),
        "the gate names the edited file as moved: {moved:?}"
    );
    assert!(!moved.out.contains("0003-a\\x07bell.md"), "{moved:?}");
}
