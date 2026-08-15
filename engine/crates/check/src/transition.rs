// SPDX-License-Identifier: Apache-2.0
//! The second check that declares `needs_prior`: a state movement the declared
//! lifecycle does not admit.
//!
//! [Spec 3](../../../../docs/spec/03-authoring-and-lifecycle.md#lifecycle)
//! states the rule and the reason a corpus run cannot reach it. "The engine
//! rejects transitions that are not in the declared machine when they land. An
//! illegal transition is only visible against the prior state, and the prior
//! state lives in the diff, not in the graph." That sentence stood with no
//! reader for the whole of the check layer. `regimes.lifecycle` was declared,
//! `taxonomy validate` read it for the soundness of the declaration, and
//! nothing read it against a document.
//!
//! # What the declaration says and what this reads out of it
//!
//! A regime names the state a document opens in and, per state, the states it
//! may reach. A state the `transitions` map does not name reaches nothing, and
//! that is what makes it terminal. So one lookup answers the question: the
//! states `from` admits, and whether `to` is among them.
//!
//! The regime comes from the kind, through the chain that binds it
//! ([`crate::shape::Shape::lifecycle_of`]), the way a voice regime and a
//! language regime already do. A kind that binds none generates no instance,
//! because a kind with no state machine has no illegal movement to make.
//!
//! # A movement is two readings of one facet, and neither may be guessed
//!
//! The facet is the one in the `state` role, so nothing here names `status`.
//! [`crate::lifecycle_state::Stood`] is what one version of a document reads
//! as, and it has three arms for the reason [`crate::change::Prior`] has three:
//! a document that declares no state is not a document at the initial state,
//! and a document whose state the vocabulary does not hold is neither of those.
//! The last one belongs to `facet.value.not_permitted`, which reports it at the
//! line that carries it. A transition rule that took an unknown value for a
//! state would put two findings on one line, and the second would name a
//! machine that never had an edge to offer.
//!
//! There is a fourth reading on the same terms, and it arrived with
//! [#219](https://github.com/headwater-ai/headwater/issues/219). A value the
//! vocabulary holds and *this regime does not name* is not an end of any
//! movement this machine can describe, and `lifecycle.state.not_admitted`
//! reports the document that stands there. So a movement with such an end skips
//! here and names that rule, and one defect stays one finding. See
//! [`crate::lifecycle_state`] for the division in full.
//!
//! Every reading this check declines is a skip with a reason rather than a
//! pass, which is what
//! [spec 4](../../../../docs/spec/04-assurance-model.md#no-silent-passes-every-document-is-accounted-for)
//! asks of an instance that decided nothing.
//!
//! # No movement is not a transition
//!
//! A document the change carries whose state did not move made no transition,
//! and this returns early on that rather than asking the machine for a self
//! edge. A regime that declared one would then not be the thing that decides
//! it. The same reading is why `Prior::Added` and `Prior::Unchanged` pass: a
//! document the change adds stood nowhere before it, and a document the change
//! does not carry stands where it stood.
//!
//! # The severity is an error, and the finding carries no patch
//!
//! A state a document may not be in is a defect of the same class as a facet
//! value the taxonomy does not admit, and that rule reports an error. What is
//! not mechanical is the remedy: a movement refused here is corrected either by
//! putting the state back or by taking the document through the state that
//! joins the two, and only the author knows which
//! ([spec 12](../../../../docs/spec/12-check-layer.md#fixability)). So the
//! finding names the admitted exits and writes nothing.

use crate::change::Prior;
use crate::finding::{at, Finding, Severity};
use crate::instance::Outcome;
use crate::lifecycle_state::{StateFacet, Stood};
use crate::scope::{DocumentCheck, DocumentView};
use crate::shape::Shape;

pub const RULE: &str = "lifecycle.transition.not_permitted";

/// The check. It carries the shape, because the machine it reads is declared
/// rather than known.
pub struct Transition<'a> {
    shape: &'a Shape,
    /// The facet in the `state` role and the values it admits, read by the one
    /// component that owns that reading. A taxonomy that declares no such facet
    /// generates no instance at all.
    facet: StateFacet,
}

impl<'a> Transition<'a> {
    pub fn over(shape: &'a Shape) -> Self {
        Transition {
            shape,
            facet: StateFacet::of(shape),
        }
    }
}

