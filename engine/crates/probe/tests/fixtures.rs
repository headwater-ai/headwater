// SPDX-License-Identifier: Apache-2.0
//! The probe fixture corpus, and the one thing a suite over a harness can hold.
//!
//! # What is testable here, and what is not
//!
//! Nothing in this file tests a model. The middle part of a probe run is a
//! session that a recorder observes, no test can fix its behavior, and
//! `fixtures/transcript.md` is written by hand and stands for what a recorder
//! writes.
//!
//! What a suite holds is the three deterministic parts. The plan is a function
//! of the tree and the declared envelope. The record is a function of the
//! transcript and the tree. The grade is a function of the record, the
//! selection and the grader version. All three are recorded whole, so a change
//! to any of them reaches a diff.
//!
//! The grade is recorded three times over, and that is the point of it. One
//! transcript satisfies every predicate form, one refutes every form, and one
//! refuses every form. A grader that returned a single verdict for everything
//! passes exactly one of the three, and nothing else in this file would notice.
//!
//!     HEADWATER_BLESS=1 cargo test -p headwater-probe --test fixtures
//!
//! Read the diff before committing it. A blessed fixture is the change.
//!
//! # Every refusal is provoked on purpose
//!
//! A harness whose whole value is that it fails closed is a harness whose
//! refusals are the thing to test. The cases below reach the budget ceiling, the
//! unknown oracle, the oracle nobody reads, the expectation over nothing, the
//! moved taxonomy, the incomplete identity and the key that carries prose. Each
//! one asserts the refusal by variant, so a message may be reworded and a
//! refusal that stops firing cannot pass.

use headwater_census::census;
use headwater_census::census::Census;
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::paint::{paint, ColorMode, Role};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use headwater_probe::budget::{self, Budgets};
use headwater_probe::grade::{Miss, Refusal as NoVerdict, Verdict, Witness};
use headwater_probe::intake::Tree;
use headwater_probe::plan::{Examined, Narrowing, Refusal, Selected};
use headwater_probe::read_set::{Provenance, Staleness, Verdict as Stale};
use headwater_probe::{Arm, Category, Expectation, Plan, Record, Results, Tier};
use headwater_yaml::Mapping;
use std::path::{Path, PathBuf};

/// The lock digest the fixture tree stands on. A pinned constant for the reason
/// the sweep's is one: the intake holds a transcript's `lock` against this
/// string, so a real digest would put the value in two files.
const LOCK: &str = "sha256:fixture";

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

/// The fixture corpus, rooted anywhere. The read-set tests below edit
/// documents, so they run over a copy of the fixture tree rather than over the
/// tree this repository commits.
fn corpus_at(dir: &Path) -> Corpus {
    Corpus::new(dir.to_path_buf(), "corpus")
}

fn load_map(path: &Path) -> Mapping {
    let source =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    headwater_yaml::load(&source)
        .unwrap_or_else(|errors| panic!("{}: {errors:?}", path.display()))
        .value
        .as_map()
        .unwrap_or_else(|| panic!("{} is not a mapping", path.display()))
        .clone()
}

fn taxonomy_map() -> Mapping {
    load_map(&fixtures_dir().join("probe.taxonomy.yml"))
}

fn fixture_census(root: &Mapping) -> Census {
    census_at(root, &fixtures_dir())
}

fn census_at(root: &Mapping, dir: &Path) -> Census {
    let taxonomy = Taxonomy::read(root).expect("the taxonomy reads");
    census::take(&corpus_at(dir), &taxonomy)
}

fn fixture_graph(root: &Mapping, taken: &Census) -> Graph {
    graph_at(root, taken, &fixtures_dir())
}

fn graph_at(root: &Mapping, taken: &Census, dir: &Path) -> Graph {
    let declarations = Declarations::read(root).expect("the declarations read");
    Graph::build(
        taken,
        &declarations,
        &Resolvers::over(&corpus_at(dir)),
        &corpus_at(dir),
        &Config::default(),
    )
}

fn budgets() -> Budgets {
    let source =
        std::fs::read_to_string(fixtures_dir().join("probe.yml")).expect("the budget declaration");
    Budgets::read(&source).expect("the budget declaration reads")
}

fn compare(recorded: &Path, actual: &str) {
    if std::env::var_os("HEADWATER_BLESS").is_some() {
        std::fs::write(recorded, actual).expect("cannot write the expectation");
        return;
    }
    let expected = std::fs::read_to_string(recorded).unwrap_or_else(|e| {
        panic!(
            "{}: {e}. Run with HEADWATER_BLESS=1 to record it.",
            recorded.display()
        )
    });
    assert_eq!(
        expected,
        actual,
        "\nthe run no longer matches {}",
        recorded.display()
    );
}

fn plan_at(tier: Tier, narrowing: &Narrowing) -> Plan {
    let root = taxonomy_map();
    let taken = fixture_census(&root);
    let graph = fixture_graph(&root, &taken);
    Plan::over(
        &taken,
        &graph,
        &Config::default(),
        &budgets(),
        LOCK,
        tier,
        narrowing,
    )
}

fn regression() -> Plan {
    plan_at(Tier::Regression, &Narrowing::default())
}

fn record_of(source: &str) -> Record {
    let root = taxonomy_map();
    let taken = fixture_census(&root);
    let config = Config::default();
    let tree = Tree {
        census: &taken,
        config: &config,
        lock: LOCK,
    };
    Record::read(source, &tree)
}

fn transcript(name: &str) -> String {
    std::fs::read_to_string(fixtures_dir().join(name)).expect("the transcript")
}

fn results_over(source: &str) -> Results {
    Results::over(&record_of(source), &regression().selected)
}

