// SPDX-License-Identifier: Apache-2.0
//! The whole chain: an import writes an edge, and the next check says it
//! resolves.
//!
//! [OBL-repo-0116](../../../../docs/obligations/0116-no-anchor-resolver-reads-a-committed-snapshot-so-every-imported-edge-lands-unresolved.md)
//! records the state this file exists to end, and it says why no case in
//! `fixtures.rs` could have found it: "No fixture of the importer can fail on
//! this, because the importer is right. The gap is one component further on, and
//! it appears only when a check runs over the result." So the measurement there
//! was a hand run of four verbs. This file is that hand run, in a suite.
//!
//! Every case builds the tree, plans the import, **writes it**, walks the result
//! and builds the graph with the resolver set a run of a verb builds. Nothing is
//! asserted against a graph assembled by hand: the defect was that the edge and
//! the resolver never met, so a fixture that put them together itself would
//! measure the fixture.
//!
//! # The two cases, and why the second one is the instrument
//!
//! The first is the defect: the edge binds. The second is the one the issue
//! calls the case that matters. A resolver that binds anything is worse than
//! none, because [`headwater_import::plan`] refuses a link into an item the
//! snapshot does not pin precisely on the belief that an anchor resolver would
//! otherwise take the string. If this resolver normalized rather than looked up,
//! that refusal would be all that stood between a typo and a bound edge, and a
//! check that reports nothing is the silent pass one level up.
//!
//! So the second case puts an item identity the snapshot does not hold into the
//! corpus by the route the importer does not police: a person types it. The
//! assertion is that it is reported.

use headwater_census::census;
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::{Cache, Context, Date, Declared, Register, Shape};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::index::Index;
use headwater_graph::{Config, Graph, Target, Unbound};
use headwater_import::{Declaration, Plan};
use std::path::{Path, PathBuf};

/// A tree that removes itself, named for the case that made it.
struct Scratch(PathBuf);

