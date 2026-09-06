// SPDX-License-Identifier: Apache-2.0
//! The states a kind admits, and the corpus that separates a document standing
//! at one from a document standing outside them.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#testing-a-check-without-a-failing-fixture-does-not-ship)
//! sets the floor: "every check ships with at least one fixture that it fails
//! and one that it passes." The tree under `fixtures/admitted-state/` is
//! written to be wrong under the two mistakes this rule is easiest to make.
//!
//! The first is a rule that reads the value and not the kind.
//! `records/authored-wrong.md` and `receipts/discharged.md` carry the same
//! front matter, down to the state and the day it was entered. Only the shelf
//! differs, and the shelf is what says which kind, and the kind is what binds
//! the regime. A rule that read `discharged` as wrong wherever it found it
//! reports both.
//!
//! The second is a rule that reads terminality. `records/admitted.md` stands at
//! `superseded`, which is terminal and which `narrow` names. A rule that
//! reported a terminal state wherever a document rested in one reports that
//! document too, and passes the first half of the first test while being
//! useless.
//!
//! No change manifest is bound anywhere below. That is the property, rather
//! than an omission: the defect is in the one version that exists.

use headwater_census::census;
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::{Cache, Context, Date, Declared, Outcome, Register, Run, Shape};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use std::path::{Path, PathBuf};

/// As every other recorded run: a verdict is a function of the injected clock.
const PINNED: &str = "2026-08-12";

const RULE: &str = headwater_check::lifecycle_state::RULE;

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn run() -> Run {
    let corpus = Corpus::new(fixtures_dir(), "admitted-state");
    let source = std::fs::read_to_string(fixtures_dir().join("admitted-state.taxonomy.yml"))
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
            lock: "sha256:admitted-state-fixture",
            taxonomy: &taxonomy,
            shape: &shape,
            relations: &declarations,
            config: &config,
            register: &register,
            adoption: None,
            source: "engine/crates/check/fixtures/admitted-state.taxonomy.yml",
        },
        &headwater_check::claim::Claims::empty(),
        &Context::at(Date::parse(PINNED).expect("the pinned date")),
        &mut Cache::disabled())
}

/// The findings this rule reported, as `(path, message)`.
fn refusals(run: &Run) -> Vec<(&str, &str)> {
    run.findings
        .iter()
        .filter(|finding| finding.rule == RULE)
        .map(|finding| (finding.path.as_str(), finding.message.as_str()))
        .collect()
}

/// The skip reasons this rule recorded, against the document each instance was
/// created over.
fn skips(run: &Run) -> Vec<(&str, &String)> {
    run.instances
        .iter()
        .filter(|instance| instance.rule == RULE)
        .filter_map(|instance| match &instance.outcome {
            Outcome::Skipped(why) => Some((over_document(instance), why)),
            _ => None,
        })
        .collect()
}

fn over_document(instance: &headwater_check::Instance) -> &str {
    instance
        .reads
        .first()
        .map(|input| input.path.as_str())
        .expect("a document instance reads the document it stands over")
}

/// A document authored at a state its kind does not admit is reported, in its
/// first and only version, and the report names the kind, the state and what
/// that kind does admit.
///
/// The decisive fixture, and the one no cheaper mechanism reaches. There is no
/// prior version to compare against and no movement to refuse, so a
/// transition rule sees nothing and a reachability rule over the declaration
/// never opens a document at all.
#[test]
fn a_document_authored_at_a_state_its_kind_does_not_admit_is_reported() {
    let run = run();
    let refusals = refusals(&run);
    assert_eq!(
        refusals.len(),
        1,
        "the value is not the defect and the kind is: {refusals:?}"
    );
    let (path, message) = refusals[0];
    assert_eq!(path, "admitted-state/records/authored-wrong.md");

    // The three things a reader needs, and none of them is derivable from the
    // other two.
    assert!(message.contains("`record`"), "the kind: {message}");
    assert!(message.contains("`discharged`"), "the state: {message}");
    assert!(message.contains("`narrow`"), "the regime: {message}");
    assert!(
        message.contains("`draft`")
            && message.contains("`current`")
            && message.contains("`superseded`"),
        "what the kind does admit: {message}"
    );
    // And it does not offer the state it is refusing as one of them.
    let offered = message.split("stands at one of").nth(1).expect("an offer");
    assert!(!offered.contains("discharged"), "{message}");
}

/// The same state, on the kind whose regime names it, is silent.
///
/// `receipts/discharged.md` is `records/authored-wrong.md` with one thing
/// changed, and the thing is not in the front matter. A rule that reported the
/// value would report this document, and a rule that reported nothing would
/// pass this test and fail the one above.
#[test]
fn the_same_state_on_a_kind_whose_regime_names_it_is_silent() {
    let run = run();
    let reported: Vec<&str> = refusals(&run).into_iter().map(|(path, _)| path).collect();
    assert!(
        !reported.contains(&"admitted-state/receipts/discharged.md"),
        "the rule read the value rather than the kind: {reported:?}"
    );
    assert_eq!(
        skips(&run)
            .iter()
            .filter(|(path, _)| { *path == "admitted-state/receipts/discharged.md" })
            .count(),
        0,
        "the receipt passed rather than declined to decide"
    );
}

