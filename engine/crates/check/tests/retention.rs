// SPDX-License-Identifier: Apache-2.0
//! The deletion of a retained document, and the corpus that separates it from
//! every other path a change names that no census row holds.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#testing-a-check-without-a-failing-fixture-does-not-ship)
//! sets the floor: "every check ships with at least one fixture that it fails
//! and one that it passes." This rule is harder to hold to that bar than a
//! defect in a document is, because its subject is a document that is not
//! there. Every case below therefore names the mistake it fails under.
//!
//! The tree under `fixtures/retention/` is what the run walks, and it holds
//! three documents that survive the change. Every departure is a manifest
//! entry naming a path that tree does not hold, with the version that stood
//! there under `fixtures/retention-before/`.
//!
//! Three regimes differ only in what they say about `retain_terminal`, and
//! that is the whole instrument. `keeps` declares it `true`, `releases`
//! declares it `false`, and `silent` declares nothing. A rule that read the
//! member's presence rather than its value passes every case but one, and a
//! rule that read silence as a `true` passes every case but the other.
//!
//! One departure is about the other half of the same reading. `withdrawn` is a
//! value the vocabulary holds and no regime here names, so it reaches nothing
//! under `keeps` for the reason that `keeps` has never heard of it. A rule that
//! called a state terminal because it reaches nothing refuses that one, and
//! `lifecycle.state.not_admitted` is what reports a document standing there.

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

const RULE: &str = headwater_check::retention::RULE;

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

/// The paths the fixture tree holds, which is what a manifest is bound against.
///
/// Written out rather than walked, for the reason `promotion.rs` writes its own
/// out: a test that took the set from the same walk the run takes would bind
/// every path a manifest could name, and binding is the whole subject here.
const HELD: [&str; 3] = [
    "retention/kept/arrived.md",
    "retention/kept/standing.md",
    "retention/released/standing.md",
];

/// One `prior` line: the path the change leaves behind, and the version that
/// stood at it.
fn departure(path: &str, prior: &str) -> String {
    format!(
        "prior\t{path}\t{}\n",
        fixtures_dir()
            .join("retention-before")
            .join(prior)
            .display()
    )
}

fn change_of(lines: &str) -> Change {
    let manifest = format!("headwater change 1\n{lines}");
    Unbound::read(&manifest, |path| std::fs::read(path))
        .expect("the manifest reads")
        .bind(|path| HELD.contains(&path))
}

/// Every departure this fixture tree can describe, in one change.
///
/// One manifest rather than one per case, because the rule has one instance
/// over the whole change: a case that ran alone would prove that the rule can
/// decide, and never that it decides these eight apart.
fn every_departure() -> String {
    [
        departure(
            "retention/kept/gone-superseded.md",
            "terminal-superseded.md",
        ),
        departure(
            "retention/kept/gone-discharged.md",
            "terminal-discharged.md",
        ),
        departure("retention/kept/gone-current.md", "live-current.md"),
        departure("retention/kept/gone-draft.md", "opening-draft.md"),
        departure(
            "retention/released/gone-superseded.md",
            "released-superseded.md",
        ),
        departure(
            "retention/unsaid/gone-superseded.md",
            "unsaid-superseded.md",
        ),
        departure("retention/loose/gone-superseded.md", "loose-superseded.md"),
        departure("retention/kept/gone-unnamed.md", "unnamed-state.md"),
        // The rename, and the one entry here that binds. The census holds a row
        // at the path the document arrived at, so it never departed.
        departure("retention/kept/arrived.md", "moved.md"),
        // A file that is no document of any corpus. Its prior version does not
        // parse as one, so nothing is held.
        departure("engine/crates/check/src/change.rs", "not-a-document.rs"),
    ]
    .concat()
}

fn run_with(ctx: &Context) -> Run {
    let corpus = Corpus::new(fixtures_dir(), "retention");
    let source = std::fs::read_to_string(fixtures_dir().join("retention.taxonomy.yml"))
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
            lock: "sha256:retention-fixture",
            taxonomy: &taxonomy,
            shape: &shape,
            relations: &declarations,
            config: &config,
            register: &register,
            adoption: None,
            source: "engine/crates/check/fixtures/retention.taxonomy.yml",
        },
        &headwater_check::claim::Claims::empty(),
        ctx,
        &mut Cache::disabled())
}

