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
//!
//! The `governing_docs_for_path` block of the recorded reads is half of the
//! test design HW-VER-0004 states, over the two stubs under `fixtures/src/`.
//! This sentence binds that verification's `cited_in` edge onto this file.

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

/// The mode every route rendered here takes.
///
/// A recorded fixture is bytes on disk and a failure message is read out of a
/// captured stream, so both are the piped case, where `paint` is the identity.
/// The color a terminal sees is asserted by `tools/engine/color-fixtures.sh`, which
/// attaches a real one; nothing in this file could, and a recorded fixture that
/// held escape sequences would be re-blessed against whatever terminal last ran
/// the suite.
const PLAIN: ColorMode = ColorMode::Plain;

/// The task descriptions the recorded file answers, in the order it holds them.
///
/// Each one is here for a property rather than for coverage. In order: a
/// rationale task that a succession pair both answers; a behavior task where a
/// satellite outscores its nucleus; a procedure task whose purpose only one
/// kind serves; a task that reaches the corpus through a code path rather than
/// through any prose; a task no purpose answers; and a task whose purpose
/// matches and whose terms reach no document, which is the confidence gate;
/// and the declared name of a decision, whose words match `behavior` alone, so
/// that only the name step offers the decision (HW-DR-0109).
const TASKS: [&str; 7] = [
    "why is throttling applied at the edge",
    "what does the ingest service do",
    "how do I rotate the keys",
    "add rate limiting to src/ingest/rate_limit.rs",
    "deploy the cluster to another region",
    "what may I rely on when the tide comes in",
    "Throttling at the edge",
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
        out.push_str(&surface.route(task, Budget::default()).render(PLAIN));
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
    for id in [
        "SPEC-FIX-ingest",
        "SPEC-FIX-ingset",
        "nothing-like-it",
        // The decisive case: no typed document carries this, and
        // `query/notes/orphan.md` carries it while the census leaves that
        // document untyped. `near_miss` is an exact match, so the identifier
        // this prints and the one asked for are the same string — that used
        // to be the defect (#845), and the rendering below no longer treats
        // it as one.
        "SPEC-FIX-orphan",
    ] {
        let _ = match surface.resolve_identifier(id) {
            Resolved::Document(pointer) => writeln!(out, "  {id} is {}", pointer.render()),
            Resolved::NearMiss(path) => {
                writeln!(out, "  {id} is not typed, and {path} carries it untyped")
            }
            Resolved::Nothing => writeln!(out, "  {id} is nothing, and nothing is near it"),
        };
    }

    out.push_str("\ngoverning_docs_for_path\n");
    for path in [
        "src/ingest/rate_limit.rs",
        // A member of the list `query/decisions/edge-throttling.md` now
        // governs alongside `src/ingest/rate_limit.rs`
        // (HW-DR-0074: "a list is one anchor ... its matched set is the union
        // of what its members match"), and also a path `query/specs/ingest.md`
        // reaches only because its anchor is a pattern now: two anchors that
        // both match one file are two nodes, and this is that file.
        "src/ingest/mod.rs",
        "src/ingest",
        "src/nothing.rs",
    ] {
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
            first.route(task, Budget::default()).render(PLAIN),
            second.route(task, Budget::default()).render(PLAIN),
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
                route.render(PLAIN)
            );
            for pointer in &anchored {
                assert!(
                    route.pointers.contains(pointer),
                    "{task} dropped the anchored {} under a budget of {pointers}\n{}",
                    pointer.path,
                    route.render(PLAIN)
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
        route.render(PLAIN)
    );
    let contract = route
        .pointers
        .iter()
        .find(|pointer| pointer.path == "docs/interfaces/headwater-check.md")
        .unwrap_or_else(|| panic!("{}", route.render(PLAIN)));
    assert_eq!(contract.kind, "interface_contract");
    assert_eq!(contract.name.as_deref(), Some("headwater check"));
    assert!(
        route.render(PLAIN).contains("(headwater check)"),
        "{}",
        route.render(PLAIN)
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
    assert!(route.silence.is_none(), "{}", route.render(PLAIN));
    assert!(!route.pointers.is_empty(), "{}", route.render(PLAIN));
    assert!(
        route
            .pointers
            .iter()
            .all(|pointer| pointer.path.starts_with("docs/")),
        "{}",
        route.render(PLAIN)
    );
}

/// One purpose no longer takes every slot of the default budget.
///
/// The task is the one #915 records. Before HW-DR-0070 its purpose scores put
/// `obligation` first, and the first twelve pointers of the route were all
/// obligation records, so the default five held one kind. The property is the
/// ruling rather than a list: the five hold more than one kind and more than
/// one purpose, and each pointer carries the terms that reached it and its
/// rank in the order by score.
#[test]
fn a_task_that_matches_several_purposes_is_offered_more_than_one_kind() {
    let built = this_repository();
    let surface = built.surface();
    let route = surface.route(
        "Decide which shelf a new document belongs on, and where its identifier comes from.",
        Budget::default(),
    );
    assert!(route.matched.len() > 1, "{}", route.render(PLAIN));
    let mut kinds: Vec<&str> = route.pointers.iter().map(|p| p.kind.as_str()).collect();
    kinds.dedup();
    kinds.sort_unstable();
    kinds.dedup();
    assert!(kinds.len() > 1, "{}", route.render(PLAIN));
    let mut purposes: Vec<Option<&str>> = route
        .pointers
        .iter()
        .map(|p| p.purpose.as_deref())
        .collect();
    purposes.sort_unstable();
    purposes.dedup();
    assert!(purposes.len() > 1, "{}", route.render(PLAIN));

    assert_eq!(route.evidence.len(), route.pointers.len());
    for evidence in &route.evidence {
        let headwater_query::Evidence::Ranked { terms, rank, of } = evidence else {
            panic!("no anchor was named: {}", route.render(PLAIN));
        };
        assert!(!terms.is_empty(), "{}", route.render(PLAIN));
        assert!(
            terms.iter().all(|term| route.distinctive.contains(term)),
            "{}",
            route.render(PLAIN)
        );
        assert!((1..=*of).contains(rank), "{}", route.render(PLAIN));
    }
}

/// The declared name of a record, with the number its title opens with
/// removed, as a person who copied the title out of the page would type it.
///
/// Written here rather than taken from the route, because the property below
/// is that the route answers a title, and a test that stripped the number with
/// the route's own function could not see that function go wrong. The shape is
/// the one the decision shelves use: `12 — Migration path for an existing
/// corpus`, or a title with no number at all on the process shelf.
fn title_as_typed(name: &str) -> String {
    let rest = name.trim_start_matches(|c: char| c.is_ascii_digit());
    if rest.len() == name.len() {
        return name.to_string();
    }
    rest.trim_start()
        .trim_start_matches(['—', '-'])
        .trim_start()
        .to_string()
}

/// Every decision record is offered, inside the default budget, for its own
/// title (#1470).
///
/// The title is the facet in the `name` role, so a person or an agent who
/// copies it into a route has named the record. Before HW-DR-0109 the record
/// still had to win the purpose gate, and a title whose words matched no
/// `rationale` question offered no decision at all: HW-DR-0004
/// (`relation storage`) matched no purpose, and HW-DR-0101 stood 6th with a
/// budget of 100,000.
///
/// Every record on both shelves is read, and a record with no name is a
/// failure here, not a skip: a record the property cannot be asked of is a
/// record nobody could reach by its title.
///
/// One record is outside the property by the rule and is named here with the
/// reason, in [`ONE_WORD_TITLES`]. A miss on any other record fails.
///
/// The test is also the measurement, with one line per record of its place in
/// the default budget and what reached it:
///
///     cargo test -p headwater-query --test reads every_decision_record -- --nocapture
#[test]
fn every_decision_record_is_offered_for_its_own_title_inside_the_default_budget() {
    let built = this_repository();
    let surface = built.surface();
    let mut asked = 0;
    let mut missed: Vec<String> = Vec::new();
    let mut one_word: Vec<&str> = Vec::new();
    for document in surface.documents() {
        if document.kind != "decision" && document.kind != "process_decision" {
            continue;
        }
        asked += 1;
        let Some(name) = surface.name(&document) else {
            missed.push(format!("{} declares no name", document.path));
            continue;
        };
        let task = title_as_typed(&name);
        let route = surface.route(&task, Budget::default());
        let offered = route
            .pointers
            .iter()
            .position(|pointer| pointer.path == document.path);
        // The measurement #1470 asks for, one line per record: its place in the
        // default five and what reached it. `-- --nocapture` prints it.
        println!(
            "{}\t{}\t{}",
            document.path,
            offered.map_or("-".to_string(), |at| (at + 1).to_string()),
            offered.map_or("-", |at| match route.evidence[at] {
                headwater_query::Evidence::Named { .. } => "anchor",
                headwater_query::Evidence::Titled { .. } => "name",
                headwater_query::Evidence::Ranked { .. } => "terms",
            })
        );
        if offered.is_none() {
            if ONE_WORD_TITLES.contains(&document.path) {
                one_word.push(document.path);
                continue;
            }
            let place = surface
                .route(&task, Budget { pointers: 100_000 })
                .pointers
                .iter()
                .position(|pointer| pointer.path == document.path)
                .map_or("not offered".to_string(), |at| {
                    format!("offered {}", at + 1)
                });
            missed.push(format!("{} for {task:?}: {place}", document.path));
        }
    }
    assert!(asked > 100, "only {asked} decision records were read");
    assert!(
        missed.is_empty(),
        "{} of {asked} decision records are outside the default budget for their own title:\n{}",
        missed.len(),
        missed.join("\n")
    );
    // A listed record that the route now offers is a stale list, and so is one
    // that left the shelf.
    assert_eq!(
        one_word, ONE_WORD_TITLES,
        "the records outside the property by the rule"
    );
}

/// The paths of a route whose pointer the name step offered.
fn named_by_the_task(route: &headwater_query::Route) -> Vec<&str> {
    route
        .pointers
        .iter()
        .zip(&route.evidence)
        .filter(|(_, evidence)| matches!(evidence, headwater_query::Evidence::Titled { .. }))
        .map(|(pointer, _)| pointer.path.as_str())
        .collect()
}

/// The sentence queries that #1470 measures, the two tasks of this corpus that
/// the tests above route, and the documents each one names by name.
///
/// A name pointer here is earned only where the task holds a declared name as
/// a run of words. The first sentence holds `taxonomy resolution`, the name of
/// the subsystem spec it asks about, the second holds `headwater mcp`, and the
/// third holds `headwater export`. The two tasks of this corpus hold no name.
/// The six tasks of the fixture tree are held by the recorded reads, where only
/// the title task carries a `repeats its name` line.
const NAMED: [(&str, &[&str]); 5] = [
    (
        "add a technical design spec for the taxonomy resolution subsystem",
        &["docs/subsystems/taxonomy-resolution.md"],
    ),
    (
        "an editor integration starts headwater mcp for each query instead of embedding the engine library",
        &["docs/interfaces/headwater-mcp.md"],
    ),
    (
        "headwater export now withholds every write when the run refuses before its first write, matching the order generate already uses",
        &["docs/interfaces/headwater-export.md"],
    ),
    (
        "why does the engine read the lock rather than the taxonomy sources",
        &[],
    ),
    (
        "Decide which shelf a new document belongs on, and where its identifier comes from.",
        &[],
    ),
];

/// A task gains a name pointer only for a name it repeats (HW-DR-0109).
#[test]
fn a_task_names_only_the_documents_whose_name_it_repeats() {
    let built = this_repository();
    let surface = built.surface();
    for (task, named) in NAMED {
        let route = surface.route(task, Budget::default());
        assert_eq!(named_by_the_task(&route), named, "{}", route.render(PLAIN));
    }
}

/// A named pointer comes after the anchored ones and before the ranked ones,
/// once, and JSON writes its evidence as `by: name` with the words, in a
/// document of version `1.1` or later (HW-DR-0109), `1.2` today (#1901).
#[test]
fn a_named_pointer_is_offered_once_before_the_ranked_ones_and_written_as_by_name() {
    let built = fixture_tree();
    let surface = built.surface();
    let route = surface.route("Throttling at the edge", Budget::default());
    assert_eq!(
        named_by_the_task(&route),
        ["query/decisions/edge-throttling.md"],
        "{}",
        route.render(PLAIN)
    );
    assert_eq!(
        route.pointers[0].path,
        "query/decisions/edge-throttling.md",
        "{}",
        route.render(PLAIN)
    );
    let json = headwater_query::json::route(&route);
    let value = headwater_yaml::load(&json).expect("the route parses").value;
    let at = |path: &[&str]| -> String {
        let mut value = &value;
        for key in path {
            value = match value {
                headwater_yaml::Value::Map(map) => &map.get(key).expect(key).value,
                headwater_yaml::Value::Seq(items) => {
                    &items[key.parse::<usize>().expect("an index")].value
                }
                headwater_yaml::Value::Scalar(_) => panic!("{key} under a scalar"),
            };
        }
        match value {
            headwater_yaml::Value::Scalar(scalar) => scalar.text.clone(),
            headwater_yaml::Value::Seq(items) => items
                .iter()
                .map(|item| item.value.as_scalar().expect("a word").text.clone())
                .collect::<Vec<_>>()
                .join(" "),
            headwater_yaml::Value::Map(_) => panic!("a mapping at {path:?}"),
        }
    };
    assert_eq!(at(&["version"]), "1.2", "{json}");
    assert_eq!(at(&["pointers", "0", "evidence", "by"]), "name", "{json}");
    assert_eq!(
        at(&["pointers", "0", "evidence", "words"]),
        "throttling at the edge",
        "{json}"
    );

    // Once, in a route of this corpus where the named record also reaches the
    // ranked list: `rationale` matches the title of HW-DR-0070.
    let built = this_repository();
    let surface = built.surface();
    let route = surface.route(
        "The matched purposes take turns at a route budget, and each pointer states what reached it",
        Budget { pointers: 100_000 },
    );
    let mut paths: Vec<&str> = route.pointers.iter().map(|p| p.path.as_str()).collect();
    let offered = paths.len();
    paths.sort_unstable();
    paths.dedup();
    assert_eq!(paths.len(), offered, "a document is offered twice");
}

/// A named pointer takes a place of the budget, as an anchored one does, and
/// the budget never removes it.
#[test]
fn a_named_pointer_takes_a_place_of_the_budget_and_is_never_cut() {
    let built = fixture_tree();
    let surface = built.surface();
    let task = "Throttling at the edge";
    // The task reaches one ranked document as well, so a budget of one holds
    // the named pointer and withholds the ranked one.
    let route = surface.route(task, Budget { pointers: 1 });
    assert_eq!(
        route
            .pointers
            .iter()
            .map(|p| p.path.as_str())
            .collect::<Vec<_>>(),
        ["query/decisions/edge-throttling.md"],
        "{}",
        route.render(PLAIN)
    );
    assert_eq!(route.withheld, 1, "{}", route.render(PLAIN));
    let route = surface.route(task, Budget { pointers: 0 });
    assert_eq!(
        named_by_the_task(&route),
        ["query/decisions/edge-throttling.md"],
        "{}",
        route.render(PLAIN)
    );
}

/// A document that the task both anchors and names is offered once, with the
/// anchored evidence.
#[test]
fn a_document_both_anchored_and_named_is_offered_once_as_anchored() {
    let built = fixture_tree();
    let surface = built.surface();
    let route = surface.route(
        "Throttling at the edge in src/ingest/rate_limit.rs",
        Budget::default(),
    );
    let at: Vec<usize> = route
        .pointers
        .iter()
        .enumerate()
        .filter(|(_, p)| p.path == "query/decisions/edge-throttling.md")
        .map(|(at, _)| at)
        .collect();
    assert_eq!(at.len(), 1, "{}", route.render(PLAIN));
    assert!(
        matches!(
            route.evidence[at[0]],
            headwater_query::Evidence::Named { .. }
        ),
        "{}",
        route.render(PLAIN)
    );
    assert!(
        named_by_the_task(&route).is_empty(),
        "{}",
        route.render(PLAIN)
    );
}

/// A corpus that declares no purpose still answers a task that repeats a name,
/// as it answers a task that names a path.
#[test]
fn a_corpus_with_no_purposes_still_offers_a_named_document() {
    let source = std::fs::read_to_string(fixtures_dir().join("query.taxonomy.yml"))
        .expect("the fixture taxonomy");
    let mut stripped = String::new();
    let mut in_purposes = false;
    for line in source.lines() {
        if line == "purposes:" {
            in_purposes = true;
            continue;
        }
        if in_purposes && !line.is_empty() && !line.starts_with(' ') {
            in_purposes = false;
        }
        if in_purposes || line.trim_start().starts_with("purpose:") {
            continue;
        }
        stripped.push_str(line);
        stripped.push('\n');
    }
    let root = headwater_yaml::load(&stripped)
        .expect("the stripped taxonomy loads")
        .value
        .as_map()
        .expect("a mapping")
        .clone();
    let corpus = Corpus::new(fixtures_dir(), "query");
    let built = Built::over(&corpus, &root);
    let surface = built.surface();
    let route = surface.route("Throttling at the edge", Budget::default());
    assert_eq!(
        route.silence,
        None,
        "the route is silent: {}",
        route.render(PLAIN)
    );
    assert_eq!(
        named_by_the_task(&route),
        ["query/decisions/edge-throttling.md"],
        "{}",
        route.render(PLAIN)
    );

    // The anchored pointer comes first and the named one after it, on the
    // branch that ranks nothing. `NoPurposeMatched` returns the same list,
    // built once, so this holds both branches.
    let route = surface.route(ANCHORED_AND_NAMED, Budget::default());
    assert_eq!(
        paths_of(&route),
        [
            "query/specs/ingest.md",
            "query/decisions/edge-throttling.md"
        ],
        "{}",
        route.render(PLAIN)
    );
}

/// A task that names a path only `query/specs/ingest.md` governs, and repeats
/// the name of a different document, `query/decisions/edge-throttling.md`.
const ANCHORED_AND_NAMED: &str = "Throttling at the edge, and src/ingest";

fn paths_of(route: &headwater_query::Route) -> Vec<&str> {
    route.pointers.iter().map(|p| p.path.as_str()).collect()
}

/// Anchored pointers come first, then named ones, then ranked ones, on the
/// branch that ranks.
#[test]
fn an_anchored_pointer_comes_before_a_named_one_and_both_before_the_ranked() {
    let built = fixture_tree();
    let surface = built.surface();
    let route = surface.route(ANCHORED_AND_NAMED, Budget::default());
    assert!(!route.matched.is_empty(), "{}", route.render(PLAIN));
    let paths = paths_of(&route);
    assert_eq!(
        paths[..2],
        [
            "query/specs/ingest.md",
            "query/decisions/edge-throttling.md"
        ],
        "{}",
        route.render(PLAIN)
    );
    assert!(
        matches!(route.evidence[0], headwater_query::Evidence::Named { .. }),
        "{}",
        route.render(PLAIN)
    );
    assert!(
        route.evidence[2..]
            .iter()
            .all(|e| matches!(e, headwater_query::Evidence::Ranked { .. })),
        "{}",
        route.render(PLAIN)
    );
}

/// Two named pointers are in derived reading precedence, as every other block
/// of a route is, and not in census order. HW-DR-0109 constrains HW-DR-0070,
/// so a task that repeats both titles is offered HW-DR-0109 first, although
/// its path sorts after HW-DR-0070's.
#[test]
fn named_pointers_are_offered_in_reading_precedence() {
    let built = this_repository();
    let surface = built.surface();
    let route = surface.route(
        "The matched purposes take turns at a route budget, and each pointer states what reached it. \
         A task that repeats the declared name of a document names that document.",
        Budget::default(),
    );
    let named = named_by_the_task(&route);
    assert_eq!(named.len(), 2, "{}", route.render(PLAIN));
    assert!(
        named[0].starts_with("docs/decisions/0109-")
            && named[1].starts_with("docs/decisions/0070-"),
        "{}",
        route.render(PLAIN)
    );
}

/// The three sentence queries of #1470, and why the name step does not reach
/// the documents that each one still misses.
///
/// Each miss has one cause, and it is the purpose gate: the verbs of the
/// sentence ("add", "starts", "uses") match `evidence` and `procedure` above
/// the purpose the target serves, so the target competes late in the turns or
/// not at all. No sentence repeats the name of the document it misses, so the
/// name step has nothing to offer for it. A change to purpose scoring would be
/// the remedy, and #1470 records the cases rather than that change.
///
/// Measured on this branch, budget 100,000: HW-DR-0102 was not offered,
/// because `rationale` did not match; spec 6 stood 51st, rank 178 of 291; and
/// `headwater-generate.md` stood 9th, rank 174 of 373. The first sentence's
/// target, `taxonomy-resolution.md`, and the third's `headwater-export.md` are
/// now offered first by name.
#[test]
fn three_sentence_queries_miss_by_the_purpose_gate_and_not_by_the_name_step() {
    let built = this_repository();
    let surface = built.surface();
    let misses: [(&str, &str); 3] = [
        (NAMED[1].0, "docs/decisions/0102-"),
        (NAMED[1].0, "docs/spec/06-"),
        (NAMED[2].0, "docs/interfaces/headwater-generate.md"),
    ];
    for (task, target) in misses {
        let route = surface.route(task, Budget::default());
        assert!(
            !route.pointers.iter().any(|p| p.path.starts_with(target)),
            "{target} is now in the default five, so the case is stale:\n{}",
            route.render(PLAIN)
        );
        let target_document = surface
            .documents()
            .into_iter()
            .find(|document| document.path.starts_with(target))
            .unwrap_or_else(|| panic!("{target} is on the tree"));
        let purpose = surface
            .pointer(&target_document)
            .purpose
            .unwrap_or_else(|| panic!("{target} serves a purpose"));
        let top = route.matched.first().expect("a purpose matched");
        let score = route
            .matched
            .iter()
            .find(|matched| matched.purpose == purpose)
            .map_or(0, |matched| matched.score);
        assert!(
            score < top.score,
            "{target} serves `{purpose}`, which scored {score} against {} for `{}`, \
             so the purpose gate is not the cause",
            top.score,
            top.purpose
        );
        let words = surface
            .name(&target_document)
            .map(|name| name.to_lowercase())
            .unwrap_or_default();
        assert!(
            !task.to_lowercase().contains(&words) || words.is_empty(),
            "the task repeats the name of {target}"
        );
    }
}

/// The decision records whose title, less its number, is one word. HW-DR-0109
/// rules that a one-word name names nothing: one word is a term, and the
/// purpose gate decides a term. `naming` is not a word of any `rationale`
/// question, so the title `10 — Naming` matches no purpose that a decision
/// serves, and the route offers no decision for it.
const ONE_WORD_TITLES: [&str; 1] = ["docs/decisions/0010-naming.md"];

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
        .unwrap_or_else(|| panic!("{}", route.render(PLAIN)));
    let satellite = offered
        .iter()
        .position(|path| *path == "query/specs/api-errors.md")
        .unwrap_or_else(|| panic!("{}", route.render(PLAIN)));
    assert!(nucleus < satellite, "{}", route.render(PLAIN));

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
        route.render(PLAIN)
    );
}

