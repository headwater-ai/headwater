// SPDX-License-Identifier: Apache-2.0
//! A live document resting on a draft one, and the ways a rule about it is easy
//! to get wrong.
//!
//! The tree is the one `terminal_dependency.rs` reads, because the pass has to
//! be the same shape of edge as the fail. The cases added for this rule are:
//!
//! **A rule that copied its sibling's relation filter** reads only the
//! relations marked `lifecycle_sensitive`, and the published base marks nothing
//! beyond `succession`. It is silent on `live/mentions-draft.md`, whose relation
//! nothing marks, and that silence is the rule reaching no adopter of the base.
//!
//! **A rule that read the edge rather than the pair** reports
//! `records/draft-mentions-standing.md`, which is the same relation with the
//! draft at the other end.
//!
//! **A rule that did not read `on_target.set_state`** reports
//! `live/supersedes-draft.md`, where the edge writes a state onto its target
//! and is a statement about the draft rather than a reliance on it.
//!
//! **A rule that read a list of state names** calls `records/proposed.md`
//! nothing at all. `proposed` carries the role `initial` and the regime of its
//! kind does not name it, so the rule stands down and names the rule that owns
//! that defect.

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

/// Named as a literal rather than through the crate, so this table compiles
/// and fails on the rule's absence before the rule exists.
const RULE: &str = "lifecycle.dependency.on_initial";

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

/// One run over a fixture tree and the taxonomy of the same name.
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
            adoption: None,
            source: &source,
        },
        &headwater_check::claim::Claims::empty(),
        &Context::at(Date::parse(PINNED).expect("the pinned date")),
        &mut Cache::disabled(),
    )
}

fn run() -> Run {
    run_over("terminal-dependency")
}

/// The findings this rule reported, as `(path, message)`.
fn refusals(run: &Run) -> Vec<(&str, &str)> {
    run.findings
        .iter()
        .filter(|finding| finding.rule == RULE)
        .map(|finding| (finding.path.as_str(), finding.message.as_str()))
        .collect()
}

/// The one finding whose message names a document, and nothing when this rule
/// reported none about it.
fn about<'a>(run: &'a Run, id: &str) -> Option<&'a str> {
    refusals(run)
        .into_iter()
        .map(|(_, message)| message)
        .find(|message| message.contains(id))
}

/// The outcome of every instance of this rule that read a file.
fn outcomes_reading<'a>(run: &'a Run, path: &str) -> Vec<&'a Outcome> {
    run.instances
        .iter()
        .filter(|instance| instance.rule == RULE)
        .filter(|instance| instance.reads.iter().any(|input| input.path == path))
        .map(|instance| &instance.outcome)
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

/// The decisive case: a relation nothing marks `lifecycle_sensitive` reaches
/// the rule, and the report names the four things a reader needs.
///
/// `mentions` is `association`. No core requirement of this taxonomy marks
/// that family and the relation declares nothing for itself, so the terminal
/// rule generates no instance over it. The published base marks nothing beyond
/// `succession`, so a rule that read the same filter reaches no pair in any
/// adopter of it.
#[test]
fn a_live_document_mentioning_a_draft_one_is_reported() {
    let run = run();
    let message = about(&run, "NOTE-FIX-mentions-draft").expect("the decisive finding");
    assert!(
        message.contains("NOTE-FIX-argued-over"),
        "the target: {message}"
    );
    assert!(message.contains("`mentions`"), "the relation: {message}");
    assert!(message.contains("`draft`"), "the state: {message}");
    assert!(
        message.contains("`current`"),
        "the source's state: {message}"
    );

    let finding = run
        .findings
        .iter()
        .find(|finding| finding.rule == RULE && finding.message.contains("NOTE-FIX-mentions-draft"))
        .expect("the finding");
    assert_eq!(finding.path, "terminal-dependency/live/mentions-draft.md");
    assert_eq!(finding.severity, headwater_check::Severity::Warn);
}

