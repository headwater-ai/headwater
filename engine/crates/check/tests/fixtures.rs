// SPDX-License-Identifier: Apache-2.0
//! The check fixture tree, and this repository's own run.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#testing-a-check-without-a-failing-fixture-does-not-ship)
//! sets the floor: "every check ships with at least one fixture that it fails
//! and one that it passes." The tree under `fixtures/check/` carries both for
//! each of the three rules, and `check.report` records every instance and every
//! finding it produces.
//!
//!     HEADWATER_BLESS=1 cargo test -p headwater-check --test fixtures
//!
//! Read the diff before committing it. A blessed fixture is the change.

use headwater_census::census;
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::{coverage, placement, reciprocity, Detail, Register, Run};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use std::path::{Path, PathBuf};

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root")
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

/// One run over one corpus: the whole pipeline, in the order spec 6 draws it.
fn run_over(corpus: &Corpus, root: &headwater_yaml::Mapping) -> Run {
    let taxonomy = Taxonomy::read(root).expect("the taxonomy reads");
    let declarations = Declarations::read(root).expect("the declarations read");
    let register = Register::read(root).expect("the register reads");
    let taken = census::take(corpus, &taxonomy);
    let graph = Graph::build(
        &taken,
        &declarations,
        &Resolvers::over(corpus),
        corpus,
        &Config::default(),
    );
    headwater_check::run(&taken, &graph, &taxonomy, &declarations, &register)
}

fn fixture_run() -> Run {
    let corpus = Corpus::new(fixtures_dir(), "check");
    let root = load_map(&fixtures_dir().join("check.taxonomy.yml"));
    run_over(&corpus, &root)
}

fn corpus_run() -> Run {
    let root = repository_root();
    let resolved = repository(&root);
    run_over(&corpus_of(&root, &resolved), &resolved.resolution.taxonomy)
}

/// One taxonomy source, loaded. The fixture taxonomy is written whole, and a
/// source with no overlays over it is a resolved taxonomy already.
fn load_map(path: &Path) -> headwater_yaml::Mapping {
    let source =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    headwater_yaml::load(&source)
        .unwrap_or_else(|errors| panic!("{}: {:?}", path.display(), errors))
        .value
        .as_map()
        .unwrap_or_else(|| panic!("{} is not a mapping", path.display()))
        .clone()
}

/// This repository, resolved. `headwater-resolve` replaced the stand-in that
/// `headwater-census` used to carry, and `corpus.checks` did not move when it
/// did.
fn repository(root: &Path) -> headwater_resolve::Repository {
    headwater_resolve::repository(root)
        .unwrap_or_else(|errors| panic!("{}", headwater_resolve::render_errors(&errors)))
}

fn corpus_of(root: &Path, resolved: &headwater_resolve::Repository) -> Corpus {
    Corpus::declared(
        root,
        &resolved.consumer.corpus_root,
        &resolved.consumer.exclusions,
    )
}

#[test]
fn the_fixture_tree_runs_to_the_recorded_report() {
    compare(
        &fixtures_dir().join("check.report"),
        &fixture_run().render(Detail::EveryInstance),
    );
}

/// This repository, checked by the taxonomy that types it.
///
/// The recorded file holds the coverage totals, the instance count per rule,
/// and every finding. It leaves out the per-document accounting for the reason
/// the census and the graph leave out their own rows: a corpus adds a document
/// most weeks, and a file that changes on every commit is a file nobody reads.
/// What stays is the number a regression moves.
#[test]
fn this_repository_runs_to_the_recorded_report() {
    let run = corpus_run();
    assert!(
        run.coverage.classified() > 30,
        "only {} classified documents, which is fewer than this repository has",
        run.coverage.classified()
    );
    compare(
        &fixtures_dir().join("corpus.checks"),
        &run.render(Detail::Findings),
    );
}

/// The reciprocity check is generated, and the generation is what is asserted.
///
/// A relation that declares no `reciprocal` produces no instance. This is the
/// property that separates a generated check from one that names a relation in
/// its own source: the fixture tree declares `assesses` and writes an edge of
/// it, and no instance appears over that edge.
#[test]
fn a_relation_that_requires_nothing_generates_no_instance() {
    let run = fixture_run();
    let instances: Vec<&headwater_check::Instance> = run
        .instances
        .iter()
        .filter(|instance| instance.rule == reciprocity::RULE)
        .collect();

    // Four `cites_evidence` pairs, and nothing from the `assesses` edge that
    // `01-one-half.md` also declares.
    assert_eq!(instances.len(), 4, "{instances:#?}");
    assert!(
        !instances.iter().any(|instance| instance
            .reads
            .contains(&"check/spec/01-one-half.md".to_string())
            && instance
                .reads
                .contains(&"check/evaluations/alpha.md".to_string())),
        "an `assesses` edge produced a reciprocity instance"
    );
}