/// A terminal state the kind's own regime names is silent.
///
/// The converse the first test cannot carry on its own. `superseded` is
/// terminal, `narrow` names it, and a document resting there is where the
/// taxonomy put it.
#[test]
fn a_terminal_state_the_regime_names_is_not_a_finding() {
    let run = run();
    let reported: Vec<&str> = refusals(&run).into_iter().map(|(path, _)| path).collect();
    assert!(
        !reported.contains(&"admitted-state/records/admitted.md"),
        "every terminal state was reported: {reported:?}"
    );
}

/// A kind that binds no lifecycle regime generates no instance at all.
///
/// `notes/exempt.md` stands at `discharged`, which is the value
/// `records/authored-wrong.md` is reported for. Its kind declares no machine,
/// so it admits no state and refuses none, and a rule that instantiated over it
/// would claim coverage of a document it could only ever pass.
#[test]
fn a_kind_that_binds_no_lifecycle_regime_generates_no_instance() {
    let run = run();
    let targets: Vec<&str> = run
        .instances
        .iter()
        .filter(|instance| instance.rule == RULE)
        .map(over_document)
        .collect();
    assert!(
        !targets.contains(&"admitted-state/notes/exempt.md"),
        "an instance exists over a kind with no machine: {targets:?}"
    );
    assert_eq!(
        targets.len(),
        5,
        "one instance per document of the three kinds that bind one: {targets:?}"
    );
}

/// A value the state vocabulary does not hold is skipped with a reason, and the
/// reason names the rule that owns it.
///
/// "This kind does not admit that state" and "that is not a state" are two
/// facts, and a report that collapsed them would send an author to add a state
/// to a regime when what they wrote was a typo.
#[test]
fn a_value_the_vocabulary_does_not_hold_is_left_to_the_rule_that_owns_it() {
    let run = run();
    let skips = skips(&run);
    let mine: Vec<&(&str, &String)> = skips
        .iter()
        .filter(|(path, _)| *path == "admitted-state/records/unknown.md")
        .collect();
    assert_eq!(mine.len(), 1, "{skips:?}");
    assert!(mine[0].1.contains("`retired`"), "{}", mine[0].1);
    assert!(
        mine[0].1.contains("facet.value.not_permitted"),
        "the skip does not name the rule that owns the value: {}",
        mine[0].1
    );
    // And `facet.value.not_permitted` did report it, so the deferral reaches a
    // rule that is looking.
    assert!(
        run.findings.iter().any(|finding| {
            finding.rule == "facet.value.not_permitted"
                && finding.path == "admitted-state/records/unknown.md"
        }),
        "the rule this one defers to reported nothing"
    );
}

/// A regime that names no state declines rather than reports.
///
/// The arm with a document under it. A regime that declares nothing is a defect
/// of the taxonomy that `taxonomy validate` reports once, and a check that
/// reported it would report it once per document of every kind that binds the
/// regime, with a remediation naming an empty set.
#[test]
fn a_regime_that_names_no_state_declines_rather_than_reporting_every_document() {
    let run = run();
    let reported: Vec<&str> = refusals(&run).into_iter().map(|(path, _)| path).collect();
    assert!(
        !reported.contains(&"admitted-state/hollows/nowhere.md"),
        "{reported:?}"
    );
    let skips = skips(&run);
    let mine: Vec<&(&str, &String)> = skips
        .iter()
        .filter(|(path, _)| *path == "admitted-state/hollows/nowhere.md")
        .collect();
    assert_eq!(mine.len(), 1, "a pass rather than a decline: {skips:?}");
    assert!(mine[0].1.contains("names no state"), "{}", mine[0].1);
    assert!(mine[0].1.contains("taxonomy validate"), "{}", mine[0].1);
}

/// Every instance of this rule reaches a verdict in a full-corpus run.
///
/// The reason the rule is document-scoped rather than change-scoped, stated as
/// a test. `lifecycle.transition.not_permitted` reports every one of its
/// instances as skipped `change-scoped-only` in a run like this one, because
/// nothing here constructs a change manifest
/// ([#178](https://github.com/headwater-ai/headwater/issues/178)). None of
/// these five is skipped for that reason.
#[test]
fn no_instance_of_this_rule_waits_for_a_change_manifest() {
    let run = run();
    assert!(run.change.is_none(), "a run with no change states one");
    for (path, why) in skips(&run) {
        assert!(
            !why.starts_with(headwater_check::scope::CHANGE_SCOPED_ONLY),
            "{path} waits for a manifest: {why}"
        );
    }
}