/// The same relation, from the draft end onto a live record, passes.
#[test]
fn the_same_edge_from_a_draft_onto_a_live_document_passes() {
    let run = run();
    assert!(
        about(&run, "NOTE-FIX-draft-mentions-standing").is_none(),
        "{:?}",
        refusals(&run)
    );
    let outcomes = outcomes_reading(
        &run,
        "terminal-dependency/records/draft-mentions-standing.md",
    );
    assert_eq!(outcomes.len(), 1, "{outcomes:?}");
    assert!(matches!(outcomes[0], Outcome::Passed), "{outcomes:?}");
}

/// A relation that marks itself is read too: this rule does not read the word.
#[test]
fn a_relation_that_marks_itself_is_read_as_any_other() {
    let run = run();
    let message = about(&run, "NOTE-FIX-cites-draft").expect("the marked relation's finding");
    assert!(message.contains("`cites`"), "{message}");
}

/// A relation that writes a state onto its target is exempt outright.
///
/// `supersedes` declares `on_target: {set_state: superseded}`. An edge of it
/// is a statement about the target, which it retires, and never a reliance on
/// what the target says.
#[test]
fn a_relation_that_writes_its_target_s_state_is_exempt() {
    let run = run();
    assert!(
        about(&run, "NOTE-FIX-supersedes-draft").is_none(),
        "a supersession of a draft was reported: {:?}",
        refusals(&run)
    );
    let outcomes = outcomes_reading(&run, "terminal-dependency/live/supersedes-draft.md");
    assert_eq!(outcomes.len(), 1, "{outcomes:?}");
    assert!(matches!(outcomes[0], Outcome::Passed), "{outcomes:?}");
}

/// A draft resting on a draft is two documents nobody relies on yet.
#[test]
fn a_draft_source_onto_a_draft_target_passes() {
    let run = run();
    assert!(
        about(&run, "NOTE-FIX-draft-mentions-draft").is_none(),
        "{:?}",
        refusals(&run)
    );
    let outcomes = outcomes_reading(&run, "terminal-dependency/records/draft-mentions-draft.md");
    assert_eq!(outcomes.len(), 1, "{outcomes:?}");
    assert!(matches!(outcomes[0], Outcome::Passed), "{outcomes:?}");
}

/// A live or terminal target is not this rule's finding, and the pair passes.
#[test]
fn a_live_or_terminal_target_passes() {
    let run = run();
    for path in [
        "terminal-dependency/live/rests-on-standing.md",
        "terminal-dependency/live/mentions-retired.md",
        "terminal-dependency/live/succeeds-replaced.md",
    ] {
        let outcomes = outcomes_reading(&run, path);
        assert_eq!(outcomes.len(), 1, "{path}: {outcomes:?}");
        assert!(
            matches!(outcomes[0], Outcome::Passed),
            "{path}: {outcomes:?}"
        );
    }
}

/// A target that declares no state skips, and says which end had nothing to
/// read, as the terminal rule does.
#[test]
fn a_target_that_declares_no_state_skips_rather_than_passing() {
    let run = run();
    let outcomes = outcomes_reading(&run, "terminal-dependency/stubs/quiet.md");
    assert_eq!(outcomes.len(), 1, "{outcomes:?}");
    let Outcome::Skipped(why) = outcomes[0] else {
        panic!("the stateless target did not skip: {outcomes:?}");
    };
    assert!(why.contains("NOTE-FIX-quiet"), "{why}");
    assert!(why.contains("target end"), "{why}");
}

/// A target at an initial state its own regime does not name is left to the
/// rule that reads the document.
#[test]
fn a_target_at_a_state_its_regime_does_not_name_is_left_to_the_rule_that_owns_it() {
    let run = run();
    let outcomes = outcomes_reading(&run, "terminal-dependency/records/proposed.md");
    assert_eq!(outcomes.len(), 1, "{outcomes:?}");
    let Outcome::Skipped(why) = outcomes[0] else {
        panic!("the target outside its machine did not skip: {outcomes:?}");
    };
    assert!(
        why.contains(headwater_check::lifecycle_state::RULE),
        "the skip does not name the rule that owns it: {why}"
    );
    assert!(
        run.findings.iter().any(|finding| {
            finding.rule == headwater_check::lifecycle_state::RULE
                && finding.path == "terminal-dependency/records/proposed.md"
        }),
        "the rule this one defers to reported nothing"
    );
}

