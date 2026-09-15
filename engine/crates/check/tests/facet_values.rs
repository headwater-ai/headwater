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
//! `rulings/excluded.md` and `opens/any.md` carry the same front matter, down
//! to `evidence_basis: asserted`. Only the shelf differs, and the shelf is what
//! says which kind, and the kind is what narrows the facet. A rule that read
//! `asserted` as wrong wherever it found it reports both.
//!
//! The second is a reading that stops at the kind's own declaration. `ruling`
//! declares no narrowing, so the set `rulings/excluded.md` is read against can
//! only come from walking to `record`. A rule that read `kind.narrows` and not
//! `Shape::ancestry` reports nothing there and still passes every other case in
//! this file.
//!
//! The third is a reading that stops instantiating where no narrowing is
//! declared. `opens/any.md` stands at a value no narrowing in this taxonomy
//! names, and its kind declares none. The whole declared set is what it admits,
//! and an instance must still stand over it — [`headwater_check::coverage`]'s
//! OB-COV-2 finding is unreachable for any corpus once a rule stops counting
//! the documents it could only ever pass.
//!
//! # Why this taxonomy is one `taxonomy validate` accepts, and why that matters
//!
//! A narrowing takes values away and adds none, so a child's set is a subset of
//! every set above it and `kind inheritance` refuses a declaration for which
//! that is false. The first draft of this fixture had `note` name a value
//! `record` excluded: the check layer read it as an intersection and reported
//! the answer this file asserted, while the resolver would have refused the
//! taxonomy outright. Two enforcement paths disagreeing about one file is worse
//! than either being wrong, so every narrowing here resolves, and
//! `engine/crates/resolve/tests/facet_narrowing.rs` is where the refusals live.
//!
//! One consequence is worth stating, because it decides what this file can
//! prove. Under a taxonomy that resolves, the deepest narrowing in a chain
//! *is* the intersection, so no corpus can separate "intersect the chain" from
//! "take the innermost narrowing". What a corpus can separate is "walk the
//! chain" from "read this kind only", and `rulings/excluded.md` is that case.
//! [`headwater_check::Shape::admitted_values`] is asserted directly for the
//! rest.
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

fn source() -> headwater_yaml::Mapping {
    let text = std::fs::read_to_string(fixtures_dir().join("facet-values.taxonomy.yml"))
        .expect("the fixture taxonomy");
    headwater_yaml::load(&text)
        .expect("the fixture taxonomy loads")
        .value
        .as_map()
        .expect("a mapping")
        .clone()
}

fn run() -> Run {
    let corpus = Corpus::new(fixtures_dir(), "facet-values");
    let root = source();

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

fn reported(run: &Run) -> Vec<&str> {
    let mut paths: Vec<&str> = refusals(run).into_iter().map(|(path, _)| path).collect();
    paths.sort_unstable();
    paths
}

fn message_over<'a>(run: &'a Run, path: &str) -> &'a str {
    refusals(run)
        .into_iter()
        .find(|(found, _)| *found == path)
        .unwrap_or_else(|| panic!("no finding over {path}: {:?}", reported(run)))
        .1
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

/// The decisive pair: one value, two kinds, one finding.
///
/// `rulings/excluded.md` and `opens/any.md` differ in no front-matter field but
/// the identifier, and the document whose kind's chain excludes the value is
/// reported while the document whose kind admits it is silent. Neither of the
/// two declarations that narrow a value set today reaches this: `evidence_basis`
/// has no machine for a lifecycle regime to name a subset of, and no shelf
/// discriminates on it.
#[test]
fn one_value_on_two_kinds_reports_the_kind_that_does_not_admit_it() {
    let run = run();
    let reported = reported(&run);
    assert!(
        reported.contains(&"facet-values/rulings/excluded.md"),
        "the kind whose chain excludes the value was not reported: {reported:?}"
    );
    assert!(
        !reported.contains(&"facet-values/opens/any.md"),
        "the rule read the value rather than the kind: {reported:?}"
    );
    assert_eq!(
        skips(&run)
            .iter()
            .filter(|(path, _)| *path == "facet-values/opens/any.md")
            .count(),
        0,
        "the open document passed rather than declining to decide"
    );
}

