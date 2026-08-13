// SPDX-License-Identifier: Apache-2.0
//! The read fixtures, and the properties a read has to hold whatever the corpus
//! says.
//!
//! Two levels, and they are recorded at different grains on purpose. The
//! fixture tree under `fixtures/` is written for these reads, so `query.reads`
//! records every answer in full: the routes, the pointers, the explanations and
//! the traversals. Nothing outside this crate can move those bytes.
//!
//! The run over this repository asserts properties instead of recording
//! answers. Every summary in `docs/spec/` is prose somebody edits, and a
//! recorded route over it would be re-blessed on the week's first paragraph.
//! What is asserted there is what a prose edit must not change: that a read
//! returns no content, that a route stays inside its budget, and that two runs
//! agree byte for byte.
//!
//!     HEADWATER_BLESS=1 cargo test -p headwater-query --test reads
//!
//! Read the diff before committing it. A blessed fixture is the change.

use headwater_census::census::{self, Census};
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::Shape;
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use headwater_query::{Budget, Resolved, Surface};
use headwater_yaml::Mapping;
use std::path::{Path, PathBuf};

/// The task descriptions the recorded file answers, in the order it holds them.
///
/// Each one is here for a property rather than for coverage. In order: a
/// rationale task that a succession pair both answers; a behavior task where a
/// satellite outscores its nucleus; a procedure task whose purpose only one
/// kind serves; a task that reaches the corpus through a code path rather than
/// through any prose; a task no purpose answers; and a task whose purpose
/// matches and whose terms reach no document, which is the confidence gate.
const TASKS: [&str; 6] = [
    "why is throttling applied at the edge",
    "what does the ingest service do",
    "how do I rotate the keys",
    "add rate limiting to src/ingest/rate_limit.rs",
    "deploy the cluster to another region",
    "what may I rely on when the tide comes in",
];

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
        "\nthe reads no longer match {}",
        recorded.display()
    );
}

/// Everything a surface borrows, owned, so that a test may build one.
struct Built {
    census: Census,
    graph: Graph,
    shape: Shape,
    taxonomy: Taxonomy,
    relations: Declarations,
}

impl Built {
    fn over(corpus: &Corpus, root: &Mapping) -> Self {
        let taxonomy = Taxonomy::read(root).expect("the taxonomy reads");
        let relations = Declarations::read(root).expect("the declarations read");
        let shape = Shape::read(root).expect("the shape reads");
        let census = census::take(corpus, &taxonomy);
        let graph = Graph::build(
            &census,
            &relations,
            &Resolvers::over(corpus),
            corpus,
            &Config::default(),
        );
        Built {
            census,
            graph,
            shape,
            taxonomy,
            relations,
        }
    }

