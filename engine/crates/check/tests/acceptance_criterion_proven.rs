// SPDX-License-Identifier: Apache-2.0
//! The decisive fixture for #935 / HW-DR-0073 ruling 1.
//!
//! `relations.verified_by` reaches an `acceptance_criterion` from a
//! `requirement`, and it stops there: no relation reached a `verification` and
//! no participation expectation stood on `acceptance_criterion` at all. So "which
//! criteria does no test reach" was a question no rule could ask. This file
//! proves that the generic `relation.participation.overdue` rule
//! (`engine/crates/check/src/participation.rs`) reports exactly that gap once
//! the taxonomy declares the new relation and the expectation over it, with no
//! new Rust rule logic.
//!
//! `fixtures/acceptance-criterion-proven/` holds four documents that differ
//! only in the clause the expectation reads. `overdue.md` is
//! `verification_method: test`, `status: current`, past the 90-day window,
//! with no `proven_by` edge, and it is the one document this rule must report.
//! `inside-window.md` holds the identical absence and stays silent because its
//! window is still open. `proven.md` holds the identical age and stays silent
//! because it declares the edge to `verifications/one.md`. `inspection.md`
//! holds the identical absence and age and stays silent because its
//! `verification_method` is `inspection` rather than `test`: HW-DR-0073 leaves
//! that cadence open, so the expectation does not read it at all.

use headwater_census::census;
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::{Cache, Context, Date, Declared, Register, Run, Shape};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use std::path::{Path, PathBuf};

/// Every `current` document in the tree entered its state on `2026-01-05`,
/// which is 219 days before this date and so past the 90-day window, except
/// `inside-window.md`, which entered on `2026-08-01` and is 11 days inside it.
/// The same pinned date and the same two entry dates as
/// `tests/participation_target.rs`, so a reader of both files reads one clock.
const PINNED: &str = "2026-08-12";

const RULE: &str = headwater_check::participation::RULE;

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn run() -> Run {
    let corpus = Corpus::new(fixtures_dir(), "acceptance-criterion-proven");
    let source =
        std::fs::read_to_string(fixtures_dir().join("acceptance-criterion-proven.taxonomy.yml"))
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
            lock: "sha256:acceptance-criterion-proven-fixture",
            taxonomy: &taxonomy,
            shape: &shape,
            relations: &declarations,
            config: &config,
            register: &register,
            adoption: None,
            source: "engine/crates/check/fixtures/acceptance-criterion-proven.taxonomy.yml",
        },
        &headwater_check::claim::Claims::empty(),
        &Context::at(Date::parse(PINNED).expect("the pinned date")),
        &mut Cache::disabled(),
    )
}

/// The one document the rule must name, and the three ways the others clear
/// it: an open window, a declared edge, and a `verification_method` the
/// expectation does not read.
#[test]
fn a_criterion_no_verification_reaches_is_reported_and_the_window_the_edge_or_the_method_clears_it(
) {
    let run = run();
    let mut reported: Vec<&str> = run
        .findings
        .iter()
        .filter(|finding| finding.rule == RULE)
        .map(|finding| finding.path.as_str())
        .collect();
    reported.sort_unstable();
    assert_eq!(
        reported,
        ["acceptance-criterion-proven/criteria/overdue.md"],
        "only the test-method criterion with no edge, past the window, is reported"
    );

    // Every criterion carries an instance, so the silence of the other three
    // is a verdict and not an absent generation.
    let instances = run
        .instances
        .iter()
        .filter(|instance| instance.rule == RULE)
        .count();
    assert_eq!(
        instances, 4,
        "every acceptance_criterion document gets an instance"
    );

    // The inspection arm specifically: past the window, no edge, and silent
    // because the expectation only reads `verification_method: test`.
    let inspection = run
        .instances
        .iter()
        .find(|instance| {
            instance.rule == RULE && instance.at() == "acceptance-criterion-proven/criteria/inspection.md"
        })
        .expect("the inspection fixture");
    assert!(
        inspection.findings().is_empty(),
        "an inspection criterion is outside the expectation's `when`, not merely inside its window"
    );
}
