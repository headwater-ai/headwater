// SPDX-License-Identifier: Apache-2.0
//! The seventeen interface contracts that hand-state `--no-color`'s meaning,
//! held against the one sentence `headwater-help.md` and `NO_COLOR_TEXT`
//! already carry.
//!
//! # What #476 left true of only one document, and #475 owed to the rest
//!
//! [HW-DR-0045](../../../../docs/decisions/0045-coloring-the-cli-and-where-the-banner-goes.md)
//! rewrites `NO_COLOR_TEXT` (`engine/crates/cli/src/lib.rs`) because the
//! sentence it stated before the ruling — that no run of this binary ever
//! writes color — became false the moment any run does. [#476](https://github.com/headwater-ai/headwater/pull/476)
//! landed that rewrite in the source and in `docs/interfaces/headwater-help.md`
//! alone. Seventeen more documents under `docs/interfaces/` still hand-state
//! the old sentence in their own Options table, each a hand-kept copy of a
//! fact the binary itself now decides. This file is the second direction
//! [#257](https://github.com/headwater-ai/headwater/issues/257) is about: a
//! table that fell out of step with what it describes.
//!
//! # Why the expected sentence is read rather than written twice
//!
//! [`crates/cli/tests/help.rs`](help.rs) states the reason its own cases read
//! `headwater_verbs::VERBS` at run time rather than pinning a string: "a
//! second copy of the verb list is what #257 was filed about." The sentence
//! this file holds every document to is read out of `headwater-help.md`
//! itself for the same reason — the alternative is a second hand-kept copy of
//! prose that could drift from the file it is supposed to match.
//!
//! # Why this is a file of its own rather than a case in `interface_contract.rs`
//!
//! `interface_contract.rs` holds the *kind* `interface_contract` declares —
//! the eight required headings, the `governs` edge, the rule against its
//! negation — against a taxonomy this repository resolves. Every case there
//! runs over a scratch corpus this file writes for the purpose. What is under
//! test here is different: it is the *content* of eighteen real documents
//! already on the shelf, and it reads them from `docs/interfaces/` rather than
//! writing anything. Mixing the two would make one file answer two different
//! questions about two different trees.

use std::path::{Path, PathBuf};

/// The eighteen documents `--no-color`'s old sentence used to reach, in the
/// order `docs/interfaces/README.md` lists the verbs. `headwater-help.md` is
/// excluded: it carries the sentence already, and it is the one this file
/// reads the expectation out of rather than holds to it a second time.
const REMAINING: [&str; 17] = [
    "headwater-capture",
    "headwater-check",
    "headwater-completions",
    "headwater-conformance",
    "headwater-explain",
    "headwater-export",
    "headwater-gate",
    "headwater-generate",
    "headwater-import",
    "headwater-infer",
    "headwater-init",
    "headwater-mcp",
    "headwater-new",
    "headwater-probe",
    "headwater-query",
    "headwater-route",
    "headwater-taxonomy",
];

/// Two of the seventeen, `headwater-probe` and `headwater-query`, name the
/// global flags in one prose sentence rather than in an Options table row —
/// see each document's own Options section. A row-shaped assertion over them
/// would fail on a document that was never wrong about the flag, so this file
/// holds them to a narrower bar: that `--no-banner` now sits beside the
/// global flags they already named, and that the paragraph never claims
/// `--no-color` changes no byte.
const PROSE_ONLY: [&str; 2] = ["headwater-probe", "headwater-query"];

fn interfaces_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../docs/interfaces")
}

fn read(slug: &str) -> String {
    let path = interfaces_dir().join(format!("{slug}.md"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// The `--no-color` row of an Options table, as the two cells between its
/// pipes. `None` for a document that states the flag in prose instead — see
/// [`PROSE_ONLY`].
fn no_color_row(text: &str) -> Option<&str> {
    text.lines()
        .find_map(|line| line.strip_prefix("| `--no-color` | "))
        .map(|rest| rest.trim_end_matches(" |"))
}

/// The sentence every rewritten row restates, read out of
/// `headwater-help.md` rather than written a second time here — see the
/// module comment.
fn canonical_sentence() -> String {
    let help = read("headwater-help");
    no_color_row(&help)
        .expect("headwater-help.md carries the --no-color row this file reads the sentence from")
        .to_string()
}

/// Every rewritten Options table restates the sentence `headwater-help.md`
/// carries, word for word, whatever else its own row goes on to say about the
/// verb in front of it.
#[test]
fn every_rewritten_options_table_states_the_sentence_headwater_help_states() {
    let canonical = canonical_sentence();
    let mut wrong = Vec::new();
    for slug in REMAINING.iter().filter(|slug| !PROSE_ONLY.contains(slug)) {
        let text = read(slug);
        match no_color_row(&text) {
            Some(row) if row.contains(&canonical) => {}
            Some(row) => wrong.push(format!("{slug}.md: {row}")),
            None => wrong.push(format!(
                "{slug}.md: no `--no-color` row in its Options table"
            )),
        }
    }
    assert!(
        wrong.is_empty(),
        "these documents do not restate the sentence headwater-help.md carries:\n{}",
        wrong.join("\n")
    );
}

/// The sentence `--no-color`'s row used to carry, refused everywhere: this is
/// the reading the case above catches only when a row is present at all, and
/// this reads the whole document, catching it in a prose sentence too.
#[test]
fn no_document_still_claims_no_run_of_this_binary_ever_writes_color() {
    let retired = [
        "changes no byte",
        "Confirms color-free output",
        "Confirm the binary's color-free output",
        "Disable color output",
        "color-free output",
    ];
    let mut offenders = Vec::new();
    for slug in REMAINING {
        let text = read(slug);
        for phrase in retired {
            if text.contains(phrase) {
                offenders.push(format!("{slug}.md: {phrase:?}"));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "these documents still state the sentence this decision made false:\n{}",
        offenders.join("\n")
    );
}

/// Every one of the seventeen gains a `--no-banner` mention: a row beside
/// `--no-color`'s for the fifteen documents that carry an Options table row,
/// and a place in the prose list of global flags for the two that do not.
#[test]
fn every_document_gains_a_no_banner_mention() {
    let mut missing = Vec::new();
    for slug in REMAINING {
        let text = read(slug);
        let carries = match PROSE_ONLY.contains(&slug) {
            true => text.contains("--no-banner"),
            false => text
                .lines()
                .any(|line| line.starts_with("| `--no-banner` |")),
        };
        if !carries {
            missing.push(slug);
        }
    }
    assert!(
        missing.is_empty(),
        "these documents name no `--no-banner`: {missing:?}"
    );
}

/// The two prose-only documents never claim `--no-color` changes no byte —
/// narrower than the row case above because neither one carries a row to
/// begin with, and this is the same refusal in the shape their own Options
/// section takes.
#[test]
fn the_two_prose_only_documents_carry_no_banner_beside_no_color() {
    for slug in PROSE_ONLY {
        let text = read(slug);
        assert!(
            text.contains("`--no-color`") && text.contains("`--no-banner`"),
            "{slug}.md names --no-color without --no-banner beside it"
        );
    }
}
