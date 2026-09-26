// SPDX-License-Identifier: Apache-2.0
//! One document that two relations tell two different states, and the three
//! ways a rule about it is easy to get wrong.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#testing-a-check-without-a-failing-fixture-does-not-ship)
//! sets the floor: "every check ships with at least one fixture that it fails
//! and one that it passes." The tree under `fixtures/state-set-twice/` is
//! [#1086](https://github.com/headwater-ai/headwater/issues/1086).
//!
//! **A rule that compared relation names** reports `notes/agree.md`, where
//! `supersedes` and `replaces` both write `superseded`.
//!
//! **A rule that read the writing file as the source** reports
//! `notes/inverse.md`. `notes/retired-by-inverse.md` wrote `retired_by` toward
//! it, so `inverse.md` is the source of that `retires` edge and is told no
//! state by it. The same rule misses `notes/self-written.md`, whose clash is
//! written entirely in inverse names by the target itself.
//!
//! **A rule that ran on every taxonomy** generates instances where one relation
//! is the only one that writes a state and no clash can exist.

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

const RULE: &str = headwater_check::state_set_twice::RULE;

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn run_with(taxonomy_file: &str) -> Run {
    let corpus = Corpus::new(fixtures_dir(), "state-set-twice");
    let source =
        std::fs::read_to_string(fixtures_dir().join(taxonomy_file)).expect("the fixture taxonomy");
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
            lock: "sha256:state-set-twice-fixture",
            taxonomy: &taxonomy,
            shape: &shape,
            relations: &declarations,
            config: &config,
            register: &register,
            observations: &headwater_check::Observations::empty(),
            adoption: None,
            source: "engine/crates/check/fixtures/state-set-twice.taxonomy.yml",
        },
        &headwater_check::claim::Claims::empty(),
        &Context::at(Date::parse(PINNED).expect("the pinned date")),
        &mut Cache::disabled(),
    )
}

fn run() -> Run {
    run_with("state-set-twice.taxonomy.yml")
}

/// The findings this rule reported, as `(path, message)`.
fn reported(run: &Run) -> Vec<(&str, &str)> {
    run.findings
        .iter()
        .filter(|finding| finding.rule == RULE)
        .map(|finding| (finding.path.as_str(), finding.message.as_str()))
        .collect()
}

fn at<'a>(run: &'a Run, file: &str) -> Vec<&'a str> {
    let path = format!("state-set-twice/notes/{file}");
    reported(run)
        .into_iter()
        .filter(|(at, _)| *at == path)
        .map(|(_, message)| message)
        .collect()
}

/// The decisive case: one finding at the target, naming both relations, both
/// states and the document that wrote each edge.
#[test]
fn a_document_told_two_states_by_two_relations_is_reported_once() {
    let run = run();
    let messages = at(&run, "clash.md");
    assert_eq!(messages.len(), 1, "one finding at the target: {messages:?}");
    let message = messages[0];
    for needle in [
        "`supersedes`",
        "`retires`",
        "`superseded`",
        "`retired`",
        "NOTE-FIX-clash-superseder",
        "NOTE-FIX-clash-retirer",
    ] {
        assert!(message.contains(needle), "{needle} in {message}");
    }
}

/// Two relations that write the same state agree, and there is nothing to
/// report.
#[test]
fn two_relations_that_write_one_state_are_silent() {
    let run = run();
    assert!(at(&run, "agree.md").is_empty(), "{:?}", reported(&run));
}

/// A document named by another file's inverse half is the source of that
/// relation, and a source is told no state by it.
#[test]
fn an_inverse_half_is_read_from_the_declared_end() {
    let run = run();
    assert!(at(&run, "inverse.md").is_empty(), "{:?}", reported(&run));
    assert!(
        at(&run, "retired-by-inverse.md").is_empty(),
        "{:?}",
        reported(&run)
    );
}

/// A clash written entirely from the target's side, in two inverse names, is
/// the same clash.
#[test]
fn a_clash_written_in_inverse_names_is_reported() {
    let run = run();
    let messages = at(&run, "self-written.md");
    assert_eq!(messages.len(), 1, "{:?}", reported(&run));
    assert!(messages[0].contains("NOTE-FIX-self-superseder"), "{}", messages[0]);
    assert!(messages[0].contains("NOTE-FIX-self-retirer"), "{}", messages[0]);
}

/// Exactly the two clashes, and nothing else in the tree.
#[test]
fn the_tree_reports_exactly_the_two_clashes() {
    let run = run();
    let mut paths: Vec<&str> = reported(&run).into_iter().map(|(path, _)| path).collect();
    paths.sort_unstable();
    assert_eq!(
        paths,
        [
            "state-set-twice/notes/clash.md",
            "state-set-twice/notes/self-written.md"
        ]
    );
    // Every other instance passed rather than declining to decide.
    assert!(
        run.instances
            .iter()
            .filter(|instance| instance.rule == RULE)
            .all(|instance| !matches!(instance.outcome, Outcome::Skipped(_))),
        "a skipped instance"
    );
}

/// Where one relation is the only one that writes a state, no document can be
/// told two, and the rule generates no instance.
#[test]
fn a_taxonomy_with_one_setting_relation_generates_no_instance() {
    let single = run_with("state-set-twice-single.taxonomy.yml");
    let instances = single
        .instances
        .iter()
        .filter(|instance| instance.rule == RULE)
        .count();
    assert_eq!(instances, 0);
    // And against the full taxonomy it does run, so the zero above is the gate
    // and not a rule that never runs.
    let full = run();
    assert!(
        full.instances.iter().any(|instance| instance.rule == RULE),
        "the rule runs where two setters disagree"
    );
}
