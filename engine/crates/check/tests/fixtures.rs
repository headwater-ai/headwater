// SPDX-License-Identifier: Apache-2.0
//! The check fixture tree, and this repository's own run.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#testing-a-check-without-a-failing-fixture-does-not-ship)
//! sets the floor: "every check ships with at least one fixture that it fails
//! and one that it passes." The tree under `fixtures/check/` carries both for
//! each of the seven rules, and `check.report` records every instance and every
//! finding it produces.
//!
//!     HEADWATER_BLESS=1 cargo test -p headwater-check --test fixtures
//!
//! Read the diff before committing it. A blessed fixture is the change.

use headwater_census::census;
use headwater_census::census::Census;
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::scope::{over_documents, over_edges, Digests};
use headwater_check::{
    coverage, endpoint, facet_required, facet_value, fragment, language, participation, placement,
    reciprocity, sections, voice, Cache, Context, Date, Declared, Detail, DocumentCheck,
    DocumentView, EdgeCheck, EdgeView, Grain, Outcome, Register, Run, Shape,
};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use std::path::{Path, PathBuf};

/// The date every recorded report is evaluated at.
///
/// A recorded report is a function of its inputs, and
/// [spec 12](../../../../docs/spec/12-check-layer.md#determinism-concretely)
/// makes the clock one of them: "same corpus, same lock, same injected clock,
/// byte-identical output". A test that read today's date would record a file
/// that a windowed rule rewrites as the calendar moves, which is a fixture
/// nobody reads.
const PINNED: &str = "2026-08-12";

fn pinned() -> Context {
    Context::at(Date::parse(PINNED).expect("the pinned date"))
}

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
fn run_over(
    corpus: &Corpus,
    root: &headwater_yaml::Mapping,
    lock: &str,
    cache: &mut Cache,
) -> Run {
    run_at(corpus, root, lock, &pinned(), cache)
}

fn run_at(
    corpus: &Corpus,
    root: &headwater_yaml::Mapping,
    lock: &str,
    ctx: &Context,
    cache: &mut Cache,
) -> Run {
    let taxonomy = Taxonomy::read(root).expect("the taxonomy reads");
    let declarations = Declarations::read(root).expect("the declarations read");
    let register = Register::read(root).expect("the register reads");
    let shape = Shape::read(root).expect("the shape reads");
    let taken = census::take(corpus, &taxonomy);
    let graph = Graph::build(
        &taken,
        &declarations,
        &Resolvers::over(corpus),
        corpus,
        &Config::default(),
    );
    headwater_check::run(
        &taken,
        &graph,
        &Declared {
            lock,
            taxonomy: &taxonomy,
            shape: &shape,
            relations: &declarations,
            register: &register,
        },
        ctx,
        cache,
    )
}

fn fixture_run() -> Run {
    let corpus = Corpus::new(fixtures_dir(), "check");
    let root = load_map(&fixtures_dir().join("check.taxonomy.yml"));
    run_over(&corpus, &root, &fixture_lock(), &mut Cache::disabled())
}

/// The fixture tree has no lock, so the digest of its taxonomy source stands
/// in for one. It is the same fact a lock digest is: the bytes every result in
/// this tree rests on.
fn fixture_lock() -> String {
    let source = std::fs::read_to_string(fixtures_dir().join("check.taxonomy.yml"))
        .expect("the fixture taxonomy");
    headwater_hash::hex(source.as_bytes())
}

/// The fixture tree as Phase A leaves it, which is what a scoped view is built
/// from.
fn fixture_census() -> Census {
    let corpus = Corpus::new(fixtures_dir(), "check");
    let root = load_map(&fixtures_dir().join("check.taxonomy.yml"));
    let taxonomy = Taxonomy::read(&root).expect("the taxonomy reads");
    census::take(&corpus, &taxonomy)
}

fn corpus_run() -> Run {
    cached_corpus_run(&mut Cache::disabled())
}

