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
    let shape = Shape::read(&root).expect("the shape reads");
    let base = fixtures_dir();
    let tree = Tree {
        root: &base,
        census: &taken,
        graph: &graph,
        relations: &relations,
        shape: &shape,
        lock: LOCK,
    };
    Report::read(source, &tree)
}

/// A one-finding return file, written around a proposal.
///
/// The cases below differ in the proposal and in nothing else, so the rest of
/// the file is written once here. Every quotation is a passage of the document
/// it names, because a finding refused for its citation would never reach the
/// proposal tests at all.
fn returned_with(documents: &[&str], proposal: &str) -> String {
    let listed = documents
        .iter()
        .map(|path| format!("      - {path}\n"))
        .collect::<String>();
    format!(
        "taxonomy: {LOCK}\nslice: corpus\nfindings:\n  - class: undeclared_conflict\n    \
         documents:\n{listed}    evidence:\n      - path: {FIRST}\n        quote: An operator \
         may disable the cache for one run.\n    message: One document gives an operator a \
         setting the other says does not exist.\n    proposal:\n{proposal}"
    )
}

/// The document every case below quotes, and the `from` end of every legal
/// proposal it makes.
const FIRST: &str = "corpus/0001-an-operator-may-disable-the-cache.md";
const SECOND: &str = "corpus/0002-the-cache-is-not-an-operator-setting.md";
const THIRD: &str = "corpus/0003-every-check-may-be-disabled-by-an-operator.md";

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
/// A claim for every identifier the fixture tree declares, as a bootstrapped
/// store holds them.
///
/// `sweep.taxonomy.yml` declares one scheme and it allocates reconcile-first,
/// so every typed document of this tree owes a claim.
fn claims_over(index: &headwater_graph::index::Index) -> headwater_check::claim::Claims {
    headwater_check::claim::Claims::of(
        index
            .typed
            .iter()
            .map(|node| headwater_check::claim::Claim {
                scheme: "decision_id".to_string(),
                id: node.id.clone(),
                claimant: node.path.clone(),
            })
            .collect(),
    )
}

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
        // A store that covers what this fixture tree already spent, because
        // the subject here is what a rule can read in a document's prose and
        // an unclaimed identifier is a fact about the tree instead. A run over
        // an empty store would name this document for the one reason that has
        // nothing to do with HW-OBL-0113.
        &claims_over(&graph.index),
        &Context::at(Date::parse(PINNED).expect("the pinned date")),
        &mut Cache::disabled(),
    );

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

// --- what the intake refuses about a proposal -------------------------------
//
// A proposal is the one thing a sweep prints that a person is told to paste
// into their own front matter, and until this section existed the whole
// explicit-proposal path had no case anywhere in the workspace. The one
// proposal in `return.yml` is a carried one whose first named document happens
// to be its own source, so nothing here could tell a printer that reads the
// proposal from one that reads the document list.

fn proposal(relation: &str, from: &str, to: &str) -> String {
    format!("      relation: {relation}\n      from: {from}\n      to: {to}\n")
}

fn refusals(report: &Report) -> Vec<String> {
    report
        .rejected
        .iter()
        .map(|rejected| rejected.reason.to_string())
        .collect()
}

/// The decisive case, and the reason it names three documents.
///
/// A finding may compare three documents and propose an edge between two of
/// them. The line that tells a person where to write the front matter then has
/// to name the source of the proposed edge, and the first path the finding
/// names is a different document. Before this, the printer read
/// `documents[0]`, so the report stated the edge on one line and named a
/// document outside it on the next.
#[test]
fn the_document_told_to_declare_an_edge_is_its_source_and_not_the_first_path_named() {
    let source = returned_with(
        &[THIRD, FIRST, SECOND],
        &proposal("conflicts_with", "DR-SWP-0001", "DR-SWP-0002"),
    );
    let report = report_of(&source);
    assert_eq!(
        report.verified.len(),
        1,
        "the finding did not carry: {:?}",
        refusals(&report)
    );
    let rendered = report.render(ColorMode::Plain);
    assert!(
        rendered.contains(&format!("in the front matter of {FIRST}:")),
        "the report tells a person to write the edge somewhere else:\n{rendered}"
    );
    assert!(
        !rendered.contains(&format!("in the front matter of {THIRD}:")),
        "the report names a document that is neither end of the edge:\n{rendered}"
    );
    assert_eq!(
        report.verified[0]
            .proposal
            .as_ref()
            .map(|found| found.path.as_str()),
        Some(FIRST)
    );
}

