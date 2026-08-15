// SPDX-License-Identifier: Apache-2.0
//! A live document resting on a terminal one, and the four ways a rule about it
//! is easy to get wrong.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#testing-a-check-without-a-failing-fixture-does-not-ship)
//! sets the floor: "every check ships with at least one fixture that it fails
//! and one that it passes." The tree under `fixtures/terminal-dependency/` goes
//! past the floor, because the pass has to be the *same edge* as the fail.
//!
//! **A rule that reported every edge into a terminal document** reports
//! `live/succeeds-replaced.md`, whose target stands at exactly the state that
//! edge wrote onto it.
//!
//! **A rule that exempted the whole `supersedes` relation** stays silent on
//! `live/succeeds-retired.md`, where the same relation reaches a document that
//! some other history deprecated.
//!
//! **A rule holding a list of terminal state names** taken from the regime of
//! the *citing* document calls `live/rests-on-receipt.md` fine, because
//! `discharged` is a state that document's own kind never names.
//!
//! **A rule reading the declaring document as the source** gets
//! `records/successor-target.md` backwards, because that file wrote the only
//! half of its pair and it wrote the inverse one.
//!
//! No change manifest is bound anywhere below, and none is needed: both states
//! this rule reads survive in the two documents in front of it.

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

const RULE: &str = headwater_check::dependency::RULE;

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn run() -> Run {
    let corpus = Corpus::new(fixtures_dir(), "terminal-dependency");
    let source = std::fs::read_to_string(fixtures_dir().join("terminal-dependency.taxonomy.yml"))
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
            lock: "sha256:terminal-dependency-fixture",
            taxonomy: &taxonomy,
            shape: &shape,
            relations: &declarations,
            config: &config,
            register: &register,
            adoption: None,
            source: "engine/crates/check/fixtures/terminal-dependency.taxonomy.yml",
        },
        &Context::at(Date::parse(PINNED).expect("the pinned date")),
        &mut Cache::disabled(),
    )
}

/// The findings this rule reported, as `(path, message)`.
fn refusals(run: &Run) -> Vec<(&str, &str)> {
    run.findings
        .iter()
        .filter(|finding| finding.rule == RULE)
        .map(|finding| (finding.path.as_str(), finding.message.as_str()))
        .collect()
}

/// The one finding whose message names a document, and nothing when this rule
/// reported none about it.
fn about<'a>(run: &'a Run, id: &str) -> Option<&'a str> {
    refusals(run)
        .into_iter()
        .map(|(_, message)| message)
        .find(|message| message.contains(id))
}

/// The skip reasons this rule recorded, with the documents each instance read.
fn skips(run: &Run) -> Vec<(Vec<&str>, &String)> {
    run.instances
        .iter()
        .filter(|instance| instance.rule == RULE)
        .filter_map(|instance| match &instance.outcome {
            Outcome::Skipped(why) => Some((
                instance
                    .reads
                    .iter()
                    .map(|input| input.path.as_str())
                    .collect(),
                why,
            )),
            _ => None,
        })
        .collect()
}

/// Every document this rule created an instance over, path by path.
fn read_by_instances(run: &Run) -> Vec<&str> {
    run.instances
        .iter()
        .filter(|instance| instance.rule == RULE)
        .flat_map(|instance| instance.reads.iter().map(|input| input.path.as_str()))
        .collect()
}

/// The decisive case, and the report names all four things a reader needs.
///
/// `revises` is a succession relation, `core.requires` declares that family
/// lifecycle-sensitive, and `revises` writes no state onto anything. So the
/// state at the far end came from somewhere else, and a live document is
/// resting on it.
#[test]
fn a_live_document_resting_on_a_terminal_one_is_reported() {
    let run = run();
    let message = about(&run, "NOTE-FIX-rests-on-retired").expect("the decisive finding");
    assert!(
        message.contains("NOTE-FIX-retired"),
        "the target: {message}"
    );
    assert!(message.contains("`revises`"), "the relation: {message}");
    assert!(message.contains("`deprecated`"), "the state: {message}");
    assert!(
        message.contains("`current`"),
        "the source's state: {message}"
    );

    let reported: Vec<&str> = refusals(&run).into_iter().map(|(path, _)| path).collect();
    assert!(
        reported.contains(&"terminal-dependency/live/rests-on-retired.md"),
        "the finding anchors at the entry that declares the edge: {reported:?}"
    );
}

