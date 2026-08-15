// SPDX-License-Identifier: Apache-2.0
//! A live document does not rest on a terminal one, where the taxonomy says
//! the lifecycle of the two ends is part of what the relation means.
//!
//! # The gap this closes
//!
//! [Spec 3](../../../../docs/spec/03-authoring-and-lifecycle.md#lifecycle)
//! lists this among the rules the engine enforces from the declaration alone:
//! "a live document may not depend on a terminal one through a relation
//! declared `lifecycle_sensitive`". The declaration existed and no rule read
//! it. `headwater_resolve::core` read the flag to decide whether a taxonomy
//! satisfies its own core, and that reading never opens a corpus.
//!
//! [#223](https://github.com/headwater-ai/headwater/issues/223) is the report.
//! It was the last unimplemented clause of that list.
//!
//! # Live and terminal are roles, not a list of state names
//!
//! The five states of the base vocabulary carry a `role`, and
//! [`crate::lifecycle_state::StateFacet::standing`] is the one component that
//! folds them into the two words spec 3 states its rule in. Nothing here holds
//! a list of state names, so a taxonomy that renames every state — spec 2's own
//! worked overlay does — is read by this rule unchanged.
//!
//! # What the relation itself declares, and the case that is not a dependency
//!
//! `supersedes` declares `on_target: {set_state: superseded}`, so an edge of it
//! **puts** its target in that state. A rule that reported every live document
//! reaching a terminal one through a succession relation would report every
//! correct supersession in the corpus, and the remedy would be to delete the
//! lineage that the same core requirement exists to retain
//! ([spec 2](../../../../docs/spec/02-taxonomy-model.md#the-immutable-core):
//! dropping the `terminal-retained` role fails resolution "because succession
//! could no longer retain lineage, which the core requires").
//!
//! So the rule reads [`headwater_graph::declarations::Relation::sets_target_state`]
//! beside the target's state. Where the two are equal, this edge is the cause
//! of the state rather than a dependency on it, and there is nothing to report.
//! Where they differ, a live document is resting on a document that some other
//! history retired, and that is spec 3's finding. The distinction is read off
//! two declarations and no exception is written into this engine.
//!
//! # Which rule owns which case
//!
//! [`crate::lifecycle_state`] owns **the state one document stands in**, against
//! the states the regime of its kind names. This rule owns **a pair**, and it
//! stands down wherever that rule is already looking: a target standing at a
//! state its own regime does not name is one defect, and reporting it again per
//! incoming edge would multiply one wrong facet by the number of documents that
//! cite it. The skip names the rule that reports it, which is the deferral
//! [`crate::transition`] and [`crate::lifecycle_state`] already make to each
//! other.
//!
//! [#225](https://github.com/headwater-ai/headwater/issues/225) owns a third
//! case and it is neither of these. It asks whether the kind at the target end
//! *admits* the state that `on_target.set_state` would write onto it, which is
//! a question about two declarations and about no document at all.
//!
//! # A terminal source is deliberately silent
//!
//! Spec 3's sentence is about a **live** document and says nothing about a
//! terminal one. A retired document's citations are part of the record it was
//! retired with: re-pointing them would rewrite what that document said when it
//! stood, and there is no live claim resting on the target either way. So a
//! terminal source passes, and so does a `draft` one, whose state carries the
//! role `initial` and whose author is still arguing.
//!
//! This corpus holds the case: six `traces_to` halves run from a discharged
//! obligation record to a live specification part. They are the direction this
//! rule does not report, and they are why the source is read rather than
//! assumed.
//!
//! # The scope, and why an edge rather than a document
//!
//! The unit is [`EdgeUnit::Pair`], which is the Q4 triple: a source, a relation
//! and a target document. Both endpoints are documents by construction there,
//! which is what this rule needs and what [`crate::target`] could not use.
//! [Spec 12](../../../../docs/spec/12-check-layer.md#scope--the-declaration-everything-else-rests-on)
//! fixes an edge-scoped read set at one relation instance and both endpoints,
//! so the two states this rule reads are both in the read set that keys it. An
//! edit to either end moves the key.
//!
//! # The severity is advisory, and the finding carries no patch
//!
//! The [fixability](../../../../docs/spec/12-check-layer.md#fixability) bar is
//! whether the remediation is mechanical and total. It is neither here. The
//! repair is a successor to point at, a retirement of the citing document, or a
//! judgment that the citation stands anyway, and only an author knows which.
//! `CT-LIFE-3` is advisory for the same reason and it says so.

use crate::finding::{at, Finding, Severity};
use crate::instance::Outcome;
use crate::lifecycle_state::{Standing, StateFacet, Stood};
use crate::scope::{EdgeCheck, EdgeUnit, EdgeView};
use crate::shape::Shape;
use headwater_graph::declarations::Relation;
use headwater_graph::Declarations;

