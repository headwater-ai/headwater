// SPDX-License-Identifier: Apache-2.0
//! An `evidenced` document whose pointer reaches an `asserted` one, and the
//! six ways a rule about it is easy to get wrong.
//!
//! [Spec 3](../../../../docs/spec/03-authoring-and-lifecycle.md#evidence-has-three-honest-states-not-two)
//! already rules it: "A pointer to a document with the `asserted` warrant does
//! not support `evidenced`, because such a document is neither external nor
//! auditable. A fabricated *why* with a file name is the failure that this
//! table exists to prevent." The clause stood in the specification and nothing
//! read it.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#testing-a-check-without-a-failing-fixture-does-not-ship)
//! sets the floor at one fixture that fails and one that passes. The tree under
//! `fixtures/evidence-basis/` goes past it, because a rule that reports nothing
//! and a rule that reports everything both look right against one case.
//!
//! **A rule that reported every evidence edge out of an evidenced document**
//! reports `claims/rests-on-accepted.md`, whose target a person read.
//!
//! **A rule holding `traces_to` alone** reports one of the eight edges that
//! `claims/every-evidence-relation.md` declares and says nothing about the
//! other seven. That is the mistake that halves the population.
//!
//! **A rule that read the source's warrant rather than its evidence basis**
//! reports `claims/unevidenced-onto-asserted.md`, which claims no evidence at
//! all and whose own warrant is `accepted`.
//!
//! **A rule that collapsed an absent warrant into "not asserted"** passes
//! `claims/onto-quiet.md` green, and a corpus can then lose every warrant it
//! declares without this rule saying a word.
//!
//! **A rule at `EdgeUnit::Entry`** reaches the anchor half that
//! `claims/traces-to-an-anchor.md` declares, where there is no second document
//! to read a warrant from.
//!
//! **A rule that read only a declared warrant** passes nothing and decides
//! nothing over `claims/onto-generated.md`, whose target this engine wrote.
//! Spec 3 derives `regenerated` from the marker and never from a declaration,
//! so a generated document has a warrant and states none, and a rule that
//! reads the declaration alone records a skip where there is an answer. The
//! set of such targets is open, because a projection can be declared over any
//! shelf, so every one of them reaches this rule undecided.
//!
//! No change manifest is bound anywhere below, and none is needed: both values
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

const RULE: &str = headwater_check::basis::RULE;

/// The `evidence` family of this repository's resolved lock, read out of
/// `.headwater/taxonomy.lock` rather than out of the one name a report quoted.
/// The fixture taxonomy declares all eight, and
/// `claims/every-evidence-relation.md` writes one edge of each.
const FAMILY: [&str; 8] = [
    "traces_to",
    "applied_in",
    "assesses",
    "cites_evidence",
    "discharges",
    "examines",
    "records",
    "verified_by",
];

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn run() -> Run {
    let corpus = Corpus::new(fixtures_dir(), "evidence-basis");
    let source = std::fs::read_to_string(fixtures_dir().join("evidence-basis.taxonomy.yml"))
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
            lock: "sha256:evidence-basis-fixture",
            taxonomy: &taxonomy,
            shape: &shape,
            relations: &declarations,
            config: &config,
            register: &register,
            adoption: None,
            source: "engine/crates/check/fixtures/evidence-basis.taxonomy.yml",
        },
        &headwater_check::claim::Claims::empty(),
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

/// Every finding of this rule whose message names a document.
fn about<'a>(run: &'a Run, id: &str) -> Vec<&'a str> {
    refusals(run)
        .into_iter()
        .map(|(_, message)| message)
        .filter(|message| message.contains(id))
        .collect()
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
#[test]
fn an_evidenced_document_resting_on_an_asserted_one_is_reported() {
    let run = run();
    let reported = about(&run, "NOTE-FIX-rests-on-asserted");
    assert_eq!(reported.len(), 1, "the decisive finding: {reported:?}");
    let message = reported[0];
    assert!(
        message.contains("NOTE-FIX-asserted"),
        "the target: {message}"
    );
    assert!(message.contains("`traces_to`"), "the relation: {message}");
    assert!(message.contains("`asserted`"), "the warrant: {message}");
    assert!(
        message.contains("`evidenced`"),
        "the claim the source made: {message}"
    );

    let paths: Vec<&str> = refusals(&run).into_iter().map(|(path, _)| path).collect();
    assert!(
        paths.contains(&"evidence-basis/claims/rests-on-asserted.md"),
        "the finding anchors at the entry that declares the edge: {paths:?}"
    );
}