fn over(lines: &str) -> Run {
    run_with(&Context::over(
        Date::parse(PINNED).expect("the pinned date"),
        change_of(lines),
    ))
}

/// The paths this rule refused, in the order it reported them.
fn refusals(run: &Run) -> Vec<&str> {
    run.findings
        .iter()
        .filter(|finding| finding.rule == RULE)
        .map(|finding| finding.path.as_str())
        .collect()
}

fn messages(run: &Run) -> Vec<&str> {
    run.findings
        .iter()
        .filter(|finding| finding.rule == RULE)
        .map(|finding| finding.message.as_str())
        .collect()
}

/// The one instance of this rule, whatever the run.
fn instance(run: &Run) -> &headwater_check::Instance {
    let mut found = run.instances.iter().filter(|one| one.rule == RULE);
    let one = found.next().expect("the rule generates an instance");
    assert!(
        found.next().is_none(),
        "a corpus-grained rule has one instance"
    );
    one
}

/// A change that deletes a document standing at a terminal state of a regime
/// that retains is refused, and every other departure in the same change is
/// not.
///
/// The decisive case, and it is one run rather than eight because the rule has
/// one instance over the whole change. Six of the eight departures are written
/// to be refused by a rule that reads one thing less than this one does.
#[test]
fn a_deletion_at_a_terminal_state_of_a_retaining_regime_is_refused() {
    let run = over(&every_departure());
    assert_eq!(
        refusals(&run),
        [
            "retention/kept/gone-discharged.md",
            "retention/kept/gone-superseded.md",
        ],
        "the two terminal states of the one regime that retains, and nothing else"
    );
    // The nine departures reached one instance, and eight of them decided
    // nothing here. A run that had refused fewer because it saw fewer would
    // read the same at the assertion above.
    assert_eq!(
        run.change
            .as_ref()
            .expect("a change-scoped run states its change")
            .named
            .unmatched,
        9,
        "the manifest names nine paths this corpus holds no row at"
    );

    // The three facts a reader needs, and none of them is derivable from the
    // other two.
    let messages = messages(&run);
    let superseded = messages
        .iter()
        .find(|message| message.contains("superseded"))
        .expect("the finding about the superseded document");
    assert!(
        superseded.contains("NOTE-FIX-gone-superseded"),
        "{superseded}"
    );
    assert!(superseded.contains("`keeps`"), "the regime: {superseded}");
    assert!(
        superseded.contains("retain_terminal: true"),
        "the member it read: {superseded}"
    );
}

/// A regime that declares `retain_terminal: false` and a regime that declares
/// nothing both permit the deletion, and they are two facts.
///
/// This is the arm the corpus of this repository cannot reach: both of its
/// regimes declare `true`. The behavior is the same either way, so the
/// behavior cannot be what holds them apart, and the shape is asserted
/// directly.
#[test]
fn a_permitted_deletion_and_an_unruled_one_are_two_facts_with_one_behavior() {
    let source = std::fs::read_to_string(fixtures_dir().join("retention.taxonomy.yml"))
        .expect("the fixture taxonomy");
    let root = headwater_yaml::load(&source)
        .expect("the taxonomy loads")
        .value
        .as_map()
        .expect("a mapping")
        .clone();
    let shape = Shape::read(&root).expect("the shape reads");
    let declared = |name: &str| {
        shape
            .lifecycle
            .iter()
            .find(|regime| regime.name == name)
            .unwrap_or_else(|| panic!("the `{name}` regime"))
            .retain_terminal
    };
    assert_eq!(declared("keeps"), Some(true));
    assert_eq!(declared("releases"), Some(false), "a declared refusal");
    assert_eq!(
        declared("silent"),
        None,
        "silence, and not a declared false"
    );

    // And neither of the last two is reported, which is the behavior the two
    // facts share.
    let run = over(
        &[
            departure(
                "retention/released/gone-superseded.md",
                "released-superseded.md",
            ),
            departure(
                "retention/unsaid/gone-superseded.md",
                "unsaid-superseded.md",
            ),
        ]
        .concat(),
    );
    assert_eq!(refusals(&run), Vec::<&str>::new());
}

