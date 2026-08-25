// SPDX-License-Identifier: Apache-2.0
//! The scratch repository that the whole chain runs over, shared by every
//! suite that needs one.
//!
//! `resolution.rs` built this first, for the four cases that end
//! [HW-OBL-0116](../../../../../docs/obligations/0116-no-anchor-resolver-reads-a-committed-snapshot-so-every-imported-edge-lands-unresolved.md).
//! `drift.rs` needs the same tree and the same four verbs, so the tree moved
//! here rather than being written a second time. A second copy would be a
//! second corpus, and two suites that disagreed about what the fixture declares
//! would each be measuring their own.

#![allow(dead_code)]

use headwater_census::census;
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::Shape;
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::index::Index;
use headwater_graph::{Config, Graph};
use headwater_import::{Declaration, Plan};
use std::path::{Path, PathBuf};

/// A tree that removes itself, named for the case that made it.
pub(crate) struct Scratch(PathBuf);

impl Scratch {
    pub(crate) fn new(case: &str) -> Self {
        let at = std::env::temp_dir().join(format!(
            "headwater-resolution-{}-{case}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).expect("the scratch tree is made");
        Scratch(at)
    }

    pub(crate) fn path(&self) -> &Path {
        &self.0
    }

    pub(crate) fn write(&self, relative: &str, text: &str) {
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
pub(crate) const TAXONOMY: &str = "\
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

pub(crate) const PAYLOAD: &str = "\
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

pub(crate) const RESOLVER: &str = "ado-snapshot";

/// Compute the release record over the snapshot directory as it stands, write
/// it, and answer with the digest that a declaration has to pin.
///
/// Called once by [`tree`] and again by any case that rewrites the snapshot,
/// because a snapshot whose bytes moved under its record binds nothing at all
/// and a case about a moved revision would then be measuring a moved pin.
pub(crate) fn publish(scratch: &Scratch) -> String {
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

/// The repository the chain runs over, and the digest of its snapshot.
pub(crate) fn tree(scratch: &Scratch) -> String {
    scratch.write(
        "docs/spec/00-first.md",
        "---\nid: SPEC-FIX-one\ndoc_type: design_spec\n---\n\n# The first part\n",
    );
    scratch.write("taxonomy.yml", TAXONOMY);
    scratch.write("imports/ado/snapshot.yml", PAYLOAD);
    publish(scratch)
}

pub(crate) fn declaration(digest: &str) -> Declaration {
    Declaration {
        name: "ado".to_string(),
        at: "imports/ado".to_string(),
        digest: Some(digest.to_string()),
        channel: Some("the platform team's fetch job".to_string()),
        resolver: Some(RESOLVER.to_string()),
    }
}

/// Everything a graph build needs, out of one walk of the tree.
pub(crate) struct Read {
    pub(crate) corpus: Corpus,
    pub(crate) census: headwater_census::census::Census,
    pub(crate) relations: Declarations,
    pub(crate) shape: Shape,
    pub(crate) taxonomy: Taxonomy,
    pub(crate) config: Config,
}

pub(crate) fn read(scratch: &Scratch) -> Read {
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
    pub(crate) fn graph(&self, root: &Path, declarations: &[Declaration]) -> Graph {
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

    pub(crate) fn index(&self) -> Index {
        Index::build(&self.census, &self.config)
    }
}

/// Plan the import and write it, which is `headwater import ado --write`.
pub(crate) fn import(scratch: &Scratch, declaration: &Declaration) -> Plan {
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