/// The same claim and the same relation onto a document a person accepted.
///
/// `rests-on-accepted.md` and `rests-on-asserted.md` differ in the identifier
/// they name and in nothing else, and the two far ends differ in one value. A
/// rule that reported the relation rather than the pair reports both.
#[test]
fn the_same_edge_onto_an_accepted_document_is_silent() {
    let run = run();
    assert!(
        about(&run, "NOTE-FIX-rests-on-accepted").is_empty(),
        "{:?}",
        refusals(&run)
    );
    // And it passed rather than declining to decide, which is the difference
    // between a rule that looked and a rule that could not.
    assert!(
        !skips(&run)
            .iter()
            .any(|(reads, _)| reads.contains(&"evidence-basis/claims/rests-on-accepted.md")),
        "the accepted pair skipped: {:?}",
        skips(&run)
    );
}

/// The whole family, one relation at a time.
///
/// The rule reads `family: evidence` and never a relation name, and this is the
/// assertion that says so. One document declares one edge of each of the eight,
/// every one of them onto the same asserted record, and all eight are reported
/// with the relation named.
#[test]
fn every_relation_of_the_evidence_family_is_read_and_not_traces_to_alone() {
    let run = run();
    let reported = about(&run, "NOTE-FIX-every-evidence-relation");
    assert_eq!(
        reported.len(),
        FAMILY.len(),
        "the family is {} relations and {} were reported: {reported:?}",
        FAMILY.len(),
        reported.len()
    );
    for relation in FAMILY {
        assert!(
            reported
                .iter()
                .any(|message| message.contains(&format!("`{relation}`"))),
            "`{relation}` reached no finding: {reported:?}"
        );
    }
}

/// A relation outside the family reaches no instance at all.
///
/// `mentions` is `association`. The pair is not silent because the rule decided
/// something about it: the rule was never generated over it.
#[test]
fn a_relation_outside_the_evidence_family_generates_no_instance() {
    let run = run();
    assert!(
        about(&run, "NOTE-FIX-mentions-asserted").is_empty(),
        "an association relation reached the rule: {:?}",
        refusals(&run)
    );
    assert!(
        !read_by_instances(&run).contains(&"evidence-basis/claims/mentions-asserted.md"),
        "an instance exists over an association relation: {:?}",
        read_by_instances(&run)
    );
}

/// The source's claim is what the rule reads, and not the source's own warrant.
///
/// Three documents point at the same asserted record through the same relation.
/// One says `unevidenced`, one leaves the template placeholder unfilled, and one
/// writes no `evidence_basis` at all. None of the three claims `evidenced`, so
/// none of them is reported, and none of the three is a skip.
#[test]
fn a_source_that_does_not_claim_evidenced_passes_in_all_three_of_its_forms() {
    let run = run();
    for id in [
        "NOTE-FIX-unevidenced-onto-asserted",
        "NOTE-FIX-placeholder-basis",
        "NOTE-FIX-no-basis",
    ] {
        assert!(
            about(&run, id).is_empty(),
            "{id} was reported: {:?}",
            refusals(&run)
        );
    }
    for path in [
        "evidence-basis/claims/unevidenced-onto-asserted.md",
        "evidence-basis/claims/placeholder-basis.md",
        "evidence-basis/claims/no-basis-onto-asserted.md",
    ] {
        assert!(
            !skips(&run).iter().any(|(reads, _)| reads.contains(&path)),
            "{path} skipped rather than passing: {:?}",
            skips(&run)
        );
    }
}

/// A warrant outside spec 3's closed set is not `asserted`, and the pass is
/// deliberate.
///
/// [HW-OBL-0125](../../../../docs/obligations/0125-nine-documents-state-a-warrant-the-closed-set-does-not-hold-and-no-check-reads-one.md)
/// holds the nine documents of this repository that stand at `proposed`. This
/// rule is not the one that reports them, and the test pins that as a decision
/// rather than leaving it to be read off a green run.
#[test]
fn a_warrant_the_closed_set_does_not_name_is_a_pass_and_not_a_finding() {
    let run = run();
    assert!(
        about(&run, "NOTE-FIX-rests-on-proposed").is_empty(),
        "a proposed target was reported: {:?}",
        refusals(&run)
    );
    assert!(
        !skips(&run)
            .iter()
            .any(|(reads, _)| reads.contains(&"evidence-basis/claims/rests-on-proposed.md")),
        "the proposed pair skipped: {:?}",
        skips(&run)
    );
}

/// The document that declares no warrant is not the document standing at one.
///
/// The wrong reading here is the one that returns green: an `Option` collapsed
/// at the parse makes "no warrant declared" and "a warrant this rule read as
/// not asserted" one answer, and both then pass. This instance skips, with a
/// reason that says which end had nothing to read.
#[test]
fn a_target_that_declares_no_warrant_skips_rather_than_passing() {
    let run = run();
    let skips = skips(&run);
    let mine: Vec<&(Vec<&str>, &String)> = skips
        .iter()
        .filter(|(reads, _)| reads.contains(&"evidence-basis/targets/quiet.md"))
        .collect();
    assert_eq!(mine.len(), 1, "{skips:?}");
    assert!(mine[0].1.contains("NOTE-FIX-quiet"), "{}", mine[0].1);
    assert!(
        mine[0].1.contains("target end"),
        "the skip does not say which end had nothing to read: {}",
        mine[0].1
    );
}

