// SPDX-License-Identifier: Apache-2.0
//! A probe result, and the one property that separates it from a snapshot.
//!
//! # The test this file exists for
//!
//! `generate --check` over a committed result has to answer three questions
//! differently, and only the third one says anything about derivation:
//!
//! 1. the committed pair agrees, and the run is green;
//! 2. an ordinary edit elsewhere in the corpus leaves it green;
//! 3. **an edit to one event of the transcript turns it red.**
//!
//! The first two hold for a file somebody typed once and never regenerated. A
//! snapshot is indistinguishable from a projection under both of them, and every
//! other check in this repository would pass a typed-in rate for as long as
//! nobody looked. The third is the whole instrument, so
//! [`a_result_goes_stale_when_its_transcript_changes`] runs all three in one
//! function and asserts the third by path.
//!
//! # The recorded file, and its grain
//!
//! [`the_fixture_tree_generates_the_recorded_result`] writes the bytes of the
//! result to `fixtures/probe-result.record`. The fixture tree is small and it
//! answers one question, so an edit to the `generate` tree does not move this
//! file and an edit to this file is always about a result.
//!
//!     HEADWATER_BLESS=1 cargo test -p headwater-generate --test probe_result
//!
//! Read the diff before committing it. A blessed fixture is the change.
//!
//! # What the recorded result shows that no assertion states as well
//!
//! Three sessions of one probe, and the three outcomes a verdict has. One
//! recorded a call that named the examined document. One recorded `calls: []`,
//! which says the recorder watched and saw nothing. One recorded no `calls` key,
//! which says nothing watched. The rate in the recorded file is over two of the
//! three, and the third is printed beside it. A grader that read a missing key
//! as an empty list would report three of three, and the recorded bytes are
//! where that shows up.

use headwater_census::census::{self, Census, Outcome};
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::Shape;
use headwater_generate::{check, plan, write, Identity, Projections, Runs, Transcript, Verdict};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use headwater_probe::plan::Narrowing;
use headwater_probe::{Budgets, Tier};
use headwater_query::Surface;
use headwater_yaml::Mapping;
use std::path::{Path, PathBuf};

/// An envelope no run of this selection fits inside.
///
/// Two probes at 25 cents against a ceiling of 1 is `Refusal::OverBudget`, and
/// it is decided after every probe has been read.
const UNAFFORDABLE: &str = "\
tiers:
  regression:
    budget_cents: 1
    session_cost_cents: 25
    repetitions: 1
    arms: [present]
";

/// The corpus root of the fixture tree, and the output path's first segment.
const ROOT: &str = "runs";

/// The path the declaration writes, once `{run}` is the transcript's file stem.
const RESULT: &str = "runs/probe-results/first-regression.md";

const TRANSCRIPT: &str = "runs/probe-runs/first-regression.md";

/// The envelope the fixture selection is projected against.
///
/// Inline rather than a file, because no branch of this test turns on it. The
/// budget decides whether a run starts, and this test grades a run that already
/// happened.
const ENVELOPE: &str = "\
tiers:
  regression:
    budget_cents: 2000
    session_cost_cents: 25
    repetitions: 1
    arms: [present]
";

/// The lock digest the fixture transcript names, which the intake holds it to.
const LOCK: &str = "sha256:fixture";

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

/// Everything a surface borrows, owned, so that a test may build one.
struct Built {
    census: Census,
    graph: Graph,
    shape: Shape,
    taxonomy: Taxonomy,
    relations: Declarations,
    config: Config,
}

impl Built {
    fn over(at: &Path) -> Self {
        let corpus = Corpus::new(at.to_path_buf(), ROOT);
        let root = load_map(&fixtures_dir().join("runs.taxonomy.yml"));
        let taxonomy = Taxonomy::read(&root).expect("the taxonomy reads");
        let relations = Declarations::read(&root).expect("the declarations read");
        let shape = Shape::read(&root).expect("the shape reads");
        let census = census::take(&corpus, &taxonomy);
        let graph = Graph::build(
            &census,
            &relations,
            &Resolvers::over(&corpus),
            &corpus,
            &Config::default(),
        );
        Built {
            census,
            graph,
            shape,
            taxonomy,
            relations,
            config: Config::default(),
        }
    }

    fn surface(&self) -> Surface<'_> {
        Surface::over(
            &self.census,
            &self.graph,
            &self.shape,
            &self.taxonomy,
            &self.relations,
            &self.config,
        )
    }
}