#[test]
fn a_proposal_the_relation_does_not_admit_at_an_end_is_refused() {
    let source = returned_with(
        &[FIRST, SECOND],
        &proposal("annotates", "DR-SWP-0001", "DR-SWP-0002"),
    );
    let report = report_of(&source);
    assert!(
        report.verified.is_empty(),
        "an illegal edge reached a reader"
    );
    let refused = refusals(&report);
    assert!(
        refused.iter().any(|reason| reason
            == "`annotates` admits `note` at the `from` end, and `DR-SWP-0001` there has the \
                kind `decision_record`"),
        "the endpoint refusal did not fire: {refused:?}"
    );
}

/// The same pair of documents, under the relation's inverse name.
///
/// `annotates` admits `note` at its `from` end and `decision_record` at its
/// `to` end. A proposal written as `annotated_by` puts its own `from` at the
/// declared `to` end, so the offending end is the other one. An intake that
/// discards the direction refuses this too, and names the wrong end while it
/// does.
#[test]
fn a_proposal_under_an_inverse_name_reads_the_ends_the_other_way_round() {
    let source = returned_with(
        &[FIRST, SECOND],
        &proposal("annotated_by", "DR-SWP-0001", "DR-SWP-0002"),
    );
    let report = report_of(&source);
    assert!(
        report.verified.is_empty(),
        "an illegal edge reached a reader"
    );
    let refused = refusals(&report);
    assert!(
        refused.iter().any(|reason| reason
            == "`annotated_by` admits `note` at the `to` end, and `DR-SWP-0002` there has the \
                kind `decision_record`"),
        "the refusal names the end the declared direction would have named: {refused:?}"
    );
}

#[test]
fn a_proposal_from_a_document_to_itself_is_refused() {
    let source = returned_with(
        &[FIRST, SECOND],
        &proposal("conflicts_with", "DR-SWP-0001", "DR-SWP-0001"),
    );
    let report = report_of(&source);
    assert!(report.verified.is_empty(), "a self-edge reached a reader");
    let refused = refusals(&report);
    assert!(
        refused
            .iter()
            .any(|reason| reason.contains("to itself, and a document declares nothing")),
        "the self-edge refusal did not fire: {refused:?}"
    );
}

/// Novelty, read backwards.
///
/// `conflicts_with` is its own inverse, and the corpus declares
/// `DR-SWP-0004 conflicts_with DR-SWP-0005`. So the same edge proposed the
/// other way round is the same edge, and a report that carried it would tell a
/// person to declare what their own front matter already says. The finding
/// names two other documents here, because the class-implied novelty test
/// already reads both orders over the pair a finding names and this test is
/// about the explicit proposal.
#[test]
fn a_proposal_that_reverses_a_declared_symmetric_edge_is_refused() {
    let source = returned_with(
        &[FIRST, SECOND],
        &proposal("conflicts_with", "DR-SWP-0005", "DR-SWP-0004"),
    );
    let report = report_of(&source);
    assert!(
        report.verified.is_empty(),
        "a restatement of a declared symmetric edge reached a reader"
    );
    let refused = refusals(&report);
    assert!(
        refused
            .iter()
            .any(|reason| reason.contains("is already declared, so this restates the graph")),
        "the symmetric novelty refusal did not fire: {refused:?}"
    );
}

/// The other direction, which is the half a refusal that never fires looks
/// exactly like.
///
/// A legal proposal between two documents the relation admits still carries,
/// and the recorded return file's one proposal is still one of them.
#[test]
fn a_legal_proposal_still_carries() {
    let source = returned_with(
        &[FIRST, SECOND],
        &proposal("conflicts_with", "DR-SWP-0001", "DR-SWP-0002"),
    );
    let report = report_of(&source);
    assert_eq!(
        report.rejected.len(),
        0,
        "a legal proposal was refused: {:?}",
        refusals(&report)
    );
    assert_eq!(report.verified.len(), 1);

    let recorded = report_of(&returned());
    let carried = recorded
        .verified
        .iter()
        .filter(|verified| verified.proposal.is_some())
        .count();
    assert_eq!(
        carried, 2,
        "the recorded return file no longer carries both of its proposals: one under a symmetric \
         relation and one under a relation that requires both ends"
    );
    assert!(
        !refusals(&recorded)
            .iter()
            .any(|reason| reason.contains("admits") || reason.contains("to itself")),
        "a new refusal now fires on the recorded file: {:?}",
        refusals(&recorded)
    );
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

// --- the round trip: what a person gets after pasting what the report printed

/// Every front-matter block the report told a reader to write, in order.
///
/// The report prints a block as four lines: one that names a document and ends
/// in a colon, then `relations:`, then the relation name, then one list entry.
/// This reads them back out of the rendered text rather than out of the
/// [`headwater_sweep::intake::Proposal`], because what a person applies is the
/// text. A block whose relation name or path the printer got wrong is a block
/// that lands in the wrong file or declares the wrong edge, and reading the
/// struct instead would agree with the printer about both.
fn blocks_of(rendered: &str) -> Vec<Block> {
    const MARKER: &str = "in the front matter of ";
    let lines: Vec<&str> = rendered.lines().collect();
    let mut found = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let Some(rest) = line.split_once(MARKER) else {
            continue;
        };
        let path = rest.1.strip_suffix(':').unwrap_or_else(|| {
            panic!("the line naming a document does not end in a colon: {line}")
        });
        let read = |offset: usize| -> &str {
            lines
                .get(index + offset)
                .unwrap_or_else(|| panic!("the block after {path} is cut short"))
                .trim()
        };
        assert_eq!(read(1), "relations:", "the block under {path} is not one");
        found.push(Block {
            path: path.to_string(),
            relation: read(2)
                .strip_suffix(':')
                .expect("the relation name ends in a colon")
                .to_string(),
            id: read(3)
                .strip_prefix("- ")
                .expect("the entry is a list item")
                .to_string(),
        });
    }
    found
}