/// A relation written under its inverse name reads as written, from both ends
/// (#1901).
///
/// The fixture writes one supersession twice: `per-service-limits.md`
/// (DR-FIX-0007) declares `superseded_by: DR-FIX-0031`, and
/// `edge-throttling.md` (DR-FIX-0031) declares `supersedes: DR-FIX-0007`.
/// Every entry of `related` reads "this document <relation> the far end",
/// whichever key either side wrote. Before #1901 the relation was named from
/// the canonical relation alone, so the superseded document printed its own
/// `superseded_by` as an outbound `supersedes`, and the successor printed that
/// edge as an inbound `superseded_by`.
#[test]
fn a_relation_declared_under_its_inverse_name_reads_as_written_from_both_ends() {
    let built = fixture_tree();
    let surface = built.surface();
    let triples = |path: &str| -> Vec<(String, bool, String)> {
        let document = surface.find(path).expect("the fixture");
        let mut triples: Vec<(String, bool, String)> = surface
            .related(&document)
            .into_iter()
            .filter(|neighbour| neighbour.relation.starts_with("supersede"))
            .map(|neighbour| (neighbour.relation, neighbour.inbound, neighbour.target))
            .collect();
        triples.sort();
        triples
    };
    let entry = |relation: &str, inbound: bool, target: &str| {
        (relation.to_owned(), inbound, target.to_owned())
    };
    assert_eq!(
        triples("query/decisions/per-service-limits.md"),
        vec![
            entry("superseded_by", false, "DR-FIX-0031"),
            entry("superseded_by", true, "query/decisions/edge-throttling.md"),
        ]
    );
    assert_eq!(
        triples("query/decisions/edge-throttling.md"),
        vec![
            entry("supersedes", false, "DR-FIX-0007"),
            entry("supersedes", true, "query/decisions/per-service-limits.md"),
        ]
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
        route.render(PLAIN)
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
            route.render(PLAIN)
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
    assert_eq!(route.anchors.len(), 2, "{}", route.render(PLAIN));
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
        route.render(PLAIN)
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
    assert!(whole.pointers.len() > 3, "{}", whole.render(PLAIN));
    assert_eq!(whole.withheld, 0, "{}", whole.render(PLAIN));

    let cut = surface.route(task, Budget { pointers: 3 });
    assert_eq!(
        cut.pointers.len() + cut.withheld,
        whole.pointers.len(),
        "the withheld count does not account for what a wider budget offers\n{}",
        cut.render(PLAIN)
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
    assert!(route.withheld > 0, "{}", route.render(PLAIN));
    let line = route
        .render(PLAIN)
        .lines()
        .find(|line| line.contains("the budget withheld"))
        .map(str::to_string)
        .unwrap_or_else(|| panic!("no withheld line\n{}", route.render(PLAIN)));
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
    // The line a cut writes opens `the budget withheld`. A bare `withheld`
    // also matches the title of a document this route may point at, such as
    // HW-DR-0100, which is a pointer and not a cut.
    assert!(
        !route.render(PLAIN).contains("the budget withheld"),
        "{}",
        route.render(PLAIN)
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

/// A pointer to a document whose warrant is outside the closed set reads no
/// acceptance from it, and says what the warrant is rather than calling it
/// `asserted` (#1438).
///
/// `acepted` is one letter from `accepted`, and a test for `asserted` alone
/// served it as vouched.
#[test]
fn a_pointer_to_a_warrant_outside_the_closed_set_reads_no_acceptance() {
    let built = fixture_tree();
    let surface = built.surface();
    let document = surface
        .find("query/specs/misspelled-notes.md")
        .expect("the fixture");
    let pointer = surface.pointer(&document);
    assert!(pointer.unwarranted, "{pointer:?}");
    assert_eq!(pointer.outside.as_deref(), Some("acepted"));
    let rendered = pointer.render();
    assert!(rendered.contains("`acepted`"), "{rendered}");
    assert!(rendered.contains("no acceptance is read"), "{rendered}");
    assert!(!rendered.contains("asserted"), "{rendered}");
}

/// How the anchor resolver and the path tools read one spelling of a path.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Reading {
    /// Both refuse it, or both accept it as the same path.
    Agree,
    /// The two readings differ on purpose, and HW-OBL-0061's `## Discharge`
    /// names the difference in a bullet that opens with this bold lead.
    Stated(&'static str),
}

/// [#1314](https://github.com/headwater-ai/headwater/issues/1314): the rule
/// HW-OBL-0061 states between the two normalizers of a path. An anchor that a
/// document writes is read by `headwater_graph::anchors::normalize`, and a
/// path that a reader types to `explain`, `related` or
/// `governing_docs_for_path` is read by `headwater_census::walk::within`. On a
/// spelling that both accept, they return the same path, and every spelling
/// where they differ is a divergence that the Discharge names. A row whose two
/// readings differ and that the table marks `Agree` fails, and so does a
/// `Stated` row whose phrase the Discharge does not carry.
#[test]
fn the_anchor_resolver_and_the_path_tools_agree_on_every_spelling_both_accept() {
    let scratch = std::env::temp_dir().join(format!(
        "headwater-reads-spellings-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("after the epoch")
            .as_nanos()
    ));
    let root = scratch.join("repo");
    let away = scratch.join("away");
    std::fs::create_dir_all(root.join("a")).expect("the repository");
    std::fs::create_dir_all(&away).expect("a directory outside it");
    std::os::unix::fs::symlink(&away, root.join("link")).expect("a link out of the root");
    let root = root.canonicalize().expect("a canonical root");
    let absolute = root.join("a/b").display().to_string();

    let rows: Vec<(&str, Option<&str>, Option<&str>, Reading)> = vec![
        ("a/b", Some("a/b"), Some("a/b"), Reading::Agree),
        ("./a/b", Some("a/b"), Some("a/b"), Reading::Agree),
        ("a/./b", Some("a/b"), Some("a/b"), Reading::Agree),
        ("a//b", Some("a/b"), Some("a/b"), Reading::Agree),
        ("a/b/", Some("a/b"), Some("a/b"), Reading::Agree),
        ("x/../a/b", Some("a/b"), Some("a/b"), Reading::Agree),
        ("../x", None, None, Reading::Agree),
        ("a/../../x", None, None, Reading::Agree),
        ("/etc/passwd", None, None, Reading::Agree),
        (
            absolute.as_str(),
            None,
            Some("a/b"),
            Reading::Stated("**An absolute path under the repository root.**"),
        ),
        (
            "link/x",
            Some("link/x"),
            None,
            Reading::Stated("**A path through a symlink that leads out of the repository.**"),
        ),
        (
            "a\\b",
            Some("a/b"),
            Some("a\\b"),
            Reading::Stated("**A backslash.**"),
        ),
        (
            " a/b ",
            Some("a/b"),
            Some(" a/b "),
            Reading::Stated("**Surrounding whitespace.**"),
        ),
        (
            "",
            None,
            Some(""),
            Reading::Stated("**The empty path, and `.`.**"),
        ),
        (
            ".",
            None,
            Some(""),
            Reading::Stated("**The empty path, and `.`.**"),
        ),
    ];

    let obligation = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../../docs/obligations/0061-an-anchor-resolver-normalizes-and-nothing-states-how.md",
    );
    let text = std::fs::read_to_string(&obligation)
        .unwrap_or_else(|e| panic!("{}: {e}", obligation.display()));
    let discharge = text
        .split_once("## Discharge")
        .map(|(_, rest)| rest)
        .expect("HW-OBL-0061 has a Discharge");

    for (spelling, anchor, tool, reading) in rows {
        let anchored = headwater_graph::anchors::normalize(spelling).ok();
        let typed = headwater_census::walk::within(&root, "docs", spelling);
        assert_eq!(
            anchored.as_deref(),
            anchor,
            "the anchor resolver reads {spelling:?}"
        );
        assert_eq!(typed.as_deref(), tool, "the path tools read {spelling:?}");
        assert_eq!(
            anchored == typed,
            reading == Reading::Agree,
            "{spelling:?}: the anchor side reads {anchored:?} and the tool side {typed:?}, \
             and the table says {reading:?}"
        );
        if let Reading::Stated(phrase) = reading {
            assert!(
                discharge.contains(phrase),
                "{spelling:?} is read two ways, and HW-OBL-0061's Discharge does not name \
                 the {phrase:?} divergence"
            );
        }
    }
    std::fs::remove_dir_all(&scratch).expect("the scratch tree goes");
}
