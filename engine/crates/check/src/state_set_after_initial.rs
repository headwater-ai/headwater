// SPDX-License-Identifier: Apache-2.0
//! A Graph-origin check: a document that replaces another over an edge that
//! writes a state onto its target, and that still stands at its own initial
//! state (#1973).
//!
//! # The gap this closes
//!
//! [HW-DR-0086](../../../../docs/decisions/0086-a-reciprocal-half-is-owed-once-its-writer-leaves-its-initial-state.md)
//! rules that a source at its initial state retires nothing yet, and
//! [`crate::state_not_set_by_edge`] keeps that clause: it passes the pair. So
//! does `relation.reciprocity.missing` for the lone half. The clause is right,
//! and it left an author with no word at all. A draft that declares
//! `supersedes` at a current document changes nothing, `check --fix` writes
//! nothing, and the target goes on reading as current. In #1881, 16 of the 28
//! failures of the `supersede_or_retire` probe left the successor at `draft`.
//!
//! This rule keeps the clause, and it writes nothing either. It reports what
//! the clause leaves pending, against the document that an author has to move.
//!
//! # What reaches an instance
//!
//! The same instances as [`crate::state_not_set_by_edge`]: every relation that
//! declares `on_target.set_state`, read as an [`EdgeUnit::Pair`] in the
//! direction the relation declares, and only where a facet carries the
//! `state` role. No relation name is written here.
//!
//! # When it reports
//!
//! The source stands at a state whose role is `initial`, and the target is an
//! authored document that stands at a state which is not initial and is not
//! already the state the edge sets. A target at its initial state is the case
//! [HW-DR-0085](../../../../docs/decisions/0085-a-live-document-that-rests-on-a-draft-one-is-reported-over-every-relation-because-a-draft-leaves-no-record-to-cite.md)
//! leaves unreported, because the base regimes give it no movement to the set
//! state. A generated target is this engine's to write.
//!
//! # The state it names
//!
//! The first exit of the source regime's `initial` state, in declaration
//! order, whose role is `live`. In a regime `{draft: [current, deprecated]}`
//! that is `current`, which is the state
//! [HW-DR-0052](../../../../docs/decisions/0052-a-document-is-proposed-at-the-state-it-will-hold-and-the-merge-activates-it.md)
//! says a settled document is proposed at. A regime whose initial state has no
//! live exit gets a message that names no state and says why. A source whose
//! kind binds no regime skips with a reason, because it has no initial state
//! to leave.
//!
//! # Advisory, with no patch
//!
//! The draft may be a draft on purpose, because the replacement is still being
//! argued over. Only its author knows, so the finding is a warning, the owner
//! ruled it advisory on #1973, and it carries no patch.

use crate::finding::{at, Finding, Severity};
use crate::instance::Outcome;
use crate::lifecycle_state::{Standing, StateFacet, Stood};
use crate::scope::{EdgeCheck, EdgeUnit, EdgeView};
use crate::shape::{LifecycleRegime, Shape};
use headwater_graph::declarations::Relation;
use headwater_graph::Declarations;

pub const RULE: &str = "lifecycle.state.set_after_initial";

/// The group carried no half at all, which the instantiation never produces.
const NO_HALF: &str = "the entry carries no declared half";

/// The far end is not a document. `EdgeUnit::Pair` never groups one.
const NO_PAIR: &str = "this edge has no second document to read";

/// The check. It carries the relations that write a state onto their target
/// and the reading of the state facet, all of which are declared.
pub struct SetAfterInitial<'a> {
    /// Every relation that declares `on_target.set_state`.
    setting: Vec<&'a Relation>,
    shape: &'a Shape,
    facet: StateFacet,
}

impl<'a> SetAfterInitial<'a> {
    pub fn over(declarations: &'a Declarations, shape: &'a Shape) -> Self {
        SetAfterInitial {
            setting: declarations
                .relations
                .iter()
                .filter(|relation| relation.sets_target_state.is_some())
                .collect(),
            shape,
            facet: StateFacet::of(shape),
        }
    }
}

/// The first exit of the regime's initial state, in declaration order, whose
/// role is `live`, and nothing where the initial state has no live exit.
pub fn first_settled<'r>(regime: &'r LifecycleRegime, facet: &StateFacet) -> Option<&'r str> {
    regime
        .exits(&regime.initial)
        .iter()
        .map(String::as_str)
        .find(|state| facet.standing(state) == Standing::Live)
}