fn load_map(path: &Path) -> Mapping {
    let source =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    headwater_yaml::load(&source)
        .unwrap_or_else(|errors| panic!("{}: {:?}", path.display(), errors))
        .value
        .as_map()
        .unwrap_or_else(|| panic!("{} is not a mapping", path.display()))
        .clone()
}

fn identity() -> Identity {
    Identity {
        corpus_root: ROOT.to_string(),
        exclusions: Vec::new(),
        package: "headwater/fixture".to_string(),
        version: "1.0.0".to_string(),
        lock: LOCK.to_string(),
    }
}

/// The two inputs the caller supplies, read the way the binary reads them.
///
/// The selection comes from `headwater probe plan` and never from the
/// transcript, so a recorded run cannot name its own denominator. It goes
/// through `Runs::graded_against` for the reason the binary does: a plan that
/// stopped partway holds a part of a selection, and reading `.selected` off it
/// grades against the probes the planner reached before it gave up.
fn runs_over(built: &Built, at: &Path, envelope: &str) -> Runs {
    let mut runs = Runs::default();
    for row in &built.census.rows {
        let Outcome::Typed { kind, .. } = &row.outcome else {
            continue;
        };
        if kind != headwater_probe::intake::KIND {
            continue;
        }
        runs.transcripts.push(Transcript {
            path: row.path.clone(),
            source: std::fs::read_to_string(at.join(&row.path)).expect("the transcript reads"),
        });
    }
    let budgets = Budgets::read(envelope).expect("the envelope reads");
    runs.graded_against(&headwater_probe::Plan::over(
        &built.census,
        &built.graph,
        &built.config,
        &budgets,
        LOCK,
        Tier::Regression,
        &Narrowing::default(),
    ));
    runs
}

/// Everything one run over the tree would write, and everything it would not.
fn plan_over(at: &Path) -> headwater_generate::Plan {
    plan_priced(at, ENVELOPE)
}

/// The same, against a stated envelope, which is what a run would have cost.
fn plan_priced(at: &Path, envelope: &str) -> headwater_generate::Plan {
    let built = Built::over(at);
    let surface = built.surface();
    let root = load_map(&fixtures_dir().join("runs.taxonomy.yml"));
    let projections = Projections::read(&root).expect("the projections read");
    plan(
        &surface,
        &built.census,
        &projections,
        &identity(),
        &runs_over(&built, at, envelope),
    )
}

/// The plan over a tree, and the result output it produced.
fn result_bytes(at: &Path) -> (headwater_generate::Plan, String) {
    let plan = plan_over(at);
    let bytes = plan
        .outputs
        .iter()
        .find(|output| output.path == RESULT)
        .unwrap_or_else(|| {
            panic!(
                "no output at {RESULT}. The plan wrote {:?} and declined {:?}",
                plan.outputs.iter().map(|o| &o.path).collect::<Vec<_>>(),
                plan.unwritten
                    .iter()
                    .map(|u| format!("{}: {}", u.at, u.reason))
                    .collect::<Vec<_>>()
            )
        })
        .bytes
        .clone();
    (plan, bytes)
}

/// A fresh copy of the fixture corpus under Cargo's own temporary tree.
///
/// A copy and never the fixture tree itself. This test writes a generated file
/// into the corpus root and then edits its source, and both belong to the run
/// rather than to the repository.
fn copied(name: &str) -> PathBuf {
    let at = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&at);
    copy_into(&fixtures_dir().join(ROOT), &at.join(ROOT));
    at
}

fn copy_into(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("a directory");
    for entry in std::fs::read_dir(from).expect("the fixture tree") {
        let entry = entry.expect("an entry");
        let target = to.join(entry.file_name());
        match entry.file_type().expect("a file type").is_dir() {
            true => copy_into(&entry.path(), &target),
            false => {
                std::fs::copy(entry.path(), &target).expect("a copy");
            }
        }
    }
}

/// The verdict the run reached over the result, by path.
///
/// By path and never by [`headwater_generate::Report::has_errors`]. Every plan
/// carries the corpus descriptor as well, and the descriptor moves whenever the
/// census does — which is on the run right after this verb writes a file into
/// the corpus root. A test that read the whole run would call that movement a
/// result going stale, which is the reading this file exists to rule out.
fn verdict_over_the_result(report: &headwater_generate::Report) -> &Verdict {
    &report
        .wrote
        .iter()
        .find(|wrote| wrote.path == RESULT)
        .expect("the result is in the report")
        .verdict
}

