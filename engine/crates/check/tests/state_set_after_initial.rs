// SPDX-License-Identifier: Apache-2.0
//! A replacing document left at its initial state, over an edge that writes a
//! state onto its target (#1973).
//!
//! HW-DR-0086 rules that a source at its initial state retires nothing yet, so
//! `lifecycle.state.not_set_by_edge` stays silent on the pair. This rule keeps
//! that clause and reports what the clause leaves pending, against the source.
//! The cases below are the ways a rule about it is easy to get wrong.
//!
//! **A rule that reported the target** puts the warning on the document that
//! nobody has to edit. The draft is the one an author must move.
//!
//! **A rule that read every relation** reports the `mentions` pair, which sets
//! no state and so has nothing pending.
//!
//! **A rule that ignored the source's state** reports every live successor,
//! such as `notes/draft-superseder.md` over the draft `notes/still-draft.md`.
//!
//! **A rule that offered a patch** moves a document out of draft, which is a
//! statement only its author can make.

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
const RULE: &str = "lifecycle.state.set_after_initial";

/// The neighboring rule, whose report this rule must leave unchanged.
const NEIGHBOR: &str = "lifecycle.state.not_set_by_edge";

/// The tree and taxonomy of `tests/state_not_set_by_edge.rs`, reused.
const TREE: &str = "state-not-set-by-edge";

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

/// One run over the fixture tree and the taxonomy of the same name.
fn run() -> Run {
    run_over(TREE)
}

/// One run over `tree` and the taxonomy of the same name.
fn run_over(tree: &str) -> Run {
    let corpus = Corpus::new(fixtures_dir(), tree);
    let source = std::fs::read_to_string(fixtures_dir().join(format!("{tree}.taxonomy.yml")))
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
    let lock = format!("sha256:{tree}-fixture");
    let source = format!("engine/crates/check/fixtures/{tree}.taxonomy.yml");
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
    format!("{TREE}/{file}")
}

/// Every finding of a rule, in report order.
fn findings<'a>(run: &'a Run, rule: &str) -> Vec<&'a Finding> {
    run.findings
        .iter()
        .filter(|finding| finding.rule == rule)
        .collect()
}

/// The decisive case: a draft that `supersedes` a current document gets one
/// warning, against the draft and at its state line, with no patch. The
/// message names the target, the target's state and the state to set.
#[test]
fn a_draft_successor_of_a_current_target_is_warned() {
    let run = run();
    let wanted = path("notes/draft-successor.md");
    let found: Vec<&Finding> = findings(&run, RULE)
        .into_iter()
        .filter(|finding| finding.path == wanted)
        .collect();
    assert_eq!(found.len(), 1, "{found:#?}");
    let finding = found[0];
    assert_eq!(finding.severity, Severity::Warn);
    assert!(finding.patch.is_none(), "{:?}", finding.patch);
    assert_eq!(finding.line, 3, "the anchor is the state facet's line");
    for wanted in [
        "only once `NOTE-FIX-draft-successor` leaves `draft`",
        "`NOTE-FIX-draft-target` stays at `current`",
        "set `status` to `current`",
    ] {
        assert!(
            finding.message.contains(wanted),
            "{wanted:?} missing from {}",
            finding.message
        );
    }
}

/// A live successor over a draft target is silent: the source is not at its
/// initial state, so nothing is pending. `notes/still-draft.md` is a draft
/// target, and the draft successor in this tree points elsewhere.
#[test]
fn a_live_successor_of_a_draft_target_is_silent() {
    let run = run();
    let reported: Vec<&str> = findings(&run, RULE)
        .iter()
        .map(|finding| finding.path.as_str())
        .collect();
    assert!(
        !reported.contains(&path("notes/draft-superseder.md").as_str()),
        "{reported:?}"
    );
    assert!(
        !reported.contains(&path("notes/still-draft.md").as_str()),
        "{reported:?}"
    );
}

/// A relation that sets no state forms no instance of this rule.
#[test]
fn a_relation_that_sets_no_state_is_not_read() {
    let run = run();
    let wanted = path("notes/mentioned.md");
    let reading: Vec<_> = run
        .instances
        .iter()
        .filter(|instance| instance.rule == RULE)
        .filter(|instance| instance.reads.iter().any(|input| input.path == wanted))
        .collect();
    assert!(reading.is_empty(), "{reading:#?}");
}