/// One block, as a reader would paste it.
#[derive(Debug)]
struct Block {
    path: String,
    relation: String,
    id: String,
}

/// A copy of the fixture corpus, under a directory named for the case.
///
/// Named for the case rather than for the process, because `cargo` runs the
/// cases of one target as threads of one process and a directory keyed on the
/// pid alone is shared between them.
fn scratch(case: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("headwater-sweep-{case}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("corpus")).expect("the scratch corpus");
    for entry in std::fs::read_dir(fixtures_dir().join("corpus")).expect("the fixture corpus") {
        let entry = entry.expect("a corpus entry");
        std::fs::copy(entry.path(), dir.join("corpus").join(entry.file_name()))
            .expect("a document copies into the scratch corpus");
    }
    dir
}

/// Paste one block into the front matter of the document it names.
///
/// The report indents a block for display, so a reader dedents it. Where the
/// document already carries a `relations:` key the entry joins that mapping,
/// and where it carries none the whole key is written before the closing
/// fence. Both are what a person does by hand, and neither is a splicer this
/// engine ships.
fn paste(dir: &Path, block: &Block) {
    let file = dir.join(&block.path);
    let source =
        std::fs::read_to_string(&file).unwrap_or_else(|e| panic!("{}: {e}", file.display()));
    let mut lines: Vec<String> = source.lines().map(str::to_string).collect();
    let entry = vec![
        format!("  {}:", block.relation),
        format!("    - {}", block.id),
    ];
    let close = lines
        .iter()
        .skip(1)
        .position(|line| line == "---")
        .expect("the front matter closes")
        + 1;
    let at = match lines[..close].iter().position(|line| line == "relations:") {
        Some(index) => index + 1,
        None => {
            lines.insert(close, "relations:".to_string());
            close + 1
        }
    };
    for (offset, written) in entry.into_iter().enumerate() {
        lines.insert(at + offset, written);
    }
    std::fs::write(&file, lines.join("\n") + "\n").expect("the pasted document writes");
}

/// Every `relation.reciprocity.missing` this check layer reports over a tree.
///
/// The whole layer runs and the findings are filtered by rule, because the
/// question is what a reader's own gate says after the paste and their gate
/// runs every rule. Other findings are the fixture corpus being a fixture and
/// are not this case's subject.
fn reciprocity_over(dir: &Path) -> Vec<String> {
    let root = taxonomy_map();
    let corpus = Corpus::new(dir, "corpus");
    let taxonomy = Taxonomy::read(&root).expect("the taxonomy reads");
    let taken = census::take(&corpus, &taxonomy);
    let relations = Declarations::read(&root).expect("the declarations read");
    let config = Config::default();
    let graph = Graph::build(
        &taken,
        &relations,
        &Resolvers::over(&corpus),
        &corpus,
        &config,
    );
    let register = Register::read(&root).expect("the register reads");
    let shape = Shape::read(&root).expect("the shape reads");
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
        &claims_over(&graph.index),
        &Context::at(Date::parse(PINNED).expect("the pinned date")),
        &mut Cache::disabled(),
    );
    run.findings
        .iter()
        .filter(|finding| finding.rule == "relation.reciprocity.missing")
        .map(|finding| format!("{}: {}", finding.path, finding.message))
        .collect()
}

