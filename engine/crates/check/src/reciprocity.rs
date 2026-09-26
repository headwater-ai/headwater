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
//! # A draft writer owes nothing yet
//!
//! The far half is owed only once the document that wrote the one half that
//! exists has left its initial state
//! ([HW-DR-0086](../../../../docs/decisions/0086-a-reciprocal-half-is-owed-once-its-writer-leaves-its-initial-state.md)).
//! A pair with one half, written by a document whose state carries the role
//! `initial`, passes. The writer is read through
//! [`crate::lifecycle_state::StateFacet::standing`], the one reader of that
//! role, so no state name is written here, and `headwater new` defers on the
//! same predicate. A writer this rule cannot read a state from (no state facet,
//! no value, or a value that is not a state) defers nothing, which keeps the
//! behavior the check had before the deferral. The deferral follows the writer
//! and never the target: a live document that writes its half onto a draft is
//! reported, and the draft owes the half now.
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
//!
//! # The scope
//!
//! [`EdgeCheck`] is the declaration, and it is the whole of it: one relation
//! instance and both endpoints. The view carries the halves and the two
//! documents they join, and the read set it fixes is what coverage counts the
//! instance against. Both ends, and never the declaring end alone.

use crate::finding::{at, Finding, Severity};
use crate::instance::Outcome;
use crate::lifecycle_state::{Standing, StateFacet, Stood};
use crate::scope::{EdgeCheck, EdgeView};
use crate::shape::Shape;
use headwater_graph::declarations::Relation;
use headwater_graph::{Declarations, Reciprocal, Target};

pub const RULE: &str = "relation.reciprocity.missing";

/// A pair that carries no half at all, which grouping never produces. Recorded
/// rather than panicked on, because a panic inside a check is a run that
/// reports nothing about the rest of the corpus.
const NO_HALF: &str = "the pair carries no declared half";

/// A pair of a relation that the declarations no longer hold. Unreachable for
/// the same reason and recorded on the same terms: the generation step took
/// the relation from those declarations.
const NO_RELATION: &str = "no declaration holds the relation this pair declares";

/// The check, generated from the relation declarations.
pub struct Reciprocity<'a> {
    /// The relations that say `required`, which is the generation step in full.
    required: Vec<&'a Relation>,
    /// The state facet, read for the writer of a lone half. See the module
    /// comment.
    facet: StateFacet,
}

impl<'a> Reciprocity<'a> {
    pub fn over(declarations: &'a Declarations, shape: &Shape) -> Self {
        Reciprocity {
            required: declarations
                .relations
                .iter()
                .filter(|relation| relation.reciprocal == Reciprocal::Required)
                .collect(),
            facet: StateFacet::of(shape),
        }
    }
}

impl EdgeCheck for Reciprocity<'_> {
    const RULE: &'static str = self::RULE;
    /// See [`crate::placement::Placement::VERSION`].
    /// 2: a lone half written by a document at an initial state passes.
    const VERSION: u32 = 2;

    fn instantiates(&self, relation: &str) -> bool {
        self.required.iter().any(|known| known.name == relation)
    }

    fn evaluate(&self, view: &EdgeView<'_>) -> Outcome {
        // One half written from the source end, and one from the target end. A
        // pair is reciprocal when a document at each end declared it.
        let (written, missing) = match (view.declared_half(), view.inverse_half()) {
            // Both ends wrote it. One instance, read against both endpoints.
            (Some(_), Some(_)) => return Outcome::Passed,
            (Some(edge), None) => (edge, Missing::TheInverseHalf),
            (None, Some(edge)) => (edge, Missing::TheDeclaredHalf),
            (None, None) => return Outcome::Skipped(NO_HALF.to_string()),
        };

        // With one half, the declaring document is the writer, because the view
        // keys the instance on the one half that exists. A writer at an initial
        // state owes nothing yet. See the module comment.
        if let Some(facets) = view.declarer_facets() {
            if let Stood::At(state) = self.facet.stood(facets) {
                if self.facet.standing(state) == Standing::Initial {
                    return Outcome::Passed;
                }
            }
        }

        let Some(relation) = self
            .required
            .iter()
            .find(|known| known.name == written.declared)
        else {
            return Outcome::Skipped(NO_RELATION.to_string());
        };

        // The far end of the half that exists is the document that owes the
        // other half, whichever half that is. See the module comment.
        let owed_by = match &written.target {
            Target::Document { path, .. } => path.clone(),
            // Unreachable: `declared_triple` returns nothing for every other
            // target, so no such edge is ever grouped into a pair.
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
        Outcome::failed_with(Finding {
            rule: self::RULE,
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
            // The patch names the far document, the relation and the target,
            // and no offset. Where the half goes inside that document is the
            // splice's answer, and `headwater_scaffold::write::splice` is the
            // one thing in this engine that gives it.
            patch: Some(crate::Patch::Half {
                path: owed_by.clone(),
                relation: name.clone(),
                id: target.clone(),
                attributes: Vec::new(),
            }),
        })
    }
}

/// Which half nobody wrote.
enum Missing {
    /// The document at the far end never wrote the relation's inverse name.
    TheInverseHalf,
    /// The document at the far end never wrote the relation's own name.
    TheDeclaredHalf,
}