    fn surface(&self) -> Surface<'_> {
        Surface::over(
            &self.census,
            &self.graph,
            &self.shape,
            &self.taxonomy,
            &self.relations,
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

fn fixture_tree() -> Built {
    let corpus = Corpus::new(fixtures_dir(), "query");
    let root = load_map(&fixtures_dir().join("query.taxonomy.yml"));
    Built::over(&corpus, &root)
}

fn this_repository() -> Built {
    let root = repository_root();
    let resolved = headwater_resolve::repository(&root)
        .unwrap_or_else(|errors| panic!("{}", headwater_resolve::render_errors(&errors)));
    let corpus = Corpus::declared(
        &root,
        &resolved.consumer.corpus_root,
        &resolved.consumer.exclusions,
    );
    Built::over(&corpus, &resolved.resolution.taxonomy)
}

/// Every read over the fixture tree, as one file.
fn reads(surface: &Surface<'_>) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    for task in TASKS {
        out.push_str(&surface.route(task, Budget::default()).render());
        out.push('\n');
    }

    out.push_str("explain\n");
    for target in [
        "query/specs/api-design.md",
        "DR-FIX-0031",
        "query/specs/asserted-notes.md",
    ] {
        let explanation = surface.explain(target).expect("the fixture");
        for line in explanation.render().lines() {
            let _ = writeln!(out, "  {line}");
        }
    }

    out.push_str("\nresolve_identifier\n");
    for id in ["SPEC-FIX-ingest", "SPEC-FIX-ingset", "nothing-like-it"] {
        let _ = match surface.resolve_identifier(id) {
            Resolved::Document(pointer) => writeln!(out, "  {id} is {}", pointer.render()),
            Resolved::NearMiss(near) => writeln!(out, "  {id} is nothing, and {near} is near it"),
            Resolved::Nothing => writeln!(out, "  {id} is nothing, and nothing is near it"),
        };
    }

    out.push_str("\ngoverning_docs_for_path\n");
    for path in ["src/ingest/rate_limit.rs", "src/ingest", "src/nothing.rs"] {
        let governing = surface.governing_docs_for_path(path);
        match governing.is_empty() {
            true => {
                let _ = writeln!(out, "  {path} is governed by nothing");
            }
            false => {
                for pointer in governing {
                    let _ = writeln!(out, "  {path} — {}", pointer.render());
                }
            }
        }
    }
    out
}

#[test]
fn the_fixture_tree_answers_the_recorded_reads() {
    let built = fixture_tree();
    compare(
        &fixtures_dir().join("query.reads"),
        &reads(&built.surface()),
    );
}

/// Two surfaces over one corpus answer identically.
///
/// [Spec 12](../../../../docs/spec/12-check-layer.md#determinism-concretely)
/// binds a run to "same corpus, same lock, same injected clock, byte-identical
/// output", and a read an agent depends on is held to the same bar. This is
/// that comparison, run over this repository rather than over the tree written
/// for it: two independent walks, two graphs, and one set of bytes.
#[test]
fn two_reads_of_one_corpus_are_byte_identical() {
    let first = this_repository();
    let second = this_repository();
    let (first, second) = (first.surface(), second.surface());
    for task in TASKS {
        assert_eq!(
            first.route(task, Budget::default()).render(),
            second.route(task, Budget::default()).render(),
            "{task}"
        );
    }
    let target = "docs/spec/05-ai-integration.md";
    assert_eq!(
        first.explain(target).expect("the spec").render(),
        second.explain(target).expect("the spec").render()
    );
}

/// A route stays inside its budget, and the budget is what says so.
#[test]
fn a_route_offers_no_more_than_its_budget() {
    let built = this_repository();
    let surface = built.surface();
    for pointers in [1, 3, 5] {
        for task in TASKS {
            let route = surface.route(task, Budget { pointers });
            assert!(
                route.pointers.len() <= pointers,
                "{task} offered {} pointers under a budget of {pointers}",
                route.pointers.len()
            );
        }
    }
}

/// A pointer carries a path and a cue, and never content.
///
/// Spec 5: "A task description resolves to a ranked, budget-capped set of
/// pointers: paths and one-line summaries, never content." The property that
/// makes it checkable is that every string a pointer carries is one the
/// document declared in its front matter, so no line of any body reaches a
/// caller. This asserts the observable half: a summary that is not the
/// document's own summary facet is a leak.
#[test]
fn a_pointer_carries_the_declared_summary_and_no_body() {
    let built = this_repository();
    let surface = built.surface();
    for document in surface.documents() {
        let pointer = surface.pointer(&document);
        assert_eq!(pointer.summary, surface.summary(&document));
        assert_eq!(pointer.path, document.path);
    }
}

/// The route over this repository's own corpus finds its own specification.
///
/// A read that never returns anything is indistinguishable from one that does
/// not work, and the fixture tree proves the mechanism against documents
/// written for it. This one runs against the real corpus, where the summaries
/// are prose nobody wrote for a test. The assertion is the property rather than
/// the list: a task in the language of this corpus reaches a document of it.
#[test]
fn a_task_in_this_corpus_reaches_a_document_of_it() {
    let built = this_repository();
    let surface = built.surface();
    let route = surface.route(
        "why does the engine read the lock rather than the taxonomy sources",
        Budget::default(),
    );
    assert!(route.silence.is_none(), "{}", route.render());
    assert!(!route.pointers.is_empty(), "{}", route.render());
    assert!(
        route
            .pointers
            .iter()
            .all(|pointer| pointer.path.starts_with("docs/")),
        "{}",
        route.render()
    );
}

/// Derived reading precedence orders the list, and it outranks the score.
///
/// Two pairs, and each one is a clause of spec 2's derivation. The satellite
/// outscores its nucleus on this task and the nucleus is still offered first,
/// which is the property a sort by score alone cannot have. The successor is
/// offered before the document it superseded, and that pair is the one that
/// caught a real defect: the superseded half declares the inverse, and reading
/// the direction off the relation name rather than off the edge made the
/// superseded document govern.
#[test]
fn precedence_offers_the_nucleus_and_the_successor_first() {
    let built = fixture_tree();
    let surface = built.surface();

    let route = surface.route(
        "in detail, what error shapes does the ingest service return",
        Budget::default(),
    );
    let offered: Vec<&str> = route
        .pointers
        .iter()
        .map(|pointer| pointer.path.as_str())
        .collect();
    let nucleus = offered
        .iter()
        .position(|path| *path == "query/specs/api-design.md")
        .unwrap_or_else(|| panic!("{}", route.render()));
    let satellite = offered
        .iter()
        .position(|path| *path == "query/specs/api-errors.md")
        .unwrap_or_else(|| panic!("{}", route.render()));
    assert!(nucleus < satellite, "{}", route.render());

    let route = surface.route("why is throttling applied at the edge", Budget::default());
    let offered: Vec<&str> = route
        .pointers
        .iter()
        .map(|pointer| pointer.path.as_str())
        .collect();
    assert_eq!(
        offered.first(),
        Some(&"query/decisions/edge-throttling.md"),
        "{}",
        route.render()
    );
}

/// A task that names a path routes by identity rather than by scent.
///
/// The anchor is a node the graph already holds, so the documents that govern
/// it are offered first and no lexical score competes with them. This is also
/// the case that has no purpose scent at all: the task below matches one
/// purpose weakly and the right answer serves another.
#[test]
fn a_task_that_names_an_anchor_is_answered_by_the_documents_that_govern_it() {
    let built = fixture_tree();
    let surface = built.surface();
    let route = surface.route(
        "add rate limiting to src/ingest/rate_limit.rs",
        Budget::default(),
    );
    assert_eq!(route.anchors, vec!["src/ingest/rate_limit.rs".to_string()]);
    assert_eq!(
        route.pointers.first().map(|pointer| pointer.path.as_str()),
        Some("query/decisions/edge-throttling.md"),
        "{}",
        route.render()
    );
}

/// A pointer to an unaccepted document says so, and one to an accepted document
/// does not.
///
/// Spec 5 reaches Q15 through this surface: "To offer such a pointer silently
/// is the failure that Q15 exists to prevent, reached through our own routing
/// surface."
#[test]
fn a_pointer_to_an_asserted_document_states_the_warrant() {
    let built = fixture_tree();
    let surface = built.surface();
    let asserted = surface
        .explain("query/specs/asserted-notes.md")
        .expect("the fixture");
    assert_eq!(asserted.warrant.as_deref(), Some("asserted"));

    let document = surface
        .find("query/specs/asserted-notes.md")
        .expect("the fixture");
    let pointer = surface.pointer(&document);
    assert!(pointer.unwarranted);
    assert!(pointer.render().contains("nobody accepted this document"));

    let accepted = surface.find("query/specs/ingest.md").expect("the fixture");
    let pointer = surface.pointer(&accepted);
    assert!(!pointer.unwarranted);
    assert!(!pointer.render().contains("accepted"));
}