/// Both directions of a missing half are found, and each names the other file.
///
/// The two cases differ in which name the missing document would write, and a
/// check that handled only the forward one would report nothing about half of
/// this corpus, where 81 of 86 pairs are written from both ends.
#[test]
fn either_missing_half_is_found_and_the_remediation_names_the_other_document() {
    let run = fixture_run();
    let findings: Vec<&headwater_check::Finding> = run
        .findings
        .iter()
        .filter(|finding| finding.rule == reciprocity::RULE)
        .collect();
    assert_eq!(findings.len(), 3, "{findings:#?}");

    // The inverse half is missing: the citing document wrote its half.
    let forward = findings
        .iter()
        .find(|finding| finding.path == "check/spec/01-one-half.md")
        .expect("the forward case");
    assert_eq!(
        forward.remediation,
        "add `cited_by: SPEC-FIX-one-half` under `relations:` in check/evaluations/beta.md"
    );

    // The declared half is missing: the cited document wrote the inverse.
    let backward = findings
        .iter()
        .find(|finding| finding.path == "check/evaluations/delta.md")
        .expect("the backward case");
    assert_eq!(
        backward.remediation,
        "add `cites_evidence: EVAL-FIX-delta` under `relations:` in check/spec/02-cited-only.md"
    );

    assert!(findings.iter().all(|finding| finding.fixable));
}

/// A homogeneous shelf forbids the discriminator, and the finding is at the key.
#[test]
fn a_restated_discriminator_is_a_finding_at_the_line_that_restates_it() {
    let run = fixture_run();
    let finding = run
        .findings
        .iter()
        .find(|finding| finding.rule == placement::RULE)
        .expect("the fixture");
    assert_eq!(finding.path, "check/evaluations/gamma.md");
    assert_eq!(finding.line, 3, "the finding is not at the `doc_type` key");
    assert!(finding.fixable);

    // And exactly one, so no document on a heterogeneous shelf produced one.
    assert_eq!(
        run.findings
            .iter()
            .filter(|f| f.rule == placement::RULE)
            .count(),
        1
    );
}

/// An instance of an edge-scoped check is counted against both endpoints.
///
/// `02-cited-only.md` declares no edge at all. It is checked because the
/// instance over the pair that names it reads it, and a coverage report that
/// counted the instance against one end would call the file unchecked and send
/// its author to look at a shelf pattern that is already right.
#[test]
fn an_edge_instance_is_counted_against_both_of_its_endpoints() {
    let run = fixture_run();
    let document = run
        .coverage
        .documents
        .iter()
        .find(|document| document.path == "check/spec/02-cited-only.md")
        .expect("the fixture");
    assert_eq!(document.ran, 1, "{document:#?}");
    assert!(run.coverage.unaccounted.is_empty());
}

/// OB-COV-2 is about classified documents, and only those.
///
/// A classified document that no rule read is a finding. An unclassified one is
/// a census row with its own outcome, and reporting it twice sends its author
/// to two places.
#[test]
fn a_classified_document_with_no_instance_is_a_finding_and_an_untyped_one_is_not() {
    let run = fixture_run();
    let paths: Vec<&str> = run
        .findings
        .iter()
        .filter(|finding| finding.rule == coverage::RULE)
        .map(|finding| finding.path.as_str())
        .collect();
    assert_eq!(paths, ["check/spec/03-no-instance.md"]);
    assert_eq!(run.coverage.seen(), 9);
    assert_eq!(run.coverage.classified(), 8);
}

/// The coverage numbers are computed against the census and never against the
/// set of documents that classified.
///
/// The failure this catches is the silent pass: a run whose denominator is the
/// set of files it managed to read reports "no findings" over a corpus it never
/// finished walking.
#[test]
fn the_denominator_is_the_census_and_not_the_classified_set() {
    let root = repository_root();
    let resolved = repository(&root);
    let corpus = corpus_of(&root, &resolved);
    let taxonomy = Taxonomy::read(&resolved.resolution.taxonomy).expect("reads");
    let taken = census::take(&corpus, &taxonomy);

    let run = corpus_run();
    assert_eq!(run.coverage.seen(), taken.rows.len());
    assert!(
        run.coverage.seen() > run.coverage.classified(),
        "this repository holds files that are not classified documents, and \
         coverage that reported otherwise would be counting the wrong set"
    );
    for (row, document) in taken.rows.iter().zip(&run.coverage.documents) {
        assert_eq!(row.path, document.path, "coverage lost the census order");
    }
}
