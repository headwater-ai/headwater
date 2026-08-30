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
//! returns no content, that a route ranks no more than its budget and carries
//! every document that governs a path the task names, and that two runs agree
//! byte for byte.
//!
//!     HEADWATER_BLESS=1 cargo test -p headwater-query --test reads
//!
//! Read the diff before committing it. A blessed fixture is the change.

use headwater_census::census::{self, Census};
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::paint::ColorMode;
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
    config: Config,
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
        for line in explanation.render(ColorMode::Plain).lines() {
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
        first
            .explain(target)
            .expect("the spec")
            .render(ColorMode::Plain),
        second
            .explain(target)
            .expect("the spec")
            .render(ColorMode::Plain)
    );
}

/// The tasks that reach the budget arm over this repository's own corpus.
///
/// Held apart from `TASKS`, which `reads()` walks to write the recorded answers
/// of the fixture tree. None of those six names an anchor of this repository,
/// so a budget run over them cuts nothing and asserts nothing. Each task here
/// is one this corpus answers: the first names a path two documents govern, and
/// the second matches a purpose that more documents answer than any budget
/// below allows.
const BUDGET_TASKS: [&str; 2] = [
    "engine/crates/cli/src/main.rs",
    "why does the engine read the lock rather than the taxonomy sources",
];

/// A route offers no more ranked pointers than its budget, and every anchored
/// pointer whatever the budget.
///
/// The budget caps the ranking. It does not cap the documents an anchor named,
/// so the total can exceed it and the property is stated over the ranked half.
/// What the ranking dropped is not silent: the route counts it.
#[test]
fn a_route_ranks_no_more_than_its_budget_and_never_cuts_an_anchor() {
    let built = this_repository();
    let surface = built.surface();
    for pointers in [1, 3, 5] {
        for task in TASKS.iter().chain(BUDGET_TASKS.iter()) {
            let route = surface.route(task, Budget { pointers });
            let anchored: Vec<_> = route
                .anchors
                .iter()
                .flat_map(|anchor| surface.governing_docs_for_path(anchor))
                .collect();
            let ranked = route.pointers.len()
                - route
                    .pointers
                    .iter()
                    .filter(|pointer| anchored.contains(pointer))
                    .count();
            assert!(
                ranked <= pointers,
                "{task} ranked {ranked} pointers under a budget of {pointers}\n{}",
                route.render()
            );
            for pointer in &anchored {
                assert!(
                    route.pointers.contains(pointer),
                    "{task} dropped the anchored {} under a budget of {pointers}\n{}",
                    pointer.path,
                    route.render()
                );
            }
        }
    }
}

/// A pointer carries a path and a cue, and never content.
///
/// Spec 5: "A task description resolves to a ranked set of pointers: a path,
/// the name the document declares, a one-line summary, and never content." The
/// property that
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

/// A pointer says what the document is called, and it reads that from the facet
/// in the `name` role rather than from the path or the identifier.
///
/// The distinction is the whole point of the field. A path and an identifier
/// are conventions an author followed, and a reader who is told
/// `docs/interfaces/headwater-check.md` has been told a file name. The `name`
/// role is a declaration, and for an `interface_contract` its value is the
/// command a caller types.
///
/// The second assertion is the one that would catch a name derived from the
/// path: `headwater check` carries a space where the file name carries a dash,
/// so a derivation from either string cannot produce it.
#[test]
fn a_pointer_names_the_document_from_the_facet_in_the_name_role() {
    let built = this_repository();
    let surface = built.surface();
    for document in surface.documents() {
        let pointer = surface.pointer(&document);
        assert_eq!(pointer.name, surface.name(&document), "{}", document.path);
    }

    let contract = surface
        .find("docs/interfaces/headwater-check.md")
        .expect("the contract for `headwater check`");
    let pointer = surface.pointer(&contract);
    assert_eq!(pointer.name.as_deref(), Some("headwater check"));
    assert!(
        pointer.render().contains("(headwater check)"),
        "{}",
        pointer.render()
    );
}

