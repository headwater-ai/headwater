// SPDX-License-Identifier: Apache-2.0
//! A document that stands at the initial state of its regime and says a person
//! accepted it (#1832).
//!
//! [HW-DR-0052](../../../../docs/decisions/0052-a-document-is-proposed-at-the-state-it-will-hold-and-the-merge-activates-it.md)
//! rules that an author writes the state the document will hold once it
//! merges, and that the initial state says the document is not proposed as
//! finished. An `accepted` warrant says a person read it as finished. A
//! document that says both contradicts itself, and before this rule nothing
//! read the two facets together.
//!
//! The tree under `fixtures/accepted-at-initial/` and its taxonomy are built so
//! that each wrong reading of "initial" reports the wrong documents:
//!
//! - **A rule that hard-coded `status`** reads nothing: the facet is `stage`.
//! - **A rule that read `role: initial`, or the literal `draft`,** reports
//!   `records/drafted-accepted.md` and passes `records/opened-accepted.md`.
//!   The regime `intake` opens at `proposed`, which carries no role.
//! - **A rule that read the warrant alone** reports `live-accepted.md`.
//! - **A rule that took an absent or misspelled warrant as accepted** reports
//!   `opened-quiet.md` or `opened-misspelled.md`, which belong to
//!   `warrant.value.not_permitted` and HW-OBL-0125.

use headwater_census::census;
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::{Cache, Context, Date, Declared, Finding, Outcome, Register, Run, Shape};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use std::path::{Path, PathBuf};

const PINNED: &str = "2026-10-23";

/// The rule name is the contract, so the tests spell it rather than reading it
/// from the crate under test.
const RULE: &str = "warrant.acceptance.at_initial";

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn path(file: &str) -> String {
    format!("accepted-at-initial/{file}")
}

fn run() -> Run {
    let corpus = Corpus::new(fixtures_dir(), "accepted-at-initial");
    let source = std::fs::read_to_string(fixtures_dir().join("accepted-at-initial.taxonomy.yml"))
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
            lock: "sha256:accepted-at-initial-fixture",
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
            source: "engine/crates/check/fixtures/accepted-at-initial.taxonomy.yml",
        },
        &headwater_check::claim::Claims::empty(),
        &Context::at(Date::parse(PINNED).expect("the pinned date")),
        &mut Cache::disabled(),
    )
}

/// The findings of the rule at one document.
fn findings<'a>(run: &'a Run, file: &str) -> Vec<&'a Finding> {
    let path = path(file);
    run.findings
        .iter()
        .filter(|finding| finding.rule == RULE && finding.path == path)
        .collect()
}

/// The outcomes of the rule's instances over `file`.
fn outcomes<'a>(run: &'a Run, file: &str) -> Vec<&'a Outcome> {
    let path = path(file);
    run.instances
        .iter()
        .filter(|instance| instance.rule == RULE)
        .filter(|instance| instance.reads.iter().any(|input| input.path == path))
        .map(|instance| &instance.outcome)
        .collect()
}

/// The one outcome of the rule over `file`, which must exist.
fn outcome<'a>(run: &'a Run, file: &str) -> &'a Outcome {
    let over = outcomes(run, file);
    assert_eq!(
        over.len(),
        1,
        "one instance of {RULE} over {file}: {over:?}"
    );
    over[0]
}

/// The decisive case: a document at the regime's initial state with an
/// `accepted` warrant and a named acceptor gets one warning, and the warning
/// names both facets and both values.
#[test]
fn an_accepted_document_at_the_initial_state_of_its_regime_is_one_warning() {
    let run = run();
    let found = findings(&run, "records/opened-accepted.md");
    assert_eq!(found.len(), 1, "one finding: {found:?}");
    let finding = found[0];
    assert_eq!(finding.severity, headwater_check::Severity::Warn);
    assert!(
        finding.patch.is_none(),
        "only the author can choose a direction"
    );
    for part in [
        "`stage`",
        "`proposed`",
        "`intake`",
        "`provenance.warrant`",
        "`accepted`",
    ] {
        assert!(
            finding.message.contains(part),
            "the message names {part}: {}",
            finding.message
        );
    }
    // The remediation names both directions.
    for part in ["`asserted`", "accepted_by", "`stage`"] {
        assert!(
            finding.remediation.contains(part),
            "the remediation names {part}: {}",
            finding.remediation
        );
    }
    // The plain case: a regime that opens at `draft`.
    let memo = findings(&run, "memos/drafted-accepted.md");
    assert_eq!(memo.len(), 1, "one finding on the memo: {memo:?}");
    assert!(memo[0].message.contains("`draft`"), "{}", memo[0].message);
}

/// The same document at the live state, and at the initial state with an
/// `asserted` warrant, both pass.
#[test]
fn the_live_state_and_an_asserted_warrant_pass() {
    let run = run();
    for file in ["records/live-accepted.md", "records/opened-asserted.md"] {
        assert!(matches!(outcome(&run, file), Outcome::Passed), "{file}");
        assert!(findings(&run, file).is_empty(), "{file}");
    }
}

/// The regime decides what is initial, and the role of a value does not.
/// `draft` has `role: initial` and `intake` opens at `proposed`, so a record
/// at `draft` passes.
#[test]
fn the_regime_decides_the_initial_state_and_the_role_does_not() {
    let run = run();
    assert!(matches!(
        outcome(&run, "records/drafted-accepted.md"),
        Outcome::Passed
    ));
}

/// What this rule leaves to another rule skips, with a reason.
#[test]
fn an_absent_or_outside_warrant_and_an_unread_state_skip() {
    let run = run();
    for file in [
        "records/opened-quiet.md",
        "records/opened-misspelled.md",
        "records/unnamed-accepted.md",
        "records/odd-accepted.md",
        "records/stateless-accepted.md",
    ] {
        assert!(
            matches!(outcome(&run, file), Outcome::Skipped(_)),
            "{file} skips: {:?}",
            outcome(&run, file)
        );
    }
}

/// A kind that binds no regime has no initial state, and a generated file
/// declares no warrant, so neither gets an instance.
#[test]
fn a_kind_with_no_regime_and_a_generated_file_get_no_instance() {
    let run = run();
    for file in ["notes/drafted-accepted.md", "records/generated.md"] {
        assert!(outcomes(&run, file).is_empty(), "{file}");
    }
}
