// SPDX-License-Identifier: Apache-2.0
//! The sweep fixture corpus, and the one thing a suite over a sampler can hold.
//!
//! # What is testable here, and what is not
//!
//! Nothing in this file tests a model. The middle part of a sweep is a model
//! reading documents, no test can fix its output, and
//! [spec 5](../../../../docs/spec/05-ai-integration.md) says as much about a
//! skill rather than pretending otherwise. `fixtures/return.yml` is written by
//! hand and it stands for what a model returns.
//!
//! What a suite holds is the two deterministic halves. The plan is a function
//! of the tree, and the report is a function of the return file and the tree.
//! Both are recorded whole, so a change to either reaches a diff.
//!
//!     HEADWATER_BLESS=1 cargo test -p headwater-sweep --test fixtures
//!
//! Read the diff before committing it. A blessed fixture is the change.
//!
//! # The test that matters most is the one where a check finds nothing
//!
//! `corpus/0003-…` is the document
//! [HW-OBL-0113](../../../../docs/obligations/0113-every-check-passes-a-document-that-is-still-the-scaffolder-s-placeholder.md)
//! records: a decision record whose two required sections each read `TODO write
//! this section.` and whose summary is the scaffolder's own prompt. The test
//! below runs this whole check layer over the fixture corpus and asserts that
//! **no finding names it**, then runs the sweep intake and asserts that a
//! finding does. A green run is no evidence that a constraint is enforced, and
//! that pair of assertions is what says so in a file rather than in a record.

use headwater_census::census;
use headwater_census::census::Census;
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::paint::ColorMode;
use headwater_check::{Cache, Context, Date, Declared, Register, Shape};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use headwater_sweep::intake::Tree;
use headwater_sweep::{Plan, Report};
use headwater_yaml::Mapping;
use std::path::{Path, PathBuf};

/// The lock digest the fixture tree stands on.
///
/// A pinned constant rather than a digest of the taxonomy source. The intake
/// holds a return file's `taxonomy` against this string, so a digest would put
/// the value in two files and a comment in the taxonomy would then rewrite
/// `return.yml`. What the value has to be is stable and named in both places,
/// which is what a constant is.
const LOCK: &str = "sha256:fixture";

/// The date the check run below is evaluated at, for the reason every recorded
/// report in this engine pins one: a run that read today's date records a file
/// that the calendar rewrites.
const PINNED: &str = "2026-08-14";

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
    load_map(&fixtures_dir().join("sweep.taxonomy.yml"))
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

fn plan_over(under: &str) -> Plan {
    let root = taxonomy_map();
    let taken = fixture_census(&root);
    let graph = fixture_graph(&root, &taken);
    Plan::over(&taken, &graph, &Config::default(), LOCK, under)
}

fn report_of(source: &str) -> Report {
    let root = taxonomy_map();
    let taken = fixture_census(&root);
    let graph = fixture_graph(&root, &taken);
    let relations = Declarations::read(&root).expect("the declarations read");
    let base = fixtures_dir();
    let tree = Tree {
        root: &base,
        census: &taken,
        graph: &graph,
        relations: &relations,
        lock: LOCK,
    };
    Report::read(source, &tree)
}

fn returned() -> String {
    std::fs::read_to_string(fixtures_dir().join("return.yml")).expect("the return file")
}

// --- the two recorded artifacts ---------------------------------------------

#[test]
fn the_plan_over_the_fixture_corpus_is_recorded() {
    compare(
        &fixtures_dir().join("plan.txt"),
        &plan_over("corpus").render(ColorMode::Plain),
    );
}

#[test]
fn the_report_over_the_return_file_is_recorded() {
    compare(
        &fixtures_dir().join("report.txt"),
        &report_of(&returned()).render(ColorMode::Plain),
    );
}

#[test]
fn the_report_in_the_finding_shape_is_recorded() {
    compare(
        &fixtures_dir().join("report.json"),
        &headwater_sweep::json::render(&report_of(&returned())),
    );
}

// --- determinism, which is the only reproducibility a sweep claims -----------

#[test]
fn a_plan_is_the_same_bytes_twice() {
    assert_eq!(
        plan_over("corpus").render(ColorMode::Plain),
        plan_over("corpus").render(ColorMode::Plain)
    );
}

#[test]
fn a_report_is_the_same_bytes_twice() {
    let source = returned();
    assert_eq!(
        report_of(&source).render(ColorMode::Plain),
        report_of(&source).render(ColorMode::Plain)
    );
    assert_eq!(
        headwater_sweep::json::render(&report_of(&source)),
        headwater_sweep::json::render(&report_of(&source))
    );
}

// --- the HW-OBL-0113 case -------------------------------------------------

