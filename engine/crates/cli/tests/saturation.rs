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
//! The root also copies the doctrine pages, whose prose writes patterns of its
//! own, so each case reads the `future_intent` count of the root before it
//! adds its sentence and holds the count after it to one more.

mod common;
use common::Root;

/// The fixture decision, which the root copies from
/// `engine/crates/cli/fixtures/change/docs`.
const DECISION: &str = "docs/decisions/0001-the-warrant-a-person-set.md";

/// The sentence the case appends to the decision. It writes one
/// `future_intent` pattern, `at a later date`, and no other forbidden pattern
/// and no retired term. The copied doctrine pages do not write that pattern,
/// which the count before the sentence shows.
const FUTURE: &str = "The record names its reader at a later date.";

/// The two terms the case retires. No copied page writes either.
const TERMS: &str = "    retired_terms:\n      - term: lighthouse\n        reason: a test retires it\n      - term: windmill\n        reason: a test retires it\n";

/// Replace the house list of retired terms with [`TERMS`] and resolve the lock
/// again.
fn retire_two(root: &Root) {
    let overlay = root.at.join(".headwater/overlay.yml");
    let text = std::fs::read_to_string(&overlay).expect("the overlay reads");
    let from = text
        .find("    retired_terms:\n")
        .expect("the overlay lists retired terms");
    let to = text[from..]
        .find("    # The front-door prose")
        .map(|at| from + at)
        .expect("the house list ends before the outside-root comment");
    let rewritten = format!("{}{TERMS}{}", &text[..from], &text[to..]);
    std::fs::write(&overlay, rewritten).expect("the overlay writes");
    let resolved = root.run(&["taxonomy", "resolve"]);
    assert_eq!(
        resolved.code,
        Some(0),
        "the lock resolves again: {resolved:?}"
    );
}

/// Write [`FUTURE`] into the decision, followed by `directive` when one is
/// given.
fn write_future(root: &Root, directive: Option<&str>) {
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
}