/// A target this engine wrote is decided, on the warrant the engine derives.
///
/// [Spec 3](../../../../docs/spec/03-authoring-and-lifecycle.md#the-warrant-and-what-each-value-requires):
/// "The engine derives `regenerated` from the marker, and never from a
/// declaration." `targets/generated.md` carries the marker in its front matter
/// and declares no provenance block, which is the shape every generated
/// document that declares an identity has. `regenerated` is not `asserted`, so
/// the pair passes.
///
/// The two assertions below are one case, and dropping either loses the point.
/// A rule that never generated an instance over the pair also reports nothing
/// about it, and a rule that still reads the declaration alone records a skip
/// rather than a finding. Neither is a decision.
#[test]
fn a_generated_target_is_decided_on_a_derived_warrant_rather_than_skipped() {
    let run = run();
    assert!(
        about(&run, "NOTE-FIX-onto-generated").is_empty(),
        "a generated target was reported: {:?}",
        refusals(&run)
    );
    assert!(
        read_by_instances(&run).contains(&"evidence-basis/claims/onto-generated.md"),
        "no instance exists over the generated pair: {:?}",
        read_by_instances(&run)
    );
    assert!(
        !skips(&run)
            .iter()
            .any(|(reads, _)| reads.contains(&"evidence-basis/targets/generated.md")),
        "the generated pair skipped rather than passing: {:?}",
        skips(&run)
    );
}

/// The document that declares no warrant and that nothing generated still skips.
///
/// This is the guard on the case above. The two targets differ in one line, the
/// marker, and a fix that answered `regenerated` for every target with no
/// provenance block turns `claims/onto-quiet.md` green and loses the reading
/// that `a_target_that_declares_no_warrant_skips_rather_than_passing` holds.
#[test]
fn deriving_a_warrant_for_a_generated_target_does_not_answer_for_an_ungenerated_one() {
    let run = run();
    let skips = skips(&run);
    let quiet: Vec<&(Vec<&str>, &String)> = skips
        .iter()
        .filter(|(reads, _)| reads.contains(&"evidence-basis/targets/quiet.md"))
        .collect();
    assert_eq!(quiet.len(), 1, "the ungenerated target stopped skipping: {skips:?}");
    assert_eq!(
        about(&run, "NOTE-FIX-rests-on-asserted").len(),
        1,
        "the decisive finding went with it: {:?}",
        refusals(&run)
    );
}

/// An anchor at the far end is not a document and carries no warrant.
///
/// `EdgeUnit::Pair` never groups a half whose target is not a document, so the
/// protection is inherited rather than written here. Nothing else in this
/// engine asserts it, and a later change to `UNIT` would pass every other test
/// on this page.
#[test]
fn an_edge_onto_an_anchor_reaches_no_instance_and_does_not_panic() {
    let run = run();
    assert!(
        about(&run, "NOTE-FIX-traces-to-an-anchor").is_empty(),
        "an anchor half was reported: {:?}",
        refusals(&run)
    );
    assert!(
        !read_by_instances(&run).contains(&"evidence-basis/claims/traces-to-an-anchor.md"),
        "an instance exists over an anchor half: {:?}",
        read_by_instances(&run)
    );
}

/// The whole corpus, in one assertion, so a case that stops being reported
/// cannot hide behind a test that names only its own document.
#[test]
fn the_tree_reports_two_documents_and_nine_edges() {
    let run = run();
    let mut reported: Vec<&str> = refusals(&run).into_iter().map(|(path, _)| path).collect();
    reported.sort_unstable();
    let mut expected = vec!["evidence-basis/claims/rests-on-asserted.md"];
    expected.extend(std::iter::repeat_n(
        "evidence-basis/claims/every-evidence-relation.md",
        FAMILY.len(),
    ));
    expected.sort_unstable();
    assert_eq!(reported, expected);
}

/// The posture is advisory, and this is the assertion behind `CT-WARRANT-2`.
///
/// This corpus holds instances of the rule today, so an error severity takes
/// `check --strict` red on the change that lands the rule, and the repair is a
/// rewrite that only an author can make. The control states the reasoning and
/// this test holds the engine to it.
#[test]
fn every_finding_is_advisory() {
    let run = run();
    assert!(!refusals(&run).is_empty(), "the tree reported nothing");
    assert!(run
        .findings
        .iter()
        .filter(|finding| finding.rule == RULE)
        .all(|finding| finding.severity == headwater_check::Severity::Warn));
}

/// No finding carries a patch, for the reason the control states.
///
/// The repair is a pointer at an auditable artifact, a promotion of the target,
/// or a demotion of the source's own claim to `reconstructed`. The engine can
/// see the pair and cannot pick among the three.
#[test]
fn no_finding_carries_a_patch() {
    let run = run();
    assert!(run
        .findings
        .iter()
        .filter(|finding| finding.rule == RULE)
        .all(|finding| finding.patch.is_none()));
}
