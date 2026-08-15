// SPDX-License-Identifier: Apache-2.0
//! The state a document stands in, against the states its kind admits.
//!
//! # The gap this closes
//!
//! A state vocabulary is one list for the whole taxonomy, and
//! `facet.value.not_permitted` reads that list and nothing else. So every kind
//! that carries the state facet admitted every value of it, and a `decision` at
//! `discharged` — a state that means "this recorded something the corpus owed
//! and the corpus paid it" — passed a full-corpus run in silence.
//! [#219](https://github.com/headwater-ai/headwater/issues/219) is the report,
//! and the measurement in it is that `superseded` and `deprecated` were admitted
//! on the same terms long before `discharged` existed. The hole is the class and
//! not the value.
//!
//! # A kind already says which states it means, and the meta-schema needed no member
//!
//! The remedy this rule implements is that a kind's admitted states are the
//! states the lifecycle regime it binds names. `regimes.lifecycle.<name>` and
//! `kinds.<name>.lifecycle` are both declared already, so the narrowing is
//! sayable in the language as it stands: a taxonomy declares a second regime
//! over one vocabulary and binds it on the kinds whose documents can reach the
//! extra state. That is the shape `facets.forbid` already has, which is a kind
//! taking away from one taxonomy-wide declaration, and the reading here needs no
//! new member on `kind` and no meta-schema version.
//!
//! What it did need is the change to `lifecycle soundness` in the resolver, which
//! asked every state of the vocabulary to be reachable from the initial state of
//! **each** regime. Two regimes over one vocabulary were not declarable while it
//! did, so the reachability change is not the cheaper alternative to this rule.
//! It is what has to land before this rule has anything to read.
//!
//! # What this owns, and what the transition rule owns
//!
//! This rule owns **the state a document stands in**.
//! [`crate::transition`] owns **the movement between two states the regime
//! names**. A movement whose either end is a value the regime does not name is
//! not a movement that machine can describe, so the transition rule skips it
//! with a reason that names this one — the same deferral it already makes for a
//! value the facet vocabulary does not hold ([`Stood::NotAState`]). One defect
//! is therefore one finding, from whichever rule owns the half that is wrong.
//!
//! The consequence worth stating is the case neither reports: a document that
//! moves *out of* a state its regime does not name, into one it does. The tree
//! after that change is correct, the version before it was reported when it
//! landed, and "put it back" is not the remedy anyone wants.
//!
//! # Document-scoped, and that is the whole point
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#scope--the-declaration-everything-else-rests-on)
//! fixes scope by what one instance is over. A wrong state survives in the
//! document, so the document is the unit, and no prior version is read. That is
//! what reaches the case the transition rule cannot: a document **authored** at
//! a state its kind does not admit has no prior version, makes no movement, and
//! is a defect on the day it lands. It is also what makes the rule run at a
//! commit here at all, because this repository constructs no change manifest
//! ([#178](https://github.com/headwater-ai/headwater/issues/178)) and every
//! `needs_prior` instance is skipped in a full-corpus run.
//!
//! # The severity is an error, and the finding carries no patch
//!
//! The same reading `facet.value.not_permitted` takes. A state a document may
//! not stand in is a defect the taxonomy decides in full, so the report is an
//! error; which of the admitted states is the right one is a statement about the
//! document that only its author can make, so nothing is written
//! ([spec 12](../../../../docs/spec/12-check-layer.md#fixability)).

use crate::finding::{at, Finding, Severity};
use crate::instance::Outcome;
use crate::scope::{DocumentCheck, DocumentView};
use crate::shape::Shape;
use headwater_yaml::Mapping;

pub const RULE: &str = "lifecycle.state.not_admitted";

