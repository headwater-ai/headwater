// SPDX-License-Identifier: Apache-2.0
//! The values a kind admits of an enumerated facet with no machine behind it,
//! and the corpus that separates a document at one from a document outside
//! them.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#testing-a-check-without-a-failing-fixture-does-not-ship)
//! sets the floor: "every check ships with at least one fixture that it fails
//! and one that it passes." The tree under `fixtures/facet-values/` is written
//! to be wrong under the three mistakes this reading is easiest to make.
//!
//! The first is a reading that reads the value and not the kind.
//! `rulings/admitted.md` and `notes/excluded.md` carry the same front matter,
//! down to the value of `evidence_basis`. Only the shelf differs, and the shelf
//! is what says which kind, and the kind is what narrows the facet. A rule that
//! read `measured` as wrong wherever it found it reports both.
//!
//! The second is a reading that takes the narrowing nearest the document.
//! `note` narrows to `cited` and `asserted`, and `record` above it narrows to
//! `measured` and `cited`. A rule that read the child alone would admit
//! `asserted` on a note, and a rule that read the topmost would admit
//! `measured`. The one value the chain admits is `cited`, and the message on
//! the single finding is where that intersection is legible.
//!
//! The third is a reading that stops instantiating where no narrowing is
//! declared. `opens/any.md` stands at a value no narrowing in this taxonomy
//! names, and its kind declares none. The whole declared set is what it admits,
//! and an instance must still stand over it — [`headwater_check::coverage`]'s
//! OB-COV-2 finding is unreachable for any corpus once a rule stops counting
//! the documents it could only ever pass.
//!
//! No lifecycle regime is bound anywhere below, and that is the property rather
//! than an omission. `evidence_basis` has no initial value and no transitions,
//! so [#219](https://github.com/headwater-ai/headwater/issues/219)'s mechanism
//! reaches nothing here.

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

const RULE: &str = headwater_check::facet_value::RULE;

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn run() -> Run {
    let corpus = Corpus::new(fixtures_dir(), "facet-values");
    let source = std::fs::read_to_string(fixtures_dir().join("facet-values.taxonomy.yml"))
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
            lock: "sha256:facet-values-fixture",
            taxonomy: &taxonomy,
            shape: &shape,
            relations: &declarations,
            config: &config,
            register: &register,
            adoption: None,
            source: "engine/crates/check/fixtures/facet-values.taxonomy.yml",
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

/// A document of a kind that does not admit a value carries that value and is a
/// finding, and the document of the kind that does admit it is silent.
///
/// The decisive fixture, and the one no existing mechanism reaches. There is no
/// machine over `evidence_basis` for a lifecycle regime to name a subset of,
/// and no shelf discriminates on it, so neither of the two declarations that
/// narrow a value set today can express the difference between these two
/// documents.
#[test]
fn a_document_of_a_kind_that_does_not_admit_a_value_is_the_only_finding() {
    let run = run();
    let refusals = refusals(&run);
    assert_eq!(
        refusals.len(),
        1,
        "the value is not the defect and the kind is: {refusals:?}"
    );
    assert_eq!(refusals[0].0, "facet-values/notes/excluded.md");
}

/// The same value, on the kind whose chain admits it, is silent and not
/// skipped.
///
/// `rulings/admitted.md` is `notes/excluded.md` with one thing changed, and the
/// thing is not in the front matter. Silence has two causes and only one of
/// them is a pass, so the skip set is read as well as the finding set.
#[test]
fn the_same_value_on_a_kind_whose_chain_admits_it_is_silent() {
    let run = run();
    let reported: Vec<&str> = refusals(&run).into_iter().map(|(path, _)| path).collect();
    assert!(
        !reported.contains(&"facet-values/rulings/admitted.md"),
        "the rule read the value rather than the kind: {reported:?}"
    );
    assert_eq!(
        skips(&run)
            .iter()
            .filter(|(path, _)| *path == "facet-values/rulings/admitted.md")
            .count(),
        0,
        "the ruling passed rather than declined to decide"
    );
}

/// The message names the narrowed set and not the facet's whole list.
///
/// The remediation is the only place an author learns what to write instead. A
/// message that quoted the declaration would send them to a value their kind
/// refuses, which is worse than no message.
#[test]
fn the_message_names_the_narrowed_set_and_not_the_declared_one() {
    let run = run();
    let refusals = refusals(&run);
    let (_, message) = refusals[0];
    assert!(message.contains("`cited`"), "the admitted value: {message}");
    for excluded in ["asserted", "observed"] {
        assert!(
            !message.contains(excluded),
            "the message offers `{excluded}`, which the chain does not admit: {message}"
        );
    }
    let remediation = &run
        .findings
        .iter()
        .find(|finding| finding.rule == RULE)
        .expect("the finding")
        .remediation;
    assert!(remediation.contains("cited"), "{remediation}");
    assert!(!remediation.contains("asserted"), "{remediation}");
}

/// A narrowing an ancestor declares is intersected with the child's own.
///
/// `notes/intersected.md` stands at the one value both narrowings name. A rule
/// that read the child alone and a rule that read the parent alone both admit
/// it, so this document alone proves nothing; it is the pair of it and the
/// message above that does.
#[test]
fn a_child_inherits_the_intersection_of_every_narrowing_above_it() {
    let run = run();
    let reported: Vec<&str> = refusals(&run).into_iter().map(|(path, _)| path).collect();
    assert!(
        !reported.contains(&"facet-values/notes/intersected.md"),
        "{reported:?}"
    );
    assert!(
        !reported.contains(&"facet-values/rulings/inherited.md"),
        "a kind that declares no narrowing of its own lost its ancestor's: {reported:?}"
    );
}

/// A kind that narrows nothing admits the whole declared set, and still
/// generates an instance over every document of it.
///
/// The trap [`headwater_check::facet_value`]'s module comment names, met from
/// this change's side. `opens/any.md` stands at a value no narrowing here
/// admits and its kind narrows nothing, so it passes; a narrowing member that
/// made `instantiates` depend on a declared narrowing would stop counting it as
/// checked at all.
#[test]
fn a_kind_that_narrows_nothing_admits_the_whole_set_and_still_instantiates() {
    let run = run();
    let reported: Vec<&str> = refusals(&run).into_iter().map(|(path, _)| path).collect();
    assert!(!reported.contains(&"facet-values/opens/any.md"), "{reported:?}");

    let targets: Vec<&str> = run
        .instances
        .iter()
        .filter(|instance| instance.rule == RULE)
        .map(over_document)
        .collect();
    assert!(
        targets.contains(&"facet-values/opens/any.md"),
        "no instance stands over the kind that narrows nothing: {targets:?}"
    );
    assert_eq!(
        targets.len(),
        5,
        "one instance per document, whatever its kind narrows: {targets:?}"
    );
}

/// Every instance of this rule reaches a verdict in a full-corpus run.
///
/// A narrowing that arrived as a skip rather than as a verdict would pass the
/// first test of this file and report nothing anywhere else.
#[test]
fn no_instance_of_this_rule_declines_to_decide() {
    let run = run();
    let skips = skips(&run);
    assert!(skips.is_empty(), "{skips:?}");
}
