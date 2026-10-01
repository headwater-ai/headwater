// SPDX-License-Identifier: Apache-2.0
//! Two documents joined by a relation whose declared `invalid_when` condition
//! holds at both ends.
//!
//! [Spec 2](../../../../docs/spec/02-taxonomy-model.md) declares
//! `invalid_when: {both: {status: current}}` on `conflicts_with` and names two
//! `current` decisions joined by it "an incoherent corpus state". Before this
//! rule, the engine read the declaration only to count the facet as read, and a
//! corpus with two live, contradictory decisions checked clean (HW-OBL-0042).
//! The cases below are the ways a rule about it is easy to get wrong.
//!
//! **A rule keyed on the relation's name** reports `constrainer.md`, whose
//! relation declares no condition, or misses `clash-source.md`, whose relation
//! is not `conflicts_with`.
//!
//! **A rule that read `status` or the state-role facet** in place of the facet
//! the condition names misses `clash-source.md`, whose ends are a draft and a
//! superseded decision, and reports `quiet-clash-source.md`, whose ends are both
//! `current`.
//!
//! **A rule that held the condition when any facet held** reports
//! `half-source.md`, whose far end holds one facet of the two.
//!
//! **A rule that de-duplicated a symmetric pair** reports one of `mutual-a.md`
//! and `mutual-b.md`, and the author of the other file never learns of it.
//!
//! **A rule that read a missing facet as held** reports `unfaceted-source.md`,
//! whose far end declares no `lifecycle`.
//!
//! **A rule that anchored on the inverse half first** reports
//! `outranks-both-target.md` in place of `outranks-both-source.md`, and **a
//! rule that read the declared half alone** misses
//! `outranks-inverse-target.md`, which wrote only the inverse.

use headwater_census::census;
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::{
    Cache, Context, Date, Declared, Finding, Outcome, Register, Run, Severity, Shape,
};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use std::path::{Path, PathBuf};

/// As every other recorded run: a verdict is a function of the injected clock.
const PINNED: &str = "2026-08-12";

/// Named as a literal rather than through the crate, so this table compiles
/// and fails on the rule's absence before the rule exists.
const RULE: &str = "relation.pair.invalid";

const TREE: &str = "conflicting-pair";

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

/// One run over the fixture tree and the taxonomy of the same name.
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
            pin: None,
            harvests: &[],
            imports: &[],
            adoption: None,
            source: &source,
        },
        &headwater_check::claim::Claims::empty(),
        &Context::at(Date::parse(PINNED).expect("the pinned date")),
        &mut Cache::disabled(),
    )
}

fn path(file: &str) -> String {
    format!("{TREE}/decisions/{file}")
}

/// Every finding of this rule, in report order.
fn findings(run: &Run) -> Vec<&Finding> {
    run.findings
        .iter()
        .filter(|finding| finding.rule == RULE)
        .collect()
}

/// The findings of this rule reported against one file.
fn against<'a>(run: &'a Run, file: &str) -> Vec<&'a Finding> {
    let wanted = path(file);
    findings(run)
        .into_iter()
        .filter(|finding| finding.path == wanted)
        .collect()
}

/// The outcome of every instance of this rule that read a file.
fn outcomes_reading<'a>(run: &'a Run, file: &str) -> Vec<&'a Outcome> {
    let wanted = path(file);
    run.instances
        .iter()
        .filter(|instance| instance.rule == RULE)
        .filter(|instance| instance.reads.iter().any(|input| input.path == wanted))
        .map(|instance| &instance.outcome)
        .collect()
}

/// The decisive case. Two `current` decisions declare `conflicts_with` at each
/// other, and each file carries a finding on the entry its author wrote.
#[test]
fn two_current_decisions_joined_by_conflicts_with_are_reported_on_both_files() {
    let run = run();
    for (file, other) in [("mutual-a.md", "mutual-b"), ("mutual-b.md", "mutual-a")] {
        let found = against(&run, file);
        assert_eq!(found.len(), 1, "one finding on {file}: {found:?}");
        let finding = found[0];
        assert_eq!(finding.severity, Severity::Warn, "{finding:?}");
        assert!(finding.patch.is_none(), "a judgment carries no patch");
        assert!(
            finding.message.contains(&format!("DEC-FIX-{other}")),
            "the far end: {}",
            finding.message
        );
        assert!(
            finding.message.contains("`conflicts_with`"),
            "the relation: {}",
            finding.message
        );
        assert!(
            finding.message.contains("`status: current`"),
            "the condition: {}",
            finding.message
        );
        // The anchor is the entry the author wrote, under `conflicts_with:`.
        assert_eq!(finding.line, 6, "{finding:?}");
    }
}

/// One declared entry is one finding, on the file that wrote it. A conflict
/// written from one end reports that end and not the other.
#[test]
fn a_one_sided_conflict_is_one_finding_on_the_declaring_file() {
    let run = run();
    assert_eq!(against(&run, "one-sided-z.md").len(), 1);
    assert!(against(&run, "one-sided-a.md").is_empty());
}

