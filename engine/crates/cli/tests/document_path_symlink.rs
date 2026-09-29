// SPDX-License-Identifier: Apache-2.0
//! `headwater check` reports a relation entry that names a governed document
//! through a symlink, and names the identifier to write (#1417).
//!
//! A target is an identifier
//! ([Q4](../../../../docs/decisions/0004-relation-storage.md)). Since #1410 the
//! path of a typed document with an identifier is a finding under a relation
//! that admits a document. That comparison read the normalized path alone,
//! which is lexical, so a symlink onto the document bound as a `code_path`
//! anchor onto its file and passed `check --strict`. The link here sits under
//! `lib/`, a plain directory: under `docs/` the census reports a symlink as an
//! entry it cannot walk, and no regime of the overlay names `lib/`.

#![cfg(unix)]

mod common;
use common::Root;

/// The link, relative to the root.
const LINK: &str = "lib/decision.md";

/// One decision that traces to `target`.
fn decision(target: &str) -> String {
    format!(
        "---\nid: HW-DR-0002\ntitle: The decision that traces through a link\nstatus: \
         current\nstatus_since: 2026-08-01\nlast_verified: 2026-08-01\nsummary: One \
         decision that traces to another decision through a symlink.\nprovenance:\n  \
         warrant: asserted\n  agency: human\n  evidence_basis: unevidenced\nrelations:\n  \
         traces_to:\n    - {target}\n---\n\n# The decision that traces through a link\n\n## \
         Context\n\nA fixture.\n\n## Decision\n\nIt traces to a decision through a \
         symlink.\n\n## Consequences\n\nThe check reads it.\n"
    )
}

/// The stub root, with `lib/decision.md` a symlink onto the file of
/// `HW-DR-0001`, and `HW-DR-0002` tracing to the link.
fn root(label: &str) -> Root {
    Root::shaped(label, move |at| {
        std::fs::create_dir_all(at.join("lib")).expect("the directory is made");
        std::os::unix::fs::symlink(
            "../docs/decisions/0001-the-warrant-a-person-set.md",
            at.join(LINK),
        )
        .expect("the link is made");
        std::fs::write(
            at.join("docs/decisions/0002-the-decision-that-traces-through-a-link.md"),
            decision(LINK),
        )
        .expect("the decision writes");
    })
}

/// The link is the finding of the document's own path: one
/// `relation.target.unresolved` that names `HW-DR-0001`, with a remedy that
/// says to write it in place of the link, and no `code_path` edge.
#[test]
fn a_symlink_onto_a_document_is_reported_with_the_identifier_to_write() {
    let root = root("symlink-document");

    let checked = root.run(&["check", "--no-cache"]);
    // A finding opens with its rule and the obligation it reaches. The report
    // also lists each rule by name in its account of the checks, and those
    // lines are not findings.
    let unresolved: Vec<&str> = checked
        .out
        .lines()
        .filter(|line| {
            line.trim_start()
                .starts_with("relation.target.unresolved (")
        })
        .collect();
    assert_eq!(unresolved.len(), 1, "{}", checked.out);
    assert!(
        checked
            .out
            .contains(&format!("write `HW-DR-0001` in place of `{LINK}`")),
        "{}",
        checked.out
    );
    // Nothing else in the output names the link.
    let naming: Vec<&str> = checked
        .out
        .lines()
        .filter(|line| line.contains(LINK))
        .collect();
    assert!(
        naming.iter().all(|line| line.contains("HW-DR-0001")),
        "{naming:?}"
    );

    let explained = root.run(&["explain", "HW-DR-0002"]);
    assert_eq!(explained.code, Some(0), "{explained:?}");
    assert!(
        !explained.out.contains(&format!("code_path `{LINK}`")),
        "{}",
        explained.out
    );
}
