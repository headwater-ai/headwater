// SPDX-License-Identifier: Apache-2.0
//! An authored document that an edge declares superseded, and that still reads
//! as live.
//!
//! [Spec 2](../../../../docs/spec/02-taxonomy-model.md) says that
//! `on_target: {set_state: superseded}` sets the target's state-entry date in
//! the same operation, and that a state change with no stamp is a defect
//! whatever caused it. Before this rule, the engine wrote that state onto a
//! generated target and nothing read an authored one. The cases below are the
//! ways a rule about it is easy to get wrong.
//!
//! **A rule that read the declaring document as the source** reports
//! `notes/inverse-successor.md`, or nothing, where the only half is
//! `superseded_by` written from the target end.
//!
//! **A rule that ignored the source's state** reports `notes/draft-target.md`,
//! whose successor is still a draft and retires nothing yet (HW-DR-0086).
//!
//! **A rule that read generated pages** reports `pages/gen-page.md`, which
//! `lifecycle.state.set_twice` and `generate --check` already own.
//!
//! **A fix that invented a date** writes a stamp where the successor declares
//! none. Spec 2 says `--fix` never reconstructs a missing date after the fact,
//! so that finding carries no patch.

use headwater_census::census;
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::{
    Cache, Context, Date, Declared, Finding, Outcome, Patch, Register, Run, Severity, Shape,
};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use std::path::{Path, PathBuf};

/// As every other recorded run: a verdict is a function of the injected clock.
const PINNED: &str = "2026-08-12";

/// Named as a literal rather than through the crate, so this table compiles
/// and fails on the rule's absence before the rule exists.
const RULE: &str = "lifecycle.state.not_set_by_edge";

const TREE: &str = "state-not-set-by-edge";

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

/// Every finding of this rule, in report order.
fn findings(run: &Run) -> Vec<&Finding> {
    run.findings
        .iter()
        .filter(|finding| finding.rule == RULE)
        .collect()
}

/// The one finding reported against a file, and nothing when there is none.
fn against<'a>(run: &'a Run, file: &str) -> Option<&'a Finding> {
    let wanted = path(file);
    let mut found = findings(run)
        .into_iter()
        .filter(|finding| finding.path == wanted);
    let first = found.next();
    assert!(found.next().is_none(), "two findings against {wanted}");
    first
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

/// The decisive case. A live successor declares `supersedes` at an authored
/// target that still reads `current`, and the rule reports the target.
#[test]
fn an_authored_target_left_current_under_a_supersedes_edge_is_reported() {
    let run = run();
    let finding = against(&run, "notes/left-current.md").expect("the decisive finding");
    assert_eq!(finding.severity, Severity::Error);
    assert!(
        finding.message.contains("NOTE-FIX-left-current"),
        "the target: {}",
        finding.message
    );
    assert!(
        finding.message.contains("NOTE-FIX-successor"),
        "the successor: {}",
        finding.message
    );
    assert!(
        finding.message.contains("`supersedes`"),
        "the relation: {}",
        finding.message
    );
    assert!(
        finding.message.contains("`superseded`"),
        "the state the edge sets: {}",
        finding.message
    );
    assert!(
        finding.message.contains("`current`"),
        "the state it reads: {}",
        finding.message
    );
    // The anchor is the state line of the target, which is what a reader edits.
    assert_eq!(finding.line, 3, "{finding:?}");
}

/// The patch writes the state and the successor's own stamp, both or neither,
/// and each write states the value it expects to replace.
#[test]
fn the_finding_s_patch_carries_the_state_and_the_successor_s_stamp() {
    let run = run();
    let finding = against(&run, "notes/left-current.md").expect("the finding");
    let Some(Patch::Facets { path: file, set }) = &finding.patch else {
        panic!("a facet patch: {:?}", finding.patch);
    };
    assert_eq!(file, &path("notes/left-current.md"));
    assert_eq!(
        set,
        &vec![
            (
                "status".to_string(),
                "current".to_string(),
                "superseded".to_string()
            ),
            (
                "status_since".to_string(),
                "2026-07-01".to_string(),
                "2026-08-05".to_string()
            ),
        ]
    );
}

/// A target that already carries the state the edge sets is silent.
#[test]
fn a_target_already_at_the_set_state_is_silent() {
    let run = run();
    assert!(against(&run, "notes/retired.md").is_none());
    let outcomes = outcomes_reading(&run, "notes/retired.md");
    assert_eq!(outcomes.len(), 1, "{outcomes:?}");
    assert!(matches!(outcomes[0], Outcome::Passed), "{outcomes:?}");
}

