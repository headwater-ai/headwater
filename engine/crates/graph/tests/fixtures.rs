// SPDX-License-Identifier: Apache-2.0
//! The graph fixture tree, and the graph of this repository.
//!
//! The tree under `fixtures/graph/` holds one document per resolution outcome,
//! and `graph.report` records what the build makes of every one of them. The
//! recorded file is the assertion, for the same reason the census records one:
//! the thing under test is what the build *reports*, and a hand-written
//! assertion per case lets a new case arrive with no report at all.
//!
//!     HEADWATER_BLESS=1 cargo test -p headwater-graph --test fixtures
//!
//! Read the diff before committing it. A blessed fixture is the change.

use headwater_census::census;
use headwater_census::shelves::Taxonomy;
use headwater_census::standin;
use headwater_census::walk::{Corpus, Exclusion};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Detail, Direction, Graph, Target, Unbound};
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
        "\nthe graph no longer matches {}",
        recorded.display()
    );
}

/// The fixture tree, in the shape the build reads it.
fn fixture_graph() -> Graph {
    let corpus = Corpus::new(fixtures_dir(), "graph").excluding(vec![Exclusion::new(
        "graph/excluded/**",
        "a declared exclusion, so that an anchor can resolve into one",
    )]);
    let root = standin::load_map(&fixtures_dir().join("graph.taxonomy.yml"));
    let taxonomy = Taxonomy::read(&root).expect("the fixture taxonomy reads");
    let declarations = Declarations::read(&root).expect("the fixture declarations read");

    let taken = census::take(&corpus, &taxonomy);
    Graph::build(
        &taken,
        &declarations,
        &Resolvers::over(&corpus),
        &corpus,
        &Config::default(),
    )
}

/// This repository, wired by the taxonomy that types it.
fn corpus_graph() -> Graph {
    let root = repository_root();
    let corpus = standin::corpus(&root);
    let resolved = standin::resolved(&root);
    let taxonomy = Taxonomy::read(&resolved).expect("the resolved taxonomy reads");
    let declarations = Declarations::read(&resolved).expect("the resolved declarations read");

    let taken = census::take(&corpus, &taxonomy);
    Graph::build(
        &taken,
        &declarations,
        &Resolvers::over(&corpus),
        &corpus,
        &Config::default(),
    )
}

#[test]
fn the_fixture_tree_builds_to_the_recorded_graph() {
    compare(
        &fixtures_dir().join("graph.report"),
        &fixture_graph().render(Detail::EveryRow),
    );
}

#[test]
fn this_repository_builds_to_the_recorded_graph() {
    let graph = corpus_graph();
    assert!(
        graph.edges.len() > 100,
        "only {} edge halves, which is fewer than this repository declares",
        graph.edges.len()
    );
    compare(
        &fixtures_dir().join("corpus.graph"),
        &graph.render(Detail::Exceptions),
    );
}

/// Every declared edge half falls in exactly one class.
///
/// The same property the census asserts about its rows, and it fails the same
/// way: a half that falls outside the closed set is a half that no count holds,
/// and coverage is computed against these counts.
#[test]
fn every_edge_half_falls_in_exactly_one_class() {
    for graph in [fixture_graph(), corpus_graph()] {
        let counted: usize = graph.counts().iter().map(|(_, n)| n).sum();
        assert_eq!(counted, graph.edges.len());
        let links: usize = graph.link_counts().iter().map(|(_, n)| n).sum();
        assert_eq!(links, graph.links.len());
    }
}

/// The two halves of one reciprocal pair are one edge.
///
/// `00-first.md` writes `cites_evidence: EVAL-FIX-alpha` and `alpha.md` writes
/// `cited_by: SPEC-FIX-first`. Both are authored, both are real, and a check
/// that looks for the half nobody wrote has to see them as one triple however
/// the two authors reached for a name.
#[test]
fn both_halves_of_a_reciprocal_pair_normalize_to_one_triple() {
    let graph = fixture_graph();
    let paired: Vec<&headwater_graph::Edge> = graph
        .edges
        .iter()
        .filter(|edge| {
            edge.declared_triple()
                == Some((
                    "SPEC-FIX-first".to_string(),
                    "cites_evidence".to_string(),
                    "EVAL-FIX-alpha".to_string(),
                ))
        })
        .collect();
    assert_eq!(paired.len(), 2, "{paired:#?}");
    let mut written: Vec<(&str, Direction)> = paired
        .iter()
        .map(|edge| (edge.name.as_str(), edge.direction))
        .collect();
    written.sort();
    assert_eq!(
        written,
        [
            ("cited_by", Direction::Inverse),
            ("cites_evidence", Direction::AsDeclared)
        ]
    );
}

