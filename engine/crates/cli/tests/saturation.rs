// SPDX-License-Identifier: Apache-2.0
//! The text report of `headwater check` counts, for each lexical pattern set,
//! how many of its entries produced a finding (HW-OBL-0168).
//!
//! Two rules read a curated pattern set: `voice.forbidden_construction` reads
//! one set per category, and `language.retired_term.used` reads one list of
//! terms per regime. A zero from either rule says nothing by itself: it is a
//! clean corpus, or a set whose entries match nothing that authors write. The
//! report separates the two by printing, for each set, the entries that
//! produced a finding against the entries the set declares, and by naming a
//! set where no entry did. `docs/interfaces/headwater-check.md` states the
//! contract, and these cases hold it.
//!
//! The fixture root copies this repository's overlay, so its decision kind
//! binds the package's `declarative` voice regime (`future_intent`,
//! `change_narration`, `phased_rollout`) and the `ste_house` language regime.
//! The case replaces the house list of retired terms with two terms, so the
//! declared count of that list is a fact of this file and not of the overlay.

mod common;
use common::Root;

/// The fixture decision, which the root copies from
/// `engine/crates/cli/fixtures/change/docs`.
const DECISION: &str = "docs/decisions/0001-the-warrant-a-person-set.md";

/// The paragraph the case appends to the decision. It writes one
/// `future_intent` pattern, `will eventually`, and no other forbidden pattern
/// and no retired term.
const FUTURE: &str = "The record will eventually name its reader.";

/// Replace the house list of retired terms with two terms, write the one
/// sentence of `future_intent` into the decision, followed by `directive`
/// when one is given, and resolve the lock again.
fn prepare(root: &Root, directive: Option<&str>) {
    let overlay = root.at.join(".headwater/overlay.yml");
    let text = std::fs::read_to_string(&overlay).expect("the overlay reads");
    let from = text
        .find("    retired_terms:\n")
        .expect("the overlay lists retired terms");
    let to = text[from..]
        .find("    # The front-door prose")
        .map(|at| from + at)
        .expect("the house list ends before the outside-root comment");
    let two = "    retired_terms:\n      - term: lighthouse\n        reason: a test retires it\n      - term: windmill\n        reason: a test retires it\n";
    let rewritten = format!("{}{two}{}", &text[..from], &text[to..]);
    std::fs::write(&overlay, rewritten).expect("the overlay writes");

    let decision = root.at.join(DECISION);
    let mut text = std::fs::read_to_string(&decision).expect("the decision reads");
    text.push('\n');
    text.push_str(FUTURE);
    if let Some(directive) = directive {
        text.push(' ');
        text.push_str(directive);
    }
    text.push('\n');
    std::fs::write(&decision, text).expect("the decision writes");

    let resolved = root.run(&["taxonomy", "resolve"]);
    assert_eq!(resolved.code, Some(0), "the lock resolves again: {resolved:?}");
}

/// The line the report prints for one voice category.
fn voice_line(category: &str, matched: usize, declared: usize) -> String {
    let mut line = format!(
        "pattern set voice.forbidden_construction {category}: {matched} of {declared} patterns produced a finding"
    );
    if matched == 0 {
        line.push_str(", so a zero here does not show a clean corpus");
    }
    line
}

/// The line the report prints for the house list of retired terms.
fn retired_line(matched: usize, declared: usize) -> String {
    let mut line = format!(
        "pattern set language.retired_term.used ste_house: {matched} of {declared} terms produced a finding"
    );
    if matched == 0 {
        line.push_str(", so a zero here does not show a clean corpus");
    }
    line
}

/// Each set prints its count, and a set where no entry produced a finding is
/// named as a zero that shows no clean corpus.
#[test]
fn each_pattern_set_prints_the_entries_that_produced_a_finding() {
    let root = Root::new("saturation-counts");
    prepare(&root, None);
    let report = root.run(&["check"]);
    assert!(
        report.out.contains("writes `will eventually`"),
        "the fixture sentence produces its finding:\n{}",
        report.out
    );
    for line in [
        voice_line("future_intent", 1, 14),
        voice_line("phased_rollout", 0, 13),
        retired_line(0, 2),
    ] {
        assert!(
            report.out.contains(&format!("{line}\n")),
            "the report prints `{line}`:\n{}",
            report.out
        );
    }
}

/// A finding a directive suppresses still shows that its pattern is live, so
/// the count reads the findings before the filters remove it.
#[test]
fn a_suppressed_finding_still_counts_its_pattern() {
    let root = Root::new("saturation-suppressed");
    prepare(
        &root,
        Some(
            "<!-- headwater allow=voice.forbidden_construction scope=block until=2099-12-31 reason=accepted_deviation note=the case suppresses its own finding -->",
        ),
    );
    let report = root.run(&["check"]);
    assert!(
        !report.out.contains("writes `will eventually`"),
        "the directive suppresses the fixture finding:\n{}",
        report.out
    );
    let line = voice_line("future_intent", 1, 14);
    assert!(
        report.out.contains(&format!("{line}\n")),
        "the suppressed finding still counts: `{line}`:\n{}",
        report.out
    );
}
