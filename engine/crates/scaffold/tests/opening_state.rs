// SPDX-License-Identifier: Apache-2.0
//! `headwater new` defers a required far half exactly when the new document
//! opens at a state whose role is `initial` (HW-DR-0086), and at no other
//! opening state.
//!
//! The pipeline test holds the deferral, and every kind it scaffolds opens at
//! `draft`. So it cannot tell the predicate `== Initial` apart from
//! `!= Terminal` or `!= Live`. This test can: one kind opens at an initial
//! state, one at a live state, and one at a state with no role.
//!
//! It proposes and composes only. Nothing is written, so it reads the
//! committed tree in place.

use headwater_census::census;
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::shape::Shape;
use headwater_check::Date;
use headwater_graph::declarations::Declarations;
use headwater_graph::index::Index;
use headwater_graph::Config;
use headwater_scaffold::{propose, write, Plan, Request, Sources};
use headwater_yaml::Mapping;
use std::path::{Path, PathBuf};

const PINNED: &str = "2026-08-14";
const TREE: &str = "opening-state";
const ANCHOR: &str = "opening-state/currents/the-anchor.md";

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
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

/// Scaffold one document of `kind` that elaborates the anchor, and return the
/// plan and the paths the composition would write.
fn scaffold(kind: &str) -> (Plan, Vec<String>) {
    let root = fixtures_dir();
    let resolved = load_map(&root.join(format!("{TREE}.taxonomy.yml")));
    let shelves = Taxonomy::read(&resolved).expect("the shelves read");
    let relations = Declarations::read(&resolved).expect("the relations read");
    let shape = Shape::read(&resolved).expect("the shape reads");
    let corpus = Corpus::new(root.clone(), TREE);
    let taken = census::take(&corpus, &shelves);
    let config = Config::default();
    let index = Index::build(&taken, &config);
    let claims = headwater_check::claim::Claims::at(&root);
    let sources = Sources {
        resolved: &resolved,
        shape: &shape,
        shelves: &shelves,
        relations: &relations,
        census: &taken,
        index: &index,
        config: &config,
        claims: &claims,
    };
    let relates = vec![("elaborates".to_string(), "NOTE-FIX-the-anchor".to_string())];
    let title = format!("A new {kind}");
    let request = Request {
        kind,
        title: &title,
        summary: None,
        now: Date::parse(PINNED).expect("the pinned date"),
        relates: &relates,
        given: &[],
        directory: None,
    };
    let plan = propose(&sources, &request).unwrap_or_else(|refusal| panic!("{kind}: {refusal}"));
    let composed =
        write::compose(&root, &plan).unwrap_or_else(|refusal| panic!("{kind}: {refusal}"));
    let paths = composed.iter().map(|file| file.path.clone()).collect();
    (plan, paths)
}

#[test]
fn a_document_that_opens_at_an_initial_state_owes_its_far_half() {
    let (plan, paths) = scaffold("at_draft");
    let edge = &plan.edges[0];
    assert!(edge.reciprocal.is_none(), "{edge:?}");
    let owed = edge.owed.as_ref().expect("the far half is owed");
    assert_eq!(
        (
            owed.half.relation.as_str(),
            owed.half.path.as_str(),
            owed.until.as_str()
        ),
        ("elaborated_by", ANCHOR, "draft")
    );
    assert_eq!(paths.len(), 1, "the anchor is left alone: {paths:?}");
}

#[test]
fn a_document_that_opens_at_a_live_state_writes_its_far_half() {
    let (plan, paths) = scaffold("at_current");
    let edge = &plan.edges[0];
    assert!(
        edge.owed.is_none(),
        "nothing is deferred at a live state: {edge:?}"
    );
    let half = edge.reciprocal.as_ref().expect("the far half is written");
    assert_eq!(
        (half.relation.as_str(), half.path.as_str()),
        ("elaborated_by", ANCHOR)
    );
    assert!(
        paths.iter().any(|path| path == ANCHOR),
        "the anchor is spliced: {paths:?}"
    );
}

#[test]
fn a_document_that_opens_at_a_state_with_no_role_writes_its_far_half() {
    let (plan, paths) = scaffold("at_proposed");
    let edge = &plan.edges[0];
    assert!(edge.owed.is_none(), "only an initial role defers: {edge:?}");
    assert!(edge.reciprocal.is_some(), "{edge:?}");
    assert!(
        paths.iter().any(|path| path == ANCHOR),
        "the anchor is spliced: {paths:?}"
    );
}