/// Render a report for one proposal, and return the blocks it printed.
fn blocks_for(relation: &str, from: &str, to: &str) -> Vec<Block> {
    let source = returned_with(&[FIRST, SECOND], &proposal(relation, from, to));
    let report = report_of(&source);
    assert_eq!(
        report.verified.len(),
        1,
        "the finding did not carry: {:?}",
        refusals(&report)
    );
    blocks_of(&report.render(ColorMode::Plain))
}

/// The case this whole section exists for, in the direction the relation
/// declares.
///
/// A reader pastes what the report printed and runs their own commit gate. The
/// assertion is the pair: the first block alone leaves an error, and every
/// block the report printed leaves none. The first half is what makes the
/// second one evidence — a run that reported zero because the rule never
/// instantiated would report zero for both.
#[test]
fn every_block_the_report_prints_for_a_required_relation_leaves_a_passing_corpus() {
    let blocks = blocks_for("supersedes", "DR-SWP-0001", "DR-SWP-0002");
    assert_eq!(
        blocks.len(),
        2,
        "a relation that requires both ends printed {} block(s): {blocks:?}",
        blocks.len()
    );

    let half = scratch("required-half");
    paste(&half, &blocks[0]);
    let owed = reciprocity_over(&half);
    assert_eq!(
        owed.len(),
        1,
        "the first block alone left no reciprocity finding, so this case proves nothing: {owed:?}"
    );

    let both = scratch("required-both");
    for block in &blocks {
        paste(&both, block);
    }
    let after = reciprocity_over(&both);
    assert!(
        after.is_empty(),
        "pasting what the report printed leaves a corpus the gate refuses: {after:?}"
    );
}

/// The same relation reached under its inverse name.
///
/// `superseded_by` is a name no `relations:` map declares, so the intake
/// resolves it through the inverse and the proposal's `from` sits at the
/// relation's `to` end. The owed name is then the declared one rather than the
/// inverse one, and a printer that reads `relation.inverse` in both directions
/// prints `superseded_by` twice. Five of the seventeen names this repository's
/// own taxonomy resolves are reachable only this way.
#[test]
fn the_same_holds_for_a_proposal_written_under_the_inverse_name() {
    let blocks = blocks_for("superseded_by", "DR-SWP-0001", "DR-SWP-0002");
    assert_eq!(blocks.len(), 2, "the inverse direction printed {blocks:?}");
    assert_eq!(
        blocks[0].relation, "superseded_by",
        "the printed half is not the name the finding wrote: {blocks:?}"
    );
    assert_eq!(
        blocks[1].relation, "supersedes",
        "the owed half is not the declared name: {blocks:?}"
    );

    let half = scratch("inverse-half");
    paste(&half, &blocks[0]);
    assert_eq!(
        reciprocity_over(&half).len(),
        1,
        "the first block alone left no reciprocity finding, so this case proves nothing"
    );

    let both = scratch("inverse-both");
    for block in &blocks {
        paste(&both, block);
    }
    let after = reciprocity_over(&both);
    assert!(
        after.is_empty(),
        "pasting what the report printed leaves a corpus the gate refuses: {after:?}"
    );
}

/// The two relations that owe nothing, which is what stops the second block
/// from being unconditional.
///
/// `conflicts_with` is symmetric and `records` declares no reciprocity. A
/// second block under either one would tell a person to declare an edge their
/// taxonomy never asked for, and the assertion is the count and the absence of
/// the second path rather than the text of the first.
#[test]
fn a_relation_that_requires_one_end_prints_one_block_and_one_path() {
    for relation in ["conflicts_with", "records"] {
        let source = returned_with(
            &[FIRST, SECOND],
            &proposal(relation, "DR-SWP-0001", "DR-SWP-0002"),
        );
        let report = report_of(&source);
        assert_eq!(
            report.verified.len(),
            1,
            "`{relation}` did not carry: {:?}",
            refusals(&report)
        );
        let blocks = blocks_of(&report.render(ColorMode::Plain));
        assert_eq!(
            blocks.len(),
            1,
            "`{relation}` requires one end and printed {blocks:?}"
        );
        assert_eq!(blocks[0].path, FIRST);
        assert!(
            report.verified[0]
                .proposal
                .as_ref()
                .expect("the proposal carried")
                .owed
                .is_none(),
            "`{relation}` requires one end and the intake recorded a second half"
        );
        let rendered = report.render(ColorMode::Plain);
        assert!(
            !rendered.contains(&format!("in the front matter of {SECOND}:")),
            "the report tells a person to write in the far document, and `{relation}` owes it \
             nothing:\n{rendered}"
        );
    }
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