/// Exactly two documents are reported, and each one is reported by a different
/// narrowing.
///
/// The denominator for the pair above. `rulings/excluded.md` is excluded by a
/// narrowing its kind does not declare, and `notes/excluded.md` is excluded by
/// one its kind does declare while the kind above it admits the value. A rule
/// that read only the kind reports the first and not the second; a rule that
/// read only the chain's top reports the second and not the first.
#[test]
fn each_of_the_two_narrowings_reports_its_own_document_and_no_other() {
    let run = run();
    assert_eq!(
        reported(&run),
        vec![
            "facet-values/notes/excluded.md",
            "facet-values/rulings/excluded.md",
        ],
        "the reported set is not the two documents the two narrowings exclude"
    );
}

/// Each message names the set that kind admits, and not the facet's list.
///
/// The remediation is the only place an author learns what to write instead. A
/// message that quoted the declaration would send them to a value their kind
/// refuses, which is worse than no message. The two messages differ, which is
/// what says the admitted set is per kind rather than per corpus.
#[test]
fn each_message_names_the_set_its_own_kind_admits() {
    let run = run();

    // `ruling` declares no narrowing and inherits `record`'s two.
    let inherited = message_over(&run, "facet-values/rulings/excluded.md");
    assert!(
        inherited.contains("admits measured, cited,"),
        "the inherited set: {inherited}"
    );
    assert!(!inherited.contains("observed"), "{inherited}");

    // `note` narrows to one of those two.
    let own = message_over(&run, "facet-values/notes/excluded.md");
    assert!(own.contains("admits cited,"), "the narrowed set: {own}");
    for excluded in ["asserted", "observed"] {
        assert!(
            !own.contains(excluded),
            "the message offers `{excluded}`, which the kind does not admit: {own}"
        );
    }

    let remediation = &run
        .findings
        .iter()
        .find(|finding| finding.path == "facet-values/notes/excluded.md")
        .expect("the finding")
        .remediation;
    assert!(remediation.contains("cited"), "{remediation}");
    assert!(!remediation.contains("asserted"), "{remediation}");
}

/// The admitted set of every kind, read off the shape the check reads.
///
/// The corpus cannot separate an intersection from the innermost narrowing on a
/// taxonomy that resolves, so the function both the check and the schema
/// emitter call is asserted here directly, over all four kinds and both
/// absences.
#[test]
fn the_admitted_set_of_each_kind_is_the_chain_narrowed() {
    let shape = Shape::read(&source()).expect("the shape reads");
    let admitted = |kind: &str| shape.admitted_values(kind, "evidence_basis");

    assert_eq!(admitted("record"), Some(vec!["measured", "cited"]));
    assert_eq!(
        admitted("ruling"),
        Some(vec!["measured", "cited"]),
        "a kind that declares no narrowing lost its ancestor's"
    );
    assert_eq!(
        admitted("note"),
        Some(vec!["cited"]),
        "a kind that narrows further did not narrow"
    );
    assert_eq!(
        admitted("open"),
        Some(vec!["measured", "cited", "asserted", "observed"]),
        "a kind outside the chain did not admit the whole declared set"
    );

    // The three absences, which are three different answers.
    assert_eq!(
        shape.admitted_values("ruling", "no_such_facet"),
        None,
        "a facet this taxonomy does not declare answered a set"
    );
    assert_eq!(
        shape.admitted_values("ruling", "summary"),
        Some(Vec::new()),
        "a facet that enumerates nothing is an empty set and not an absence"
    );
    assert_eq!(
        admitted("no_such_kind"),
        Some(vec!["measured", "cited", "asserted", "observed"]),
        "a kind nothing declares narrows nothing, which is the declared set"
    );
}

/// A kind that narrows nothing still generates an instance over every document
/// of it.
///
/// The trap [`headwater_check::facet_value`]'s module comment names, met from
/// this change's side. A narrowing member that made `instantiates` depend on a
/// declared narrowing would stop counting the kinds that declare none, which is
/// every kind of most taxonomies.
#[test]
fn a_kind_that_narrows_nothing_still_instantiates() {
    let run = run();
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