fn cached_corpus_run(cache: &mut Cache) -> Run {
    let root = repository_root();
    let resolved = repository(&root);
    let lock = headwater_lock::at(&root).expect("the committed lock");
    run_over(
        &corpus_of(&root, &resolved),
        &resolved.resolution.taxonomy,
        &lock.digest,
        cache,
    )
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
/// and what each rule serves. It leaves out the per-document accounting for the
/// reason the census and the graph leave out their own rows: a corpus adds a
/// document most weeks, and a file that changes on every commit is a file
/// nobody reads. What stays is the number a regression moves.
///
/// It now leaves out the findings for the same reason, one step further in.
/// Four of the eleven rules read prose, and a prose finding is a function of a
/// sentence: recording them here would re-bless this file on every commit that
/// touched a paragraph of `docs/spec/`. So the properties that a prose edit
/// must not change are asserted instead, and the count that a prose edit
/// legitimately moves is left to the run itself. CI publishes it.
#[test]
fn this_repository_runs_to_the_recorded_report() {
    let run = corpus_run();
    assert!(
        run.coverage.classified() > 30,
        "only {} classified documents, which is fewer than this repository has",
        run.coverage.classified()
    );

    // This repository holds no blocking defect, which is the property the
    // recorded finding list used to carry. It survives a prose edit, and the
    // finding list did not.
    let errors: Vec<&str> = run
        .findings
        .iter()
        .filter(|finding| finding.severity == headwater_check::Severity::Error)
        .map(|finding| finding.rule)
        .collect();
    assert!(errors.is_empty(), "{errors:?}");

    // A check that never fires on any input is indistinguishable from one that
    // does not work, and the fixture tree proves that per rule against a
    // constructed document. These two run against real prose, which is what
    // Q5's instrument is: the count is not recorded, and that it is not zero
    // is.
    for rule in [voice::RULE, language::RULE] {
        assert!(
            run.findings.iter().any(|finding| finding.rule == rule),
            "{rule} found nothing over this repository's own prose"
        );
    }

    compare(
        &fixtures_dir().join("corpus.checks"),
        &run.render(Detail::Totals),
    );
}

/// The differential of the correctness root, over the real corpus.
///
/// `tests/cache.rs` holds the same property over the fixture tree, where a
/// test may edit a document and re-run. This one runs it over the corpus the
/// recorded report above is of, so the report and the property are about one
/// set of numbers. The cache is written to this crate's target directory,
/// because a test that wrote into `.headwater/` would leave a repository
/// dirtier than it found it.
#[test]
fn this_repository_reports_the_same_run_from_a_cache_as_from_none() {
    let store = Path::new(env!("CARGO_TARGET_TMPDIR")).join("corpus-cache");
    let _ = std::fs::remove_dir_all(&store);
    let lock = headwater_lock::at(&repository_root()).expect("the committed lock");

    let without = corpus_run();

    let mut cold = Cache::at(&store, &lock.digest);
    let first = cached_corpus_run(&mut cold);
    cold.write(&store);

    let mut warm = Cache::at(&store, &lock.digest);
    let second = cached_corpus_run(&mut warm);

    let expected = without.render(Detail::Findings);
    assert_eq!(expected, first.render(Detail::Findings));
    assert_eq!(expected, second.render(Detail::Findings));

    assert!(first.cache.misses > 0, "{:?}", first.cache);
    assert_eq!(second.cache.hits, first.cache.misses, "{:?}", second.cache);
    assert_eq!(second.cache.misses, 0, "{:?}", second.cache);
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
        !instances.iter().any(|instance| {
            let paths = instance.paths();
            paths.contains(&"check/spec/01-one-half.md")
                && paths.contains(&"check/evaluations/alpha.md")
        }),
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

/// Every finding of one verdict is reported, and not the first of them.
///
/// A document can omit several facets its kind requires, and each omission is a
/// separate line for its author to write. Spec 12 gives a check the signature
/// `-> [Finding]` for this, and a verdict that named the first omission would
/// make the rest cost one run each to find.
#[test]
fn a_required_facet_that_is_absent_is_a_finding_with_no_line_and_no_fix() {
    let run = fixture_run();
    let findings: Vec<&headwater_check::Finding> = run
        .findings
        .iter()
        .filter(|finding| finding.rule == facet_required::RULE)
        .collect();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].path, "check/spec/05-no-summary.md");
    assert!(findings[0].message.contains("`summary`"), "{findings:#?}");
    // The finding is that a key is absent, so it anchors at no line, and its
    // value is a sentence somebody has to write rather than one to derive.
    assert_eq!(findings[0].line, 0);
    assert!(!findings[0].fixable);

    // The requirement is inherited: `governed_document` declares it and
    // `design_spec` is one. A check that read only the kind's own declaration
    // would report nothing here at all.
    let instance = run
        .instances
        .iter()
        .find(|instance| {
            instance.rule == facet_required::RULE && instance.at() == "check/spec/05-no-summary.md"
        })
        .expect("the fixture");
    assert_eq!(instance.findings().len(), 1);
}