/// An edit to a crate a contract governs routes to that contract, and the route
/// says which command the contract describes.
///
/// This is what the `PostToolUse` position of `.claude/hooks/write.sh` prints,
/// asserted against the engine rather than against the hook, because the hook
/// carries no rule of its own and this is the rule.
///
/// The assertion is not the substring `headwater check`, which the contract's
/// own summary also holds and which would pass whether the name was read or
/// not. It is the parenthesized form, which only [`Pointer::render`] writes.
#[test]
fn an_edit_to_a_governed_crate_routes_to_the_contract_and_names_its_verb() {
    let built = this_repository();
    let surface = built.surface();
    let route = surface.route("engine/crates/check/src/lib.rs", Budget::default());
    assert_eq!(
        route.anchors,
        vec!["engine/crates/check/src/lib.rs".to_string()],
        "{}",
        route.render()
    );
    let contract = route
        .pointers
        .iter()
        .find(|pointer| pointer.path == "docs/interfaces/headwater-check.md")
        .unwrap_or_else(|| panic!("{}", route.render()));
    assert_eq!(contract.kind, "interface_contract");
    assert_eq!(contract.name.as_deref(), Some("headwater check"));
    assert!(
        route.render().contains("(headwater check)"),
        "{}",
        route.render()
    );
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

/// A document that governs the named path is carried whatever the budget is.
///
/// Spec 5 caps the ranking and not the identities. A document that declares
/// `governs` over the path was named rather than ranked, so the budget has no
/// basis on which to remove it, and `governing_docs_for_path` already answers
/// this question with no budget at all.
///
/// The corpus is this repository rather than the tree under `fixtures/`. Each
/// governed path of that tree has one governing document, so no budget of one
/// or more can cut anything there, and a case placed there would assert
/// nothing. This repository has a path with two governors.
///
/// The expectation is derived from `governing_docs_for_path` rather than
/// written as a number, so the case states that the two surfaces agree rather
/// than that a count is two.
#[test]
fn a_route_carries_every_governing_document_whatever_the_budget() {
    let built = this_repository();
    let surface = built.surface();
    let path = "engine/crates/cli/src/main.rs";
    let governing = surface.governing_docs_for_path(path);
    // Loose, and first on purpose: if this is the assertion that fires, the
    // corpus moved rather than the mechanism broke.
    assert!(
        governing.len() > 1,
        "{path} no longer has more than one governing document, so this case \
         reaches nothing. Find a path that does."
    );
    let route = surface.route(path, Budget { pointers: 1 });
    for pointer in &governing {
        assert!(
            route.pointers.contains(pointer),
            "the budget dropped {}, which governs {path}\n{}",
            pointer.path,
            route.render()
        );
    }
}

/// One document is offered once, however many anchors of the task it governs.
///
/// The anchored set is the union over every anchor the task named.
/// `governing_docs_for_path` deduplicates inside one anchor and cannot see
/// across them, so a document that governs two named paths arrived twice. The
/// truncation used to hide this at a low budget; carrying the identities whole
/// shows it.
#[test]
fn a_document_that_governs_two_named_anchors_is_offered_once() {
    let built = this_repository();
    let surface = built.surface();
    let task = "engine/crates/cli/src/main.rs engine/crates/check/src/lib.rs";
    let route = surface.route(task, Budget::default());
    assert_eq!(route.anchors.len(), 2, "{}", route.render());
    let mut paths: Vec<&str> = route
        .pointers
        .iter()
        .map(|pointer| pointer.path.as_str())
        .collect();
    let offered = paths.len();
    paths.sort_unstable();
    paths.dedup();
    assert_eq!(
        paths.len(),
        offered,
        "a pointer is offered twice\n{}",
        route.render()
    );
}

/// A route says how many ranked pointers the budget removed.
///
/// The count is what a silent `truncate` destroyed: the difference between
/// "there were three" and "there were fifteen and you were shown three". The
/// property is stated against a wider budget rather than against a number, so
/// it holds whatever this corpus grows into: what a route offers plus what it
/// says it withheld is what a route with room for everything offers.
#[test]
fn a_route_says_how_many_ranked_pointers_the_budget_withheld() {
    let built = this_repository();
    let surface = built.surface();
    let task = "why does the engine read the lock rather than the taxonomy sources";
    let whole = surface.route(task, Budget { pointers: 512 });
    // Loose, and first on purpose: if this fires, the corpus stopped having
    // more answers to this task than the budget below allows.
    assert!(whole.pointers.len() > 3, "{}", whole.render());
    assert_eq!(whole.withheld, 0, "{}", whole.render());

    let cut = surface.route(task, Budget { pointers: 3 });
    assert_eq!(
        cut.pointers.len() + cut.withheld,
        whole.pointers.len(),
        "the withheld count does not account for what a wider budget offers\n{}",
        cut.render()
    );
}

/// The withheld count reaches the reader, and it is not mistaken for a pointer.
///
/// `.claude/hooks/write.sh` selects the pointer lines of a rendered route by
/// grepping for the separator a pointer carries, so a count line carrying that
/// separator would be shown to an author as though it were a document, and a
/// count line is not one.
#[test]
fn the_withheld_line_is_rendered_and_is_not_shaped_like_a_pointer() {
    let built = this_repository();
    let surface = built.surface();
    let task = "why does the engine read the lock rather than the taxonomy sources";
    let route = surface.route(task, Budget { pointers: 3 });
    assert!(route.withheld > 0, "{}", route.render());
    let line = route
        .render()
        .lines()
        .find(|line| line.contains("withheld"))
        .map(str::to_string)
        .unwrap_or_else(|| panic!("no withheld line\n{}", route.render()));
    assert!(
        line.contains(&route.withheld.to_string()),
        "the line does not carry the count: {line}"
    );
    assert!(
        !line.contains(" — "),
        "the withheld line is shaped like a pointer: {line}"
    );
}

/// A route that cuts nothing says nothing about a cut.
#[test]
fn a_route_within_its_budget_renders_no_withheld_line() {
    let built = this_repository();
    let surface = built.surface();
    let route = surface.route(
        "why does the engine read the lock rather than the taxonomy sources",
        Budget { pointers: 512 },
    );
    assert_eq!(route.withheld, 0);
    assert!(!route.render().contains("withheld"), "{}", route.render());
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