/// The state one version of a document stood in.
///
/// Three arms, and each one is a different fact. Folding the last two together
/// would make a document that declares no state and a document whose state
/// nothing declares one value, and they are answered by two different rules.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Stood<'a> {
    /// A state the vocabulary holds.
    At(&'a str),
    /// The document writes no value for the state facet. `facet.required.missing`
    /// is the rule that reports it where the facet is required.
    Undeclared,
    /// The document writes a value the state facet does not admit.
    /// `facet.value.not_permitted` is the rule that reports it.
    NotAState(&'a str),
}

/// The facet in the `state` role, resolved once, with the values it admits.
///
/// One reader for two rules. The alternative was a second copy of the three-way
/// reading below inside [`crate::transition`], and a copy of an invariant is
/// what a test suite cannot see.
pub(crate) struct StateFacet {
    /// The name of the facet in the `state` role. A taxonomy that declares none
    /// generates no instance of either rule that reads this.
    pub(crate) name: Option<String>,
    /// The values that facet admits, so a value outside them reads as
    /// [`Stood::NotAState`] rather than as a state.
    pub(crate) values: Vec<String>,
}

impl StateFacet {
    pub(crate) fn of(shape: &Shape) -> Self {
        let facet = shape.facet_in_role("state");
        StateFacet {
            name: facet.map(|facet| facet.name.clone()),
            values: facet.map(|facet| facet.values.clone()).unwrap_or_default(),
        }
    }

    /// What one version of a document reads as.
    pub(crate) fn stood<'b>(&self, facets: &'b Mapping) -> Stood<'b> {
        let Some(name) = self.name.as_deref() else {
            return Stood::Undeclared;
        };
        let Some(entry) = facets.entry(name) else {
            return Stood::Undeclared;
        };
        // A value this engine cannot read as a scalar is the meta-schema's
        // business, and it is not a state either way.
        let Some(scalar) = entry.value.value.as_scalar() else {
            return Stood::Undeclared;
        };
        match self.values.iter().any(|value| value == &scalar.text) {
            true => Stood::At(&scalar.text),
            false => Stood::NotAState(&scalar.text),
        }
    }
}

/// The check. It carries the shape, because the states it reads are declared
/// rather than known.
pub struct StateAdmitted<'a> {
    shape: &'a Shape,
    facet: StateFacet,
}

impl<'a> StateAdmitted<'a> {
    pub fn over(shape: &'a Shape) -> Self {
        StateAdmitted {
            shape,
            facet: StateFacet::of(shape),
        }
    }
}

impl DocumentCheck for StateAdmitted<'_> {
    const RULE: &'static str = self::RULE;
    /// See [`crate::placement::Placement::VERSION`].
    const VERSION: u32 = 1;

    /// A kind that binds a lifecycle regime. A kind that binds none declares no
    /// admitted set, which is an absence rather than a machine that admits
    /// everything, and an instance over it could only ever pass. The same
    /// reading [`crate::transition`] takes, and it is why the two rules
    /// instantiate over exactly one set of kinds.
    fn instantiates(&self, kind: &str) -> bool {
        self.facet.name.is_some() && self.shape.lifecycle_of(kind).is_some()
    }

    fn evaluate(&self, view: &DocumentView<'_>) -> Outcome {
        let Some(regime) = self.shape.lifecycle_of(view.kind()) else {
            return Outcome::Passed;
        };
        let standing = match self.facet.stood(view.facets()) {
            Stood::At(state) => state,
            Stood::Undeclared => return Outcome::Skipped(UNDECLARED.to_string()),
            Stood::NotAState(value) => {
                return Outcome::Skipped(format!(
                    "this document stands at `{value}`, which the state facet does not admit, and \
                     `facet.value.not_permitted` reports that"
                ))
            }
        };
        let admitted = regime.states();
        // A regime that names no state is a declaration `lifecycle soundness`
        // refuses in the resolver. Reporting it here would put one taxonomy
        // defect on every document of every kind that binds the regime, and
        // the remediation would name an empty set.
        if admitted.is_empty() {
            return Outcome::Skipped(format!(
                "the lifecycle regime `{}` names no state, and `taxonomy validate` reports that \
                 against the declaration",
                regime.name
            ));
        }
        if admitted.contains(&standing) {
            return Outcome::Passed;
        }

        let offers = admitted
            .iter()
            .map(|state| format!("`{state}`"))
            .collect::<Vec<_>>()
            .join(", ");
        let facet = self.facet.name.as_deref().unwrap_or_default();
        let (line, column) = at(view.facets().key_span(facet));
        Outcome::Failed(vec![Finding {
            rule: self::RULE,
            severity: Severity::Error,
            obligation: None,
            path: view.path().to_string(),
            line,
            column,
            message: format!(
                "a `{}` stands at `{standing}`, and the lifecycle regime `{}` that this kind binds \
                 names no such state: a `{}` stands at one of {offers}",
                view.kind(),
                regime.name,
                view.kind()
            ),
            remediation: format!(
                "change `{facet}` in {} to one of: {}, or bind `{}` to a lifecycle regime that \
                 names `{standing}`",
                view.path(),
                admitted.join(", "),
                view.kind()
            ),
            // No patch. Which admitted state the document belongs in is a
            // statement about the document that only its author can make.
            patch: None,
        }])
    }
}