/// A kind that forbids every enumerated facet generates no instance of the enum
/// rule, which is what keeps the coverage finding reachable.
#[test]
fn the_enum_rule_generates_per_kind_and_reports_at_the_value() {
    let run = fixture_run();
    let findings: Vec<&headwater_check::Finding> = run
        .findings
        .iter()
        .filter(|finding| finding.rule == facet_value::RULE)
        .collect();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].path, "check/spec/06-retired.md");
    // At the value, because that is the text to change.
    assert_eq!(findings[0].line, 4);
    assert!(findings[0].message.contains("retired"), "{findings:#?}");

    // `03-no-instance.md` is a `note`, which forbids every enumerated facet.
    // No instance of either Shape rule is generated over it, and that is what
    // leaves it as the coverage rule's failing fixture.
    for rule in [facet_value::RULE, facet_required::RULE] {
        assert!(
            !run.instances
                .iter()
                .any(|instance| instance.rule == rule
                    && instance.at() == "check/spec/03-no-instance.md"),
            "{rule} generated an instance over a kind that forbids what it reads"
        );
    }
}

/// An endpoint is permitted when the kind at that end descends from one the
/// relation names, and the two ends do not swap when an author writes the
/// inverse.
#[test]
fn an_endpoint_is_read_through_the_is_a_chain() {
    let run = fixture_run();
    let findings: Vec<&headwater_check::Finding> = run
        .findings
        .iter()
        .filter(|finding| finding.rule == endpoint::RULE)
        .collect();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].path, "check/evaluations/eta.md");
    assert!(findings[0].message.contains("from"), "{findings:#?}");

    // The passing half of the same relation: `01-one-half.md` writes
    // `assesses: EVAL-FIX-alpha`, whose target is an `evaluation` against a
    // declared `to: [governed_document]`. A check that compared the two names
    // directly would report every inherited endpoint in a corpus.
    let inherited = run
        .instances
        .iter()
        .find(|instance| {
            instance.rule == endpoint::RULE
                && instance.paths() == ["check/spec/01-one-half.md", "check/evaluations/alpha.md"]
        })
        .expect("the fixture");
    assert!(inherited.findings().is_empty(), "{inherited:#?}");
}

/// The clock decides this rule, and the fixture holds one document on each side
/// of the window.
#[test]
fn a_participation_expectation_is_met_by_either_half_or_by_an_open_window() {
    let run = fixture_run();
    let findings: Vec<&headwater_check::Finding> = run
        .findings
        .iter()
        .filter(|finding| finding.rule == participation::RULE)
        .collect();

    // `epsilon.md` is outside the window and nothing reaches it.
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].path, "check/evaluations/epsilon.md");
    // The severity is the taxonomy's, because the check is generated from the
    // declaration that carries it.
    assert_eq!(findings[0].severity, headwater_check::Severity::Warn);
    // At the origin facet, which is the line that started the window.
    assert_eq!(findings[0].line, 4);

    // `zeta.md` is in the same state with the same absent edge, and it passes
    // because its window is still open. Move the injected date and this is the
    // verdict that changes.
    let inside = run
        .instances
        .iter()
        .find(|instance| {
            instance.rule == participation::RULE && instance.at() == "check/evaluations/zeta.md"
        })
        .expect("the fixture");
    assert!(inside.findings().is_empty(), "{inside:#?}");

    // `beta.md` wrote no half of its own. The half `01-one-half.md` wrote
    // reaches it at the target end of the same relation, and the expectation is
    // satisfied by it: matching runs on the relation type and the end, never on
    // the name an author reached for.
    let other_end = run
        .instances
        .iter()
        .find(|instance| {
            instance.rule == participation::RULE && instance.at() == "check/evaluations/beta.md"
        })
        .expect("the fixture");
    assert!(other_end.findings().is_empty(), "{other_end:#?}");
    assert_eq!(
        other_end.paths(),
        ["check/evaluations/beta.md", "check/spec/01-one-half.md"],
        "the neighbourhood read set is the centre and its neighbours"
    );
}