/// A draft successor retires nothing yet (HW-DR-0086), so its target is silent.
#[test]
fn a_draft_successor_is_silent() {
    let run = run();
    assert!(against(&run, "notes/draft-target.md").is_none());
    let outcomes = outcomes_reading(&run, "notes/draft-target.md");
    assert_eq!(outcomes.len(), 1, "{outcomes:?}");
    assert!(matches!(outcomes[0], Outcome::Passed), "{outcomes:?}");
}

/// A target still at its initial state is silent. The regime gives a draft no
/// movement to `superseded`, so the fix would write a transition the lifecycle
/// refuses, and HW-DR-0085 leaves the same pair unreported. The tutorial
/// walks a reader through this case.
#[test]
fn a_draft_target_of_a_live_successor_is_silent() {
    let run = run();
    assert!(against(&run, "notes/still-draft.md").is_none());
    let outcomes = outcomes_reading(&run, "notes/still-draft.md");
    assert_eq!(outcomes.len(), 1, "{outcomes:?}");
    assert!(matches!(outcomes[0], Outcome::Passed), "{outcomes:?}");
}

/// The half written only as `superseded_by`, from the target end, is still
/// the same edge in the same direction, and it is still reported.
#[test]
fn a_half_written_only_from_the_target_end_is_still_reported() {
    let run = run();
    let finding = against(&run, "notes/inverse-target.md").expect("the inverse finding");
    assert!(
        finding.message.contains("NOTE-FIX-inverse-successor"),
        "{}",
        finding.message
    );
    assert!(against(&run, "notes/inverse-successor.md").is_none());
    let Some(Patch::Facets { set, .. }) = &finding.patch else {
        panic!("a facet patch: {:?}", finding.patch);
    };
    assert_eq!(set[1].2, "2026-08-08");
}

/// A generated target is not read: its state is this engine's to write.
#[test]
fn a_generated_target_is_not_read() {
    let run = run();
    assert!(against(&run, "pages/gen-page.md").is_none());
    let outcomes = outcomes_reading(&run, "pages/gen-page.md");
    assert_eq!(outcomes.len(), 1, "{outcomes:?}");
    assert!(
        matches!(outcomes[0], Outcome::Skipped(reason) if reason.contains("generated")),
        "{outcomes:?}"
    );
}

/// A successor that declares no stamp still reports its target, and the
/// finding carries no patch, because the fix never invents a date.
#[test]
fn no_patch_where_the_successor_declares_no_stamp() {
    let run = run();
    let finding = against(&run, "notes/unstamped-target.md").expect("the unstamped finding");
    assert_eq!(finding.patch, None);
    assert!(
        finding.remediation.contains("status_since"),
        "the remediation names the missing stamp: {}",
        finding.remediation
    );
}

/// A target at a terminal state other than the one the edge sets is reported
/// with no patch: which terminal state it ends at is the author's call.
#[test]
fn a_target_at_another_terminal_state_is_reported_with_no_patch() {
    let run = run();
    let finding = against(&run, "notes/deprecated-target.md").expect("the finding");
    assert_eq!(finding.patch, None);
    // Advisory: the remedy is a judgment between two terminal states, so it
    // is not mechanical and total, and it must not fail a strict run.
    assert_eq!(finding.severity, Severity::Warn);
    assert!(
        finding.message.contains("`deprecated`"),
        "{}",
        finding.message
    );
}

/// A target whose kind binds no regime that names the set state is skipped,
/// and the reason names the record that owns the question.
#[test]
fn a_target_whose_kind_names_no_such_state_is_skipped() {
    let run = run();
    assert!(against(&run, "loose/loose-target.md").is_none());
    let outcomes = outcomes_reading(&run, "loose/loose-target.md");
    assert_eq!(outcomes.len(), 1, "{outcomes:?}");
    assert!(
        matches!(outcomes[0], Outcome::Skipped(reason) if reason.contains("HW-OBL-0196")),
        "{outcomes:?}"
    );
}

/// A relation that sets no state forms no instance of this rule.
#[test]
fn a_relation_that_sets_no_state_is_not_read() {
    let run = run();
    assert!(outcomes_reading(&run, "notes/mentioned.md").is_empty());
}

/// The whole report of the rule over the tree, so a new finding is a failure
/// rather than a silence.
#[test]
fn the_rule_reports_exactly_the_four_targets() {
    let run = run();
    let mut reported: Vec<&str> = findings(&run)
        .iter()
        .map(|finding| finding.path.as_str())
        .collect();
    reported.sort_unstable();
    assert_eq!(
        reported,
        vec![
            "state-not-set-by-edge/notes/deprecated-target.md",
            "state-not-set-by-edge/notes/inverse-target.md",
            "state-not-set-by-edge/notes/left-current.md",
            "state-not-set-by-edge/notes/unstamped-target.md",
        ]
    );
}
