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
use headwater_check::paint::ColorMode;
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

/// The selection digest the fixture transcript names.
///
/// A real digest of the two fixture identifiers and not a placeholder, because
/// the value is compared now. A placeholder here would make the recorded pair
/// look self-consistent while one half of it was never read.
const SELECTION: &str = "sha256:c6f58f5bd22dfb4b353528edb188b7de55e447426fd4ad335559d172e000a9f9";

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
        headwater_verbs::VERBS,
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
        held.render(ColorMode::Plain)
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
        still.render(ColorMode::Plain)
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
        drifted.render(ColorMode::Plain)
    );
    assert!(drifted.has_errors());
}

/// A third probe, well formed, which moves the selection digest and nothing else.
const THIRD: &str = "\
---
id: PROBE-FIX-third
status: current
status_since: 2026-08-14
summary: A third probe, added after the run was recorded, which the run never met.
probe_category: consistency
expectation: answered
oracle: \"none\"
---

# A probe the recorded run never met

## Task

Say whether this probe was in the selection. Answer `yes` or `no`.

## Expectation

```yaml
answers: [yes, no]
```
";

/// The sentence the result carries when the two selections agree.
const AGREES: &str = "The selection this transcript names is the selection this corpus composes";

/// The comparison a result reports about the selection it was recorded over.
///
/// Three directions, because a comparison that never moves and one that always
/// moves are both useless and both look identical from a green run.
///
/// The `selection` digest is the one member of the pre-run identity that a
/// result can compare and still be a file somebody can leave committed. The
/// `tree` digest is the alternative and it is why the second direction is here:
/// a tree digest covers every classified document, so a result that tracked it
/// would move on the edit below and `generate --check` would ask for a fresh
/// commit of every result after every prose change.
#[test]
fn a_result_reports_whether_the_selection_it_recorded_is_the_one_this_corpus_composes() {
    let at = copied("selection");

    // Direction 1: the recorded pair agrees, and the result says so.
    let (_, agreeing) = result_bytes(&at);
    assert!(
        agreeing.contains(AGREES),
        "the result does not report the selection it agrees with:\n{agreeing}"
    );

    // Direction 2: a probe's prose moves, and the selection does not. This is
    // the direction that rules out comparing the corpus tree instead.
    edit(
        &at,
        "runs/probes/0002-answered.md",
        "Say whether the cache may change a verdict.",
        "Say whether a cache may change any verdict at all.",
    );
    let (_, edited) = result_bytes(&at);
    assert!(
        edited.contains(AGREES),
        "an edit to a probe's prose moved the selection this result compares:\n{edited}"
    );

    // Direction 3: a probe is added, so the population moved, and the result
    // says which digest it recorded and which one this corpus composes.
    std::fs::write(at.join("runs/probes/0003-third.md"), THIRD).expect("the probe lands");
    let (_, moved) = result_bytes(&at);
    assert!(
        moved.contains("is not the selection this corpus composes"),
        "a probe was added and the result did not report the moved selection:\n{moved}"
    );
    assert!(
        moved.contains(SELECTION),
        "the result does not name the digest the transcript recorded:\n{moved}"
    );
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
        headwater_verbs::VERBS,
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
        held.render(ColorMode::Plain)
    );
    write(&at, &refused);
    let after = check(&at, &plan_over(&at));
    assert!(
        after.has_errors(),
        "running the printed remedy made the gate green over a refused plan:\n{}",
        after.render(ColorMode::Plain)
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

/// The one type a grading caller may build a selection into refuses one that a
/// plan did not finish composing.
///
/// The end-to-end cases above assert this through the bytes a projection wrote,
/// which is the right grain for the projection and the wrong grain for the
/// rule. This asserts the rule where it lives, so a regression names
/// `Runs::graded_against` rather than a missing file.
///
/// It is the second half of a fix that is otherwise structural. `Runs` keeps
/// its selection, its digest and its refusal private, so `graded_against` is
/// the only writer of any of them and the defect this replaced — a caller
/// assigning `plan.selected` and dropping the refusal beside it — is now
/// unwritable outside this crate rather than merely wrong
/// ([#174](https://github.com/headwater-ai/headwater/issues/174)).
#[test]
fn a_plan_that_stopped_partway_hands_a_grading_caller_no_selection() {
    let at = copied("partial-selection");
    std::fs::write(at.join("runs/probes/0003-malformed.md"), MALFORMED).expect("the probe lands");
    let built = Built::over(&at);
    let runs = runs_over(&built, &at, ENVELOPE);
    assert!(
        runs.selected().is_empty(),
        "a caller was handed the probes a planner reached before it gave up: {:?}",
        runs.selected()
            .iter()
            .map(|selected| &selected.id)
            .collect::<Vec<_>>()
    );
    assert!(
        runs.refusal().is_some(),
        "the refusal beside the selection was dropped, so nothing can say why"
    );
}

/// A transcript planned against another taxonomy fails the run, and a matching
/// one reports nothing.
///
/// The pair, because neither half is a test on its own. An absence assertion is
/// satisfied by a walk that reached no transcript at all, and a presence
/// assertion is satisfied by a rule that fires on every transcript forever.
///
/// The defect this holds is the one `f615fb86` landed in the repository above:
/// a transcript whose `lock` member names a taxonomy this tree no longer
/// carries is refused whole by the first of the five confirmations
/// [spec 15](../../../../docs/spec/15-the-recorder-contract.md#what-the-engine-confirms-and-what-it-records-without-confirming)
/// states, the refusal text *is* the derived output, so `generate --check`
/// regenerates it faithfully and stays green, and no check rule reads a probe
/// result. The corpus then published three measurements taken off a result
/// that carries zero verdicts of four, and nothing anywhere reported it.
///
/// So the assertion is on the run and not on the bytes. The bytes were already
/// right.
#[test]
fn a_transcript_planned_against_another_taxonomy_fails_the_run() {
    let matched = copied("probe-result-lock-matches");
    let held = write(&matched, &plan_over(&matched));
    // The walk reached a transcript, before anything is asserted about what it
    // did not find. An empty refusal list is what a run over a corpus with no
    // transcript in it also produces, and a negative arm that cannot tell the
    // two apart tests the subject and not the instrument.
    let wrote = held
        .wrote
        .iter()
        .find(|wrote| wrote.path == RESULT)
        .expect("the run wrote a result, so it reached the transcript that result comes from");
    assert!(
        !wrote.verdict.is_error(),
        "the run reached the transcript and then failed over it: {:?}",
        wrote.verdict
    );
    assert!(
        std::fs::read_to_string(matched.join(RESULT))
            .expect("the result reads")
            .contains("## The verdicts"),
        "the matching arm grades: a result with no verdicts section would satisfy every \
         assertion below and measure nothing"
    );
    assert!(
        held.refused.is_empty(),
        "a transcript this tree's taxonomy matches is refused by nothing"
    );
    assert_eq!(
        held.remedy(),
        None,
        "a transcript this tree's taxonomy matches does not fail the run"
    );

    let moved = copied("probe-result-lock-moved");
    edit(
        &moved,
        TRANSCRIPT,
        "lock: sha256:fixture",
        "lock: sha256:another-taxonomy",
    );
    let plan = plan_over(&moved);
    let report = write(&moved, &plan);

    let refused = match report.refused.as_slice() {
        [one] => one,
        other => panic!(
            "the run reported {} refused transcripts, not one",
            other.len()
        ),
    };
    assert_eq!(refused.transcript, TRANSCRIPT);
    assert_eq!(refused.output, RESULT);
    assert_eq!(
        refused.confirmation,
        headwater_probe::intake::CONFIRMATIONS[0],
        "the report names the confirmation that failed, from the closed set rather than a literal"
    );
    assert!(
        refused.why.contains("sha256:another-taxonomy") && refused.why.contains("sha256:fixture"),
        "the report names both digests, and it names neither: {}",
        refused.why
    );
    assert!(
        report
            .render(ColorMode::Plain)
            .contains("refused transcripts"),
        "the run prints the refusal where a reader of the run sees it, rather than only inside \
         the generated document"
    );
    assert!(
        report.remedy().is_some(),
        "a transcript planned against another taxonomy leaves a result with no verdict in it, \
         and the run that wrote it exits 0"
    );
}

/// One tree whose transcript is refused, standing at the state named.
///
/// The lock edit is the one
/// [`a_transcript_planned_against_another_taxonomy_fails_the_run`] makes, so
/// every tree below is refused by the first of the five confirmations and the
/// state is the only thing that differs between them.
fn refused_at(scratch: &str, state: &str) -> (PathBuf, headwater_generate::Report) {
    let at = copied(scratch);
    edit(
        &at,
        TRANSCRIPT,
        "lock: sha256:fixture",
        "lock: sha256:another-taxonomy",
    );
    if state != "current" {
        edit(
            &at,
            TRANSCRIPT,
            "status: current",
            &format!("status: {state}"),
        );
    }
    let plan = plan_over(&at);
    let report = write(&at, &plan);
    (at, report)
}

/// The refusal this run reported over the fixture transcript, and nothing else.
fn only_refusal(report: &headwater_generate::Report) -> &headwater_generate::RefusedTranscript {
    match report.refused.as_slice() {
        [one] => one,
        other => panic!(
            "the run reported {} refused transcripts, not one",
            other.len()
        ),
    }
}

/// What the state a refused transcript stands in decides, over every answer a
/// role can give.
///
/// [#814](https://github.com/headwater-ai/headwater/issues/814) is the report.
/// The predicate used to be "this document is not a draft", and the two are not
/// the same question: the standard lifecycle has no transition back to `draft`,
/// so a promoted recording could never be allowed to go stale and every change
/// that moved the lock owed four fresh sessions before it could merge. Two
/// verified branches were blocked on it the day this was filed.
///
/// The table is the whole rule. A role that says nothing relies on the document
/// releases the refusal, and everything else holds it — including the two
/// readings where this engine cannot tell what the document claims, which is
/// the silent pass `probe_result` exists to close. Nothing here names a state
/// value that the engine reads; the values are the fixture taxonomy's and the
/// decision is over the role beside each one.
#[test]
fn only_the_role_on_a_state_decides_whether_a_refusal_fails_the_run() {
    for (state, holds, why) in [
        (
            "current",
            true,
            "`live` is the role that says a reader may rely on the document, and a refused \
             recording relied on is the case this gate exists for",
        ),
        (
            "draft",
            false,
            "`initial` is a recording somebody is still working on, and the remedy for a \
             refusal is a fresh recording rather than an edit",
        ),
        (
            "deprecated",
            false,
            "a `terminal-` role keeps the document as a record and lets nothing new rest on \
             it, so a stale recording contradicts nothing this corpus asserts",
        ),
        (
            "filed",
            true,
            "a value the vocabulary declares with no role says nothing about reliance, and a \
             recording may not be released by a state this engine cannot read",
        ),
        (
            "retired",
            true,
            "a value the vocabulary does not admit is no state at all, and a recording may not \
             escape the gate by declaring one",
        ),
    ] {
        let (at, report) = refused_at(&format!("refused-at-{state}"), state);
        let refused = only_refusal(&report);
        assert_eq!(
            refused.transcript, TRANSCRIPT,
            "the run reported a refusal over some other file at `{state}`"
        );
        assert_eq!(
            refused.held, holds,
            "a transcript at `{state}` holds the refusal {holds}: {why}"
        );
        assert_eq!(
            report.remedy().is_some(),
            holds,
            "the run over a transcript at `{state}` fails {holds}: {why}. It printed:\n{}",
            report.render(ColorMode::Plain)
        );

        // The result is written either way, and it carries the refusal and no
        // verdict. A state that released the refusal by writing no file would
        // take the measurement out of the corpus rather than mark it stale.
        let written = std::fs::read_to_string(at.join(RESULT)).expect("the result reads");
        assert!(
            written.contains("sha256:another-taxonomy"),
            "the result written for a transcript at `{state}` does not carry the refusal:\n\
             {written}"
        );
        assert!(
            !written.contains("## The verdicts"),
            "the result written for a transcript at `{state}` carries verdicts, and a refused \
             transcript reaches no grader:\n{written}"
        );
        assert!(
            report
                .render(ColorMode::Plain)
                .contains("refused transcripts"),
            "the run over a transcript at `{state}` did not report the refusal to a reader"
        );
    }
}

/// Promotion is what fails, and it fails on the promotion alone.
///
/// The table above reads five trees that differ in their state. This reads one
/// tree twice, so that nothing but the promotion can account for the change:
/// green at `draft`, red at `current`, over the same refused recording.
#[test]
fn promoting_a_refused_transcript_turns_a_green_run_red() {
    let (at, report) = refused_at("refused-then-promoted", "draft");
    assert_eq!(
        report.remedy(),
        None,
        "a refused transcript at `draft` failed the run before anything promoted it:\n{}",
        report.render(ColorMode::Plain)
    );

    edit(&at, TRANSCRIPT, "status: draft", "status: current");
    let promoted = write(&at, &plan_over(&at));
    assert!(
        only_refusal(&promoted).held,
        "promoting a refused transcript to a state a reader may rely on left the refusal \
         released"
    );
    let remedy = promoted
        .remedy()
        .expect("promoting a refused transcript fails the run");
    assert!(
        remedy.contains(TRANSCRIPT) && remedy.contains("Record the session again"),
        "the remedy for a promoted refusal names the transcript and the recording: {remedy}"
    );
}

/// The note that links the result, which is the reader a refusal has.
const CITING: &str = "runs/notes/citing.md";

/// The note that links nothing, which is the file the list may not name.
const UNRELATED: &str = "runs/notes/scratch.md";

/// A refusal a role released is still reported to every document that reads it.
///
/// [HW-DR-0062](../../../../docs/decisions/0062-a-refused-recording-is-held-by-the-reliance-its-state-claims-and-not-by-promotion.md)
/// releases the gate where the state a recording stands in says that no reader
/// relies on it. A document that links the recording, or the result derived
/// from it, is a reader the state of the recording knows nothing about. That is
/// the case this corpus met on `0149a92c`: a transcript moved to `deprecated`,
/// the run went green, and two obligation records went on stating a rate the
/// result does not carry.
///
/// So the run reports the readers and fails on none of them, which is the
/// posture the ruling takes for a released refusal. The list goes into the
/// result as well as into the run, because a reader of a committed result is
/// not the person who ran the verb, and `generate --check` then holds the list
/// to the corpus: a document that starts citing a refused result moves these
/// bytes and stops a merge.
///
/// The fixture note links the result and the transcript, so one reader is named
/// once rather than twice. The unrelated note links neither, and a list that
/// named it would be a list of the corpus rather than of the readers.
#[test]
fn a_released_refusal_names_every_document_that_reads_it() {
    let (at, report) = refused_at("refused-with-a-reader", "deprecated");
    let refused = only_refusal(&report);
    assert!(
        !refused.held,
        "a `terminal-` role releases the refusal, and this case is about what it still reports"
    );
    assert_eq!(
        refused.readers,
        vec![CITING.to_string()],
        "the run names the documents that read a refused recording, once each"
    );

    let printed = report.render(ColorMode::Plain);
    assert!(
        printed.contains(CITING),
        "the run reports the reader where a reader of the run meets it:\n{printed}"
    );
    assert!(
        !printed.contains(UNRELATED),
        "the run named a document that links neither the transcript nor its result:\n{printed}"
    );

    let written = std::fs::read_to_string(at.join(RESULT)).expect("the result reads");
    assert!(
        written.contains(CITING),
        "the result a refused transcript wrote does not name the document that reads it:\n\
         {written}"
    );
    assert!(
        !written.contains(UNRELATED),
        "the result named a document that links neither it nor its transcript:\n{written}"
    );
}

/// A refused recording that nothing reads says so, rather than printing an
/// empty list a reader has to interpret.
///
/// The two directions matter together. A list that is always empty and a list
/// that always names something are both useless, and both look the same from a
/// green run. This is the corpus the case above would be wrong about.
#[test]
fn a_refused_recording_with_no_reader_says_that_nothing_reads_it() {
    let at = copied("refused-with-no-reader");
    std::fs::remove_file(at.join(CITING)).expect("the citing note is removed");
    edit(
        &at,
        TRANSCRIPT,
        "lock: sha256:fixture",
        "lock: sha256:another-taxonomy",
    );
    edit(&at, TRANSCRIPT, "status: current", "status: deprecated");
    let report = write(&at, &plan_over(&at));

    assert!(
        only_refusal(&report).readers.is_empty(),
        "a corpus whose only reader was removed still reported one"
    );
    let written = std::fs::read_to_string(at.join(RESULT)).expect("the result reads");
    assert!(
        written.contains("No document of this corpus"),
        "the result of a refusal nothing reads does not say that nothing reads it:\n{written}"
    );
}