/// Two spellings of one anchor are one node, and the second is a repeated
/// triple rather than a second edge.
///
/// This is the correctness root spec 12 names, asserted the only way it can be:
/// a resolver that mis-normalizes reports two edges here and fails nothing.
#[test]
fn two_spellings_of_one_anchor_are_one_node() {
    let graph = fixture_graph();
    let anchors = graph.anchor_nodes();
    let second = anchors
        .iter()
        .find(|node| node.normalized == "graph/spec/01-second.md")
        .expect("the anchor");
    assert_eq!(second.edges, 1, "{anchors:#?}");
    assert!(
        graph.problems.iter().any(|problem| matches!(
            &problem.problem,
            headwater_graph::edges::Problem::RepeatedTriple { target, .. }
                if target == "graph/spec/01-second.md"
        )),
        "the second spelling declared a second edge instead of repeating one"
    );
}

/// An anchor inside a declared exclusion resolves, and says which rule claims
/// it.
#[test]
fn an_anchor_inside_an_exclusion_resolves_and_names_the_rule() {
    let graph = fixture_graph();
    let node = graph
        .anchor_nodes()
        .into_iter()
        .find(|node| node.normalized == "graph/excluded/note.md")
        .expect("the anchor");
    assert_eq!(node.excluded_by.as_deref(), Some("graph/excluded/**"));
}

/// A target that names an untyped document is a different report from a target
/// that names nothing.
///
/// One says fix that document and the other says fix this link. Reported as one
/// class, the first sends its author to repair a link that is already right.
#[test]
fn an_untyped_target_and_a_target_that_is_nothing_are_two_reports() {
    let graph = fixture_graph();
    let unbound: Vec<(&str, &Unbound)> = graph
        .edges
        .iter()
        .filter_map(|edge| match &edge.target {
            Target::Unbound(unbound) => Some((edge.raw_target.as_str(), unbound)),
            _ => None,
        })
        .collect();

    let untyped = unbound
        .iter()
        .find(|(raw, _)| *raw == "SPEC-FIX-unshelved")
        .expect("the untyped target");
    assert!(
        matches!(untyped.1, Unbound::NotTyped { path, .. } if path == "graph/notes/loose.md"),
        "{:?}",
        untyped.1
    );

    let nothing = unbound
        .iter()
        .find(|(raw, _)| *raw == "EVAL-FIX-nowhere")
        .expect("the target that is nothing");
    assert!(
        matches!(nothing.1, Unbound::NoSuchTarget { .. }),
        "{:?}",
        nothing.1
    );

    let no_resolver = unbound
        .iter()
        .find(|(raw, _)| *raw == "12345")
        .expect("the anchor kind with no resolver");
    assert!(
        matches!(no_resolver.1, Unbound::NoResolver { resolver, .. } if resolver == "ado-snapshot"),
        "{:?}",
        no_resolver.1
    );
}

/// A quoted link belongs to another author, and never to this document.
#[test]
fn a_quoted_link_is_counted_and_never_bound() {
    let graph = fixture_graph();
    assert_eq!(graph.skipped.quoted, 1);
    // `00-first.md` writes the same destination twice: once in its own prose,
    // and once inside a block quote. One of the two is this document's.
    let bound = graph
        .links
        .iter()
        .filter(|link| {
            link.source_path == "graph/spec/00-first.md" && link.destination == "../no-such-file.md"
        })
        .count();
    assert_eq!(bound, 1, "the quoted link reached the binding");
}

/// The graph and the census are two accounts of one corpus, and every node of
/// the first is a typed row of the second.
#[test]
fn every_node_of_the_graph_is_a_typed_row_of_the_census() {
    let root = repository_root();
    let corpus = standin::corpus(&root);
    let taxonomy = Taxonomy::read(&standin::resolved(&root)).expect("reads");
    let taken = census::take(&corpus, &taxonomy);
    let graph = corpus_graph();

    for node in &graph.index.typed {
        let row = taken
            .rows
            .iter()
            .find(|row| row.path == node.path)
            .unwrap_or_else(|| panic!("{} is a node and absent from the census", node.path));
        assert!(
            matches!(row.outcome, census::Outcome::Typed { .. }),
            "{} is a node and not a typed row",
            node.path
        );
    }
    for edge in &graph.edges {
        assert!(
            graph.index.node(&edge.source.id).is_some(),
            "{} declares an edge and is not a node",
            edge.source.path
        );
    }
}