impl DocumentCheck for Transition<'_> {
    const RULE: &'static str = self::RULE;
    /// 2 with the fourth reading below: a movement whose end the regime does
    /// not name now skips and names `lifecycle.state.not_admitted`, where it
    /// used to be refused here as a movement out of a state with no exits.
    const VERSION: u32 = 2;
    const NEEDS_PRIOR: bool = true;

    /// A kind that binds a lifecycle regime. A kind that binds none has no
    /// declared machine, so no movement of one of its documents is illegal and
    /// an instance over it could only ever pass.
    fn instantiates(&self, kind: &str) -> bool {
        self.facet.name.is_some() && self.shape.lifecycle_of(kind).is_some()
    }

    fn evaluate(&self, view: &DocumentView<'_>) -> Outcome {
        // Unreachable: the runner skips an instance with no prior version to
        // bind. A pass rather than a panic, because one row must not silence
        // the rest of the corpus.
        let Some(prior) = view.prior() else {
            return Outcome::Passed;
        };
        let Prior::Committed { facets, .. } = prior else {
            // Added, or not carried by the change. Neither is a movement.
            return Outcome::Passed;
        };
        let Some(regime) = self.shape.lifecycle_of(view.kind()) else {
            return Outcome::Passed;
        };

        let (before, after) = (self.facet.stood(facets), self.facet.stood(view.facets()));
        let (before, after) = match (before, after) {
            (Stood::At(before), Stood::At(after)) => (before, after),
            (Stood::Undeclared, _) | (_, Stood::Undeclared) => {
                return Outcome::Skipped(UNDECLARED.to_string())
            }
            (Stood::NotAState(value), _) | (_, Stood::NotAState(value)) => {
                return Outcome::Skipped(format!(
                    "one version of this document stands at `{value}`, which the state facet does \
                     not admit, and `facet.value.not_permitted` reports that"
                ))
            }
        };
        // The fourth reading: a value of the vocabulary that this regime does
        // not name is not an end of any movement it can describe. The document
        // standing there is what is wrong, and
        // `lifecycle.state.not_admitted` reports it against the version that
        // stands there. Refusing here as well would put two findings on one
        // line, and the exits this rule would offer come from a state the
        // machine never had.
        let named = regime.states();
        if let Some(unnamed) = [before, after].into_iter().find(|state| {
            !named.contains(state)
        }) {
            return Outcome::Skipped(format!(
                "one version of this document stands at `{unnamed}`, which the lifecycle regime \
                 `{}` does not name, and `lifecycle.state.not_admitted` reports that",
                regime.name
            ));
        }
        // Not a transition. A regime that declared a self edge would then not
        // be the thing that decides this, so it is stated here and read from
        // nowhere.
        if before == after {
            return Outcome::Passed;
        }
        if regime.admits(before, after) {
            return Outcome::Passed;
        }

        let name = view
            .facets()
            .get("id")
            .and_then(|node| node.value.as_scalar())
            .map(|scalar| scalar.text.clone())
            .unwrap_or_else(|| view.path().to_string());
        let exits = regime.exits(before);
        let offered = match exits.is_empty() {
            true => format!(
                "`{before}` is terminal in `{}` and admits none",
                regime.name
            ),
            false => format!(
                "`{before}` admits {}",
                exits
                    .iter()
                    .map(|state| format!("`{state}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        };
        let (line, column) = at(self
            .facet
            .name
            .as_deref()
            .and_then(|name| view.facets().key_span(name)));
        Outcome::Failed(vec![Finding {
            rule: self::RULE,
            severity: Severity::Error,
            obligation: None,
            path: view.path().to_string(),
            line,
            column,
            message: format!(
                "{name} moved from `{before}` to `{after}` in this change, and the lifecycle \
                 regime `{}` does not admit that movement: {offered}",
                regime.name
            ),
            remediation: format!(
                "put the state of {} back to `{before}`, or move it through a state that `{}` \
                 joins to `{after}`",
                view.path(),
                regime.name
            ),
            // No patch. Which of the two corrections is right is a statement
            // about the document that only its author can make.
            patch: None,
        }])
    }
}

/// The reason an instance skips when one version of the document writes no
/// state. It is a skip rather than a pass because a movement out of nothing is
/// a reading this rule declines rather than one it approved.
const UNDECLARED: &str =
    "one version of this document declares no value for the state facet, so there are not two \
     states to read a movement between";

#[cfg(test)]
mod tests {
    use super::*;
    use headwater_yaml::Mapping;

    fn shape() -> Shape {
        let source = load(concat!(
            "facets:\n",
            "  status:\n",
            "    role: state\n",
            "    values: [draft, current, superseded]\n",
            "regimes:\n",
            "  lifecycle:\n",
            "    standard:\n",
            "      initial: draft\n",
            "      transitions: {draft: [current], current: [superseded]}\n",
            "kinds:\n",
            "  decision: {lifecycle: standard}\n",
            "  note: {}\n",
        ));
        Shape::read(&source).expect("a shape")
    }

    /// One mapping, from source text.
    fn load(source: &str) -> Mapping {
        headwater_yaml::load(source)
            .expect("the source loads")
            .value
            .as_map()
            .expect("a mapping")
            .clone()
    }

    /// The machine answers out of the declaration, and a state the map does not
    /// name reaches nothing.
    #[test]
    fn a_regime_admits_what_the_declaration_admits() {
        let shape = shape();
        let regime = shape.lifecycle_of("decision").expect("a regime");
        assert!(regime.admits("draft", "current"));
        assert!(!regime.admits("draft", "superseded"));
        assert!(regime.admits("current", "superseded"));
        assert!(regime.exits("superseded").is_empty());
        assert!(!regime.admits("superseded", "current"));
        assert_eq!(regime.initial, "draft");
    }

    /// A kind with no regime generates no instance.
    #[test]
    fn a_kind_that_binds_no_lifecycle_generates_no_instance() {
        let shape = shape();
        let check = Transition::over(&shape);
        assert!(check.instantiates("decision"));
        assert!(!check.instantiates("note"));
    }

    /// The three readings of one version live in [`crate::lifecycle_state`],
    /// which owns them, and this rule reads them through it.
    #[test]
    fn one_version_is_read_by_the_component_that_owns_the_state_facet() {
        let shape = shape();
        let check = Transition::over(&shape);
        assert_eq!(
            check.facet.stood(&load("status: draft\n")),
            Stood::At("draft")
        );
        assert_eq!(check.facet.stood(&load("id: A\n")), Stood::Undeclared);
    }
}