pub const RULE: &str = "lifecycle.dependency.on_terminal";

/// The group carried no half at all, which the instantiation never produces.
/// Recorded rather than panicked on, for [`crate::target`]'s reason.
const NO_HALF: &str = "the entry carries no declared half";

/// The far end is not a document. `EdgeUnit::Pair` never groups one, so this is
/// unreachable rather than tolerated.
const NO_PAIR: &str = "this edge has no second document to read";

/// The check. It carries the relations the taxonomy marked and the reading of
/// the state facet, both of which are declared rather than known.
pub struct Dependency<'a> {
    /// Every relation whose family a core requirement declares
    /// `lifecycle_sensitive`. Empty for a taxonomy that declares no such
    /// requirement, and then this rule generates no instance at all.
    sensitive: Vec<&'a Relation>,
    shape: &'a Shape,
    facet: StateFacet,
}

impl<'a> Dependency<'a> {
    pub fn over(declarations: &'a Declarations, shape: &'a Shape) -> Self {
        Dependency {
            sensitive: declarations
                .relations
                .iter()
                .filter(|relation| relation.lifecycle_sensitive)
                .collect(),
            shape,
            facet: StateFacet::of(shape),
        }
    }

    /// Whether a kind's own regime names a state, and `true` for a kind that
    /// binds no regime.
    ///
    /// A kind with no declared machine has no admitted set to hold a value
    /// against, so there is no second rule looking at it and nothing to defer
    /// to. The role on the value still says what the state is.
    fn regime_names(&self, kind: &str, state: &str) -> bool {
        match self.shape.lifecycle_of(kind) {
            Some(regime) => regime.states().contains(&state),
            None => true,
        }
    }
}

impl EdgeCheck for Dependency<'_> {
    const RULE: &'static str = self::RULE;
    /// See [`crate::placement::Placement::VERSION`].
    const VERSION: u32 = 1;
    /// The Q4 pair. See the module comment: both ends have to be documents,
    /// because the rule reads a facet at each of them.
    const UNIT: EdgeUnit = EdgeUnit::Pair;

    /// A relation the taxonomy marked, and only where a facet carries the
    /// `state` role. A taxonomy with no state facet has no live and no terminal
    /// document, and an instance over one could only ever pass.
    fn instantiates(&self, relation: &str) -> bool {
        self.facet.name.is_some() && self.sensitive.iter().any(|known| known.name == relation)
    }

    fn evaluate(&self, view: &EdgeView<'_>) -> Outcome {
        // The declared half when a document wrote it, because a finding anchors
        // at the entry an author is looking at.
        let Some(half) = view.declared_half().or_else(|| view.inverse_half()) else {
            return Outcome::Skipped(NO_HALF.to_string());
        };
        let Some(relation) = self
            .sensitive
            .iter()
            .find(|known| known.name == half.declared)
        else {
            return Outcome::Skipped(NO_PAIR.to_string());
        };
        let Some((source, target)) = view.ends() else {
            return Outcome::Skipped(NO_PAIR.to_string());
        };

        // Three states at each end, and the two absences are kept apart from
        // the values. A document the census parsed nothing for is not a
        // document that declared nothing, and a document that declares no state
        // is not one standing at a state this rule read as not terminal.
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

        // A source that is not live is not what spec 3's sentence is about, and
        // that is a pass rather than a skip: the rule looked at both ends and
        // found nothing to report.
        if self.facet.standing(source_state) != Standing::Live {
            return Outcome::Passed;
        }
        if self.facet.standing(target_state) != Standing::Terminal {
            return Outcome::Passed;
        }
        // The state this relation writes onto its target. An edge that put the
        // target where it stands is the record of a retirement rather than a
        // dependency on one.
        if relation.sets_target_state.as_deref() == Some(target_state) {
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
                 terminal state `{target_state}`: the taxonomy declares this relation \
                 lifecycle-sensitive, so a live document does not rest on a retired one through it",
                source.id, relation.name, target.id
            ),
            remediation: format!(
                "point `{}` in {} at the document that replaced {}, or retire {} on its own terms \
                 if it still means what the retired one said",
                half.name, half.source.path, target.path, source.path
            ),
            // No patch. Which live document replaced the retired one is a
            // statement about the corpus that only an author can make, and
            // there may be none.
            patch: None,
        })
    }
}

/// The reason an instance skips where one end declares no state.
fn undeclared(end: &str, id: &str) -> String {
    format!(
        "the document at the {end} end, `{id}`, declares no value for the state facet, so there is \
         no state to read there"
    )
}

/// The reason an instance skips where one end writes a value the state facet
/// does not admit. `facet.value.not_permitted` reports the value itself.
fn not_a_state(end: &str, id: &str, value: &str) -> String {
    format!(
        "the document at the {end} end, `{id}`, writes `{value}`, which the state facet does not \
         admit, and `facet.value.not_permitted` reports that"
    )
}