/// The total is one finding per declared entry whose condition holds, and
/// nothing else in the tree reports.
#[test]
fn the_count_is_one_per_declared_entry() {
    let run = run();
    let mut paths: Vec<&str> = findings(&run)
        .iter()
        .map(|finding| finding.path.as_str())
        .collect();
    paths.sort_unstable();
    assert_eq!(
        paths,
        vec![
            path("both-source.md"),
            path("clash-source.md"),
            path("mutual-a.md"),
            path("mutual-b.md"),
            path("one-sided-z.md"),
            path("outranks-both-source.md"),
            path("outranks-inverse-target.md"),
        ]
    );
}

/// The remedy names only edits that clear the finding. Superseding one end
/// moves its `status` off `current`, and the condition no longer holds. An
/// `overrides` edge leaves the loser `current` (Q18), so the rule still fires,
/// and a taxonomy may not declare `overrides` at all.
#[test]
fn the_remedy_names_only_edits_that_clear_the_finding() {
    let run = run();
    for finding in findings(&run) {
        assert!(
            !finding.remediation.contains("overrides"),
            "an `overrides` edge leaves both ends current: {}",
            finding.remediation
        );
        assert!(
            finding.remediation.contains("supersede"),
            "the remedy names supersession: {}",
            finding.remediation
        );
    }
    let found = against(&run, "clash-source.md");
    assert!(
        found[0].remediation.contains("`lifecycle: live`"),
        "the remedy names the condition to move off: {}",
        found[0].remediation
    );
}

/// A facet one end does not declare is a condition that does not hold there.
/// `unfaceted-source.md` is `lifecycle: live` and its far end declares no
/// `lifecycle`, so the instance passes and nothing is reported.
#[test]
fn a_facet_the_far_end_does_not_declare_is_a_pass() {
    let run = run();
    assert!(against(&run, "unfaceted-source.md").is_empty());
    let outcomes = outcomes_reading(&run, "unfaceted-target.md");
    assert_eq!(outcomes.len(), 1, "{outcomes:?}");
    assert!(matches!(outcomes[0], Outcome::Passed), "{outcomes:?}");
}

/// A relation with an inverse, written from both ends, is one instance and one
/// finding, on the file that wrote the declared half.
#[test]
fn an_instance_written_from_both_ends_anchors_on_the_declared_half() {
    let run = run();
    let found = against(&run, "outranks-both-source.md");
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].line, 6, "{:?}", found[0]);
    assert!(against(&run, "outranks-both-target.md").is_empty());
    let outcomes = outcomes_reading(&run, "outranks-both-target.md");
    assert_eq!(
        outcomes.len(),
        1,
        "one instance for both halves: {outcomes:?}"
    );
}

/// The same relation written only as its inverse reports on the file that
/// wrote the inverse, because no other file carries an entry to anchor on.
#[test]
fn an_instance_written_only_as_the_inverse_anchors_on_the_inverse_half() {
    let run = run();
    let found = against(&run, "outranks-inverse-target.md");
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].line, 6, "{:?}", found[0]);
    assert!(
        found[0].message.contains("`DEC-FIX-outranks-inverse-source` declares `outranks` to `DEC-FIX-outranks-inverse-target`"),
        "the message names the ends in the declared direction: {}",
        found[0].message
    );
    assert!(against(&run, "outranks-inverse-source.md").is_empty());
}

/// A pair with one end superseded does not hold the condition.
#[test]
fn a_superseded_end_is_silent() {
    let run = run();
    assert!(against(&run, "live-one.md").is_empty());
    let outcomes = outcomes_reading(&run, "retired-one.md");
    assert_eq!(outcomes.len(), 1, "{outcomes:?}");
    assert!(matches!(outcomes[0], Outcome::Passed), "{outcomes:?}");
}

/// A relation that declares no condition forms no instance of the rule, even
/// between two `current` ends.
#[test]
fn a_relation_with_no_condition_forms_no_instance() {
    let run = run();
    assert!(against(&run, "constrainer.md").is_empty());
    assert!(outcomes_reading(&run, "constrained.md").is_empty());
}

/// The condition names `lifecycle`, and the rule reads `lifecycle`: a draft and
/// a superseded decision that are both `live` are reported, and two `current`
/// decisions with one end `retired` are not.
#[test]
fn the_rule_reads_the_facet_the_condition_names() {
    let run = run();
    let found = against(&run, "clash-source.md");
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].message.contains("`lifecycle: live`"),
        "the condition: {}",
        found[0].message
    );
    assert!(against(&run, "quiet-clash-source.md").is_empty());
}

/// A condition of two facets holds only where both hold at both ends.
#[test]
fn every_facet_of_the_condition_must_hold_at_both_ends() {
    let run = run();
    assert_eq!(against(&run, "both-source.md").len(), 1);
    assert!(against(&run, "half-source.md").is_empty());
}
