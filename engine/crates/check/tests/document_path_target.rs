// SPDX-License-Identifier: Apache-2.0
//! A relation entry that names a governed document by its path (#1410).
//!
//! [Q4](../../../../docs/decisions/0004-relation-storage.md) rules that a
//! target is an identifier. Where a relation admits a document and a
//! `code_path` anchor, the index is asked first, and it matches an identifier
//! only. A path fell through to the `source-tree` resolver, which binds any
//! file that exists, so the entry became an anchor onto a Markdown file. It
//! passed `check --strict`, and the document it meant never showed the edge.
//!
//! The tree under `fixtures/document-path-target/` holds the one case that is
//! now a finding and each case beside it that stays as it was:
//!
//! **The decisive case.** `notes/a.md` writes the path of `NOTE-FIX-b`.
//! `relation.target.unresolved` reports it once, names `NOTE-FIX-b`, and the
//! graph holds no anchor for it.
//!
//! **A rule that compared the spelling.** `notes/a2.md` writes the same path
//! with `./` and a `..` segment. The resolver normalizes it, and the
//! comparison reads the normalized path.
//!
//! **The identifier form.** `notes/c.md` writes `NOTE-FIX-b`, which binds to
//! the document, so `NOTE-FIX-b` has the reverse edge.
//!
//! **A rule that reported every path.** `notes/d.md` traces to a source file,
//! `notes/e.md` to a wildcard over the documents, and `notes/g.md` to a
//! Markdown file with no identifier. Each is an anchor and no finding.
//!
//! **A rule that ignored the declaration.** `notes/f.md` writes the path of
//! `NOTE-FIX-b` under `governs`, which admits only a `code_path` anchor. It is
//! an anchor and no finding.

use headwater_census::census;
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::{Cache, Context, Date, Declared, Register, Run, Shape};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::edges::{Target, Unbound};
use headwater_graph::{Config, Graph};
use std::path::{Path, PathBuf};

/// As every other recorded run: a verdict is a function of the injected clock.
const PINNED: &str = "2026-08-12";

const RULE: &str = "relation.target.unresolved";

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn build() -> (Graph, Run) {
    let corpus = Corpus::new(fixtures_dir(), "document-path-target");
    let source = std::fs::read_to_string(fixtures_dir().join("document-path-target.taxonomy.yml"))
        .expect("the fixture taxonomy");
    let root = headwater_yaml::load(&source)
        .expect("the fixture taxonomy loads")
        .value
        .as_map()
        .expect("a mapping")
        .clone();

    let taxonomy = Taxonomy::read(&root).expect("the taxonomy reads");
    let declarations = Declarations::read(&root).expect("the declarations read");
    let register = Register::read(&root).expect("the register reads");
    let shape = Shape::read(&root).expect("the shape reads");
    let taken = census::take(&corpus, &taxonomy);
    let config = Config::default();
    let graph = Graph::build(
        &taken,
        &declarations,
        &Resolvers::over(&corpus),
        &corpus,
        &config,
    );
    let run = headwater_check::run(
        &taken,
        &graph,
        &Declared {
            lock: "sha256:document-path-target-fixture",
            taxonomy: &taxonomy,
            shape: &shape,
            relations: &declarations,
            config: &config,
            register: &register,
            observations: &headwater_check::Observations::empty(),
            pin: None,
            harvests: &[],
            adoption: None,
            source: "engine/crates/check/fixtures/document-path-target.taxonomy.yml",
        },
        &headwater_check::claim::Claims::empty(),
        &Context::at(Date::parse(PINNED).expect("the pinned date")),
        &mut Cache::disabled(),
    );
    (graph, run)
}

/// The messages of one rule at one file of the tree.
fn at<'a>(run: &'a Run, file: &str) -> Vec<&'a str> {
    let path = format!("document-path-target/{file}");
    run.findings
        .iter()
        .filter(|finding| finding.rule == RULE && finding.path == path)
        .map(|finding| finding.message.as_str())
        .collect()
}

/// The one target the document with this identifier declares.
fn target_of<'a>(graph: &'a Graph, id: &str) -> &'a Target {
    let edges: Vec<&Target> = graph
        .edges
        .iter()
        .filter(|edge| edge.source.id == id)
        .map(|edge| &edge.target)
        .collect();
    let [only] = edges.as_slice() else {
        panic!("{id} declares one edge, and the graph holds {edges:?}");
    };
    only
}

/// The decisive case: the path of a typed document with an identifier is a
/// finding that names the identifier, and it is no anchor.
#[test]
fn a_path_to_an_identified_document_is_reported_with_the_identifier_to_write() {
    let (graph, run) = build();

    let found = at(&run, "notes/a.md");
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].contains("NOTE-FIX-b"), "{found:?}");

    assert!(
        matches!(
            target_of(&graph, "NOTE-FIX-a"),
            Target::Unbound(Unbound::DocumentByPath { id, path })
                if id == "NOTE-FIX-b" && path == "document-path-target/notes/b.md"
        ),
        "{:?}",
        target_of(&graph, "NOTE-FIX-a")
    );
}

/// The same path, spelled with `./` and a `..` segment, is the same finding.
#[test]
fn a_path_that_is_not_in_canonical_form_is_compared_after_normalization() {
    let (graph, run) = build();

    let found = at(&run, "notes/a2.md");
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].contains("NOTE-FIX-b"), "{found:?}");
    assert!(matches!(
        target_of(&graph, "NOTE-FIX-a2"),
        Target::Unbound(Unbound::DocumentByPath { .. })
    ));
}

/// The identifier form binds to the document, and the document is the target
/// end of the edge, which is the reverse line `explain` prints.
#[test]
fn the_identifier_form_binds_to_the_document_and_makes_the_reverse_edge() {
    let (graph, run) = build();

    assert!(at(&run, "notes/c.md").is_empty());
    assert!(matches!(
        target_of(&graph, "NOTE-FIX-c"),
        Target::Document { id, .. } if id == "NOTE-FIX-b"
    ));
    let incoming: Vec<&str> = graph
        .edges
        .iter()
        .filter(|edge| matches!(&edge.target, Target::Document { id, .. } if id == "NOTE-FIX-b"))
        .map(|edge| edge.source.id.as_str())
        .collect();
    assert_eq!(incoming, ["NOTE-FIX-c"]);
}

/// A source file, a wildcard, a path under a relation that admits no
/// document, and a Markdown file with no identifier all stay anchors.
#[test]
fn every_other_path_stays_an_anchor_and_is_not_reported() {
    let (graph, run) = build();

    for (file, id) in [
        ("notes/d.md", "NOTE-FIX-d"),
        ("notes/e.md", "NOTE-FIX-e"),
        ("notes/f.md", "NOTE-FIX-f"),
        ("notes/g.md", "NOTE-FIX-g"),
    ] {
        assert!(at(&run, file).is_empty(), "{file}: {:?}", at(&run, file));
        assert!(
            matches!(
                target_of(&graph, id),
                Target::Anchor { anchor_kind, .. } if anchor_kind == "code_path"
            ),
            "{file}: {:?}",
            target_of(&graph, id)
        );
    }

    // The whole tree: two findings, both the decisive case.
    let all: Vec<&str> = run
        .findings
        .iter()
        .filter(|finding| finding.rule == RULE)
        .map(|finding| finding.path.as_str())
        .collect();
    assert_eq!(
        all,
        [
            "document-path-target/notes/a.md",
            "document-path-target/notes/a2.md"
        ]
    );
}