impl EdgeCheck for SetAfterInitial<'_> {
    const RULE: &'static str = self::RULE;
    /// The first edition. See [`crate::placement::Placement::VERSION`].
    ///
    /// Raise the rule version when a rule's behavior changes.
    const VERSION: u32 = 1;
    /// The Q4 pair: the rule reads a facet at each end.
    const UNIT: EdgeUnit = EdgeUnit::Pair;

    fn instantiates(&self, relation: &str) -> bool {
        self.facet.name.is_some() && self.setting.iter().any(|known| known.name == relation)
    }

    fn evaluate(&self, view: &EdgeView<'_>) -> Outcome {
        let Some(half) = view.declared_half().or_else(|| view.inverse_half()) else {
            return Outcome::Skipped(NO_HALF.to_string());
        };
        let Some(relation) = self
            .setting
            .iter()
            .find(|known| known.name == half.declared)
        else {
            return Outcome::Skipped(NO_PAIR.to_string());
        };
        let Some(set) = relation.sets_target_state.as_deref() else {
            return Outcome::Skipped(NO_PAIR.to_string());
        };
        let Some((source, target)) = view.ends() else {
            return Outcome::Skipped(NO_PAIR.to_string());
        };

        if target.generated() {
            return Outcome::Skipped(format!(
                "`{}` is a generated document, so this engine writes its state, and `{}` and \
                 `generate --check` read it",
                target.id,
                crate::state_set_twice::RULE
            ));
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
            Stood::Undeclared => {
                return Outcome::Skipped(crate::dependency::undeclared("source", source.id))
            }
            Stood::NotAState(value) => {
                return Outcome::Skipped(crate::dependency::not_a_state("source", source.id, value))
            }
        };
        // Only a source at its initial state has anything pending. Every other
        // source is `lifecycle.state.not_set_by_edge`'s.
        if self.facet.standing(source_state) != Standing::Initial {
            return Outcome::Passed;
        }
        let Some(regime) = self.shape.lifecycle_of(source.kind) else {
            return Outcome::Skipped(format!(
                "`{}` is a `{}`, whose kind binds no lifecycle regime, so it has no initial state \
                 to leave",
                source.id, source.kind
            ));
        };
        let target_state = match self.facet.stood(target_facets) {
            Stood::At(state) => state,
            Stood::Undeclared => {
                return Outcome::Skipped(crate::dependency::undeclared("target", target.id))
            }
            Stood::NotAState(value) => {
                return Outcome::Skipped(crate::dependency::not_a_state("target", target.id, value))
            }
        };
        // A draft target is the pair HW-DR-0085 leaves unreported, and a target
        // already at the set state has nothing pending.
        if self.facet.standing(target_state) == Standing::Initial || target_state == set {
            return Outcome::Passed;
        }

        let state_name = self.facet.name.as_deref().unwrap_or_default();
        let (line, column) = at(source_facets.key_span(state_name));
        let advice = match first_settled(regime, &self.facet) {
            Some(settled) => {
                format!("if the replacement is settled, set `{state_name}` to `{settled}`")
            }
            None => format!(
                "no state that `{source_state}` may move to in regime `{}` is live, so no state \
                 to set is named here",
                regime.name
            ),
        };
        Outcome::failed_with(Finding {
            rule: self::RULE,
            severity: Severity::Warn,
            obligation: None,
            path: source.path.to_string(),
            line,
            column,
            message: format!(
                "`{}` declares `{}` to `{}`, and the supersession takes effect only once `{}` \
                 leaves `{source_state}`: `{}` stays at `{target_state}` until then; {advice}",
                source.id, relation.name, target.id, source.id, target.id
            ),
            remediation: format!(
                "in {}, {advice}, and `check --fix` then writes `{set}` onto {} and the far half; \
                 or leave it at `{source_state}` while the replacement is still argued over",
                source.path, target.path
            ),
            // No patch. Whether the draft is settled is a statement only its
            // author can make.
            patch: None,
        })
    }
}

/// What the account of `check --fix` says about the findings of this rule,
/// one line for each, and the line that no finding carries a patch where this
/// run composed nothing and this rule found nothing.
///
/// A source at its initial state retires nothing yet (HW-DR-0086), so no patch
/// writes the target's state or the far half. Before #1973 the account said
/// only that no finding carried a patch, and a caller read that silence as a
/// fix that had nothing left to do. The text is the finding's own, so a
/// terminal and the write tool of the MCP server read the same bytes. It lives
/// here and not in the binary, because `src/main.rs` holds a line ratchet
/// (#1959).
pub fn fix_account(findings: &[Finding], composed_nothing: bool) -> String {
    use std::fmt::Write;
    let mut account = String::new();
    for finding in findings.iter().filter(|finding| finding.rule == RULE) {
        let _ = writeln!(
            account,
            "headwater: deferred {}: the target's state and the far half are written once it \
             leaves its initial state. {}",
            finding.path, finding.message
        );
    }
    if composed_nothing && account.is_empty() {
        account.push_str("headwater: no finding of this run carries a patch\n");
    }
    account
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shape::FacetValue;

    fn facet() -> StateFacet {
        let value = |value: &str, role: &str| FacetValue {
            value: value.to_string(),
            role: Some(role.to_string()),
        };
        StateFacet {
            name: Some("status".to_string()),
            values: vec![
                value("draft", "initial"),
                value("review", "initial"),
                value("current", "live"),
                value("deprecated", "terminal-retained"),
            ],
        }
    }

    fn regime(transitions: Vec<(&str, Vec<&str>)>) -> LifecycleRegime {
        LifecycleRegime {
            name: "standard".to_string(),
            initial: "draft".to_string(),
            transitions: transitions
                .into_iter()
                .map(|(from, to)| {
                    (
                        from.to_string(),
                        to.into_iter().map(str::to_string).collect(),
                    )
                })
                .collect(),
            retain_terminal: None,
            span: Default::default(),
        }
    }

    /// The first live exit in declaration order, past a terminal one and past
    /// another initial one.
    #[test]
    fn the_first_live_exit_is_named() {
        let regime = regime(vec![("draft", vec!["deprecated", "review", "current"])]);
        assert_eq!(first_settled(&regime, &facet()), Some("current"));
    }

    /// An initial state with no live exit names nothing.
    #[test]
    fn no_live_exit_names_nothing() {
        let regime = regime(vec![("draft", vec!["deprecated"])]);
        assert_eq!(first_settled(&regime, &facet()), None);
    }
}