// --- the read set of a recorded result ---------------------------------------
//
// The question is which recorded results a change voided, and the whole of the
// design is in the negative direction. A report that fires on every commit is
// indistinguishable from one that does not work, and the blunt whole-tree
// comparison passes every positive test anybody writes. So the fixture corpus
// carries one classified document that no probe examines, and the tests below
// assert that the tree digest moves over it while the read set holds still.

/// The transcript this corpus commits, which is the run the tests below ask
/// about.
const COMMITTED: &str = "corpus/probe-runs/committed.md";

/// The sentence a report opens with when a change voided a result. Asserted as
/// the whole sentence rather than as the word `stale`, which every report of
/// this module carries somewhere.
const VOIDED: &str = "**This result is stale.**";

/// The instrument, before any subject. A fixture corpus whose read set is its
/// corpus tree cannot tell a narrowing from a whole-tree comparison, and every
/// test below would pass under the blunt one.
#[test]
fn the_read_set_of_the_fixture_selection_is_not_the_corpus_tree() {
    let plan = regression();
    assert_ne!(
        plan.read_set, plan.tree,
        "the read set and the tree agree, so this corpus proves no narrowing"
    );
    assert!(
        plan.reads.len() < plan.corpus,
        "every classified document is in the read set: {} of {}",
        plan.reads.len(),
        plan.corpus
    );
}

fn plan_over(dir: &Path) -> Plan {
    let root = taxonomy_map();
    let taken = census_at(&root, dir);
    let graph = graph_at(&root, &taken, dir);
    Plan::over(
        &taken,
        &graph,
        &Config::default(),
        &budgets(),
        LOCK,
        Tier::Regression,
        &Narrowing::default(),
    )
}

fn staleness_at(dir: &Path) -> Staleness {
    let root = taxonomy_map();
    let taken = census_at(&root, dir);
    let graph = graph_at(&root, &taken, dir);
    let config = Config::default();
    let plan = Plan::over(
        &taken,
        &graph,
        &config,
        &budgets(),
        LOCK,
        Tier::Regression,
        &Narrowing::default(),
    );
    let tree = Tree {
        census: &taken,
        config: &config,
        lock: LOCK,
    };
    let source = std::fs::read_to_string(dir.join(COMMITTED)).expect("the committed transcript");
    let record = Record::read(&source, &tree);
    Staleness::over(&record, &plan, &taken)
}

/// A copy of the fixture corpus under Cargo's own temporary tree, because these
/// tests edit documents and the edits belong to the run rather than to the
/// repository.
fn copied(name: &str) -> PathBuf {
    let at = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&at);
    copy_into(&fixtures_dir().join("corpus"), &at.join("corpus"));
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
fn the_read_set_over_the_committed_run_is_recorded() {
    compare(
        &fixtures_dir().join("read-set.txt"),
        &staleness_at(&fixtures_dir()).render(),
    );
}

/// The negative direction, which is the whole test.
///
/// The edit is to a classified document that no probe examines and that no
/// recorded call named. The corpus tree digest moves over it, so a comparison
/// over the tree would report this result as voided, and the read set holds
/// still. The assertion on the tree digest is what makes the assertion on the
/// read set mean anything: without it, a read set that never moved at all would
/// pass this test.
#[test]
fn an_edit_the_read_set_does_not_cover_voids_no_result() {
    let at = copied("read-set-outside");
    let before = plan_over(&at);
    edit(
        &at,
        COMMITTED,
        "An edit to the prose here",
        "One edit to the prose here",
    );
    let after = plan_over(&at);
    assert_ne!(
        before.tree, after.tree,
        "the blunt instrument did not fire, so this edit proves nothing about the sharp one"
    );
    assert_eq!(
        before.read_set, after.read_set,
        "an edit to a document no probe examines moved the read set"
    );

    let staleness = staleness_at(&at);
    assert_eq!(staleness.verdict(), Stale::Stands);
    assert!(
        staleness.moved().is_empty(),
        "a member is reported as moved: {:?}",
        staleness.moved()
    );
    let report = staleness.render();
    assert!(
        !report.contains(VOIDED),
        "the report calls this result stale:\n{report}"
    );
}

/// The positive direction over a member a call witnessed, which is the case
/// that names the offender.
#[test]
fn an_edit_to_an_examined_document_voids_the_result_and_names_it() {
    let at = copied("read-set-witnessed");
    edit(
        &at,
        "corpus/probes/0002-answered.md",
        "# The session answers",
        "# The session answers the question",
    );
    let staleness = staleness_at(&at);
    assert_eq!(staleness.verdict(), Stale::SetMoved);
    let moved: Vec<&str> = staleness
        .moved()
        .iter()
        .map(|member| member.path.as_str())
        .collect();
    assert_eq!(moved, vec!["corpus/probes/0002-answered.md"]);
    let report = staleness.render();
    assert!(report.contains("corpus/probes/0002-answered.md"));
    assert!(report.contains("so this document moved"), "{report}");
}

/// The positive direction over a member no call witnessed.
///
/// The digest decides and the transcript names no offender, so the report says
/// the result is stale and says which documents it cannot single out. A design
/// that only compared recorded calls would report this result as standing.
#[test]
fn an_edit_to_a_probe_no_call_witnessed_still_voids_the_result() {
    let at = copied("read-set-unwitnessed");
    edit(
        &at,
        "corpus/probes/0005-patched.md",
        "# The produced patch passes",
        "# The produced patch survives",
    );
    let staleness = staleness_at(&at);
    assert_eq!(staleness.verdict(), Stale::SetMoved);
    assert!(
        staleness.moved().is_empty(),
        "no call witnessed this document, so nothing can name it as the mover"
    );
    let report = staleness.render();
    assert!(report.contains("corpus/probes/0005-patched.md"));
    assert!(
        report.contains("no recorded call named it"),
        "the report does not say what it cannot decide:\n{report}"
    );
}

