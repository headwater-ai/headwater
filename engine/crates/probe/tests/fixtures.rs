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
//! What a suite holds is the two deterministic halves. The plan is a function of
//! the tree and the declared envelope. The record is a function of the
//! transcript and the tree. Both are recorded whole, so a change to either
//! reaches a diff.
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
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use headwater_probe::budget::{self, Budgets};
use headwater_probe::intake::Tree;
use headwater_probe::plan::{Narrowing, Refusal};
use headwater_probe::{Arm, Category, Plan, Record, Tier};
use headwater_yaml::Mapping;
use std::path::{Path, PathBuf};

/// The lock digest the fixture tree stands on. A pinned constant for the reason
/// the sweep's is one: the intake holds a transcript's `lock` against this
/// string, so a real digest would put the value in two files.
const LOCK: &str = "sha256:fixture";

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn corpus() -> Corpus {
    Corpus::new(fixtures_dir(), "corpus")
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
    let taxonomy = Taxonomy::read(root).expect("the taxonomy reads");
    census::take(&corpus(), &taxonomy)
}

fn fixture_graph(root: &Mapping, taken: &Census) -> Graph {
    let declarations = Declarations::read(root).expect("the declarations read");
    Graph::build(
        taken,
        &declarations,
        &Resolvers::over(&corpus()),
        &corpus(),
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

// --- the two recorded artifacts ---------------------------------------------

#[test]
fn the_regression_plan_over_the_fixture_corpus_is_recorded() {
    compare(&fixtures_dir().join("plan.txt"), &regression().render());
}

#[test]
fn the_record_over_the_transcript_is_recorded() {
    compare(
        &fixtures_dir().join("record.txt"),
        &record_of(&transcript("transcript.md")).render(),
    );
}

// --- determinism, which is the only reproducibility a probe claims -----------

/// The claim the whole layer rests on: a probe result is a function of the
/// transcript, the expectations and the grader version. The first two halves of
/// that are here, and each is the same bytes twice.
#[test]
fn a_plan_and_a_record_are_each_the_same_bytes_twice() {
    assert_eq!(regression().render(), regression().render());
    let source = transcript("transcript.md");
    assert_eq!(record_of(&source).render(), record_of(&source).render());
}

// --- the budget, which is the reason the harness exists ----------------------

#[test]
fn the_regression_tier_clears_its_ceiling_and_the_campaign_tier_does_not() {
    let plan = regression();
    assert!(plan.runs(), "{:?}", plan.refusal);
    assert_eq!(plan.sessions, 2, "two probes, one arm, one repetition");
    assert_eq!(plan.projected, 50);

    let campaign = plan_at(Tier::Campaign, &Narrowing::default());
    assert_eq!(
        campaign.refusal,
        Some(Refusal::OverBudget {
            sessions: 232,
            projected: 5800,
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

#[test]
fn a_category_that_no_probe_declares_refuses_rather_than_planning_nothing() {
    let plan = plan_at(
        Tier::Regression,
        &Narrowing {
            category: Some(Category::Sufficiency),
            ..Narrowing::default()
        },
    );
    assert_eq!(
        plan.refusal,
        Some(Refusal::SelectionEmpty {
            category: Some(Category::Sufficiency)
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
            assert_eq!(key, "reasoning")
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(
        record.calls, 0,
        "a refused transcript reports nothing about the run it recorded"
    );
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
    assert_eq!(record.read, 3);
    assert_eq!(record.rejected.len(), 1);
    assert_eq!(record.probes.len(), 2);
    assert_eq!(
        record.declared, 2,
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
