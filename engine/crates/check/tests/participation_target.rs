// SPDX-License-Identifier: Apache-2.0
//! Which neighbour satisfies a participation window, when the relation admits
//! more target kinds than the expectation names.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#testing-a-check-without-a-failing-fixture-does-not-ship)
//! sets the floor: "every check ships with at least one fixture that it fails
//! and one that it passes." `relation.participation.overdue` already had both.
//! What it had no fixture for is the shape below, and that is how the defect
//! this file pins reached a merged branch unseen.
//!
//! `Expectation.to_kind` is an `Option<String>` and a relation's `to:` is a
//! list. So an expectation over a relation that admits two kinds can name only
//! one of them, and a document verified by the other is reported as reaching
//! nothing. The remediation text then tells an author to declare an edge they
//! have already correctly declared.
//!
//! The tree under `fixtures/participation-target/` holds two arms that differ
//! in one clause. `requirement_narrow` declares `to_kind: acceptance_criterion`
//! and `requirement_broad` declares no `to_kind` at all. Every other member of
//! the two kinds matches character for character, and both arms hold the same
//! five documents, so a difference between the two columns below can come from
//! nothing else.
//!
//! **The broad arm is a shape no taxonomy can currently declare, and that is
//! the point of writing it here.** `engine/crates/meta/meta-schema.yml` makes
//! `to_kind` a required member, so `Participation::satisfied`'s `None => true`
//! branch and `evaluate`'s second message format are both reachable from this
//! crate and from no `.headwater/overlay.yml` anywhere.
//! [HW-OBL-0039](../../../../docs/obligations/0039-a-participation-expectation-names-one-target-kind.md)
//! holds the obligation and
//! [HW-OBL-0128](../../../../docs/obligations/0128-nothing-holds-a-crate-to-having-a-contract-under-a-root-that-excludes-it.md)
//! records the same measurement from the other side. That record proposes an
//! abstract kind over the target kinds as the remedy; it does not run, because
//! expectation well-formedness refuses a `to_kind` that no shelf reaches.
//!
//! So this file is two things at once. It is the passing-and-failing pair that
//! spec 12 requires for a branch the engine implements, and it is the measured
//! cost of the member the meta-schema has not yet made optional. When that
//! member yields, the broad arm becomes declarable and this file already says
//! what changes.

use headwater_census::census;
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::{Cache, Context, Date, Declared, Register, Run, Shape};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use std::path::{Path, PathBuf};

/// As every other recorded run: a verdict is a function of the injected clock.
///
/// Every `current` document in the tree entered its state on `2026-01-05`,
/// which is 219 days before this date and so past the 90-day window, except
/// `inside-the-window.md`, which entered on `2026-08-01` and is 11 days inside
/// it.
const PINNED: &str = "2026-08-12";

const RULE: &str = headwater_check::participation::RULE;

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn run() -> Run {
    let corpus = Corpus::new(fixtures_dir(), "participation-target");
    let source = std::fs::read_to_string(fixtures_dir().join("participation-target.taxonomy.yml"))
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
            lock: "sha256:participation-target-fixture",
            taxonomy: &taxonomy,
            shape: &shape,
            relations: &declarations,
            config: &config,
            register: &register,
            adoption: None,
            source: "engine/crates/check/fixtures/participation-target.taxonomy.yml",
        },
        &headwater_check::claim::Claims::empty(),
        &Context::at(Date::parse(PINNED).expect("the pinned date")),
        &mut Cache::disabled())
}

/// The documents this rule reported, under one arm, as bare file names.
fn reported(run: &Run, arm: &str) -> Vec<String> {
    let prefix = format!("participation-target/{arm}/");
    let mut names: Vec<String> = run
        .findings
        .iter()
        .filter(|finding| finding.rule == RULE)
        .filter_map(|finding| finding.path.strip_prefix(&prefix))
        .map(|name| name.to_string())
        .collect();
    names.sort();
    names
}

/// The decisive case. One relation, two admitted target kinds, and the only
/// difference between the two arms is whether the expectation names one of
/// them.
///
/// `by-probe.md` is the whole subject. The relation admits a probe at its
/// target end, both arms declare the identical edge, and only the arm that
/// names a `to_kind` reports it. An expectation that read the relation's own
/// `to:` list rather than a scalar of its own would report neither.
#[test]
fn a_target_kind_the_relation_admits_and_the_expectation_does_not_name_is_reported_anyway() {
    let run = run();

    assert_eq!(
        reported(&run, "narrow"),
        vec!["by-nothing.md", "by-probe.md"],
        "the narrow arm reports the probe-verified document, which is the defect"
    );
    assert_eq!(
        reported(&run, "broad"),
        vec!["by-nothing.md"],
        "the broad arm reports only the document that reaches nothing"
    );
}

/// The remediation an author reads, and why the narrow arm's is the harm.
///
/// A finding is not only a count. The narrow arm tells an author to declare a
/// `verified_by` to an `acceptance_criterion` on a document that already
/// declares a `verified_by`, so acting on the message means adding a second
/// verifier to satisfy a rule rather than to settle a claim. The broad arm's
/// message names the relation alone, which is the only thing it can be owed.
#[test]
fn the_message_names_the_relation_alone_when_the_expectation_names_no_target_kind() {
    let run = run();
    let message = |arm: &str, name: &str| -> String {
        run.findings
            .iter()
            .find(|finding| {
                finding.rule == RULE && finding.path == format!("participation-target/{arm}/{name}")
            })
            .map(|finding| finding.message.clone())
            .unwrap_or_else(|| panic!("{arm}/{name} is reported"))
    };

    let narrow = message("narrow", "by-probe.md");
    assert!(
        narrow.contains("reaches no `verified_by` to a `acceptance_criterion`"),
        "the narrow arm names a target kind the document was never owed: {narrow}"
    );

    let broad = message("broad", "by-nothing.md");
    assert!(
        broad.contains("reaches no `verified_by` inside"),
        "the broad arm names the relation and no target kind: {broad}"
    );
    assert!(
        !broad.contains("acceptance_criterion"),
        "the broad arm names no target kind at all: {broad}"
    );
}

/// Widening the expectation does not disable it.
///
/// The three documents that must stay silent under the broad arm are silent
/// for three different reasons, and a fix that bought its silence by
/// generating nothing would take the fourth down with them. So the fixture
/// asserts the reported set exactly rather than asserting that one document
/// left it.
#[test]
fn the_broad_arm_still_reports_a_document_that_reaches_nothing_and_still_respects_the_window() {
    let run = run();

    // The window, the `when` clause, and a satisfied edge each keep one
    // document out of the report, and `by-nothing.md` has none of the three.
    assert_eq!(reported(&run, "broad"), vec!["by-nothing.md"]);

    // And the rule did generate over every document of both kinds, so the
    // silence above is a verdict rather than an absent instance. Ten documents
    // carry an expectation; the criterion and the probe declare none.
    let instances = run
        .instances
        .iter()
        .filter(|instance| instance.rule == RULE)
        .count();
    assert_eq!(
        instances, 10,
        "every document of a kind that declares an expectation gets an instance"
    );
}