fn edit(at: &Path, relative: &str, from: &str, to: &str) {
    let path = at.join(relative);
    let source = std::fs::read_to_string(&path).expect("it reads");
    assert_eq!(
        source.matches(from).count(),
        1,
        "{relative} does not hold exactly one `{from}`"
    );
    std::fs::write(&path, source.replace(from, to)).expect("it writes");
}

#[test]
fn the_fixture_tree_generates_the_recorded_result() {
    let (_, bytes) = result_bytes(&fixtures_dir());
    let recorded = fixtures_dir().join("probe-result.record");
    if std::env::var_os("HEADWATER_BLESS").is_some() {
        std::fs::write(&recorded, &bytes).expect("cannot write the expectation");
        return;
    }
    let expected = std::fs::read_to_string(&recorded).unwrap_or_else(|e| {
        panic!(
            "{}: {e}. Run with HEADWATER_BLESS=1 to record it.",
            recorded.display()
        )
    });
    assert_eq!(
        expected,
        bytes,
        "\nthe result no longer matches {}",
        recorded.display()
    );
}

/// The three directions, in one function, in the order that makes the third one
/// mean something.
///
/// The first two are true of a file somebody typed. The third is true only of a
/// file that is derived from the transcript, and it is asserted by path rather
/// than by `has_errors`, because a run that failed for any other reason would
/// otherwise read as a pass here.
#[test]
fn a_result_goes_stale_when_its_transcript_changes() {
    let at = copied("stale");

    // Direction 0: the pair agrees the moment it is written.
    let (first, _) = result_bytes(&at);
    let written = write(&at, &first);
    assert_eq!(verdict_over_the_result(&written), &Verdict::Written);
    assert!(
        at.join(RESULT).exists(),
        "the result landed inside the corpus root"
    );

    // Direction 1: the committed pair is green. The census now walks the file
    // this run wrote, so this is also the run that proves a generated result
    // does not become a finding against the corpus that produced it.
    let (again, committed) = result_bytes(&at);
    let held = check(&at, &again);
    assert_eq!(
        verdict_over_the_result(&held),
        &Verdict::Unchanged,
        "the pair did not agree the run after it was written:\n{}",
        held.render()
    );

    // Direction 2: an ordinary edit elsewhere leaves it alone. The note is on a
    // shelf that no probe examines and no projection reads.
    edit(
        &at,
        "runs/notes/scratch.md",
        "This document exists so that",
        "This paragraph was edited so that",
    );
    let (unrelated, after_note) = result_bytes(&at);
    assert_eq!(
        committed, after_note,
        "an edit to an unrelated document moved the result"
    );
    let still = check(&at, &unrelated);
    assert_eq!(
        verdict_over_the_result(&still),
        &Verdict::Unchanged,
        "an unrelated edit failed the gate:\n{}",
        still.render()
    );

    // Direction 3: one event of the transcript moves, and the committed result
    // is no longer what this corpus produces.
    edit(&at, TRANSCRIPT, "answer: \"no\"", "answer: \"maybe\"");
    let (stale, regraded) = result_bytes(&at);
    assert_ne!(
        committed, regraded,
        "an edit to one event of the transcript left the result unchanged, so nothing derives it"
    );
    let drifted = check(&at, &stale);
    assert_eq!(
        verdict_over_the_result(&drifted),
        &Verdict::Differs,
        "the gate did not report the result as stale:\n{}",
        drifted.render()
    );
    assert!(drifted.has_errors());
}

/// A corpus with no transcript writes no result, and the plan says why.
///
/// This is the state of this repository, and the reason the reason is a
/// sentence rather than a silence: an empty arm that no run prints is an empty
/// arm that a reader has to already know about.
#[test]
fn a_corpus_with_no_transcript_reports_the_reason_rather_than_writing_nothing() {
    let at = copied("no-transcript");
    std::fs::remove_file(at.join(TRANSCRIPT)).expect("the transcript goes");

    let built = Built::over(&at);
    let surface = built.surface();
    let root = load_map(&fixtures_dir().join("runs.taxonomy.yml"));
    let projections = Projections::read(&root).expect("the projections read");
    let made = plan(
        &surface,
        &built.census,
        &projections,
        &identity(),
        &runs_over(&built, &at, ENVELOPE),
    );

    assert!(
        !made
            .outputs
            .iter()
            .any(|output| output.kind == headwater_generate::Kind::ProbeResult),
        "a corpus with no transcript wrote {:?}",
        made.outputs.iter().map(|o| &o.path).collect::<Vec<_>>()
    );
    let declined = made
        .unwritten
        .iter()
        .find(|unwritten| unwritten.kind == headwater_generate::Kind::ProbeResult)
        .expect("the declaration reports itself");
    assert!(
        declined
            .reason
            .contains("holds no `probe_transcript` document"),
        "the reason does not name what is missing: {}",
        declined.reason
    );
}

