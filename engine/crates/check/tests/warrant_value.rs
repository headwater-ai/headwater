// SPDX-License-Identifier: Apache-2.0
//! A warrant outside spec 3's closed set, and an acceptor the warrant does not
//! pair with.
//!
//! [Spec 3](../../../../docs/spec/03-authoring-and-lifecycle.md#the-warrant-and-what-each-value-requires)
//! states both halves in one paragraph: the warrant's "value set is closed",
//! and "each value requires a different part of the block, and the engine
//! checks the pairing". Before these two rules nothing read either half. A
//! document at `warrant: acepted` raised no finding, and a reader that tested
//! for `asserted` alone took the misspelling for a human acceptance.
//!
//! The tree under `fixtures/warrant-value/` holds one document for each case a
//! rule about it is easy to get wrong:
//!
//! - **A test for one bad value** passes `cased.md` (`Accepted`) and
//!   `listed.md` (a list), which are outside the set and are not `acepted`.
//! - **A rule that collapsed an absent warrant into a bad one** reports
//!   `quiet.md` and `blank.md`. Spec 3's "absent is a finding" is a separate
//!   rule that no edition of this engine has written yet, and
//!   [HW-OBL-0125](../../../../docs/obligations/0125-nine-documents-state-a-warrant-the-closed-set-does-not-hold-and-no-check-reads-one.md)
//!   holds it. These two instances skip and say why.
//! - **A rule that read the front matter of a generated file** reports
//!   `generated.md`, whose warrant the engine derives from the marker.
//! - **A pairing rule that read a non-member** reports `misspelled.md` twice,
//!   once for the value and once for an acceptor it cannot pair with anything.
//! - **A pairing rule that tested presence of the key** passes
//!   `accepted-blank.md`, which names nobody.

use headwater_census::census;
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::{Cache, Context, Date, Declared, Finding, Outcome, Register, Run, Shape};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use std::path::{Path, PathBuf};

const PINNED: &str = "2026-09-30";

/// The two rule names are the contract, so the tests spell them rather than
/// reading them from the crate that is under test.
const VALUE: &str = "warrant.value.not_permitted";
const UNPAIRED: &str = "warrant.acceptance.unpaired";
const BASIS: &str = "warrant.evidence.unsupported";

/// Spec 3's closed set, in the order of its table.
const CLOSED: [&str; 4] = ["accepted", "regenerated", "transcribed", "asserted"];

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn path(file: &str) -> String {
    format!("warrant-value/{file}")
}

fn run() -> Run {
    let corpus = Corpus::new(fixtures_dir(), "warrant-value");
    let source = std::fs::read_to_string(fixtures_dir().join("warrant-value.taxonomy.yml"))
        .expect("the fixture taxonomy");
    let root = headwater_yaml::load(&source)
        .expect("the fixture taxonomy loads")
        .value
        .as_map()
        .expect("a mapping")
        .clone();

    let taxonomy = Taxonomy::read(&root).expect("the taxonomy reads");
    let declarations = Declarations::read(&root).expect("the declarations read");
    let register = Register::read(&root).expect("the register reads");
    let shape = Shape::read(&root).expect("the shape reads");
    let taken = census::take(&corpus, &taxonomy);
    let config = Config::default();
    let graph = Graph::build(
        &taken,
        &declarations,
        &Resolvers::over(&corpus),
        &corpus,
        &config,
    );
    headwater_check::run(
        &taken,
        &graph,
        &Declared {
            lock: "sha256:warrant-value-fixture",
            taxonomy: &taxonomy,
            shape: &shape,
            relations: &declarations,
            config: &config,
            register: &register,
            observations: &headwater_check::Observations::empty(),
            pin: None,
            harvests: &[],
            imports: &[],
            adoption: None,
            source: "engine/crates/check/fixtures/warrant-value.taxonomy.yml",
        },
        &headwater_check::claim::Claims::empty(),
        &Context::at(Date::parse(PINNED).expect("the pinned date")),
        &mut Cache::disabled(),
    )
}

/// The findings of one rule at one document.
fn findings<'a>(run: &'a Run, rule: &str, file: &str) -> Vec<&'a Finding> {
    let path = path(file);
    run.findings
        .iter()
        .filter(|finding| finding.rule == rule && finding.path == path)
        .collect()
}

/// The outcome of the one instance of `rule` over `file`, which must exist.
fn outcome<'a>(run: &'a Run, rule: &str, file: &str) -> &'a Outcome {
    let path = path(file);
    let over: Vec<&Outcome> = run
        .instances
        .iter()
        .filter(|instance| instance.rule == rule)
        .filter(|instance| instance.reads.iter().any(|input| input.path == path))
        .map(|instance| &instance.outcome)
        .collect();
    assert_eq!(
        over.len(),
        1,
        "one instance of {rule} over {file}: {over:?}"
    );
    over[0]
}

/// The decisive case: a misspelled warrant is an error, and an evidenced claim
/// resting on it is not supported.
#[test]
fn a_misspelled_warrant_is_an_error_and_supports_no_evidenced_claim() {
    let run = run();
    let value = findings(&run, VALUE, "misspelled.md");
    assert_eq!(value.len(), 1, "the decisive finding: {:?}", run.findings);
    assert_eq!(value[0].severity, headwater_check::Severity::Error);
    // At the value, which is the text to change: line 7 of the fixture.
    assert_eq!(value[0].line, 7, "{:?}", value[0]);

    let basis = findings(&run, BASIS, "claim-onto-misspelled.md");
    assert_eq!(basis.len(), 1, "the pointer onto it: {:?}", run.findings);
    assert!(
        basis[0].message.contains("`acepted`"),
        "the value as written: {}",
        basis[0].message
    );
}