/// The reason an instance skips when the document writes no state. It is a skip
/// rather than a pass because a document with no state is a reading this rule
/// declines rather than one it approved, and `facet.required.missing` is the
/// rule that reports the absence where the facet is required.
const UNDECLARED: &str =
    "this document declares no value for the state facet, so there is no state to hold against \
     the ones its kind admits";

#[cfg(test)]
mod tests {
    use super::*;

    /// Two regimes over one vocabulary, which is the declaration this rule was
    /// written to read. `narrow` names four of the five states and `wide` names
    /// all five.
    fn shape() -> Shape {
        let source = load(concat!(
            "facets:\n",
            "  status:\n",
            "    role: state\n",
            "    values: [draft, current, superseded, deprecated, discharged]\n",
            "regimes:\n",
            "  lifecycle:\n",
            "    narrow:\n",
            "      initial: draft\n",
            "      transitions: {draft: [current, deprecated], current: [superseded, deprecated]}\n",
            "    wide:\n",
            "      initial: draft\n",
            "      transitions: {draft: [current], current: [discharged]}\n",
            "    empty:\n",
            "      transitions: {}\n",
            "kinds:\n",
            "  decision: {lifecycle: narrow}\n",
            "  obligation_record: {lifecycle: wide}\n",
            "  broken: {lifecycle: empty}\n",
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

    /// The admitted set is what the regime names, in declaration order, and two
    /// regimes over one vocabulary name two different sets.
    #[test]
    fn a_regime_names_the_states_its_kind_admits() {
        let shape = shape();
        let narrow = shape.lifecycle_of("decision").expect("a regime");
        assert_eq!(
            narrow.states(),
            vec!["draft", "current", "deprecated", "superseded"]
        );
        assert!(!narrow.states().contains(&"discharged"));
        let wide = shape.lifecycle_of("obligation_record").expect("a regime");
        assert_eq!(wide.states(), vec!["draft", "current", "discharged"]);
    }

    /// A regime that declares nothing names nothing, and the empty string that
    /// an absent `initial` reads as is not a state.
    #[test]
    fn a_regime_that_declares_no_state_names_none() {
        let shape = shape();
        let empty = shape.lifecycle_of("broken").expect("a regime");
        assert!(empty.states().is_empty());
    }

    /// A kind with no regime generates no instance, and a kind with one does.
    #[test]
    fn a_kind_that_binds_no_lifecycle_generates_no_instance() {
        let shape = shape();
        let check = StateAdmitted::over(&shape);
        assert!(check.instantiates("decision"));
        assert!(check.instantiates("obligation_record"));
        assert!(!check.instantiates("note"));
    }

    /// The three readings of a document's state, kept apart.
    #[test]
    fn a_state_a_document_does_not_declare_is_not_the_state_it_declares_wrongly() {
        let facet = StateFacet::of(&shape());
        assert_eq!(facet.stood(&load("status: draft\n")), Stood::At("draft"));
        assert_eq!(facet.stood(&load("id: A\n")), Stood::Undeclared);
        assert_eq!(
            facet.stood(&load("status: retired\n")),
            Stood::NotAState("retired")
        );
    }
}
