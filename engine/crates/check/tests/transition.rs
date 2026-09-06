// SPDX-License-Identifier: Apache-2.0
//! The state movements a declared lifecycle admits, and the corpus that
//! separates them from the movements it refuses.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#testing-a-check-without-a-failing-fixture-does-not-ship)
//! sets the floor: "every check ships with at least one fixture that it fails
//! and one that it passes." The tree under `fixtures/transition/` is written to
//! be wrong under the mistake this rule exists to avoid: every record in it
//! stands at a state the vocabulary holds, and `permitted.md` and `refused.md`
//! read alike in the front matter that a full-corpus run can see. Only the
//! prior version tells them apart.
//!
//! Six properties, and each one names the defect it fails under.

use headwater_census::census;
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::change::{Change, Unbound};
use headwater_check::{Cache, Context, Date, Declared, Outcome, Register, Run, Shape};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use std::path::{Path, PathBuf};

/// As every other recorded run: a verdict is a function of the injected clock.
const PINNED: &str = "2026-08-12";

const RULE: &str = headwater_check::transition::RULE;

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

/// The corpus paths the fixture tree holds. Written out rather than walked, for
/// the reason `promotion.rs` writes its own out: a test that took the set from
/// the walk the run takes would bind every path a manifest could name.
const HELD: [&str; 6] = [
    "transition/notes/exempt.md",
    "transition/records/out-of-terminal.md",
    "transition/records/permitted.md",
    "transition/records/refused.md",
    "transition/records/skipped.md",
    "transition/records/unmoved.md",
];

/// A manifest naming one document of the tree and the version that stood
/// before it. `slug` names both, because the fixture pairs them by name.
fn manifest(entries: &[(&str, &str)]) -> String {
    let mut text = String::from("headwater change 1\n");
    for (path, slug) in entries {
        text.push_str(&format!(
            "prior\t{path}\t{}\n",
            fixtures_dir()
                .join("before-transition")
                .join(format!("{slug}.md"))
                .display()
        ));
    }
    text
}

fn change(entries: &[(&str, &str)]) -> Change {
    Unbound::read(&manifest(entries), |path| std::fs::read(path))
        .expect("the manifest reads")
        .bind(|path| HELD.contains(&path))
}

fn run_with(ctx: &Context) -> Run {
    let corpus = Corpus::new(fixtures_dir(), "transition");
    let source = std::fs::read_to_string(fixtures_dir().join("transition.taxonomy.yml"))
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
            lock: "sha256:transition-fixture",
            taxonomy: &taxonomy,
            shape: &shape,
            relations: &declarations,
            config: &config,
            register: &register,
            adoption: None,
            source: "engine/crates/check/fixtures/transition.taxonomy.yml",
        },
        &headwater_check::claim::Claims::empty(),
        ctx,
        &mut Cache::disabled(),
    )
}