/// The placeholder document, held twice: once against every check this engine
/// runs, and once against the sweep.
///
/// The first half is the finding HW-OBL-0113 records, turned into a standing
/// assertion. If a rule ever starts to report a document whose sections are the
/// scaffolder's prompt, this test fails and the record is discharged. Until
/// then it says, in a file that runs, that the run names nothing.
#[test]
fn no_check_names_the_placeholder_document_and_the_sweep_does() {
    let placeholder = "corpus/0003-every-check-may-be-disabled-by-an-operator.md";
    let root = taxonomy_map();
    let taken = fixture_census(&root);
    let graph = fixture_graph(&root, &taken);
    let taxonomy = Taxonomy::read(&root).expect("the taxonomy reads");
    let relations = Declarations::read(&root).expect("the declarations read");
    let register = Register::read(&root).expect("the register reads");
    let shape = Shape::read(&root).expect("the shape reads");
    let config = Config::default();
    let run = headwater_check::run(
        &taken,
        &graph,
        &Declared {
            lock: LOCK,
            taxonomy: &taxonomy,
            shape: &shape,
            relations: &relations,
            config: &config,
            register: &register,
            adoption: None,
            source: "engine/crates/sweep/fixtures/sweep.taxonomy.yml",
        },
        &headwater_check::claim::Claims::empty(),
        &Context::at(Date::parse(PINNED).expect("the pinned date")),
        &mut Cache::disabled());

    // It is a classified document, so it was seen and it was checked. That is
    // the half that makes the silence a result rather than an omission.
    assert!(
        taken
            .rows
            .iter()
            .any(|row| row.path == placeholder
                && matches!(row.outcome, census::Outcome::Typed { .. })),
        "the placeholder document is not classified, so this test proves nothing"
    );
    let named: Vec<&str> = run
        .findings
        .iter()
        .filter(|finding| finding.path == placeholder)
        .map(|finding| finding.rule)
        .collect();
    assert!(
        named.is_empty(),
        "a check now names the placeholder document: {named:?}. \
         HW-OBL-0113 is discharged and this test is what should change"
    );

    let report = report_of(&returned());
    let seen: Vec<&str> = report
        .verified
        .iter()
        .filter(|verified| verified.finding.path == placeholder)
        .map(|verified| verified.finding.rule)
        .collect();
    assert_eq!(
        seen,
        ["sweep.unwritten_section"],
        "the sweep no longer carries a finding about the document every check passes"
    );
}

// --- what the intake refuses ------------------------------------------------

#[test]
fn a_quotation_the_document_does_not_hold_is_refused() {
    let report = report_of(&returned());
    let refused: Vec<String> = report
        .rejected
        .iter()
        .map(|rejected| rejected.reason.to_string())
        .collect();
    assert!(
        refused
            .iter()
            .any(|reason| reason.contains("does not hold the quotation")),
        "a fabricated quotation reached a reader: {refused:?}"
    );
}

#[test]
fn a_finding_that_restates_a_declared_edge_is_refused() {
    let report = report_of(&returned());
    assert!(
        report
            .rejected
            .iter()
            .any(|rejected| rejected.reason.to_string().contains("already declared")),
        "a restatement of the graph reached a reader"
    );
}

#[test]
fn a_path_that_is_not_a_classified_document_is_refused() {
    let report = report_of(&returned());
    assert!(
        report.rejected.iter().any(|rejected| rejected
            .reason
            .to_string()
            .contains("is not a classified document")),
        "a finding about a document nobody governs reached a reader"
    );
}

#[test]
fn a_class_this_sweep_does_not_carry_is_refused() {
    let report = report_of(&returned());
    assert!(
        report
            .rejected
            .iter()
            .any(|rejected| rejected.reason.to_string().contains("is not a class")),
        "a finding of an invented class reached a reader"
    );
}

#[test]
fn a_file_planned_against_another_taxonomy_reports_nothing() {
    let source = returned().replace("taxonomy: sha256:fixture", "taxonomy: sha256:another");
    let report = report_of(&source);
    assert!(report.verified.is_empty());
    assert!(
        matches!(
            report.refusal,
            Some(headwater_sweep::Refusal::TaxonomyMoved { .. })
        ),
        "a sweep of one taxonomy was read back against a second"
    );
}

#[test]
fn a_file_that_is_not_yaml_refuses_itself_and_never_the_process() {
    let report = report_of("findings: [\n  - class: undeclared_conflict\n");
    assert!(report.verified.is_empty());
    assert!(report.refusal.is_some());
}

// --- the quotation match ----------------------------------------------------

#[test]
fn a_quotation_matches_across_a_line_break_and_a_paragraph_break() {
    let source = "One line.\nA second line.\n\nA third, after a blank.\n";
    assert_eq!(
        headwater_sweep::intake::locate(source, "One line. A second line."),
        Some(1)
    );
    assert_eq!(
        headwater_sweep::intake::locate(source, "A second line. A third, after a blank."),
        Some(2)
    );
    assert_eq!(
        headwater_sweep::intake::locate(source, "A fourth line."),
        None
    );
}

#[test]
fn a_paraphrase_is_not_a_quotation() {
    let source = "The cache is keyed on the lock digest.\n";
    assert_eq!(
        headwater_sweep::intake::locate(source, "The cache is keyed on the lock's digest."),
        None
    );
}

// --- what a control cannot say about a sweep --------------------------------

/// The measurement behind
/// [HW-OBL-0114](../../../../docs/obligations/0114-a-control-that-names-a-sweep-marks-its-obligation-verified-with-nothing-run.md).
///
/// `sweep.taxonomy.yml` declares one coherence obligation and one control whose
/// mechanism is `sweep:undeclared_conflict`. The engine reads two mechanism
/// prefixes and this is neither, so the register calls the control external and
/// the obligation reads `verified` — from the declaration alone, with no sweep
/// having run and none reachable from a run of the checks. A derived state has
/// to be derived from what runs, and this one is not.
#[test]
fn a_control_that_names_a_sweep_verifies_its_obligation_with_nothing_run() {
    let root = taxonomy_map();
    let register = Register::read(&root).expect("the register reads");
    let projection = headwater_check::register::Projection::of(&register);
    let disposed = projection
        .obligations
        .iter()
        .find(|disposed| disposed.id == "OB-SWP-1")
        .expect("the coherence obligation");
    assert!(
        disposed.discharged(),
        "the register no longer reads an external control as discharging, so HW-OBL-0114 \
         is discharged and this test is what should change"
    );
    assert!(
        disposed.unimplemented.is_empty(),
        "the engine now reports a `sweep:` mechanism as unimplemented, which is the other \
         repair HW-OBL-0114 admits"
    );
}