/// An event with no `calls` key recorded nothing, and a read set that read it as
/// an empty list would report that the session opened no document.
///
/// The direction of that error is why it is here. Such a session would then be
/// voided by no change at all, and the result with the least evidence behind it
/// would be the one that never went stale.
#[test]
fn an_absent_calls_key_is_not_a_session_that_opened_nothing() {
    let staleness = staleness_at(&fixtures_dir());
    assert_eq!(
        staleness.unwatched,
        vec![("PROBE-FIX-not-opened".to_string(), "1".to_string())],
        "the session that recorded no `calls` key is not named"
    );
    let witnessed = staleness
        .members
        .iter()
        .find(|member| member.path == "corpus/probes/0001-opened.md")
        .expect("the document the unwatched session was pointed at is a member");
    assert!(
        witnessed.witness.is_none(),
        "an unwatched session produced a witness"
    );
    assert!(staleness.render().contains("recorded no `calls` key"));
}

/// A path a call named that this corpus classifies no document at is on no read
/// set, and the report says so rather than dropping it.
#[test]
fn a_call_that_named_no_document_of_this_corpus_is_reported_apart() {
    let staleness = staleness_at(&fixtures_dir());
    assert!(
        staleness
            .members
            .iter()
            .all(|member| member.because != Provenance::Opened),
        "every call of this transcript names a document the plan already covers"
    );
    assert!(staleness.outside.is_empty());
}

// --- the recorded artifacts --------------------------------------------------

#[test]
fn the_regression_plan_over_the_fixture_corpus_is_recorded() {
    compare(
        &fixtures_dir().join("plan.txt"),
        &regression().render(ColorMode::Plain),
    );
}

/// A plan's `harness` field names [`headwater_resolve::release::ENGINE`] and
/// not this crate's own `env!("CARGO_PKG_VERSION")`.
///
/// The recorded fixture above would also catch a regression here, but only by
/// accident: `harness` is one line inside a much larger recorded document, so
/// a drift there reads as a whole-fixture diff rather than a named version
/// disagreement, and `HEADWATER_BLESS=1` re-records over it without a word
/// said about the version (#308 shipped exactly this way). This asserts the
/// one field, by name, against the constant.
#[test]
fn the_plan_names_the_harness_by_the_shared_constant() {
    assert_eq!(
        regression().harness,
        headwater_resolve::release::ENGINE,
        "harness names this crate's own env!(\"CARGO_PKG_VERSION\") rather than the constant a \
         `requires_engine` range is read against"
    );
}

#[test]
fn the_record_over_the_transcript_is_recorded() {
    compare(
        &fixtures_dir().join("record.txt"),
        &record_of(&transcript("transcript.md")).render(),
    );
}

/// Three recorded results, and the three are the fixture set that spec 12 asks
/// a correctness root for. The first satisfies every predicate form, the second
/// refutes every one, and the third refuses every one. A grader that returned
/// one verdict for everything passes exactly one of the three.
#[test]
fn every_predicate_form_is_recorded_satisfied_refuted_and_refused() {
    for (transcript_name, recorded) in [
        ("transcript.md", "grade.txt"),
        ("transcript-missed.md", "grade-missed.txt"),
        ("transcript-refused.md", "grade-refused.txt"),
    ] {
        compare(
            &fixtures_dir().join(recorded),
            &results_over(&transcript(transcript_name)).render(),
        );
    }
}

// --- determinism, which is the only reproducibility a probe claims -----------

/// The claim the whole layer rests on: a probe result is a function of the
/// transcript, the expectations and the grader version. All three halves of
/// that are here, and each is the same bytes twice.
///
/// It is necessary and it is not sufficient. A grader that answered
/// `satisfied` to everything writes the same bytes twice as well, which is why
/// the recorded fixtures above exist and why this test is not the instrument.
#[test]
fn a_plan_a_record_and_a_grade_are_each_the_same_bytes_twice() {
    assert_eq!(
        regression().render(ColorMode::Plain),
        regression().render(ColorMode::Plain)
    );
    let source = transcript("transcript.md");
    assert_eq!(record_of(&source).render(), record_of(&source).render());
    assert_eq!(
        results_over(&source).render(),
        results_over(&source).render()
    );
}

// --- the verdicts, by variant, so a message may be reworded ------------------

/// Every satisfied verdict names the thing in the transcript that satisfied
/// it. This is the property that earns the grader the right to grade, and it
/// is asserted over the witness rather than over the rendered sentence.
#[test]
fn every_satisfied_verdict_carries_a_witness_a_reader_can_check() {
    let results = results_over(&transcript("transcript.md"));
    assert_eq!(results.graded(), 5, "five forms, one session each");
    assert_eq!(results.satisfied(), 5);
    assert_eq!(results.refused(), 0);

    let witnesses: Vec<&Witness> = results
        .rows
        .iter()
        .flat_map(|row| &row.sessions)
        .filter_map(|graded| match &graded.verdict {
            Verdict::Satisfied(witness) => Some(witness),
            _ => None,
        })
        .collect();
    assert_eq!(witnesses.len(), 5);
    for witness in witnesses {
        match witness {
            Witness::Read { event, call, .. } => assert!(*event > 0 && *call > 0),
            Witness::NoneOf { calls, over } => assert!(*calls > 0 && *over > 0),
            Witness::Cites { event, .. } | Witness::Answered { event, .. } => assert!(*event > 0),
            Witness::Clean { event, .. } => assert!(*event > 0),
        }
    }
}

