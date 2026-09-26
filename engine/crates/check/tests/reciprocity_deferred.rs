// SPDX-License-Identifier: Apache-2.0
//! The far half of a reciprocal pair is owed only once the document that wrote
//! the near half has left its initial state, and the ways a rule about it is
//! easy to get wrong.
//!
//! One document per row, each writing one half onto a document that wrote
//! none. The rule reads the writer's state through the role its value carries,
//! so no row depends on a state's name.
//!
//! **A rule that read the word `draft`** reports `notes/proposed-inverse.md`,
//! whose state is a second value with the role `initial`.
//!
//! **A rule that deferred on an absence** passes `stubs/stateless-declares.md`,
//! whose kind carries no state. An absence keeps the behavior the check had
//! before the deferral.
//!
//! **A rule that read the target's state** passes `notes/current-onto-draft.md`,
//! a live writer onto a draft. The deferral follows the writer.

use headwater_census::census;
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::{
    Cache, Context, Date, Declared, Finding, Outcome, Patch, Register, Run, Shape,
};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use std::path::{Path, PathBuf};

const PINNED: &str = "2026-08-12";

/// Named as a literal, as the sibling tables do.
const RULE: &str = "relation.reciprocity.missing";

const TREE: &str = "reciprocity-deferred";

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn run() -> Run {
    let corpus = Corpus::new(fixtures_dir(), TREE);
    let source = std::fs::read_to_string(fixtures_dir().join(format!("{TREE}.taxonomy.yml")))
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
    let lock = format!("sha256:{TREE}-fixture");
    let source = format!("engine/crates/check/fixtures/{TREE}.taxonomy.yml");
    headwater_check::run(
        &taken,
        &graph,
        &Declared {
            lock: &lock,
            taxonomy: &taxonomy,
            shape: &shape,
            relations: &declarations,
            config: &config,
            register: &register,
            observations: &headwater_check::Observations::empty(),
            adoption: None,
            source: &source,
        },
        &headwater_check::claim::Claims::empty(),
        &Context::at(Date::parse(PINNED).expect("the pinned date")),
        &mut Cache::disabled(),
    )
}

fn path(file: &str) -> String {
    format!("{TREE}/{file}")
}

/// The findings of this rule anchored on one writer.
fn findings_on<'a>(run: &'a Run, file: &str) -> Vec<&'a Finding> {
    let path = path(file);
    run.findings
        .iter()
        .filter(|finding| finding.rule == RULE && finding.path == path)
        .collect()
}

/// The outcome of every instance of this rule that read a file.
fn outcomes_reading<'a>(run: &'a Run, file: &str) -> Vec<&'a Outcome> {
    let path = path(file);
    run.instances
        .iter()
        .filter(|instance| instance.rule == RULE)
        .filter(|instance| instance.reads.iter().any(|input| input.path == path))
        .map(|instance| &instance.outcome)
        .collect()
}

/// A writer whose own instance passed and reported nothing.
fn deferred(run: &Run, file: &str) {
    assert_eq!(
        findings_on(run, file),
        Vec::<&Finding>::new(),
        "{file} stands at an initial state, so nothing is owed yet"
    );
    let outcomes = outcomes_reading(run, file);
    assert_eq!(outcomes.len(), 1, "{file} is one pair: {outcomes:?}");
    assert!(
        matches!(outcomes[0], Outcome::Passed),
        "{file} passes: {outcomes:?}"
    );
}

/// A writer with one finding, owed by `far` under `name`, with the patch that
/// writes it.
fn owed(run: &Run, file: &str, far: &str, name: &str, id: &str) {
    let findings = findings_on(run, file);
    assert_eq!(findings.len(), 1, "{file} owes one half: {findings:?}");
    let finding = findings[0];
    let far = path(far);
    assert!(
        finding.remediation.contains(&far) && finding.remediation.contains(name),
        "the remediation names {far} and `{name}`: {}",
        finding.remediation
    );
    match &finding.patch {
        Some(Patch::Half {
            path,
            relation,
            id: target,
            ..
        }) => {
            assert_eq!(path, &far);
            assert_eq!(relation, name);
            assert_eq!(target, id);
        }
        other => panic!("{file} carries a half patch, and carries {other:?}"),
    }
}

#[test]
fn a_draft_writer_owes_nothing_yet() {
    deferred(&run(), "notes/draft-declares.md");
}

#[test]
fn a_second_initial_state_defers_as_the_first_does() {
    deferred(&run(), "notes/proposed-inverse.md");
}

#[test]
fn a_live_writer_of_the_declared_half_is_owed_the_inverse() {
    owed(
        &run(),
        "notes/current-declares.md",
        "notes/target.md",
        "elaborated_by",
        "NOTE-FIX-current-declares",
    );
}

#[test]
fn a_live_writer_of_the_inverse_half_is_owed_the_declared_name() {
    owed(
        &run(),
        "notes/current-inverse.md",
        "notes/target.md",
        "elaborates",
        "NOTE-FIX-current-inverse",
    );
}

#[test]
fn a_terminal_writer_is_owed_the_half() {
    owed(
        &run(),
        "notes/superseded-declares.md",
        "notes/target.md",
        "elaborated_by",
        "NOTE-FIX-superseded-declares",
    );
}

#[test]
fn a_writer_with_no_state_defers_nothing() {
    owed(
        &run(),
        "stubs/stateless-declares.md",
        "notes/target.md",
        "elaborated_by",
        "NOTE-FIX-stateless-declares",
    );
}

#[test]
fn a_live_writer_onto_a_draft_is_owed_by_the_draft() {
    owed(
        &run(),
        "notes/current-onto-draft.md",
        "notes/draft-target.md",
        "elaborated_by",
        "NOTE-FIX-current-onto-draft",
    );
}

#[test]
fn a_draft_with_both_halves_passes() {
    deferred(&run(), "notes/draft-both.md");
}

/// The whole tree, counted, so that a row nobody asserted cannot move.
#[test]
fn the_tree_has_eight_pairs_and_five_findings() {
    let run = run();
    let instances = run
        .instances
        .iter()
        .filter(|instance| instance.rule == RULE)
        .count();
    let findings = run
        .findings
        .iter()
        .filter(|finding| finding.rule == RULE)
        .count();
    assert_eq!((instances, findings), (8, 5));
}