/// The same corpus on a later date reaches a different verdict.
///
/// The clock is an input, and this is the test that says so from outside the
/// cache: nothing in the tree moves, and the report does.
#[test]
fn the_injected_clock_changes_a_verdict_and_nothing_else_does() {
    let corpus = Corpus::new(fixtures_dir(), "check");
    let root = load_map(&fixtures_dir().join("check.taxonomy.yml"));
    let later = run_at(
        &corpus,
        &root,
        &fixture_lock(),
        &Context::at(Date::parse("2026-09-30").expect("a date")),
        &mut Cache::disabled(),
    );
    let overdue: Vec<&str> = later
        .findings
        .iter()
        .filter(|finding| finding.rule == participation::RULE)
        .map(|finding| finding.path.as_str())
        .collect();
    // `zeta.md` and `eta.md` were both inside their windows at the pinned
    // date and are outside them here. Nothing in the tree moved.
    assert_eq!(
        overdue,
        [
            "check/evaluations/epsilon.md",
            "check/evaluations/eta.md",
            "check/evaluations/zeta.md"
        ],
        "the windows did not close"
    );
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
/// `02-cited-only.md` declares no edge at all, and the reciprocity instance
/// over the pair that names it still reads it. A coverage report that counted
/// that instance against one end would call the file unchecked by the rule that
/// did look at it, and send its author to a shelf pattern that is already
/// right.
#[test]
fn an_edge_instance_is_counted_against_both_of_its_endpoints() {
    let run = fixture_run();
    let counted = run
        .instances
        .iter()
        .filter(|instance| {
            instance.rule == reciprocity::RULE
                && instance.paths().contains(&"check/spec/02-cited-only.md")
        })
        .count();
    assert_eq!(counted, 1, "{:#?}", run.instances);

    let document = run
        .coverage
        .documents
        .iter()
        .find(|document| document.path == "check/spec/02-cited-only.md")
        .expect("the fixture");
    assert!(document.ran >= counted, "{document:#?}");
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
    assert_eq!(run.coverage.seen(), 17);
    assert_eq!(run.coverage.classified(), 16);
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

/// Every rule reports a scope, and each one comes from the trait that binds it.
///
/// The order is [`headwater_check::RULES`], because a report that listed the
/// scopes in one order and the instance counts in another would be two lists
/// that a reader has to reconcile.
#[test]
fn the_scope_of_every_rule_comes_from_the_trait_that_binds_it() {
    let run = fixture_run();
    let listed: Vec<&str> = run.served.iter().map(|served| served.rule).collect();
    assert_eq!(listed, headwater_check::RULES.to_vec());

    let grains: Vec<Grain> = run
        .served
        .iter()
        .map(|served| served.scope.grain())
        .collect();
    assert_eq!(
        grains,
        [
            Grain::Document,
            Grain::Document,
            Grain::Document,
            Grain::Edge,
            Grain::Edge,
            Grain::Neighbourhood { depth: 1 },
            // The four Document-origin rules, which read the body rather than
            // the front matter. The grain is the same and the view is not:
            // each one declares `NEEDS_BODY`.
            Grain::Document,
            Grain::Document,
            Grain::Document,
            Grain::Document,
            Grain::Corpus,
        ]
    );
    let bodies: Vec<&str> = run
        .served
        .iter()
        .filter(|served| served.scope.needs_body())
        .map(|served| served.rule)
        .collect();
    assert_eq!(
        bodies,
        [voice::RULE, language::RULE, sections::RULE, fragment::RULE]
    );

    // The required-facet check reads front matter and never a body, and the
    // report says so rather than leaving a reader to infer it from the name.
    assert!(!run.served[0].scope.needs_body());
    assert_eq!(
        run.served[0].scope.render(),
        "document scope, one document and its front matter"
    );

    // Exactly one rule reads the clock, and the report names it. A reader who
    // asks why a warm run re-evaluated one rule and not another reads it here.
    let clocked: Vec<&str> = run
        .served
        .iter()
        .filter(|served| served.scope.needs_clock())
        .map(|served| served.rule)
        .collect();
    assert_eq!(clocked, [participation::RULE]);
    assert_eq!(
        run.served[5].scope.render(),
        "neighbourhood scope, one document and the documents one relation away from it, \
         and the injected clock"
    );
}

/// A document check receives the body only when it declares that it needs one.
///
/// This is scope enforcement at the one grain where it is observable at run
/// time. The rest is structural: `DocumentView` has no accessor for a second
/// document, so a document-scoped check cannot read a sibling and no test can
/// be written that catches it doing so.
#[test]
fn a_document_check_receives_the_body_only_when_it_declares_it() {
    struct Reader<const BODY: bool>;

    impl<const BODY: bool> DocumentCheck for Reader<BODY> {
        const RULE: &'static str = "test.body_arrives";
        const VERSION: u32 = 1;
        const NEEDS_BODY: bool = BODY;

        fn evaluate(&self, view: &DocumentView<'_>) -> Outcome {
            match view.body() {
                Some(_) => Outcome::Skipped("the body arrived".to_string()),
                None => Outcome::Passed,
            }
        }
    }

    let census = fixture_census();
    let declared = over_documents(&Reader::<true>, &census, &pinned(), &mut Cache::disabled());
    let did_not = over_documents(&Reader::<false>, &census, &pinned(), &mut Cache::disabled());

    assert_eq!(declared.len(), 16, "one instance per typed document");
    assert_eq!(declared.len(), did_not.len());
    assert!(declared
        .iter()
        .all(|instance| matches!(instance.outcome, Outcome::Skipped(_))));
    assert!(did_not
        .iter()
        .all(|instance| matches!(instance.outcome, Outcome::Passed)));
}

/// The runner records what the view carried, and never what the check says.
///
/// A check that reports nothing at all still produces instances that read what
/// its scope fixes: one document at document grain, and both endpoints at edge
/// grain. That is the property every cache key and every coverage number
/// rests on, and it is the one a returned scope cannot give.
#[test]
fn the_read_set_of_an_instance_comes_from_the_view_and_not_from_the_check() {
    struct Quiet;

    impl EdgeCheck for Quiet {
        const RULE: &'static str = "test.every_pair";
        const VERSION: u32 = 1;

        fn evaluate(&self, _view: &EdgeView<'_>) -> Outcome {
            Outcome::Passed
        }
    }

    struct Silent;

    impl DocumentCheck for Silent {
        const RULE: &'static str = "test.every_document";
        const VERSION: u32 = 1;

        fn evaluate(&self, _view: &DocumentView<'_>) -> Outcome {
            Outcome::Passed
        }
    }

    let census = fixture_census();
    let corpus = Corpus::new(fixtures_dir(), "check");
    let root = load_map(&fixtures_dir().join("check.taxonomy.yml"));
    let declarations = Declarations::read(&root).expect("the declarations read");
    let graph = Graph::build(
        &census,
        &declarations,
        &Resolvers::over(&corpus),
        &corpus,
        &Config::default(),
    );

    for instance in over_documents(&Silent, &census, &pinned(), &mut Cache::disabled()) {
        assert_eq!(instance.reads.len(), 1, "{instance:#?}");
        // And each read carries the hash of what it read, which is the half
        // that #54 left for the cache key to need.
        assert!(instance.reads[0].digest.is_some(), "{instance:#?}");
    }

    // A check with no generation step of its own gets every declared pair,
    // where the reciprocity rule declares `required` and gets four of them.
    let every = over_edges(
        &Quiet,
        &graph,
        &Digests::of(&census),
        &pinned(),
        &mut Cache::disabled(),
    );
    assert!(
        every.len() > 4,
        "the fixture tree declares a pair that requires no reciprocity"
    );
    for instance in &every {
        assert_eq!(instance.reads.len(), 2, "{instance:#?}");
        assert!(
            instance.reads.iter().all(|input| input.digest.is_some()),
            "{instance:#?}"
        );
    }
}