fn over(entries: &[(&str, &str)]) -> Run {
    let now = Date::parse(PINNED).expect("the pinned date");
    run_with(&Context::over(now, change(entries)))
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
/// created over. A document check reads the document it is instantiated over,
/// so the first input of the read set is that document.
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

/// One movement is refused and one is accepted, and the front matter of the two
/// documents is the same shape.
///
/// The decisive fixture. Both stand at a state the vocabulary holds, both are
/// carried by the change, and both belong to the kind that binds the regime. A
/// rule that read the current state alone reports either two or none. Only the
/// prior version separates them.
#[test]
fn a_movement_the_regime_does_not_declare_is_refused_and_one_it_declares_is_not() {
    let run = over(&[
        ("transition/records/permitted.md", "permitted"),
        ("transition/records/refused.md", "refused"),
    ]);
    let refusals = refusals(&run);
    assert_eq!(
        refusals.len(),
        1,
        "the current state is not the movement: {refusals:?}"
    );
    let (path, message) = refusals[0];
    assert_eq!(path, "transition/records/refused.md");

    // The message names the record and both states, because a reader who is
    // told only that a transition was refused has to reconstruct which one.
    assert!(message.contains("NOTE-FIX-refused"), "{message}");
    assert!(message.contains("`draft`"), "{message}");
    assert!(message.contains("`superseded`"), "{message}");
    assert!(message.contains("`standard`"), "{message}");
    // And it names what the state it left does admit, which is the shortest
    // route to the correction.
    assert!(message.contains("admits `current`"), "{message}");
}

/// A movement out of a terminal state is refused, and the message says the
/// state is terminal rather than listing an empty set.
///
/// The other shape of the same defect. A state the `transitions` map does not
/// name has no exits, and a report that rendered that as "admits " with nothing
/// after it would send a reader to look for the list that was cut off.
#[test]
fn a_movement_out_of_a_terminal_state_is_refused_and_says_the_state_is_terminal() {
    let run = over(&[("transition/records/out-of-terminal.md", "out-of-terminal")]);
    let refusals = refusals(&run);
    assert_eq!(refusals.len(), 1, "{refusals:?}");
    let (path, message) = refusals[0];
    assert_eq!(path, "transition/records/out-of-terminal.md");
    assert!(
        message.contains("is terminal in `standard` and admits none"),
        "{message}"
    );
}

/// A document the change carries whose state did not move passes.
///
/// No regime here declares a self edge, so a rule that asked the machine
/// whether `current` reaches `current` would refuse every document a change
/// touches without moving. That is most of them.
#[test]
fn a_document_whose_state_did_not_move_passes() {
    let run = over(&[("transition/records/unmoved.md", "unmoved")]);
    assert_eq!(refusals(&run), Vec::new());
    assert_eq!(skips(&run).len(), 0, "the unmoved document skipped");
}

/// A kind that binds no lifecycle regime generates no instance at all.
///
/// `exempt.md` moves from `draft` to `superseded` in the same change, which is
/// the movement `refused.md` is refused for. Its kind declares no machine, so
/// there is no edge to hold it to and no instance to report a skip on. A rule
/// that instantiated over every kind would claim coverage of a document it
/// could only ever pass.
#[test]
fn a_kind_that_binds_no_lifecycle_regime_generates_no_instance() {
    let run = over(&[("transition/notes/exempt.md", "exempt")]);
    assert_eq!(refusals(&run), Vec::new());
    let targets: Vec<&str> = run
        .instances
        .iter()
        .filter(|instance| instance.rule == RULE)
        .map(over_document)
        .collect();
    assert!(
        !targets.contains(&"transition/notes/exempt.md"),
        "an instance exists over a kind with no machine: {targets:?}"
    );
    assert_eq!(targets.len(), 5, "one instance per record: {targets:?}");
}

/// A state the vocabulary does not hold is skipped with a reason, and never
/// reported as a movement.
///
/// `facet.value.not_permitted` reports the value, at the line that carries it.
/// A transition rule that took an unknown value for a state would put a second
/// finding on the same line, naming a machine that never had an edge to offer.
/// The skip is what makes the decline visible rather than a pass.
#[test]
fn a_state_the_vocabulary_does_not_hold_is_skipped_rather_than_read_as_a_movement() {
    let run = over(&[("transition/records/skipped.md", "skipped")]);
    assert_eq!(refusals(&run), Vec::new());
    let skips = skips(&run);
    let mine: Vec<&(&str, &String)> = skips
        .iter()
        .filter(|(target, _)| *target == "transition/records/skipped.md")
        .collect();
    assert_eq!(mine.len(), 1, "{skips:?}");
    assert!(mine[0].1.contains("`retired`"), "{}", mine[0].1);
    assert!(
        mine[0].1.contains("facet.value.not_permitted"),
        "the skip does not name the rule that owns the value: {}",
        mine[0].1
    );
}

/// A run with no change reports every instance as skipped, and never as passed.
///
/// [Spec 12](../../../../docs/spec/12-check-layer.md#temporal-inputs-the-clock-and-the-prior-version):
/// "In a full-corpus run, instances of such a check are counted and reported as
/// skipped, with the reason `change-scoped-only` — visible, never silent." This
/// repository runs no change-scoped gate today, so this is the mode every one
/// of these instances is actually in.
#[test]
fn a_full_corpus_run_reports_every_instance_as_skipped() {
    let run = run_with(&Context::at(Date::parse(PINNED).expect("a date")));
    assert!(run.change.is_none(), "a run with no change states one");
    assert_eq!(refusals(&run), Vec::new());
    let skips = skips(&run);
    assert_eq!(skips.len(), 5, "one instance per record: {skips:?}");
    for (target, why) in &skips {
        assert!(
            why.starts_with(headwater_check::scope::CHANGE_SCOPED_ONLY),
            "{target} skipped for another reason: {why}"
        );
    }
}