/// One draft over three targets: a draft target is silent (HW-DR-0085), a
/// target already at the set state is silent, and a target terminal at another
/// state is pending, so the draft is warned once, and only for that target.
#[test]
fn a_draft_is_warned_only_for_a_target_the_edge_would_move() {
    let run = run();
    let wanted = path("notes/draft-replacer.md");
    let found: Vec<&Finding> = findings(&run, RULE)
        .into_iter()
        .filter(|finding| finding.path == wanted)
        .collect();
    assert_eq!(found.len(), 1, "{found:#?}");
    let message = &found[0].message;
    assert!(
        message.contains("`NOTE-FIX-deprecated-early` stays at `deprecated`"),
        "{message}"
    );
    for silent in [
        "NOTE-FIX-draft-replaced",
        "NOTE-FIX-already-retired",
        "NOTE-FIX-drafted-page",
    ] {
        assert!(!message.contains(silent), "{message}");
    }
}

/// A generated target is skipped with a reason: its state is this engine's
/// to write, and `lifecycle.state.set_twice` and `generate --check` read it.
#[test]
fn a_generated_target_is_skipped() {
    let run = run();
    let wanted = path("pages/drafted-page.md");
    let outcomes: Vec<&Outcome> = run
        .instances
        .iter()
        .filter(|instance| instance.rule == RULE)
        .filter(|instance| instance.reads.iter().any(|input| input.path == wanted))
        .map(|instance| &instance.outcome)
        .collect();
    assert_eq!(outcomes.len(), 1, "{outcomes:?}");
    assert!(
        matches!(outcomes[0], Outcome::Skipped(reason) if reason.contains("generated")),
        "{outcomes:?}"
    );
}

/// The whole report of the rule over the tree, so a new finding is a failure
/// rather than a silence.
#[test]
fn the_rule_reports_exactly_the_two_drafts() {
    let run = run();
    let mut reported: Vec<&str> = findings(&run, RULE)
        .iter()
        .map(|finding| finding.path.as_str())
        .collect();
    reported.sort_unstable();
    assert_eq!(
        reported,
        vec![
            "state-not-set-by-edge/notes/draft-replacer.md",
            "state-not-set-by-edge/notes/draft-successor.md",
        ]
    );
}

/// The rule forms an instance for every edge of `supersedes`, and only the
/// two pending pairs fail.
#[test]
fn only_the_pending_pairs_fail() {
    let run = run();
    let ours: Vec<_> = run
        .instances
        .iter()
        .filter(|instance| instance.rule == RULE)
        .collect();
    assert!(ours.len() > 1, "{ours:#?}");
    let failed = ours
        .iter()
        .filter(|instance| matches!(instance.outcome, Outcome::Failed(_)))
        .count();
    assert_eq!(failed, 2, "{ours:#?}");
}

/// The neighbor keeps its report: HW-DR-0086's clause still holds, and
/// `lifecycle.state.not_set_by_edge` reports exactly its four targets.
#[test]
fn the_neighboring_rule_reports_exactly_its_four_targets() {
    let run = run();
    let mut reported: Vec<&str> = findings(&run, NEIGHBOR)
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

/// A second tree, with a copy of the same taxonomy, whose sources stand past
/// their initial state. The neighboring tree has none over a live target, because
/// there the neighbor would report each one and move its count of four.
const SOURCES: &str = "state-set-after-initial-sources";

/// A source that left its initial state has nothing pending, whether it is
/// live or not: a deprecated and a superseded source over a `current` target
/// stay silent, and the draft control in the same tree is warned. A gate that
/// passed only live sources would warn the two terminal ones (#1973, round 1).
#[test]
fn a_source_past_its_initial_state_is_silent_whatever_its_standing() {
    let run = run_over(SOURCES);
    let reported: Vec<&str> = findings(&run, RULE)
        .iter()
        .map(|finding| finding.path.as_str())
        .collect();
    assert_eq!(
        reported,
        vec!["state-set-after-initial-sources/notes/draft-source.md"]
    );
    let ours: Vec<_> = run
        .instances
        .iter()
        .filter(|instance| instance.rule == RULE)
        .collect();
    let passed = ours
        .iter()
        .filter(|instance| matches!(instance.outcome, Outcome::Passed))
        .count();
    assert_eq!(passed, 2, "{ours:#?}");
}