/// The message names the value as written and all four members, and no finding
/// carries a patch: a typo and an intent are the author's to tell apart.
#[test]
fn the_message_names_the_value_and_the_closed_set_and_offers_no_patch() {
    let run = run();
    let finding = findings(&run, VALUE, "misspelled.md")[0];
    assert!(finding.message.contains("`acepted`"), "{}", finding.message);
    for member in CLOSED {
        assert!(
            finding.message.contains(member),
            "{member}: {}",
            finding.message
        );
    }
    assert!(run
        .findings
        .iter()
        .filter(|finding| finding.rule == VALUE || finding.rule == UNPAIRED)
        .all(|finding| finding.patch.is_none()));
}

/// Every value outside the set, not the one misspelling the report quoted.
#[test]
fn every_value_outside_the_set_is_reported() {
    let run = run();
    for file in ["misspelled.md", "proposed.md", "cased.md", "listed.md"] {
        let reported = findings(&run, VALUE, file);
        assert_eq!(reported.len(), 1, "{file}: {:?}", run.findings);
        assert_eq!(reported[0].severity, headwater_check::Severity::Error);
    }
}

/// A list is named as what it is, once: "the warrant is a sequence".
#[test]
fn a_warrant_that_is_not_one_value_is_named_by_its_shape() {
    let run = run();
    let finding = findings(&run, VALUE, "listed.md")[0];
    assert!(
        finding.message.starts_with("the warrant is a sequence,"),
        "{}",
        finding.message
    );
    assert!(!finding.message.contains("a a "), "{}", finding.message);
}

/// A pairing finding stands at the text to change: the acceptor where one is
/// written, and the warrant where none is.
#[test]
fn a_pairing_finding_stands_at_the_line_to_change() {
    let run = run();
    for (file, line) in [
        // `accepted_by:` with no value, on line 10.
        ("accepted-blank.md", 10),
        // No acceptor at all, so the warrant on line 7.
        ("accepted-unsigned.md", 7),
        // A forbidden acceptor, on line 10.
        ("asserted-signed.md", 10),
    ] {
        let reported = findings(&run, UNPAIRED, file);
        assert_eq!(reported.len(), 1, "{file}");
        assert_eq!(reported[0].line, line, "{file}: {:?}", reported[0]);
    }
}

/// Each of the four values passes the value rule.
#[test]
fn each_member_of_the_closed_set_passes() {
    let run = run();
    for file in [
        "accepted.md",
        "asserted.md",
        "regenerated.md",
        "transcribed.md",
    ] {
        assert!(
            matches!(outcome(&run, VALUE, file), Outcome::Passed),
            "{file}: {:?}",
            outcome(&run, VALUE, file)
        );
    }
}

/// An absent warrant is not a bad one. It skips, and says why, because the rule
/// that reports an absence is not written yet and this one is not it.
#[test]
fn an_absent_warrant_skips_with_a_reason() {
    let run = run();
    for file in ["quiet.md", "blank.md"] {
        assert!(findings(&run, VALUE, file).is_empty(), "{file}");
        match outcome(&run, VALUE, file) {
            Outcome::Skipped(why) => assert!(why.contains("no warrant"), "{file}: {why}"),
            other => panic!("{file} did not skip: {other:?}"),
        }
    }
}

/// A generated document's warrant is derived from the marker, so a stray
/// declaration in its front matter is read by neither rule.
#[test]
fn a_generated_document_is_not_judged_on_its_declaration() {
    let run = run();
    assert!(findings(&run, VALUE, "generated.md").is_empty());
    assert!(findings(&run, UNPAIRED, "generated.md").is_empty());
}

/// Spec 3's table, row by row: `accepted` requires an acceptor, and the other
/// three forbid one.
#[test]
fn the_pairing_follows_spec_three_row_by_row() {
    let run = run();
    for file in [
        "accepted-unsigned.md",
        "accepted-blank.md",
        // `accepted_by: []` lists nobody.
        "accepted-empty-list.md",
        "asserted-signed.md",
        "regenerated-signed.md",
        "transcribed-signed.md",
    ] {
        let reported = findings(&run, UNPAIRED, file);
        assert_eq!(reported.len(), 1, "{file}: {:?}", run.findings);
        assert_eq!(reported[0].severity, headwater_check::Severity::Error);
        assert!(
            reported[0].message.contains("accepted_by"),
            "{file}: {}",
            reported[0].message
        );
    }
    for file in [
        "accepted.md",
        // `accepted_by: [a.person]` names a person, written as a list.
        "accepted-listed.md",
        "asserted.md",
        "regenerated.md",
        "transcribed.md",
    ] {
        assert!(
            matches!(outcome(&run, UNPAIRED, file), Outcome::Passed),
            "{file}: {:?}",
            outcome(&run, UNPAIRED, file)
        );
    }
}

/// A value outside the set is the value rule's to report, and it gets no
/// second finding for an acceptor that pairs with nothing.
#[test]
fn a_value_outside_the_set_gets_no_pairing_finding() {
    let run = run();
    for file in [
        "misspelled.md",
        "proposed.md",
        "cased.md",
        "listed.md",
        "quiet.md",
    ] {
        assert!(
            findings(&run, UNPAIRED, file).is_empty(),
            "{file}: {:?}",
            findings(&run, UNPAIRED, file)
        );
    }
}