/// Flipping the one member on the one regime is what decides the verdict.
///
/// The treatment arm. The same document, the same state, the same path and the
/// same change, under two taxonomies that differ in one word. A rule that
/// reported a terminal deletion without reading the member passes every other
/// test in this file and fails this one.
#[test]
fn the_member_is_what_decides_and_not_the_state_alone() {
    let refused = over(&departure(
        "retention/kept/gone-superseded.md",
        "terminal-superseded.md",
    ));
    assert_eq!(refusals(&refused), ["retention/kept/gone-superseded.md"]);

    // The same shape, one word apart: the departed document is of the kind that
    // binds the regime declaring `false`, and it stands at the same terminal
    // state on the same day.
    let permitted = over(&departure(
        "retention/released/gone-superseded.md",
        "released-superseded.md",
    ));
    assert_eq!(refusals(&permitted), Vec::<&str>::new());
}

/// A rename is not a deletion, and the census is what decides it.
///
/// The manifest names the path the document arrived at, the census holds a row
/// there, and the entry binds. A rule that read a `prior` line as a departure
/// refuses this change, and this repository renames a governed document often
/// enough that such a rule would be turned off in a week.
#[test]
fn a_renamed_document_at_a_terminal_state_is_not_a_departure() {
    let run = over(&departure("retention/kept/arrived.md", "moved.md"));
    assert_eq!(refusals(&run), Vec::<&str>::new());

    let scoped = run
        .change
        .clone()
        .expect("a change-scoped run states its change");
    assert_eq!(scoped.named.carried, 1, "the entry bound to a census row");
    assert_eq!(scoped.named.unmatched, 0, "and it was not a departure");

    // And the same prior version at a path the tree does not hold is refused,
    // which is what makes the pass above a statement about the binding rather
    // than about the document.
    let left = over(&departure("retention/kept/left.md", "moved.md"));
    assert_eq!(refusals(&left), ["retention/kept/left.md"]);
}

/// A run that names no change reports the reason rather than a pass.
///
/// A corpus-grained instance covers the whole change, so a pass here would say
/// that a full-corpus run had cleared every deletion the history holds. Spec 4
/// asks for the visible skip, and this is the corpus-grained half of the arm
/// `promotion.rs` and `transition.rs` already hold at document grain.
#[test]
fn a_run_with_no_change_skips_the_instance_with_the_reason() {
    let run = run_with(&Context::at(Date::parse(PINNED).expect("the pinned date")));
    assert_eq!(refusals(&run), Vec::<&str>::new());
    let Outcome::Skipped(why) = &instance(&run).outcome else {
        panic!("a full-corpus run has no change to read a departure from");
    };
    assert!(
        why.contains(headwater_check::scope::CHANGE_SCOPED_ONLY),
        "{why}"
    );
}

/// A change that names no departure decides, and it decides that nothing left.
///
/// The arm between the two above: there is a change, so the instance is not
/// skipped, and there is nothing in it for this rule, so it passes. A rule that
/// skipped on an empty departed set would report the same thing for a run that
/// looked and a run that could not.
#[test]
fn a_change_that_departs_nothing_passes_rather_than_skipping() {
    let run = over(&departure("retention/kept/arrived.md", "moved.md"));
    assert!(
        matches!(instance(&run).outcome, Outcome::Passed),
        "{:?}",
        instance(&run).outcome
    );
}

/// The prior version of a departed path is in the read set, so it is in the key.
///
/// Without it a verdict about a document that left would survive the change
/// that put it back, and the differential that would catch that compares
/// verdicts rather than keys. The instance reads the census and the departure,
/// and the departed path is not a row of this corpus.
#[test]
fn the_departed_version_is_an_input_of_the_instance() {
    let run = over(&departure(
        "retention/kept/gone-superseded.md",
        "terminal-superseded.md",
    ));
    let read: Vec<&str> = instance(&run)
        .reads
        .iter()
        .map(|input| input.path.as_str())
        .collect();
    assert!(
        read.contains(&"retention/kept/gone-superseded.md"),
        "the departed path is not an input: {read:?}"
    );
    for held in HELD {
        assert!(read.contains(&held), "the census is not an input: {read:?}");
    }
    let departed = instance(&run)
        .reads
        .iter()
        .find(|input| input.path == "retention/kept/gone-superseded.md")
        .expect("the departed input");
    assert!(
        departed.digest.is_some(),
        "an input with no digest leaves the instance unkeyed forever"
    );

    // And the coverage block says the denominator does not hold it, which is
    // the one place a reader learns that this run read outside the census.
    assert_eq!(
        run.coverage.unaccounted,
        ["retention/kept/gone-superseded.md"]
    );
}