/// The same edge onto a document that still stands is silent.
///
/// `live/rests-on-standing.md` and `live/rests-on-retired.md` differ in the
/// identifier they name and in nothing else, and the documents at the two far
/// ends differ in one facet. A rule that reported the relation rather than the
/// pair reports both.
#[test]
fn the_same_edge_onto_a_live_document_is_silent() {
    let run = run();
    assert!(
        about(&run, "NOTE-FIX-rests-on-standing").is_none(),
        "{:?}",
        refusals(&run)
    );
    // And it passed rather than declining to decide, which is the difference
    // between a rule that looked and a rule that could not.
    assert!(
        !skips(&run)
            .iter()
            .any(|(reads, _)| reads.contains(&"terminal-dependency/records/standing.md")),
        "the live pair skipped: {:?}",
        skips(&run)
    );
}

/// A target whose own regime names `discharged` is terminal, and the citing
/// document's regime does not name it at all.
///
/// This is what separates reading the declaration from reading a list. `record`
/// binds `narrow`, which names four states and not this one. The terminality of
/// the state is a role on the value, the legitimacy of the document standing
/// there is the regime of the *target's* kind, and a rule that took either
/// question to the wrong end gets this pair wrong.
#[test]
fn a_receipt_at_a_state_only_its_own_regime_names_is_terminal() {
    let run = run();
    let message = about(&run, "NOTE-FIX-rests-on-receipt").expect("the receipt finding");
    assert!(message.contains("NOTE-FIX-paid"), "{message}");
    assert!(message.contains("`discharged`"), "{message}");
}

/// A relation whose family carries no flag reaches no instance at all.
///
/// `mentions` is `association`, and the core requirement that names that family
/// writes no `lifecycle_sensitive`. So the pair is not silent because the rule
/// decided something about it: the rule was never generated over it.
#[test]
fn a_relation_no_core_requirement_marks_generates_no_instance() {
    let run = run();
    assert!(
        about(&run, "NOTE-FIX-mentions-retired").is_none(),
        "an unmarked family reached the rule: {:?}",
        refusals(&run)
    );
    assert!(
        !read_by_instances(&run).contains(&"terminal-dependency/live/mentions-retired.md"),
        "an instance exists over an unmarked relation: {:?}",
        read_by_instances(&run)
    );
}

/// The edge that wrote the state it points at is not resting on it.
///
/// `supersedes` declares `on_target: {set_state: superseded}` and the target
/// stands at `superseded`. Reporting this would report every correct
/// supersession and ask its author to delete the lineage that the same core
/// requirement exists to retain.
#[test]
fn an_edge_that_wrote_its_target_s_state_is_the_cause_and_not_a_dependency() {
    let run = run();
    assert!(
        about(&run, "NOTE-FIX-succeeds-replaced").is_none(),
        "a correct supersession was reported: {:?}",
        refusals(&run)
    );
}

/// The exemption is the equality of two declared values, not the identity of
/// the relation.
///
/// The same `supersedes`, onto a document standing at `deprecated`. That state
/// is not the one this relation writes, so some other history put the target
/// there and the successor is resting on it. A rule that exempted the relation
/// rather than the state it writes passes the test above and fails this one.
#[test]
fn the_same_relation_onto_a_state_it_does_not_write_is_reported() {
    let run = run();
    let message = about(&run, "NOTE-FIX-succeeds-retired").expect("the deprecated-target finding");
    assert!(message.contains("`supersedes`"), "{message}");
    assert!(message.contains("`deprecated`"), "{message}");
}

/// The document that declares no state is not the document standing at one.
///
/// The wrong reading here is the one that returns green: an `Option` collapsed
/// at the parse makes "no state declared" and "a state that is not terminal"
/// one answer, and both then pass. This instance skips, with a reason that says
/// which end had nothing to read.
#[test]
fn a_target_that_declares_no_state_skips_rather_than_passing() {
    let run = run();
    let skips = skips(&run);
    let mine: Vec<&(Vec<&str>, &String)> = skips
        .iter()
        .filter(|(reads, _)| reads.contains(&"terminal-dependency/stubs/quiet.md"))
        .collect();
    assert_eq!(mine.len(), 1, "{skips:?}");
    assert!(mine[0].1.contains("NOTE-FIX-quiet"), "{}", mine[0].1);
    assert!(
        mine[0].1.contains("target end"),
        "the skip does not say which end had nothing to read: {}",
        mine[0].1
    );
}

