// SPDX-License-Identifier: Apache-2.0
//! A document that reaches another document more than once is one neighbour
//! per identity and one read per path.
//!
//! `NeighbourhoodView::neighbours` states "every document one relation away,
//! without repeats", and `NeighbourhoodView::reads` states "the centre and
//! every neighbour". `Instance::reads` states "without repeats". Two sets in
//! `scope.rs` keep those statements true in constant time for each edge
//! (#1968): `Neighbours::seen` in `Adjacency::push`, and `read_paths` in
//! `over_neighbourhoods`. Before this file, no test read either one. The one
//! neighbourhood rule, `participation`, reads its neighbours through `any()`,
//! so a doubled neighbour or a doubled read changed no verdict, and a mutant
//! that dropped either set survived the crate's suite.
//!
//! A doubled read is not harmless. The read set goes into the cache key and
//! into the published gate input
//! ([spec 12](../../../../docs/spec/12-check-layer.md), *Determinism,
//! concretely*), so a repeat there changes bytes that a reader compares.
//!
//! The tree under `fixtures/neighbourhood-dedup/` holds the two shapes, one
//! for each set, and each shape has its own test, so that one set cannot hide
//! behind the other:
//!
//! - `a.md` writes `depends_on` at `b.md`, and `b.md` writes the inverse
//!   `required_by` back. That is two edges with one identity: the relation,
//!   the end and the far document. Each end has exactly one neighbour.
//! - `c.md` writes `depends_on` and `constrains` at `d.md`. That is two
//!   neighbours, which is correct, with one path. `c.md` reads `d.md` once.

use headwater_census::census::{self, Census};
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::scope::{
    over_neighbourhoods, Digests, End, NeighbourhoodCheck, NeighbourhoodView,
};
use headwater_check::{Cache, Context, Date, Instance, Outcome};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::edges::Target;
use headwater_graph::{Config, Graph};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const A: &str = "neighbourhood-dedup/notes/a.md";
const B: &str = "neighbourhood-dedup/notes/b.md";
const C: &str = "neighbourhood-dedup/notes/c.md";
const D: &str = "neighbourhood-dedup/notes/d.md";

/// What one centre's view held, as owned values.
#[derive(Debug, Default)]
struct Seen {
    neighbours: Vec<(String, End, String)>,
    reads: Vec<String>,
}

/// A neighbourhood check that decides nothing and records what it was handed.
#[derive(Default)]
struct Recorder {
    by_centre: RefCell<BTreeMap<String, Seen>>,
}

impl NeighbourhoodCheck for Recorder {
    const RULE: &'static str = "test.neighbourhood.recorder";
    const VERSION: u32 = 1;

    fn evaluate(&self, view: &NeighbourhoodView<'_>) -> Outcome {
        let seen = Seen {
            neighbours: view
                .neighbours()
                .iter()
                .map(|n| (n.relation.to_string(), n.end, n.path.to_string()))
                .collect(),
            reads: view.reads().iter().map(|r| r.path.clone()).collect(),
        };
        self.by_centre
            .borrow_mut()
            .insert(view.path().to_string(), seen);
        Outcome::Passed
    }
}

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

struct Run {
    graph: Graph,
    instances: Vec<Instance>,
    recorder: Recorder,
}

fn run() -> Run {
    let corpus = Corpus::new(fixtures_dir(), "neighbourhood-dedup");
    let source = std::fs::read_to_string(fixtures_dir().join("neighbourhood-dedup.taxonomy.yml"))
        .expect("the fixture taxonomy");
    let root = headwater_yaml::load(&source)
        .expect("the fixture taxonomy loads")
        .value
        .as_map()
        .expect("a mapping")
        .clone();
    let taxonomy = Taxonomy::read(&root).expect("the taxonomy reads");
    let declarations = Declarations::read(&root).expect("the declarations read");
    let taken: Census = census::take(&corpus, &taxonomy);
    let graph = Graph::build(
        &taken,
        &declarations,
        &Resolvers::over(&corpus),
        &corpus,
        &Config::default(),
    );
    let recorder = Recorder::default();
    let instances = over_neighbourhoods(
        &recorder,
        &taken,
        &graph,
        &Digests::of(&taken),
        &Context::at(Date::parse("2026-08-12").expect("a date")),
        &mut Cache::disabled(),
    );
    Run {
        graph,
        instances,
        recorder,
    }
}

/// The edges whose source is `from` and whose target document is `to`.
fn edges_between(graph: &Graph, from: &str, to: &str) -> usize {
    graph
        .edges
        .iter()
        .filter(|edge| {
            edge.source.path == from
                && matches!(&edge.target, Target::Document { path, .. } if path == to)
        })
        .count()
}

fn reads_of<'r>(instances: &'r [Instance], centre: &str) -> Vec<&'r str> {
    let instance = instances
        .iter()
        .find(|instance| instance.reads.first().is_some_and(|r| r.path == centre))
        .unwrap_or_else(|| panic!("an instance centred on {centre}"));
    instance.reads.iter().map(|r| r.path.as_str()).collect()
}

/// Both halves of one pair are two edges and one neighbour at each end. This
/// holds the dedup on identity in `Adjacency::push`.
#[test]
fn a_pair_that_both_ends_declare_is_one_neighbour_at_each_end() {
    let run = run();

    // The guard against a vacuous pass: the graph keeps both halves.
    assert_eq!(
        (edges_between(&run.graph, A, B), edges_between(&run.graph, B, A)),
        (1, 1),
        "a.md writes depends_on and b.md writes required_by, so the graph holds two edges"
    );

    let seen = run.recorder.by_centre.borrow();
    assert_eq!(
        seen[A].neighbours,
        vec![("depends_on".to_string(), End::Source, B.to_string())],
        "a.md reaches b.md by one relation from one end, so it has one neighbour"
    );
    assert_eq!(
        seen[B].neighbours,
        vec![("depends_on".to_string(), End::Target, A.to_string())],
        "b.md reaches a.md by one relation from one end, so it has one neighbour"
    );
}

/// One far document reached by two relations is two neighbours and one read.
/// This holds the dedup on path in `over_neighbourhoods`.
#[test]
fn a_document_reached_by_two_relations_is_read_once() {
    let run = run();

    let seen = run.recorder.by_centre.borrow();
    assert_eq!(
        seen[C].neighbours,
        vec![
            ("depends_on".to_string(), End::Source, D.to_string()),
            ("constrains".to_string(), End::Source, D.to_string()),
        ],
        "c.md reaches d.md by two relations, so it has two neighbours"
    );
    assert_eq!(
        seen[C].reads,
        vec![C.to_string(), D.to_string()],
        "the view reads c.md and then d.md, each once"
    );
    assert_eq!(
        reads_of(&run.instances, C),
        vec![C, D],
        "the instance records the read set the view was handed"
    );
}
