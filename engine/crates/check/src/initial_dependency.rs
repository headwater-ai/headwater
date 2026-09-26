// SPDX-License-Identifier: Apache-2.0
//! A Graph-origin check: a live document does not rest on a draft one.
//!
//! # The gap this closes
//!
//! [`crate::dependency`] reports a live document that rests on a terminal one.
//! Nothing reported the other end of the lifecycle: a document a reader may
//! rely on, pointing at a document nobody has promoted.
//! [#569](https://github.com/headwater-ai/headwater/issues/569) is the report,
//! and in this repository a shelf of obligation records stood at `draft` for
//! weeks while live documents cited them.
//!
//! # Every relation between two documents, and not the sensitive ones
//!
//! The terminal rule reads the relations the taxonomy marks
//! `lifecycle_sensitive`, and the published base marks nothing beyond
//! `succession` ([HW-DR-0065](../../../../docs/decisions/0065-a-relation-declares-lifecycle-sensitive-for-itself-and-a-core-requirement-demands-it-of-a-family.md)).
//! That rule leaves `traces_to` unmarked because a live document may cite a
//! retired one legitimately, as a citation of the record the retired one left.
//! A draft leaves no record. The guidance its state declares is that nothing
//! may rely on it, so there is no legitimate citation of one to exempt, and a
//! rule gated on the same flag would reach no pair in any adopter of the base.
//! So this rule reads every relation whose two ends are documents.
//!
//! # The one exemption: a relation that writes a state onto its target
//!
//! `supersedes` declares `on_target: {set_state: superseded}`. An edge of it is
//! a statement about its target, which it retires, and never a reliance on
//! what the target says. The exemption is the relation outright, whatever state
//! the target stands at, because superseding a draft is how a draft that lost
//! an argument is closed.
//!
//! # What the rule shares with its sibling
//!
//! Live, initial and terminal are the `role` on each state value, read through
//! [`crate::lifecycle_state::StateFacet::standing`], so a taxonomy that renames
//! every state is read unchanged. The unit is [`EdgeUnit::Pair`], so an edge
//! onto an anchor forms no instance. A target standing at a state its own
//! regime does not name is `lifecycle.state.not_admitted`'s finding, and this
//! rule stands down with the same skip [`crate::dependency`] records.
//!
//! # The severity is advisory, and the finding carries no patch
//!
//! The repair is to promote the target, point the source somewhere else, or
//! move the source back to draft, and only an author can choose among the
//! three. `CT-LIFE-5` is advisory and permanently so for that reason.

use crate::dependency::{not_a_state, undeclared};
use crate::finding::{at, Finding, Severity};
use crate::instance::Outcome;
use crate::lifecycle_state::{Standing, StateFacet, Stood};
use crate::scope::{EdgeCheck, EdgeUnit, EdgeView};
use crate::shape::Shape;
use headwater_graph::declarations::Relation;
use headwater_graph::Declarations;

pub const RULE: &str = "lifecycle.dependency.on_initial";

/// The group carried no half at all, which the instantiation never produces.
const NO_HALF: &str = "the entry carries no declared half";

/// The far end is not a document. `EdgeUnit::Pair` never groups one.
const NO_PAIR: &str = "this edge has no second document to read";

/// The check. It carries every declared relation and the reading of the state
/// facet.
pub struct InitialDependency<'a> {
    relations: &'a [Relation],
    shape: &'a Shape,
    facet: StateFacet,
}

impl<'a> InitialDependency<'a> {
    pub fn over(declarations: &'a Declarations, shape: &'a Shape) -> Self {
        InitialDependency {
            relations: &declarations.relations,
            shape,
            facet: StateFacet::of(shape),
        }
    }

    /// Whether a kind's own regime names a state, and `true` for a kind that
    /// binds no regime. See [`crate::dependency`].
    fn regime_names(&self, kind: &str, state: &str) -> bool {
        match self.shape.lifecycle_of(kind) {
            Some(regime) => regime.states().contains(&state),
            None => true,
        }
    }
}

impl EdgeCheck for InitialDependency<'_> {
    const RULE: &'static str = self::RULE;
    /// See [`crate::placement::Placement::VERSION`].
    const VERSION: u32 = 1;
    /// The Q4 pair: both ends are documents, because the rule reads a facet at
    /// each of them.
    const UNIT: EdgeUnit = EdgeUnit::Pair;

    /// Every declared relation, where a facet carries the `state` role. A
    /// taxonomy with no state facet has no live and no draft document.
    fn instantiates(&self, relation: &str) -> bool {
        self.facet.name.is_some() && self.relations.iter().any(|known| known.name == relation)
    }

    fn evaluate(&self, view: &EdgeView<'_>) -> Outcome {
        let Some(half) = view.declared_half().or_else(|| view.inverse_half()) else {
            return Outcome::Skipped(NO_HALF.to_string());
        };
        let Some(relation) = self
            .relations
            .iter()
            .find(|known| known.name == half.declared)
        else {
            return Outcome::Skipped(NO_PAIR.to_string());
        };
        let Some((source, target)) = view.ends() else {
            return Outcome::Skipped(NO_PAIR.to_string());
        };

        // An edge that writes a state onto its target is a statement about the
        // target, and never a reliance on it.
        if relation.sets_target_state.is_some() {
            return Outcome::Passed;
        }

        let (Some(source_facets), Some(target_facets)) = (source.facets(), target.facets()) else {
            return Outcome::Skipped(
                "the census parsed no document at one end of this edge, so there is no state to \
                 read there"
                    .to_string(),
            );
        };
        let source_state = match self.facet.stood(source_facets) {
            Stood::At(state) => state,
            Stood::Undeclared => return Outcome::Skipped(undeclared("source", source.id)),
            Stood::NotAState(value) => {
                return Outcome::Skipped(not_a_state("source", source.id, value))
            }
        };
        let target_state = match self.facet.stood(target_facets) {
            Stood::At(state) => state,
            Stood::Undeclared => return Outcome::Skipped(undeclared("target", target.id)),
            Stood::NotAState(value) => {
                return Outcome::Skipped(not_a_state("target", target.id, value))
            }
        };

        if self.facet.standing(source_state) != Standing::Live {
            return Outcome::Passed;
        }
        if self.facet.standing(target_state) != Standing::Initial {
            return Outcome::Passed;
        }
        // A state the target's own regime does not name is a defect of that
        // document, and `lifecycle.state.not_admitted` is reporting it.
        if !self.regime_names(target.kind, target_state) {
            return Outcome::Skipped(format!(
                "`{}` stands at `{target_state}`, which the lifecycle regime of a `{}` does not \
                 name, and `{}` reports that",
                target.id,
                target.kind,
                crate::lifecycle_state::RULE
            ));
        }

        let (line, column) = at(Some(half.span));
        Outcome::failed_with(Finding {
            rule: self::RULE,
            severity: Severity::Warn,
            obligation: None,
            path: half.source.path.clone(),
            line,
            column,
            message: format!(
                "`{}` stands at `{source_state}` and declares `{}` to `{}`, which stands at the \
                 initial state `{target_state}`: nothing has promoted that document, so a live \
                 document does not rest on it",
                source.id, relation.name, target.id
            ),
            remediation: format!(
                "promote {} once it is settled, point `{}` in {} at a document that stands, or \
                 move {} back to its initial state until the target is settled",
                target.path, half.name, half.source.path, source.path
            ),
            // No patch. Which of the three repairs is right is a judgment
            // about both documents that only an author can make.
            patch: None,
        })
    }
}