/// The report with every run of white space read as one space. The text
/// report is laid out at the width of the run, so one line of the projection
/// can wrap onto two.
fn flat(report: &str) -> String {
    report.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The count the report prints for one set: the entries that produced a
/// finding and the entries the set declares. `name` is the rule and the set,
/// as in `voice.forbidden_construction future_intent`.
fn count(report: &str, name: &str) -> (usize, usize) {
    let flat = flat(report);
    let prefix = format!("pattern set {name}: ");
    let at = flat
        .find(&prefix)
        .unwrap_or_else(|| panic!("the report prints the set `{name}`:\n{report}"));
    let mut words = flat[at + prefix.len()..].split(' ');
    let matched = words.next().and_then(|word| word.parse().ok());
    let of = words.next();
    let declared = words.next().and_then(|word| word.parse().ok());
    match (matched, of, declared) {
        (Some(matched), Some("of"), Some(declared)) => (matched, declared),
        _ => panic!("the set `{name}` prints `<n> of <m>`:\n{report}"),
    }
}

/// The warning a set where no entry produced a finding carries.
const ZERO: &str = "produced a finding, so a zero here does not show a clean corpus";

/// Each set prints its count, and a set where no entry produced a finding is
/// named as a zero that shows no clean corpus.
#[test]
fn each_pattern_set_prints_the_entries_that_produced_a_finding() {
    let root = Root::new("saturation-counts");
    retire_two(&root);
    let before = root.run(&["check"]);
    let (base, declared) = count(&before.out, "voice.forbidden_construction future_intent");
    assert_eq!(declared, 14, "future_intent declares 14 patterns");

    write_future(&root, None);
    let report = root.run(&["check"]);
    assert!(
        flat(&report.out).contains("writes `at a later date`"),
        "the fixture sentence produces its finding:\n{}",
        report.out
    );
    assert_eq!(
        count(&report.out, "voice.forbidden_construction future_intent"),
        (base + 1, 14),
        "the sentence adds one pattern to future_intent:\n{}",
        report.out
    );
    assert_eq!(
        count(&report.out, "voice.forbidden_construction phased_rollout"),
        (0, 13)
    );
    assert_eq!(
        count(&report.out, "language.retired_term.used ste_house"),
        (0, 2)
    );
    for name in [
        "voice.forbidden_construction phased_rollout: 0 of 13 patterns",
        "language.retired_term.used ste_house: 0 of 2 terms",
    ] {
        assert!(
            flat(&report.out).contains(&format!("pattern set {name} {ZERO}")),
            "the report names `{name}` as a zero that shows no clean corpus:\n{}",
            report.out
        );
    }
}

/// A finding a directive suppresses still shows that its pattern is live, so
/// the count reads the findings before the filters remove it.
#[test]
fn a_suppressed_finding_still_counts_its_pattern() {
    let root = Root::new("saturation-suppressed");
    retire_two(&root);
    let before = root.run(&["check"]);
    let (base, _) = count(&before.out, "voice.forbidden_construction future_intent");

    write_future(
        &root,
        Some(
            "<!-- headwater allow=voice.forbidden_construction scope=block until=2099-12-31 reason=accepted_deviation note=the case suppresses its own finding -->",
        ),
    );
    let report = root.run(&["check"]);
    assert!(
        flat(&report.out).contains("0 migration-pending, 1 suppressed"),
        "the directive suppresses the fixture finding:\n{}",
        report.out
    );
    assert_eq!(
        count(&report.out, "voice.forbidden_construction future_intent"),
        (base + 1, 14),
        "the suppressed finding still counts its pattern:\n{}",
        report.out
    );
}

/// A set is a set that a binding reads: a category no bound regime forbids
/// gets no line, and a regime that only paths outside the corpus root bind
/// gets one.
///
/// The case narrows the package's `declarative` regime to two categories, and
/// the `requirement_prospective` regime of a copied bundle to one, with an
/// overlay `override`. So no bound regime forbids `change_narration`. It adds a language regime that retires one term and
/// binds `NOTICE.md` alone, through `outside_root`, and no kind.
#[test]
fn a_set_is_counted_only_where_a_binding_reads_it() {
    let root = Root::new("saturation-bindings");
    let overlay = root.at.join(".headwater/overlay.yml");
    let text = std::fs::read_to_string(&overlay).expect("the overlay reads");
    assert!(text.contains("\nadd:\n"), "the overlay has an add block");
    assert!(
        text.contains("\noverride:\n"),
        "the overlay has an override block"
    );
    let front_door = "\nadd:\n  regimes.language.front_door:\n    tag: en-US\n    controlled: none\n    retired_terms:\n      - term: lantern\n        reason: a test retires it\n    outside_root:\n      - NOTICE.md\n";
    let narrowed = "\noverride:\n  regimes.voice.declarative.forbid: [future_intent, phased_rollout]\n  regimes.voice.requirement_prospective.forbid: [phased_rollout]\n";
    let text = text
        .replacen("\nadd:\n", front_door, 1)
        .replacen("\noverride:\n", narrowed, 1);
    std::fs::write(&overlay, text).expect("the overlay writes");
    std::fs::write(
        root.at.join("NOTICE.md"),
        "# A front door\n\nThis page names no term.\n",
    )
    .expect("the front door writes");
    let resolved = root.run(&["taxonomy", "resolve"]);
    assert_eq!(
        resolved.code,
        Some(0),
        "the lock resolves again: {resolved:?}"
    );

    let report = root.run(&["check"]);
    assert!(
        !flat(&report.out).contains("pattern set voice.forbidden_construction change_narration"),
        "no bound regime forbids change_narration, so it has no line:\n{}",
        report.out
    );
    assert_eq!(
        count(&report.out, "voice.forbidden_construction phased_rollout"),
        (0, 13)
    );
    assert_eq!(
        count(&report.out, "language.retired_term.used front_door"),
        (0, 1),
        "a regime that binds only a path outside the root has its line:\n{}",
        report.out
    );
}
