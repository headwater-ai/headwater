// SPDX-License-Identifier: Apache-2.0
//! Whether a declared `filter` on a shelf projection reaches the file it writes.
//!
//! # The defect this file was written against
//!
//! `projections[].filter` is declared in the meta-schema for **every**
//! projection kind. Before this file existed, [`Filter::admits`] reached exactly
//! one caller in the engine, which is the graph export. `shelf_index` and
//! `shelf_sections` took every document on the shelf and never asked.
//!
//! So a declared filter on a shelf index validated, resolved, generated and was
//! discarded, at exit 0, with nothing anywhere reporting it. Measured against
//! the repository's own corpus before the fix: a `shelf_index` over the
//! obligations shelf with `filter: {include: {waiting_on: [ruling]}}` wrote
//! **129 rows for a filter that named 40**, and `headwater check --strict` was
//! green over the result.
//!
//! It is worse than inert. `facets_read` in `headwater_resolve::rules` counts a
//! facet named in *any* projection's filter as a reader for the facet relevance
//! canon. So declaring the filter is what makes the facet pass the guard against
//! a facet nothing reads, and the guard is satisfied by a reader that does
//! nothing.
//!
//! # The fixture
//!
//! `fixtures/filter.taxonomy.yml` is `generate.taxonomy.yml` with a different
//! `projections` block and nothing else changed, over the same `fixtures/generate`
//! tree. That tree's `decisions` shelf holds two documents, one `superseded` and
//! one `current`, so an `include` clause over `status` separates them by one.
//! An emitter that drops the filter writes two rows where the filter names one,
//! which is the smallest shape in which the defect is visible.

use headwater_census::census::{self, Census};
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::Shape;
use headwater_generate::{plan, Identity, Kind, Plan, Projections, Runs};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use headwater_query::Surface;
use headwater_yaml::Mapping;
use std::path::{Path, PathBuf};

/// The row of the `current` document, named by the link target rather than by
/// the label. Two things make that the right handle. The parentheses matter for
/// the reason the brackets used to: the summary of one document names the other
/// in prose, so a bare string is not a test of what the rows are. And the link
/// target is the one part of a row that no labeling decision moves.
///
/// It was `[DR-FIX-0002]` until #427, which is the identifier the row used to
/// carry, and that reading tied this file to a choice it does not test. A shelf
/// index labels a row with the facet in the `name` role from that change, so
/// the assertion failed on a fixture corpus whose decisions all declare a title
/// — and it failed in the two emitters unequally, because `shelf_sections`
/// writes the name as a heading and still writes the identifier as link text.
/// This file is about whether a declared filter reaches an emitter. It should
/// not fail again the next time somebody changes what a row is called.
const CURRENT: &str = "(0002-rebuild-the-graph.md)";

/// The `superseded` document, which the `live` filter withholds.
const SUPERSEDED: &str = "(0001-store-the-graph.md)";

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
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

fn identity() -> Identity {
    Identity {
        corpus_root: "generate".to_string(),
        exclusions: Vec::new(),
        package: "headwater/fixture".to_string(),
        version: "1.0.0".to_string(),
        lock: "sha256:0000000000000000000000000000000000000000000000000000000000000000".to_string(),
    }
}

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

/// The plan over the fixture tree, under the filtered taxonomy.
fn planned() -> Plan {
    let corpus = Corpus::new(fixtures_dir(), "generate");
    let source = load_map(&fixtures_dir().join("filter.taxonomy.yml"));
    let built = Built::over(&corpus, &source);
    let projections = Projections::read(&source).expect("the projections read");
    let surface = built.surface();
    plan(
        &surface,
        &built.census,
        &projections,
        &identity(),
        &Runs::default(),
        &[],
    )
}

fn written<'a>(plan: &'a Plan, path: &str) -> &'a str {
    &plan
        .outputs
        .iter()
        .find(|output| output.path == path)
        .unwrap_or_else(|| {
            panic!(
                "nothing written at {path}. Unwritten: {:?}",
                plan.unwritten
                    .iter()
                    .map(|entry| format!("{}: {}", entry.at, entry.reason))
                    .collect::<Vec<_>>()
            )
        })
        .bytes
}

fn unwritten_reason<'a>(plan: &'a Plan, path: &str) -> &'a str {
    &plan
        .unwritten
        .iter()
        .find(|entry| entry.at == path)
        .unwrap_or_else(|| {
            panic!(
                "{path} is not among the unwritten. Written: {:?}",
                plan.outputs
                    .iter()
                    .map(|output| &output.path)
                    .collect::<Vec<_>>()
            )
        })
        .reason
}

/// The control. With no filter, the index names every document on the shelf.
///
/// It fails for the same reason as the test below if the filter is applied to
/// the wrong entry, which is the arm a fix could plausibly break.
#[test]
fn an_unfiltered_shelf_index_names_every_document_on_the_shelf() {
    let plan = planned();
    let bytes = written(&plan, "generate/decisions/README.md");
    assert!(bytes.contains(CURRENT), "the current document is named");
    assert!(
        bytes.contains(SUPERSEDED),
        "the superseded document is named too, because nothing filters this entry:\n{bytes}"
    );
}

/// The test this file exists for, and it was watched to fail.
///
/// Before the fix it reported both documents, because `shelf_index` never asked
/// the filter. The failure is the whole instrument: a declared filter that the
/// emitter drops is invisible to `validate`, to `resolve`, to `generate` and to
/// `check --strict`.
#[test]
fn a_filtered_shelf_index_names_only_what_the_filter_admits() {
    let plan = planned();
    let bytes = written(&plan, "generate/decisions/LIVE.md");
    assert!(
        bytes.contains(CURRENT),
        "the filter admits the current document:\n{bytes}"
    );
    assert!(
        !bytes.contains(SUPERSEDED),
        "the filter withholds the superseded document, and the index must not name it:\n{bytes}"
    );
}

/// The sections emitter takes every document of the shelf by the same route, so
/// it drops a filter the same way.
#[test]
fn a_filtered_shelf_sections_writes_only_what_the_filter_admits() {
    let plan = planned();
    let bytes = written(&plan, "generate/decisions/LIVE-SECTIONS.md");
    assert!(
        bytes.contains(CURRENT),
        "the filter admits the current document:\n{bytes}"
    );
    assert!(
        !bytes.contains(SUPERSEDED),
        "the filter withholds the superseded document, and the file must not carry its \
         section:\n{bytes}"
    );
}

/// A filter that admits nothing is not an empty shelf, and the two reasons differ.
///
/// An index of an empty shelf asserts that a shelf is there, which is the
/// existing refusal. A filter that names a value no document states is a
/// declaration that produced nothing, and a reader acts on the difference.
#[test]
fn a_filter_that_admits_nothing_is_reported_apart_from_an_empty_shelf() {
    let plan = planned();
    let reason = unwritten_reason(&plan, "generate/decisions/NONE.md");
    assert!(
        reason.contains("filter"),
        "the reason names the filter rather than the shelf: {reason}"
    );
    assert!(
        !plan
            .outputs
            .iter()
            .any(|output| output.path == "generate/decisions/NONE.md"),
        "a filter that admits nothing writes no file"
    );
}

/// The kinds the plan carries, so that a fix which stopped emitting is caught.
#[test]
fn the_filtered_declarations_still_reach_their_emitters() {
    let plan = planned();
    let indexes = plan
        .outputs
        .iter()
        .filter(|output| output.kind == Kind::ShelfIndex)
        .count();
    assert_eq!(indexes, 2, "the control and the `live` index are written");
}
