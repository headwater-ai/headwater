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
//! # The direction is the one the author wrote
//!
//! A relation declares a direction, and an author may write either of its two
//! names. The document that writes the line is the one that cites, so each
//! written half is judged from its writer to the document it names. A live
//! document that writes `verifies` onto a draft rests on that draft, although
//! the relation declares the draft as its source. Where both halves are
//! written, each is judged, and each finding anchors at its own entry.
//!
//! # Every relation is read, including the ones that are not a reliance
//!
//! `constrains` runs from the constraining decision to the constrained one,
//! and `conflicts_with` is valid with a live end only while the other end is
//! not live. Neither is a reliance of the writer on the document named. This
//! rule reads both anyway, because the ruling is every relation, and a
//! relation is exempt only by a declaration the engine can read. A live
//! document that constrains a draft or conflicts with one names a text that
//! can still change under it, and the warning says so.
//!
//! # The one exemption: a relation that writes a state onto its target
//!
//! `supersedes` declares `on_target: {set_state: superseded}`. An edge of it
//! states that its target is replaced, and it is not a reliance on what the
//! target says. The exemption is the relation outright, whatever state the
//! target stands at. That exemption has a silent case: a live document that
//! supersedes a draft which still stands at `draft`. The base regimes do not
//! let a draft move to `superseded`, and no verb writes that state onto a
//! target, so the draft stays where it is and this rule does not report it.
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
use crate::scope::{EdgeCheck, EdgeEnd, EdgeUnit, EdgeView};
use crate::shape::Shape;
use headwater_graph::declarations::Relation;
use headwater_graph::edges::Edge;
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
        let Some(first) = view.declared_half().or_else(|| view.inverse_half()) else {
            return Outcome::Skipped(NO_HALF.to_string());
        };
        let Some(relation) = self
            .relations
            .iter()
            .find(|known| known.name == first.declared)
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

        // Each half an author wrote, read from the document that wrote it. The
        // declared half is written at the source and points at the target, and
        // the inverse half is written at the target and points at the source.
        let written = [
            view.declared_half().map(|half| (half, source, target)),
            view.inverse_half().map(|half| (half, target, source)),
        ];
        let mut findings = Vec::new();
        let mut skipped = None;
        for (half, writer, cited) in written.into_iter().flatten() {
            match self.judge(half, writer, cited) {
                Judged::Passed => {}
                Judged::Skipped(why) => {
                    skipped.get_or_insert(why);
                }
                Judged::Failed(finding) => findings.push(*finding),
            }
        }
        if !findings.is_empty() {
            return Outcome::failed(findings);
        }
        match skipped {
            Some(why) => Outcome::Skipped(why),
            None => Outcome::Passed,
        }
    }
}

/// What one written half comes to.
enum Judged {
    Passed,
    Skipped(String),
    /// Boxed, because a finding is many times the size of the other two arms.
    Failed(Box<Finding>),
}

impl InitialDependency<'_> {
    /// One half, from the document that wrote it to the document it names.
    fn judge(&self, half: &Edge, writer: EdgeEnd<'_>, cited: EdgeEnd<'_>) -> Judged {
        // Three states at each end, and the two absences are kept apart from
        // the values, as in [`crate::dependency`].
        let (Some(writer_facets), Some(cited_facets)) = (writer.facets(), cited.facets()) else {
            return Judged::Skipped(
                "the census parsed no document at one end of this edge, so there is no state to \
                 read there"
                    .to_string(),
            );
        };
        let writer_state = match self.facet.stood(writer_facets) {
            Stood::At(state) => state,
            Stood::Undeclared => return Judged::Skipped(undeclared("source", writer.id)),
            Stood::NotAState(value) => {
                return Judged::Skipped(not_a_state("source", writer.id, value))
            }
        };
        let cited_state = match self.facet.stood(cited_facets) {
            Stood::At(state) => state,
            Stood::Undeclared => return Judged::Skipped(undeclared("target", cited.id)),
            Stood::NotAState(value) => {
                return Judged::Skipped(not_a_state("target", cited.id, value))
            }
        };

        if self.facet.standing(writer_state) != Standing::Live {
            return Judged::Passed;
        }
        if self.facet.standing(cited_state) != Standing::Initial {
            return Judged::Passed;
        }
        // A state the cited document's own regime does not name is a defect
        // of that document, and `lifecycle.state.not_admitted` is reporting it.
        if !self.regime_names(cited.kind, cited_state) {
            return Judged::Skipped(format!(
                "`{}` stands at `{cited_state}`, which the lifecycle regime of a `{}` does not \
                 name, and `{}` reports that",
                cited.id,
                cited.kind,
                crate::lifecycle_state::RULE
            ));
        }

        let (line, column) = at(Some(half.span));
        Judged::Failed(Box::new(Finding {
            rule: self::RULE,
            severity: Severity::Warn,
            obligation: None,
            path: half.source.path.clone(),
            line,
            column,
            message: format!(
                "`{}` stands at `{writer_state}` and writes `{}` to `{}`, which stands at the \
                 initial state `{cited_state}`: nothing has promoted that document, so a live \
                 document does not rest on it",
                writer.id, half.name, cited.id
            ),
            remediation: format!(
                "promote {} once it is settled, point `{}` in {} at a document that stands, or \
                 move {} back to its initial state until the target is settled",
                cited.path, half.name, half.source.path, writer.path
            ),
            // No patch. Which of the three repairs is right is a judgment
            // about both documents that only an author can make.
            patch: None,
        }))
    }
}
