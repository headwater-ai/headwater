// SPDX-License-Identifier: Apache-2.0
//! The whole chain: an import writes an edge, and the next check says it
//! resolves.
//!
//! [HW-OBL-0116](../../../../docs/obligations/0116-no-anchor-resolver-reads-a-committed-snapshot-so-every-imported-edge-lands-unresolved.md)
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

mod chain;

use chain::{declaration, import, read, tree, Scratch, PAYLOAD, RESOLVER};
use headwater_check::{Cache, Context, Date, Declared, Register};
use headwater_graph::{Graph, Target, Unbound};

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

/// The defect HW-OBL-0116 recorded, and its end.
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
/// reports nothing are two claims. HW-OBL-0116's measurement is this one.
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
        &headwater_check::claim::Claims::empty(),
        &Context::at(Date::parse("2026-08-14").expect("the pinned date")),
        &mut Cache::disabled());
    let reported: Vec<&headwater_check::Finding> = run
        .findings
        .iter()
        .filter(|finding| finding.rule == "relation.target.unresolved")
        .collect();
    assert!(reported.is_empty(), "{reported:?}");
}
