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
//! The same tree holds two more rulings (HW-DR-0101). A symmetric relation gets
//! no far half at any opening state, because one half states the edge and a
//! far half written into a live target is read by
//! `lifecycle.dependency.on_initial`. And `supersedes` writes no state onto its
//! target at any opening state: `lifecycle.state.not_set_by_edge` reads that
//! transition and `headwater check --fix` writes it.
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

/// Scaffold one document of `kind` that relates to the anchor through
/// `relation`, and return the plan and what the composition would write.
fn scaffold(kind: &str, relation: &str) -> (Plan, Vec<(String, String)>) {
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
    let relates = vec![(relation.to_string(), "NOTE-FIX-the-anchor".to_string())];
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
    let files = composed
        .iter()
        .map(|file| (file.path.clone(), file.text.clone()))
        .collect();
    (plan, files)
}

/// The paths of the composed files.
fn paths(files: &[(String, String)]) -> Vec<String> {
    files.iter().map(|(path, _)| path.clone()).collect()
}

/// The composed bytes of the anchor, if this run writes it.
fn anchor(files: &[(String, String)]) -> Option<&str> {
    files
        .iter()
        .find(|(path, _)| path == ANCHOR)
        .map(|(_, contents)| contents.as_str())
}

#[test]
fn a_document_that_opens_at_an_initial_state_owes_its_far_half() {
    let (plan, files) = scaffold("at_draft", "elaborates");
    let paths = paths(&files);
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
    let (plan, files) = scaffold("at_current", "elaborates");
    let paths = paths(&files);
    let edge = &plan.edges[0];
    assert!(
        !edge.symmetric && edge.sets_target_state.is_none(),
        "{edge:?}"
    );
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
    let (plan, files) = scaffold("at_proposed", "elaborates");
    let paths = paths(&files);
    let edge = &plan.edges[0];
    assert!(edge.owed.is_none(), "only an initial role defers: {edge:?}");
    assert!(edge.reciprocal.is_some(), "{edge:?}");
    assert!(
        paths.iter().any(|path| path == ANCHOR),
        "the anchor is spliced: {paths:?}"
    );
}

/// The decisive case of #1187. A draft that declares a symmetric relation onto
/// a live document leaves the live document's bytes alone, so no line runs
/// from a live document to a draft.
#[test]
fn a_draft_writes_no_far_half_of_a_symmetric_relation() {
    let (plan, files) = scaffold("at_draft", "pairs_with");
    let edge = &plan.edges[0];
    assert!(
        edge.reciprocal.is_none() && edge.owed.is_none(),
        "a symmetric edge has no far half: {edge:?}"
    );
    assert!(edge.symmetric, "the report reads this: {edge:?}");
    assert_eq!(edge.sets_target_state, None, "{edge:?}");
    assert_eq!(
        paths(&files).len(),
        1,
        "the anchor is left alone: {files:?}"
    );
}

#[test]
fn a_live_document_writes_no_far_half_of_a_symmetric_relation() {
    let (plan, files) = scaffold("at_current", "pairs_with");
    let edge = &plan.edges[0];
    assert!(
        edge.reciprocal.is_none() && edge.owed.is_none(),
        "a symmetric edge has no far half at any opening state: {edge:?}"
    );
    assert_eq!(
        paths(&files).len(),
        1,
        "the anchor is left alone: {files:?}"
    );
}

#[test]
fn a_live_supersession_writes_the_far_half_and_no_state() {
    let (plan, files) = scaffold("at_current", "supersedes");
    let edge = &plan.edges[0];
    assert!(edge.owed.is_none(), "{edge:?}");
    assert!(!edge.symmetric, "{edge:?}");
    assert_eq!(
        edge.sets_target_state.as_deref(),
        Some("superseded"),
        "the report names the state this run leaves to check --fix: {edge:?}"
    );
    let half = edge.reciprocal.as_ref().expect("the far half is written");
    assert_eq!(
        (half.relation.as_str(), half.path.as_str()),
        ("superseded_by", ANCHOR)
    );
    let anchor = anchor(&files).expect("the anchor is spliced");
    assert!(anchor.contains("superseded_by:"), "{anchor}");
    assert!(anchor.contains(half.id.as_str()), "{anchor}");
    assert!(
        anchor.contains("\nstatus: current\n"),
        "the anchor keeps its state, which check --fix sets: {anchor}"
    );
}

#[test]
fn a_draft_supersession_owes_its_far_half_and_leaves_the_target_alone() {
    let (plan, files) = scaffold("at_draft", "supersedes");
    let edge = &plan.edges[0];
    assert!(edge.reciprocal.is_none(), "{edge:?}");
    let owed = edge.owed.as_ref().expect("the far half is owed");
    assert_eq!(
        (owed.half.relation.as_str(), owed.until.as_str()),
        ("superseded_by", "draft")
    );
    assert_eq!(
        paths(&files).len(),
        1,
        "the anchor is left alone: {files:?}"
    );
}

/// Written through its inverse, `supersedes` retires the new document and not
/// the anchor, so the edge names no state for the anchor.
#[test]
fn an_inverse_supersession_names_no_state_for_its_target() {
    let (plan, _) = scaffold("at_current", "superseded_by");
    let edge = &plan.edges[0];
    assert_eq!(edge.sets_target_state, None, "{edge:?}");
    let half = edge.reciprocal.as_ref().expect("the far half is written");
    assert_eq!(half.relation, "supersedes", "{edge:?}");
}