/// An anchor at the far end is not a document and has no state, so no pair
/// forms and no instance exists. The evidence-basis tree holds the one fixture
/// whose only edge reaches an anchor.
#[test]
fn an_edge_onto_an_anchor_generates_no_instance() {
    let run = run_over("evidence-basis");
    let read = read_by_instances(&run);
    assert!(
        !read.is_empty(),
        "the rule generated no instance in the tree, so the absence below says nothing"
    );
    assert!(
        !read.contains(&"evidence-basis/claims/traces-to-an-anchor.md"),
        "an instance exists over an anchor half: {read:?}"
    );
}

/// The author who writes the line is the one who cites, whichever name of the
/// relation they wrote.
///
/// `verified_by` runs from the verified document to the verifier, and its
/// inverse is `verifies`. `live/verifies-draft.md` writes `verifies` onto a
/// draft, so the declared source is the draft. A rule that read the declared
/// direction calls the source not live and passes. The document that wrote the
/// line is live, and it rests on a draft.
#[test]
fn a_live_document_writing_the_inverse_name_onto_a_draft_is_reported() {
    let run = run();
    let message =
        about(&run, "NOTE-FIX-verifies-draft").expect("the inverse-written finding");
    assert!(message.contains("`verifies`"), "the name written: {message}");
    assert!(
        message.contains("NOTE-FIX-argued-over"),
        "the draft: {message}"
    );
    // Writer first, far end second, in the direction the author wrote.
    let writer_at = message.find("NOTE-FIX-verifies-draft").expect("the writer");
    let draft_at = message.find("NOTE-FIX-argued-over").expect("the draft");
    assert!(writer_at < draft_at, "the ends are the wrong way round: {message}");
}

/// The mirror: a draft writes the inverse name onto a live document. The
/// declared direction runs from the live document to the draft, and the draft
/// wrote the line, so nothing is reported.
#[test]
fn a_draft_writing_the_inverse_name_onto_a_live_document_passes() {
    let run = run();
    assert!(
        about(&run, "NOTE-FIX-draft-verifies-standing").is_none(),
        "{:?}",
        refusals(&run)
    );
    let outcomes = outcomes_reading(
        &run,
        "terminal-dependency/records/draft-verifies-standing.md",
    );
    assert_eq!(outcomes.len(), 1, "{outcomes:?}");
    assert!(matches!(outcomes[0], Outcome::Passed), "{outcomes:?}");
}

/// A terminal source onto a draft passes. Only a live source is the finding,
/// and a rule that tested "not initial" rather than "live" reports this pair.
#[test]
fn a_terminal_source_onto_a_draft_target_passes() {
    let run = run();
    assert!(
        about(&run, "NOTE-FIX-retired-mentions-draft").is_none(),
        "{:?}",
        refusals(&run)
    );
    let outcomes = outcomes_reading(
        &run,
        "terminal-dependency/records/retired-mentions-draft.md",
    );
    assert_eq!(outcomes.len(), 1, "{outcomes:?}");
    assert!(matches!(outcomes[0], Outcome::Passed), "{outcomes:?}");
}

/// The whole tree, in one assertion, so a case that stops being reported
/// cannot hide behind a test that names only its own document.
#[test]
fn the_tree_reports_three_pairs_and_no_others() {
    let run = run();
    let mut reported: Vec<&str> = refusals(&run).into_iter().map(|(path, _)| path).collect();
    reported.sort_unstable();
    assert_eq!(
        reported,
        [
            "terminal-dependency/live/cites-draft.md",
            "terminal-dependency/live/mentions-draft.md",
            "terminal-dependency/live/verifies-draft.md",
        ]
    );
    // Advisory, for the reason `CT-LIFE-5` states: the repair is to promote the
    // target, re-point the source or move the source back to draft, and only an
    // author can choose.
    assert!(run
        .findings
        .iter()
        .filter(|finding| finding.rule == RULE)
        .all(|finding| finding.severity == headwater_check::Severity::Warn));
}