/// A target standing at a state its own regime does not name is one defect, and
/// it belongs to the rule that reads the document.
///
/// `records/outside.md` is a `record` at `discharged`, which `narrow` never
/// names. `lifecycle.state.not_admitted` reports it once. This rule would
/// report it again for every document that cites it, so it stands down and
/// names the rule that owns it.
#[test]
fn a_target_standing_outside_its_own_machine_is_left_to_the_rule_that_owns_it() {
    let run = run();
    let skips = skips(&run);
    let mine: Vec<&(Vec<&str>, &String)> = skips
        .iter()
        .filter(|(reads, _)| reads.contains(&"terminal-dependency/records/outside.md"))
        .collect();
    assert_eq!(mine.len(), 1, "{skips:?}");
    assert!(
        mine[0].1.contains(headwater_check::lifecycle_state::RULE),
        "the skip does not name the rule that owns it: {}",
        mine[0].1
    );
    // And the rule it defers to did report, so the deferral reaches a rule that
    // is looking.
    assert!(
        run.findings.iter().any(|finding| {
            finding.rule == headwater_check::lifecycle_state::RULE
                && finding.path == "terminal-dependency/records/outside.md"
        }),
        "the rule this one defers to reported nothing"
    );
}

/// A source that is not live is not what spec 3's sentence is about, in both of
/// the two ways a source can fail to be live.
///
/// `draft` carries the role `initial`, and a document being argued over rests
/// on nothing yet. A terminal source is the case the specification is silent
/// about, and the silence is deliberate: the citations of a retired document
/// are part of the record it was retired with.
#[test]
fn a_source_that_is_not_live_is_silent_in_both_of_its_forms() {
    let run = run();
    assert!(
        about(&run, "NOTE-FIX-argued-over").is_none(),
        "a draft source was reported: {:?}",
        refusals(&run)
    );
    assert!(
        about(&run, "NOTE-FIX-retired-onto-retired").is_none(),
        "a terminal source was reported: {:?}",
        refusals(&run)
    );
    // Both passed. A skip here would mean the rule could not read one of the
    // two states, which is a different report from the one this corpus earns.
    for path in [
        "terminal-dependency/records/argued-over.md",
        "terminal-dependency/records/retired-onto-retired.md",
    ] {
        assert!(
            !skips(&run).iter().any(|(reads, _)| reads.contains(&path)),
            "{path} skipped rather than passing: {:?}",
            skips(&run)
        );
    }
}

/// The half an author wrote does not say which end is the source.
///
/// `records/successor-target.md` wrote `superseded_by`, and it is the only file
/// of its pair that wrote anything. The relation runs the other way, so the
/// source is the live document that declared nothing and the target is the file
/// in front of the author. The finding anchors at the entry that exists and it
/// names the pair in the declared direction.
#[test]
fn the_direction_comes_from_the_relation_and_never_from_the_file_that_wrote_it() {
    let run = run();
    let message = about(&run, "NOTE-FIX-successor").expect("the inverse-half finding");
    assert!(
        message.contains("NOTE-FIX-half-written"),
        "the target: {message}"
    );
    // Source first, target second, whichever end held the pen.
    let source_at = message.find("NOTE-FIX-successor").expect("the source");
    let target_at = message.find("NOTE-FIX-half-written").expect("the target");
    assert!(
        source_at < target_at,
        "the ends are the wrong way round: {message}"
    );

    let reported: Vec<&str> = refusals(&run).into_iter().map(|(path, _)| path).collect();
    assert!(
        reported.contains(&"terminal-dependency/records/successor-target.md"),
        "the finding anchors away from the only entry that exists: {reported:?}"
    );
}

/// The whole corpus, in one assertion, so a case that stops being reported
/// cannot hide behind a test that names only its own document.
#[test]
fn the_tree_reports_four_pairs_and_no_others() {
    let run = run();
    let mut reported: Vec<&str> = refusals(&run).into_iter().map(|(path, _)| path).collect();
    reported.sort_unstable();
    assert_eq!(
        reported,
        [
            "terminal-dependency/live/rests-on-receipt.md",
            "terminal-dependency/live/rests-on-retired.md",
            "terminal-dependency/live/succeeds-retired.md",
            "terminal-dependency/records/successor-target.md",
        ]
    );
    // Advisory, for the reason `CT-LIFE-3` states: the repair is a rewrite and
    // the engine cannot pick among three answers.
    assert!(run
        .findings
        .iter()
        .filter(|finding| finding.rule == RULE)
        .all(|finding| finding.severity == headwater_check::Severity::Warn));
}