impl Scratch {
    fn new(case: &str) -> Self {
        let at = std::env::temp_dir().join(format!(
            "headwater-resolution-{}-{case}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).expect("the scratch tree is made");
        Scratch(at)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn write(&self, relative: &str, text: &str) {
        let path = self.0.join(relative);
        std::fs::create_dir_all(path.parent().expect("it has a parent"))
            .expect("the parent is made");
        std::fs::write(path, text).expect("the file is written");
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// The taxonomy of the fixture corpus. `audited_by` is the relation an importer
/// may write, and `ado_work_item` is the anchor kind at its far end. The
/// resolver name is what an import declaration has to supply for anything the
/// import writes to bind.
const TAXONOMY: &str = "\
kinds:
  governed_document: {abstract: true}
  design_spec: {is_a: governed_document}

shelves:
  spec_series:
    path: docs/spec/**
    homogeneous: true
    kind: design_spec

relations:
  audited_by:
    family: evidence
    from: [design_spec]
    to:   [ado_work_item]
    created_by: import
    attributes:
      verified_revision: {type: string, owner: edge}

anchors:
  ado_work_item: {resolver: ado-snapshot}
";

const PAYLOAD: &str = "\
snapshot:
  format: 1
  source: acme/work-items
  fetched: 2026-08-14
  items:
    - id: \"12345\"
      revision: \"7\"
      title: The service refuses an unauthenticated request
  links:
    - from: SPEC-FIX-one
      relation: audited_by
      to: \"12345\"
";

const RESOLVER: &str = "ado-snapshot";

/// The repository the chain runs over, and the digest of its snapshot.
fn tree(scratch: &Scratch) -> String {
    scratch.write(
        "docs/spec/00-first.md",
        "---\nid: SPEC-FIX-one\ndoc_type: design_spec\n---\n\n# The first part\n",
    );
    scratch.write("taxonomy.yml", TAXONOMY);
    scratch.write("imports/ado/snapshot.yml", PAYLOAD);

    let dir = scratch.path().join("imports/ado");
    let manifest = headwater_yaml::load("package: acme/work-items\nversion: \"2026-08-14\"\n")
        .expect("the manifest reads")
        .value
        .as_map()
        .expect("it is a mapping")
        .clone();
    let release = headwater_resolve::release::compute(&dir, &manifest).expect("it computes");
    std::fs::write(
        dir.join(headwater_resolve::release::RECORD),
        headwater_resolve::release::render(&release),
    )
    .expect("the record is written");
    release.digest
}

fn declaration(digest: &str) -> Declaration {
    Declaration {
        name: "ado".to_string(),
        at: "imports/ado".to_string(),
        digest: Some(digest.to_string()),
        channel: Some("the platform team's fetch job".to_string()),
        resolver: Some(RESOLVER.to_string()),
    }
}

/// Everything a graph build needs, out of one walk of the tree.
struct Read {
    corpus: Corpus,
    census: headwater_census::census::Census,
    relations: Declarations,
    shape: Shape,
    taxonomy: Taxonomy,
    config: Config,
}

fn read(scratch: &Scratch) -> Read {
    let source = std::fs::read_to_string(scratch.path().join("taxonomy.yml")).expect("it is there");
    let root = headwater_yaml::load(&source)
        .expect("the fixture taxonomy reads")
        .value
        .as_map()
        .expect("it is a mapping")
        .clone();
    let corpus = Corpus::new(scratch.path(), "docs");
    let taxonomy = Taxonomy::read(&root).expect("the shelves read");
    let census = census::take(&corpus, &taxonomy);
    Read {
        relations: Declarations::read(&root).expect("the relations read"),
        shape: Shape::read(&root).expect("the kinds read"),
        corpus,
        census,
        taxonomy,
        config: Config::default(),
    }
}

impl Read {
    /// The graph a verb builds: the corpus resolvers, plus whatever the declared
    /// imports supply. This is the one line of `load` under test.
    fn graph(&self, root: &Path, declarations: &[Declaration]) -> Graph {
        let mut resolvers = Resolvers::over(&self.corpus);
        for items in headwater_import::anchors::over(root, declarations) {
            resolvers = resolvers.with(Box::new(items)).expect("one of each name");
        }
        Graph::build(
            &self.census,
            &self.relations,
            &resolvers,
            &self.corpus,
            &self.config,
        )
    }

    fn index(&self) -> Index {
        Index::build(&self.census, &self.config)
    }
}

/// Plan the import and write it, which is `headwater import ado --write`.
fn import(scratch: &Scratch, declaration: &Declaration) -> Plan {
    let read = read(scratch);
    let index = read.index();
    let view = headwater_import::Corpus {
        index: &index,
        relations: &read.relations,
        shape: &read.shape,
    };
    let plan =
        headwater_import::plan(scratch.path(), declaration, None, &view).expect("the import plans");
    let pending = plan.to_write();
    let composed = headwater_import::write::compose(scratch.path(), &pending).expect("it composes");
    headwater_import::write::apply(scratch.path(), &composed).expect("it writes");
    plan
}

/// Every target of every edge in the graph, in path order.
fn targets(graph: &Graph) -> Vec<&Target> {
    graph.edges.iter().map(|edge| &edge.target).collect()
}

/// Every edge that bound to nothing, with the string its author wrote.
fn unresolved(graph: &Graph) -> Vec<(&str, &Unbound)> {
    graph
        .edges
        .iter()
        .filter_map(|edge| match &edge.target {
            Target::Unbound(unbound) => Some((edge.raw_target.as_str(), unbound)),
            _ => None,
        })
        .collect()
}

/// The defect OBL-repo-0116 recorded, and its end.
///
/// Before the resolver existed this ran the same four steps and the last one
/// reported `relation.target.unresolved` against the document the import had
/// just written.
#[test]
fn an_edge_an_import_wrote_resolves_to_the_item_the_snapshot_pinned() {
    let scratch = Scratch::new("resolves");
    let digest = tree(&scratch);
    let declaration = declaration(&digest);
    let plan = import(&scratch, &declaration);
    assert_eq!(plan.edges.len(), 1);

    let read = read(&scratch);
    let graph = read.graph(scratch.path(), std::slice::from_ref(&declaration));
    let bound = targets(&graph);
    assert_eq!(bound.len(), 1, "{bound:?}");
    let Target::Anchor {
        anchor_kind,
        resolver,
        normalized,
        ..
    } = bound[0]
    else {
        panic!("the imported edge did not bind: {:?}", bound[0]);
    };
    assert_eq!(anchor_kind, "ado_work_item");
    assert_eq!(resolver, RESOLVER);
    // The identity is the string the far end spells, and never a form this
    // engine invented for it.
    assert_eq!(normalized, "12345");
    assert!(unresolved(&graph).is_empty(), "{:?}", unresolved(&graph));
}

/// **The case that matters.** An item identity the snapshot does not hold is
/// reported rather than bound.
///
/// The string reaches the corpus by the one route the importer does not police:
/// a person writes it. A resolver that normalized instead of looking up would
/// bind it, no rule would report anything, and the graph would carry an edge
/// into an item that no pin holds.
#[test]
fn an_identity_the_snapshot_does_not_hold_is_reported_rather_than_bound() {
    let scratch = Scratch::new("typo");
    let digest = tree(&scratch);
    let declaration = declaration(&digest);
    import(&scratch, &declaration);

    // One digit different from the item the snapshot pins, written by hand into
    // the document the import already wrote to.
    scratch.write(
        "docs/spec/01-second.md",
        "---\nid: SPEC-FIX-two\ndoc_type: design_spec\nrelations:\n  audited_by:\n    - \"12346\"\n---\n\n# The second part\n",
    );

    let read = read(&scratch);
    let graph = read.graph(scratch.path(), std::slice::from_ref(&declaration));
    let unbound = unresolved(&graph);
    assert_eq!(unbound.len(), 1, "{:?}", targets(&graph));
    let Unbound::AnchorUnresolved { anchor_kind, why } = unbound[0].1 else {
        panic!(
            "the wrong identity was not reported as one: {:?}",
            unbound[0]
        );
    };
    assert_eq!(anchor_kind, "ado_work_item");
    assert!(why.contains("12346"), "{why}");
    assert!(why.contains("acme/work-items"), "{why}");
}

/// A snapshot whose bytes moved after the pin was written binds nothing, and
/// every edge into it says why.
///
/// The alternative is an absent resolver, which makes the graph report that the
/// run does not have a resolver over a repository that declares one and has it.
#[test]
fn a_snapshot_that_is_not_the_pinned_artifact_binds_nothing_and_names_the_pin() {
    let scratch = Scratch::new("moved");
    let digest = tree(&scratch);
    let declaration = declaration(&digest);
    import(&scratch, &declaration);

    // The item the edge was checked against is renumbered after the import. The
    // pin is what catches it, and this is the whole reason the resolver verifies
    // rather than reading.
    scratch.write(
        "imports/ado/snapshot.yml",
        &PAYLOAD.replace("id: \"12345\"", "id: \"99999\""),
    );

    let read = read(&scratch);
    let graph = read.graph(scratch.path(), std::slice::from_ref(&declaration));
    let unbound = unresolved(&graph);
    assert_eq!(unbound.len(), 1, "{:?}", targets(&graph));
    let Unbound::AnchorUnresolved { why, .. } = unbound[0].1 else {
        panic!("a moved snapshot bound something: {:?}", unbound[0]);
    };
    assert!(why.contains("not the pinned artifact"), "{why}");
    // And never the message about a resolver the run does not have, which sends
    // a reader looking for a feature rather than for a byte.
    assert!(!why.contains("does not have"), "{why}");
}

/// The check layer over the same tree, because a bound target and a rule that
/// reports nothing are two claims. OBL-repo-0116's measurement is this one.
#[test]
fn the_check_layer_reports_no_unresolved_target_over_an_imported_edge() {
    let scratch = Scratch::new("checked");
    let digest = tree(&scratch);
    let declaration = declaration(&digest);
    import(&scratch, &declaration);

    let read = read(&scratch);
    let graph = read.graph(scratch.path(), std::slice::from_ref(&declaration));
    let source = std::fs::read_to_string(scratch.path().join("taxonomy.yml")).expect("it is there");
    let root = headwater_yaml::load(&source)
        .expect("it reads")
        .value
        .as_map()
        .expect("it is a mapping")
        .clone();
    let register = Register::read(&root).expect("the register reads");
    let run = headwater_check::run(
        &read.census,
        &graph,
        &Declared {
            lock: "sha256:a fixture tree has no lock",
            taxonomy: &read.taxonomy,
            shape: &read.shape,
            relations: &read.relations,
            config: &read.config,
            register: &register,
            adoption: None,
            source: "taxonomy.yml",
        },
        &Context::at(Date::parse("2026-08-14").expect("the pinned date")),
        &mut Cache::disabled(),
    );
    let reported: Vec<&headwater_check::Finding> = run
        .findings
        .iter()
        .filter(|finding| finding.rule == "relation.target.unresolved")
        .collect();
    assert!(reported.is_empty(), "{reported:?}");
}