/// The absolute argument and the boundary, in the transcript rather than in a
/// unit test: a session driven from another working directory names the same
/// document, and a copy of it is a different document.
#[test]
fn an_opened_verdict_cites_the_call_that_named_the_document() {
    let results = results_over(&transcript("transcript.md"));
    let row = row_of(&results, "PROBE-FIX-opened");
    match &row.sessions[0].verdict {
        Verdict::Satisfied(Witness::Read { call, argument, .. }) => {
            assert_eq!(*call, 2, "the first call named a copy and not the document");
            assert!(argument.starts_with('/'), "{argument}");
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn every_predicate_form_has_a_transcript_that_refutes_it() {
    let results = results_over(&transcript("transcript-missed.md"));
    assert_eq!(results.graded(), 5);
    assert_eq!(results.satisfied(), 0, "not one of the five is satisfied");
    for (probe, expected) in [
        ("PROBE-FIX-opened", "NeverRead"),
        ("PROBE-FIX-answered", "Outside"),
        ("PROBE-FIX-not-opened", "Read"),
        ("PROBE-FIX-cited", "NeverCited"),
        ("PROBE-FIX-patched", "OracleReported"),
    ] {
        let row = row_of(&results, probe);
        let Verdict::NotSatisfied(miss) = &row.sessions[0].verdict else {
            panic!("{probe}: {:?}", row.sessions[0].verdict);
        };
        let named = match miss {
            Miss::NeverRead { .. } => "NeverRead",
            Miss::Read { .. } => "Read",
            Miss::NeverCited { .. } => "NeverCited",
            Miss::NoAnswerGiven => "NoAnswerGiven",
            Miss::Outside { .. } => "Outside",
            Miss::OracleReported { .. } => "OracleReported",
        };
        assert_eq!(named, expected, "{probe}");
    }
}

/// The six refusals, which are the six places a grader that wanted a number
/// could have returned a green one.
#[test]
fn a_key_the_recorder_never_wrote_reaches_a_refusal_and_never_a_pass() {
    let source = transcript("transcript-refused.md");
    let results = results_over(&source);
    assert_eq!(results.graded(), 0, "not one session reached a verdict");
    assert_eq!(results.satisfied(), 0);
    assert!(
        results.rate().is_none(),
        "a rate over nothing is a number about nothing"
    );

    for (probe, expected) in [
        ("PROBE-FIX-opened", NoVerdict::Unrecorded { what: "calls" }),
        (
            "PROBE-FIX-answered",
            NoVerdict::Unrecorded { what: "answer" },
        ),
        ("PROBE-FIX-not-opened", NoVerdict::NothingObserved),
        (
            "PROBE-FIX-cited",
            NoVerdict::Unrecorded { what: "produced" },
        ),
        (
            "PROBE-FIX-patched",
            NoVerdict::NotChecked {
                artifact: "out/patch.md".into(),
            },
        ),
    ] {
        let row = row_of(&results, probe);
        assert_eq!(
            row.sessions[0].verdict,
            Verdict::Refused(expected),
            "{probe}"
        );
    }

    // The sixth: a probe the selection carries and the transcript never names.
    let without = source.replace(
        "- probe: PROBE-FIX-answered\n  session: 1\n  calls: []\n  produced: []\n",
        "",
    );
    let results = results_over(&without);
    let row = row_of(&results, "PROBE-FIX-answered");
    assert_eq!(row.sessions[0].verdict, Verdict::Refused(NoVerdict::NotRun));
}

/// A transcript the intake refused reaches no grader, which is the input
/// contract this component inherited rather than restated.
#[test]
fn a_refused_transcript_produces_no_verdict_at_all() {
    let results = results_over(&transcript("transcript-with-prose.md"));
    assert!(results.unusable.is_some());
    assert!(results.rows.is_empty());
    assert!(results.render().contains("reached no grader"));
}

/// Three refusals that no transcript can provoke, because `headwater probe
/// plan` stops the run before one is recorded. The grader holds them anyway: a
/// caller that assembled a selection by hand is a caller the plan never saw,
/// and a component that trusted its caller for the `patched` sentinel would
/// pass the one case [HW-OBL-0123] says must be refused.
#[test]
fn the_grader_refuses_a_selection_the_plan_would_not_have_produced() {
    let one = |expectation: Expectation, oracle: Option<&str>, answers: Vec<String>| Selected {
        path: "corpus/probes/0001-opened.md".into(),
        id: "PROBE-FIX-opened".into(),
        category: Category::Discovery,
        expectation,
        examines: vec![Examined {
            id: None,
            path: "corpus/probes/0002-answered.md".into(),
        }],
        oracle: oracle.map(str::to_string),
        answers,
    };
    let record = record_of(&transcript("transcript.md"));

    for (selected, expected) in [
        (
            one(Expectation::Patched, None, Vec::new()),
            NoVerdict::OracleUndeclared,
        ),
        (
            one(Expectation::Answered, None, Vec::new()),
            NoVerdict::AnswersUndeclared,
        ),
        (
            one(Expectation::Cited, None, Vec::new()),
            NoVerdict::NothingCitable { over: 1 },
        ),
    ] {
        let results = Results::over(&record, std::slice::from_ref(&selected));
        assert_eq!(
            results.rows[0].sessions[0].verdict,
            Verdict::Refused(expected)
        );
    }
}

/// The denominator, stated where a reader of the report sees it. A refused
/// session is outside both halves of the fraction, so a run that refused four
/// of five sessions reports a rate over one and says so.
#[test]
fn the_rate_is_over_the_graded_sessions_and_names_what_it_left_out() {
    let results = results_over(&transcript("transcript.md"));
    let rendered = results.render();
    assert!(rendered.contains("5 of 5 graded sessions"), "{rendered}");
    assert!(rendered.contains("95% interval"), "{rendered}");
    assert!(
        rendered.contains("The denominator is the graded sessions"),
        "{rendered}"
    );
}

fn row_of<'a>(results: &'a Results, probe: &str) -> &'a headwater_probe::grade::Row {
    results
        .rows
        .iter()
        .find(|row| row.probe == probe)
        .unwrap_or_else(|| panic!("{probe} is not in the results"))
}

// --- the budget, which is the reason the harness exists ----------------------

#[test]
fn the_regression_tier_clears_its_ceiling_and_the_campaign_tier_does_not() {
    let plan = regression();
    assert!(plan.runs(), "{:?}", plan.refusal);
    assert_eq!(plan.sessions, 5, "five probes, one arm, one repetition");
    assert_eq!(plan.projected, 125);

    let campaign = plan_at(Tier::Campaign, &Narrowing::default());
    assert_eq!(
        campaign.refusal,
        Some(Refusal::OverBudget {
            sessions: 580,
            projected: 14500,
            budget: 100,
        }),
        "a run that does not happen is the cheaper error"
    );
}

#[test]
fn a_campaign_narrowed_to_one_arm_is_refused() {
    let plan = plan_at(
        Tier::Campaign,
        &Narrowing {
            arm: Some(Arm::Present),
            ..Narrowing::default()
        },
    );
    assert_eq!(plan.refusal, Some(Refusal::CampaignNarrowed));
}

/// An arm the tier does not declare refuses, rather than silently planning the
/// arm the tier does declare.
///
/// The `regression` envelope carries `present` alone. Narrowing it to `absent`
/// selects nothing, and the plan used to put the tier's own list back and
/// print `arms: [present]` under a command line that asked for `absent`.
#[test]
fn an_arm_the_tier_does_not_declare_refuses_rather_than_planning_the_other_one() {
    let plan = plan_at(
        Tier::Regression,
        &Narrowing {
            arm: Some(Arm::Absent),
            ..Narrowing::default()
        },
    );
    assert_eq!(
        plan.refusal,
        Some(Refusal::ArmNotDeclared {
            tier: Tier::Regression,
            arm: Arm::Absent,
            declared: vec![Arm::Present],
        }),
    );
    assert!(plan.arms.is_empty(), "a refused narrowing plans no arm");
}

/// The measurement [#337](https://github.com/headwater-ai/headwater/issues/337)
/// made, held as a case.
///
/// Three captures of `headwater probe plan` — unnarrowed, `--arm present` and
/// `--arm absent` — compared byte for byte and all three identical. The case
/// above would pass on the enum alone. This one fails for the reason the issue
/// is about: the caller could not tell what they had asked for from what came
/// back.
#[test]
fn a_narrowed_plan_is_not_the_plan_the_caller_would_have_got_by_asking_for_nothing() {
    let asked_for_nothing = regression().render(ColorMode::Plain);
    let asked_for_absent = plan_at(
        Tier::Regression,
        &Narrowing {
            arm: Some(Arm::Absent),
            ..Narrowing::default()
        },
    )
    .render(ColorMode::Plain);
    assert_ne!(
        asked_for_nothing, asked_for_absent,
        "an undeclared arm handed back the plan of a run nobody asked for"
    );

    // And the arm the tier does declare still plans, so the case above is the
    // narrowing and not the flag.
    let asked_for_present = plan_at(
        Tier::Regression,
        &Narrowing {
            arm: Some(Arm::Present),
            ..Narrowing::default()
        },
    );
    assert_eq!(asked_for_present.refusal, None);
    assert_eq!(asked_for_present.arms, vec![Arm::Present]);
}

#[test]
fn a_category_that_no_probe_declares_refuses_rather_than_planning_nothing() {
    let plan = plan_at(
        Tier::Regression,
        &Narrowing {
            category: Some(Category::Consistency),
            ..Narrowing::default()
        },
    );
    assert_eq!(
        plan.refusal,
        Some(Refusal::SelectionEmpty {
            category: Some(Category::Consistency)
        }),
        "an empty selection is a run over a denominator nobody declared"
    );
}

#[test]
fn the_seed_is_recorded_as_the_caller_stated_it() {
    let plan = plan_at(
        Tier::Regression,
        &Narrowing {
            seed: 7,
            ..Narrowing::default()
        },
    );
    assert_eq!(plan.seed, 7);
    assert_eq!(
        plan.selection,
        regression().selection,
        "the seed rotates the phrasing and never the selection, so the two are \
         separable in a comparison"
    );
}

// --- the refusals a probe document can provoke -------------------------------
//
// Each case edits one facet of one fixture probe in memory. The document on
// disk stays well-formed, because a fixture corpus that shipped a broken probe
// would report the same defect through every other test in this file.

fn plan_over_probe(edit: &dyn Fn(&str) -> String) -> Plan {
    let directory = std::env::temp_dir().join(format!(
        "headwater-probe-fixture-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let shelf = directory.join("corpus/probes");
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&shelf).expect("the scratch shelf");
    for name in ["0001-opened.md", "0002-answered.md"] {
        let source = std::fs::read_to_string(fixtures_dir().join("corpus/probes").join(name))
            .expect("the fixture probe");
        std::fs::write(shelf.join(name), edit(&source)).expect("the scratch probe");
    }
    let scratch = Corpus::new(directory.clone(), "corpus");
    let root = taxonomy_map();
    let taxonomy = Taxonomy::read(&root).expect("the taxonomy reads");
    let taken = census::take(&scratch, &taxonomy);
    let declarations = Declarations::read(&root).expect("the declarations read");
    let config = Config::default();
    let graph = Graph::build(
        &taken,
        &declarations,
        &Resolvers::over(&scratch),
        &scratch,
        &config,
    );
    let plan = Plan::over(
        &taken,
        &graph,
        &config,
        &budgets(),
        LOCK,
        Tier::Regression,
        &Narrowing::default(),
    );
    let _ = std::fs::remove_dir_all(&directory);
    plan
}

#[test]
fn a_patched_probe_naming_a_rule_this_engine_does_not_carry_stops_the_run() {
    let plan = plan_over_probe(&|source| {
        source
            .replace("expectation: opened", "expectation: patched")
            .replace("oracle: \"none\"", "oracle: section.required.absent")
    });
    assert!(
        matches!(plan.refusal, Some(Refusal::OracleUnknown { .. })),
        "{:?}",
        plan.refusal
    );
}

#[test]
fn a_patched_probe_that_declares_the_sentinel_stops_the_run() {
    let plan =
        plan_over_probe(&|source| source.replace("expectation: opened", "expectation: patched"));
    assert!(
        matches!(plan.refusal, Some(Refusal::OracleUnnamed { .. })),
        "{:?}",
        plan.refusal
    );
}

#[test]
fn a_probe_that_is_not_the_oracle_route_may_not_name_a_rule() {
    let plan = plan_over_probe(&|source| {
        source.replace("oracle: \"none\"", "oracle: section.required.missing")
    });
    assert!(
        matches!(plan.refusal, Some(Refusal::OracleNotUsed { .. })),
        "a rule declared and never read is a key nobody audits: {:?}",
        plan.refusal
    );
}

#[test]
fn an_opened_probe_that_names_no_document_stops_the_run() {
    let plan = plan_over_probe(&|source| {
        source
            .replace("expectation: answered", "expectation: opened")
            .replace("expectation: not_opened", "expectation: opened")
    });
    assert!(
        matches!(plan.refusal, Some(Refusal::ExpectationNamesNothing { .. })),
        "`opened` over an empty set is never satisfied, so the rate would report \
         the declaration: {:?}",
        plan.refusal
    );
}

/// A plan that refused composed no read set, and an empty digest is not a read
/// set that covers nothing.
///
/// `Plan::over` returns from inside the loop that composes the selection, so a
/// refusal leaves the read set empty. A comparison against an empty digest
/// reports every committed result as voided by a refusal that has nothing to do
/// with the tree, which is what a hand run of `headwater probe stale` over a
/// half-copied corpus printed before this guard existed.
#[test]
fn a_plan_that_composed_no_read_set_decides_nothing_about_a_result() {
    let plan = plan_over_probe(&|source| {
        source
            .replace("expectation: answered", "expectation: opened")
            .replace("expectation: not_opened", "expectation: opened")
    });
    assert!(
        plan.read_set.is_empty(),
        "this refusal composed a read set, so it tests the wrong thing: {:?}",
        plan.refusal
    );
    let root = taxonomy_map();
    let taken = census_at(&root, &fixtures_dir());
    let source =
        std::fs::read_to_string(fixtures_dir().join(COMMITTED)).expect("the committed transcript");
    let staleness = Staleness::over(&record_of(&source), &plan, &taken);
    assert_eq!(staleness.verdict(), Stale::Unusable);
    let report = staleness.render();
    assert!(report.contains("composed no read set"), "{report}");
    assert!(
        !report.contains(VOIDED),
        "a refused plan reports a result as stale:\n{report}"
    );
}

#[test]
fn an_answered_probe_that_declares_no_closed_set_stops_the_run() {
    let plan = plan_over_probe(&|source| source.replace("answers: [yes, no]", "answers: []"));
    assert!(
        matches!(plan.refusal, Some(Refusal::AnswersUndeclared { .. })),
        "a probe with no declared set is satisfied by every string a session returns: {:?}",
        plan.refusal
    );
}

#[test]
fn a_probe_that_reads_no_answer_may_not_declare_a_closed_set() {
    let plan = plan_over_probe(&|source| {
        source.replace(
            "`opened` over the document it names.",
            "`opened` over the document it names.\n\n```yaml\nanswers: [yes, no]\n```",
        )
    });
    assert!(
        matches!(plan.refusal, Some(Refusal::AnswersNotUsed { .. })),
        "a set declared and never read is a key nobody audits: {:?}",
        plan.refusal
    );
}

#[test]
fn a_category_outside_the_closed_set_stops_the_run() {
    let plan = plan_over_probe(&|source| {
        source.replace("probe_category: discovery", "probe_category: vibes")
    });
    assert!(
        matches!(plan.refusal, Some(Refusal::Undeclared { .. })),
        "{:?}",
        plan.refusal
    );
}

// --- the refusals a transcript can provoke -----------------------------------

/// The rule this fixture set exists for. Spec 5: the transcript "holds no model
/// prose. That omission is the enforcement." An omission nothing tests is a
/// claim, so the failing arm is a transcript with one extra key.
#[test]
fn a_key_outside_the_closed_set_refuses_the_transcript() {
    let record = record_of(&transcript("transcript-with-prose.md"));
    match record.refusal {
        Some(headwater_probe::intake::Refusal::KeyNotPermitted { ref key, .. }) => {
            assert_eq!(key, "reasoning");
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(
        record.calls, 0,
        "a refused transcript reports nothing about the run it recorded"
    );
}

/// The same rule, in the third place a key can appear. The closed-key test ran
/// over an event and over a tool call and never over a produced artifact, so a
/// transcript could carry the model's account of itself in the one field the
/// grader was about to read. Only a component that read `produced` could find
/// it, which is why this case arrives with the grader.
#[test]
fn a_key_outside_the_closed_set_of_a_produced_artifact_refuses_the_transcript() {
    let source = transcript("transcript.md").replace(
        "    - path: out/report.md\n",
        "    - path: out/report.md\n      note: I cited it because the descriptor named it.\n",
    );
    let record = record_of(&source);
    match record.refusal {
        Some(headwater_probe::intake::Refusal::KeyNotPermitted { ref key, .. }) => {
            assert_eq!(key, "note");
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_transcript_planned_against_another_taxonomy_is_refused_whole() {
    let source = transcript("transcript.md").replace("lock: sha256:fixture", "lock: sha256:other");
    assert!(
        matches!(
            record_of(&source).refusal,
            Some(headwater_probe::intake::Refusal::TaxonomyMoved { .. })
        ),
        "a rate over documents another taxonomy typed is a rate about another corpus"
    );
}

#[test]
fn an_identity_missing_the_served_version_is_refused() {
    let source = transcript("transcript.md").replace("served_version: a-model-20260701\n", "");
    assert_eq!(
        record_of(&source).refusal,
        Some(headwater_probe::intake::Refusal::Missing("served_version")),
        "a model name is not a pin"
    );
}

#[test]
fn a_run_that_recorded_no_cost_is_refused() {
    let source = transcript("transcript.md").replace("cost_cents: 41\n", "");
    assert_eq!(
        record_of(&source).refusal,
        Some(headwater_probe::intake::Refusal::Missing("cost_cents")),
        "the layer reports the cost of its own instrument or it reports a guess"
    );
}

#[test]
fn an_event_naming_a_probe_this_corpus_does_not_declare_does_not_count() {
    let record = record_of(&transcript("transcript.md"));
    assert!(record.refusal.is_none(), "{:?}", record.refusal);
    assert_eq!(record.read, 6);
    assert_eq!(record.rejected.len(), 1);
    assert_eq!(record.probes.len(), 5);
    assert_eq!(
        record.declared, 5,
        "the denominator is what the corpus declares and never what the run named"
    );
}

/// The seam to [#85](https://github.com/headwater-ai/headwater/issues/85),
/// asserted rather than described. A `Record` carries counts and refusals and
/// no verdict, so a grader written here would have to add a field, and this
/// test is what a reviewer of that change reads first.
#[test]
fn the_intake_grades_nothing() {
    let rendered = record_of(&transcript("transcript.md")).render();
    assert!(
        rendered.contains("It graded nothing"),
        "the report says so where a reader sees it"
    );
    for word in ["satisfied", "passed", "failed", "verdict:"] {
        assert!(
            !rendered.contains(word),
            "`{word}` in a record is a grader that arrived without an issue"
        );
    }
}

// --- the budget declaration, refused rather than defaulted -------------------

#[test]
fn a_budget_declaration_this_engine_cannot_read_names_the_path() {
    let refusal = Budgets::read("tiers: []").expect_err("a sequence is not a mapping of tiers");
    assert!(refusal.to_string().contains(budget::PATH));
}

// --- what a caller may grade a recorded run against --------------------------
//
// [#174](https://github.com/headwater-ai/headwater/issues/174): two components
// have now read the wrong value out of a plan, and both defects survived a full
// suite. `Plan::gradable` is the one route to a selection for grading, and the
// two cases below are the two arms it separates.

/// A plan that stopped partway holds a part of a selection, and no caller may
/// grade against it.
///
/// The second fixture probe declares a category outside the closed set, so
/// `Plan::over` returns after it has read the first one. The state that leaves
/// behind is the whole point: `selected` is **not** empty. A guard that tested
/// `selected.is_empty()` passed this plan through and graded a recorded run
/// against the probes a planner reached before it gave up, which is a
/// denominator no document declares and one that moves with the order the paths
/// sort in. That is the defect #173 fixed in `headwater probe grade`, and until
/// this test nothing failed when it was put back.
#[test]
fn a_plan_that_stopped_partway_is_not_gradable_and_names_the_refusal() {
    let plan = plan_over_probe(&|source| {
        source.replace("probe_category: sufficiency", "probe_category: vibes")
    });
    assert!(
        matches!(plan.refusal, Some(Refusal::Undeclared { .. })),
        "the wrong refusal, so this tests the wrong thing: {:?}",
        plan.refusal
    );
    assert!(
        !plan.selected.is_empty(),
        "an empty selection makes this case indistinguishable from the one an          `is_empty` guard already caught, so it would prove nothing"
    );
    let refusal = plan
        .gradable()
        .expect_err("a partial selection is not a denominator");
    assert!(matches!(refusal, Refusal::Undeclared { .. }), "{refusal:?}");
}

/// A ceiling the run would have exceeded leaves the plan gradable.
///
/// The other arm, and the one a guard that stopped on every refusal would
/// break. `OverBudget` is decided after every probe has been read, so the
/// selection beside it is whole, and a grade of a run that already happened
/// spends nothing. A fix that returned `Err` here would stop grading every
/// recorded run whose corpus later outgrew its envelope.
#[test]
fn a_ceiling_the_run_would_have_exceeded_leaves_the_plan_gradable() {
    let plan = plan_at(Tier::Campaign, &Narrowing::default());
    assert!(
        matches!(plan.refusal, Some(Refusal::OverBudget { .. })),
        "{:?}",
        plan.refusal
    );
    let selected = plan
        .gradable()
        .expect("a cost is a fact about a run that has not happened");
    assert_eq!(selected.len(), plan.selected.len());
}

/// Every SGR sequence in a string, taken back off it.
fn stripped(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(start) = rest.find('\u{1b}') {
        out.push_str(&rest[..start]);
        match rest[start..].find('m') {
            Some(end) => rest = &rest[start + end + 1..],
            None => {
                rest = "";
                break;
            }
        }
    }
    out.push_str(rest);
    out
}

/// The colored plan strips to the plain plan, byte for byte.
///
/// Nothing in this briefing folds, so there is no early break to catch here.
/// What this holds is the other half: the color is added and no other byte
/// moves, which is what a recorded fixture of the plain plan already assumes
/// and nothing else states.
#[test]
fn the_colored_plan_strips_to_the_plain_plan() {
    let ansi = regression().render(ColorMode::Ansi);
    let plain = regression().render(ColorMode::Plain);
    assert!(
        ansi.contains('\u{1b}'),
        "the colored plan has to carry color, or this comparison holds nothing"
    );
    assert_eq!(stripped(&ansi), plain);
}

/// The roles `Plan::render` declares, enumerated from the renderer.
///
/// Two `paint(Role::` families reach this briefing: `Heading` on the `##`
/// lines and `Path` on the two document-path positions. A third role wired in
/// and not added here fails `the_plan_paints_no_role_this_enumeration_omits`.
const PROBE_ROLES: [Role; 2] = [Role::Heading, Role::Path];

/// The opening SGR sequence a role writes, with no text and no reset.
fn opening(role: Role) -> String {
    let painted = paint(role, "x", ColorMode::Ansi);
    painted
        .strip_suffix("x\u{1b}[0m")
        .expect("paint wraps its text and closes with a reset")
        .to_string()
}

/// The headings this plan writes, decided by the plan's own state.
///
/// Three of the six are conditional in the renderer, so a fixed list would
/// either overclaim or undercount. `## The cost` is written only where a cost
/// was computed, and a refusal replaces the last three sections with one and
/// returns.
fn headings_of(plan: &Plan) -> Vec<&'static str> {
    let mut headings = vec!["## The run identity this plan fixes"];
    if plan.budget > 0 {
        headings.push("## The cost");
    }
    match plan.refusal.is_some() {
        true => headings.push("## This run does not start"),
        false => headings.extend([
            "## The selection",
            "## The read set",
            "## What the recorder writes back",
        ]),
    }
    headings
}

/// How many times each declared role is painted, against a count derived from
/// the plan rather than from the report.
///
/// The count half of the bar. Setting either of the two `Role::Path` call
/// sites to `Plain` leaves the other standing, and before this case the whole
/// suite stayed green while the briefing lost four of its ten color sequences
/// under a terminal.
#[test]
fn every_declared_role_of_the_plan_is_painted_the_number_of_times_it_is_reached() {
    let plan = regression();
    let ansi = plan.render(ColorMode::Ansi);
    assert!(
        plan.refusal.is_none(),
        "this plan refuses, so it writes no selection and holds neither count"
    );
    for role in PROBE_ROLES {
        let wanted = match role {
            // One per heading this plan's own state says it writes.
            Role::Heading => headings_of(&plan).len(),
            // One per selected probe, and one per document of the read set.
            Role::Path => plan.selected.len() + plan.reads.len(),
            other => panic!("{other:?} is in PROBE_ROLES with no expected count"),
        };
        assert!(
            wanted > 0,
            "{role:?} has an empty population on this plan, so its count holds nothing"
        );
        let got = ansi.matches(&opening(role)).count();
        assert_eq!(
            got, wanted,
            "{role:?} is painted {got} times and this plan reaches it {wanted} times"
        );
    }
}

/// No role outside the enumeration is painted, so the enumeration is the set.
#[test]
fn the_plan_paints_no_role_this_enumeration_omits() {
    let ansi = regression().render(ColorMode::Ansi);
    for role in [
        Role::Error,
        Role::Warn,
        Role::Info,
        Role::Path,
        Role::Verb,
        Role::Obligation,
        Role::Heading,
    ] {
        let painted = ansi.contains(&opening(role));
        // `Role` derives no `PartialEq`, and every opening sequence is distinct,
        // so the enumeration is searched by what each member writes.
        let declared = PROBE_ROLES
            .iter()
            .any(|member| opening(*member) == opening(role));
        assert_eq!(
            painted, declared,
            "{role:?} is painted={painted} and enumerated={declared}. Add it to \
             PROBE_ROLES with a position case and a count, or stop painting it"
        );
    }
}

/// `Role::Heading`, in the position only this surface writes it.
///
/// A whole painted literal per heading, `##` markers included. A bare
/// `\x1b[1m` would be satisfied by any one of the six.
#[test]
fn every_plan_heading_is_painted_whole() {
    let plan = regression();
    let ansi = plan.render(ColorMode::Ansi);
    for heading in headings_of(&plan) {
        let wanted = paint(Role::Heading, heading, ColorMode::Ansi);
        assert!(
            ansi.contains(&wanted),
            "the heading {heading:?} is not painted in the colored plan"
        );
    }
}

/// `Role::Path`, in **both** of the two positions this surface writes it.
///
/// The selection line puts the path in parentheses after the identifier, and
/// the read-set line puts it first with the reason after it. Those two
/// positions cannot be satisfied by one another, so setting either call site
/// to `Plain` fails here. This is the case the audit side already had and the
/// probe side did not: a bare escape-prefix assertion is satisfied by any
/// sibling caller of the same role.
#[test]
fn both_path_positions_of_the_plan_are_painted() {
    let plan = regression();
    let ansi = plan.render(ColorMode::Ansi);
    assert!(
        !plan.selected.is_empty() && !plan.reads.is_empty(),
        "both populations have to be non-empty, or this case asserts nothing"
    );
    for selected in &plan.selected {
        let wanted = format!(
            "- {} ({})",
            selected.id,
            paint(Role::Path, &selected.path, ColorMode::Ansi)
        );
        assert!(
            ansi.contains(&wanted),
            "the selection line of {} does not paint its path:\n{wanted:?}",
            selected.id
        );
    }
    for read in &plan.reads {
        let wanted = format!(
            "- {} ({})",
            paint(Role::Path, &read.path, ColorMode::Ansi),
            read.because.name()
        );
        assert!(
            ansi.contains(&wanted),
            "the read-set line of {} does not paint its path:\n{wanted:?}",
            read.path
        );
    }
}

/// The plain plan carries no escape byte at all.
#[test]
fn the_plain_plan_writes_no_escape_byte() {
    let plain = regression().render(ColorMode::Plain);
    assert!(!plain.contains('\u{1b}'), "the plain plan has to be plain");
}