/// The malformed probe the two orderings below drop into the fixture tree.
///
/// It declares an identifier, so `Plan::over` gets past `Unnameable` and stops
/// on the category, which is the refusal a reader has to be told about by name.
const MALFORMED: &str = "\
---
id: PROBE-FIX-malformed
status: current
status_since: 2026-08-14
summary: A probe that declares no category, so no run starts over this corpus.
expectation: opened
oracle: \"none\"
---

# The probe that stops the plan

## Task

Nothing runs over this probe. The planner returns before it reads this section.

## Expectation

None is reachable.
";

/// A plan that did not compose grades nothing, and the reason names the probe.
///
/// `Plan::over` returns from inside its composition loop, so the state it
/// leaves behind depends on where the offending probe sorts. The defect this
/// was written against read `.selected` and dropped `.refusal`, and the two
/// orderings therefore returned opposite verdicts over one broken corpus: a
/// probe sorting last left a partial selection and published a rate over a
/// silently reduced denominator, and one sorting first left an empty selection
/// and advised deleting the committed result. Neither named the probe.
fn a_refused_plan_grades_nothing(scratch: &str, probe: &str) {
    let at = copied(scratch);

    // The healthy pair, committed. A refusal has to be visible over a corpus
    // that was green a moment ago, because that is the corpus a reader has.
    let first = plan_over(&at);
    assert_eq!(
        verdict_over_the_result(&write(&at, &first)),
        &Verdict::Written
    );

    std::fs::write(at.join("runs/probes").join(probe), MALFORMED).expect("the probe lands");

    let refused = plan_over(&at);
    assert!(
        !refused
            .outputs
            .iter()
            .any(|output| output.kind == headwater_generate::Kind::ProbeResult),
        "a corpus whose plan is refused still wrote {:?}",
        refused
            .outputs
            .iter()
            .filter(|output| output.kind == headwater_generate::Kind::ProbeResult)
            .map(|output| &output.path)
            .collect::<Vec<_>>()
    );
    let declined = refused
        .unwritten
        .iter()
        .find(|unwritten| unwritten.kind == headwater_generate::Kind::ProbeResult)
        .expect("the declaration reports itself");
    assert!(
        declined.reason.contains(probe) && declined.reason.contains("probe_category"),
        "the reason does not name the probe that stopped the plan: {}",
        declined.reason
    );

    // And the gate does not go green over it. `generate` is the remedy the
    // failing run prints, so the run after that remedy is the one that would
    // hide this.
    let held = check(&at, &refused);
    assert!(
        held.has_errors(),
        "a refused plan left the gate green:\n{}",
        held.render()
    );
    write(&at, &refused);
    let after = check(&at, &plan_over(&at));
    assert!(
        after.has_errors(),
        "running the printed remedy made the gate green over a refused plan:\n{}",
        after.render()
    );
}

/// A run this corpus could not have afforded still grades, and grades the same.
///
/// The other arm of [`headwater_probe::plan::Refusal::stops_a_grade`], and the
/// arm a fix that stopped on every refusal would break in silence: the result
/// would simply stop being written, and every assertion about a refused plan
/// would still pass. The bytes are compared against the ones the affordable
/// envelope produces, because a ceiling is a fact about a run that has not
/// happened and this one has.
#[test]
fn a_ceiling_the_run_would_have_exceeded_leaves_the_grade_alone() {
    let at = copied("unaffordable");
    let priced = plan_priced(&at, UNAFFORDABLE);
    let over = priced
        .outputs
        .iter()
        .find(|output| output.path == RESULT)
        .unwrap_or_else(|| {
            panic!(
                "a run that was too expensive to take stopped a grade of one already taken: {:?}",
                priced
                    .unwritten
                    .iter()
                    .map(|unwritten| format!("{}: {}", unwritten.at, unwritten.reason))
                    .collect::<Vec<_>>()
            )
        });
    let (_, affordable) = result_bytes(&at);
    assert_eq!(
        over.bytes, affordable,
        "the declared ceiling moved the bytes of a result over a run that already happened"
    );
}

#[test]
fn a_probe_that_sorts_last_and_stops_the_plan_writes_no_result() {
    a_refused_plan_grades_nothing("refused-last", "0003-malformed.md");
}

#[test]
fn a_probe_that_sorts_first_and_stops_the_plan_writes_no_result() {
    a_refused_plan_grades_nothing("refused-first", "0000-malformed.md");
}
