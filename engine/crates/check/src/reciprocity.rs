// SPDX-License-Identifier: Apache-2.0
//! A Graph-origin check, generated from `reciprocal: required`.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#the-five-origins-of-a-check)
//! puts reciprocity first among the Graph-origin examples and rules that Graph
//! checks are "generated, not written": a new relation in the taxonomy produces
//! its checks with no code. Nothing below names a relation. The instance set is
//! every declared pair of every relation whose declaration says `required`, so
//! a taxonomy that declares one more gets one more check and this file does not
//! change.
//!
//! # The unit is the pair, and not the half
//!
//! Either half may be the one an author wrote. This corpus writes both names:
//! `cites_evidence` on the citing document and `cited_by` on the cited one.
//! [`headwater_graph::Edge::declared_triple`] normalizes the two halves of one
//! pair to one triple however the two authors reached for a name, so the check
//! is a set difference over triples rather than a search for a key.
//!
//! A pair is complete when **both documents declared it**: one half written
//! from the source end, and one from the target end. The direction an edge
//! carries is what says which end wrote it, so the test is that both directions
//! are present on one triple.
//!
//! # Where the finding reports, and where the fix goes
//!
//! Against the document that *did* write its half, at the line of the entry it
//! wrote. [Spec 4](../../../../docs/spec/04-assurance-model.md#findings) gives
//! that shape in its own worked example, and the reason is practical: the
//! author who is looking at a file is the one who just declared the edge. The
//! remediation names the other document and the line to add there.
//!
//! Whichever half is missing, the document that owes it is the one at the far
//! end of the half that exists. So the remediation names one path in both
//! cases, and only the relation name changes.
//!
//! The fix is mechanical and total, which is the bar
//! [spec 12](../../../../docs/spec/12-check-layer.md#fixability) sets: the
//! missing half is derivable from the half that exists, with no judgment.

use crate::finding::{at, Finding, Severity};
use crate::instance::Instance;
use headwater_graph::declarations::Relation;
use headwater_graph::{Declarations, Direction, Edge, Graph, Reciprocal, Target};

pub const RULE: &str = "relation.reciprocity.missing";

/// Instantiate the check over a graph: one instance per declared pair.
pub fn run(graph: &Graph, declarations: &Declarations) -> Vec<Instance> {
    // The generation step, in full: read the declarations, and take the ones
    // that say `required`.
    let required: Vec<&Relation> = declarations
        .relations
        .iter()
        .filter(|relation| relation.reciprocal == Reciprocal::Required)
        .collect();

    // Group the halves by the triple they normalize to. The edges arrive in
    // path order, so the groups come out in a stable order with no sort here.
    let mut pairs: Vec<(String, Vec<&Edge>)> = Vec::new();
    for edge in &graph.edges {
        // An anchor and an unbound target are not document pairs, and
        // `declared_triple` says so by returning nothing.
        let Some((source, relation, target)) = edge.declared_triple() else {
            continue;
        };
        if !required.iter().any(|r| r.name == relation) {
            continue;
        }
        let key = format!("{source}\u{1f}{relation}\u{1f}{target}");
        match pairs.iter_mut().find(|(known, _)| known == &key) {
            Some((_, halves)) => halves.push(edge),
            None => pairs.push((key, vec![edge])),
        }
    }

    pairs
        .into_iter()
        .map(|(_, halves)| instance(&halves, declarations))
        .collect()
}

fn instance(halves: &[&Edge], declarations: &Declarations) -> Instance {
    // One half written from the source end, and one from the target end. A
    // pair is reciprocal when a document at each end declared it.
    let forward = halves
        .iter()
        .find(|edge| edge.direction == Direction::AsDeclared);
    let backward = halves
        .iter()
        .find(|edge| edge.direction == Direction::Inverse);

    let (written, missing) = match (forward, backward) {
        // Both ends wrote it. One instance, read against both endpoints.
        (Some(edge), Some(_)) => return Instance::passed(RULE, ends(edge)),
        (Some(edge), None) => (*edge, Missing::TheInverseHalf),
        (None, Some(edge)) => (*edge, Missing::TheDeclaredHalf),
        (None, None) => unreachable!("a pair with no half is never grouped"),
    };

    let relation = declarations
        .relations
        .iter()
        .find(|r| r.name == written.declared)
        .expect("the pair was grouped by a relation that requires reciprocity");

    // The far end of the half that exists is the document that owes the other
    // half, whichever half that is. See the module comment.
    let owed_by = match &written.target {
        Target::Document { path, .. } => path.clone(),
        // Unreachable: `declared_triple` returns nothing for every other
        // target. Stated rather than unwrapped, because a panic inside a check
        // is a run that reports nothing about the rest of the corpus.
        _ => written.raw_target.clone(),
    };
    let (name, target) = match missing {
        Missing::TheInverseHalf => (
            relation
                .inverse
                .clone()
                .unwrap_or_else(|| relation.name.clone()),
            written.source.id.clone(),
        ),
        Missing::TheDeclaredHalf => (relation.name.clone(), written.source.id.clone()),
    };

    let (line, column) = at(Some(written.span));
    Instance::failed(
        RULE,
        ends(written),
        Finding {
            rule: RULE,
            severity: Severity::Error,
            obligation: None,
            path: written.source.path.clone(),
            line,
            column,
            message: format!(
                "`{}` declares `{}: {}`, and `{}` requires both ends, so {owed_by} owes `{name}`",
                written.source.id, written.name, written.raw_target, relation.name,
            ),
            remediation: format!("add `{name}: {target}` under `relations:` in {owed_by}"),
            fixable: true,
        },
    )
}

/// The two documents an edge-scoped instance reads: the declaring end and the
/// far end, in that order.
///
/// The far end of an edge that bound to no document is not a document, so an
/// instance over it reads one file. `declared_triple` never groups such an
/// edge, so the fallback is unreachable and it is written rather than
/// unwrapped: a panic inside a check silences the rest of the corpus.
fn ends(edge: &Edge) -> Vec<String> {
    let mut reads = vec![edge.source.path.clone()];
    if let Target::Document { path, .. } = &edge.target {
        reads.push(path.clone());
    }
    reads
}

/// Which half nobody wrote.
enum Missing {
    /// The document at the far end never wrote the relation's inverse name.
    TheInverseHalf,
    /// The document at the far end never wrote the relation's own name.
    TheDeclaredHalf,
}
