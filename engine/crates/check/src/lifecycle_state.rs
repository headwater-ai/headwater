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
use crate::shape::{FacetValue, Shape};
use headwater_yaml::Mapping;

pub const RULE: &str = "lifecycle.state.not_admitted";

/// The state one version of a document stood in.
///
/// Three arms, and each one is a different fact. Folding the last two together
/// would make a document that declares no state and a document whose state
/// nothing declares one value, and they are answered by two different rules.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stood<'a> {
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
/// One reader for two rules, and now for a reader outside this crate as well:
/// `headwater-generate` asks the same question of a transcript. The alternative
/// was a second copy of the reading below inside [`crate::transition`], and a
/// copy of an invariant is what a test suite cannot see. It is public for that
/// reason and for no other, so the fold stays in one place as the taxonomy
/// moves.
pub struct StateFacet {
    /// The name of the facet in the `state` role. A taxonomy that declares none
    /// generates no instance of either rule that reads this.
    pub(crate) name: Option<String>,
    /// The values that facet admits, with the role the vocabulary gives each
    /// one. A value outside them reads as [`Stood::NotAState`] rather than as a
    /// state, and the role is what [`StateFacet::standing`] folds.
    pub(crate) values: Vec<FacetValue>,
}

/// What spec 3 asks of a state, which is neither the value nor the machine.
///
/// [Spec 3](../../../../docs/spec/03-authoring-and-lifecycle.md#lifecycle):
/// "a live document may not depend on a terminal one". The sentence needs the
/// five values of this repository's vocabulary folded into two answers, and
/// the fold is a reading of the `role` beside each value rather than a list of
/// state names in this engine. A taxonomy that renames every state keeps the
/// roles, and this reading moves with it — which is spec 2's worked overlay
/// holding at the check layer.
///
/// Four arms, and the last two are what keeps the rule honest. `draft` carries
/// the role `initial`: a document being argued over is not live, so nothing it
/// points at is spec 3's finding, and it is not terminal either. A role this
/// engine does not know is the fourth, and deciding nothing about it is the
/// same posture the rest of the check layer takes toward a value the
/// meta-schema owns.
///
/// **`Initial` and `Neither` are two facts, and a reader outside this crate
/// needs both.** Spec 3's rule reads each of them as "not live", so the rules
/// here could fold the pair. `headwater-generate` cannot: it decides whether a
/// refused recording fails a run, and its answers for the two are opposite. A
/// document at the initial state is one somebody is still working on, and a
/// document whose state this engine cannot read is one that would otherwise
/// escape the decision. One arm for both would hand that caller a single answer
/// for two questions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Standing {
    /// `role: live`. A reader may rely on this document.
    Live,
    /// A `terminal-` role. The document is kept as a record and nothing new
    /// may rest on it.
    Terminal,
    /// `role: initial`. Nothing has promoted this document, so nothing relies
    /// on it and nothing is kept by it.
    Initial,
    /// None of the three, including a state whose value declares no role at
    /// all.
    Neither,
}

/// The role that names a state a reader may rely on.
const LIVE: &str = "live";

/// The role that names the state a document is authored at, before anything
/// promotes it.
const INITIAL: &str = "initial";

impl StateFacet {
    pub fn of(shape: &Shape) -> Self {
        let facet = shape.facet_in_role("state");
        StateFacet {
            name: facet.map(|facet| facet.name.clone()),
            values: facet.map(|facet| facet.values.clone()).unwrap_or_default(),
        }
    }

    /// What one state is, in the two terms spec 3 states its rule in.
    ///
    /// The one reader of the role, so that no rule holds a second list of
    /// which states end a lifecycle. `terminal` is [`headwater_resolve::core::role_is_terminal`],
    /// which is the same predicate `lifecycle soundness` asks of a declaration,
    /// so a taxonomy cannot be sound under one reading and checked under
    /// another.
    ///
    /// **A regime is available at the one call site that reads this, and the
    /// reading declines to take it.** [`crate::dependency`] holds the target's
    /// kind and calls `Shape::lifecycle_of` on it eleven lines below, so a
    /// per-regime reading is one line away. That reading was built and
    /// measured, and it costs the deferral. A target standing at a state its
    /// own regime never names reads as not terminal, the instance passes, and
    /// the account naming `lifecycle.state.not_admitted` as the rule that owns
    /// the defect is gone. Three facts fold into two, and the one that goes is
    /// the decline to judge.
    /// [Q26](../../../../docs/decisions/0026-q26-whether-terminality-belongs-to-a-state-or-to-a-state-and-a-regime.md)
    /// is the ruling, and it carries the survey of published traditions behind
    /// it.
    pub fn standing(&self, state: &str) -> Standing {
        match self
            .values
            .iter()
            .find(|held| held.value == state)
            .and_then(|held| held.role.as_deref())
        {
            Some(role) if headwater_resolve::core::role_is_terminal(role) => Standing::Terminal,
            Some(LIVE) => Standing::Live,
            Some(INITIAL) => Standing::Initial,
            _ => Standing::Neither,
        }
    }

    /// What one version of a document reads as.
    pub fn stood<'b>(&self, facets: &'b Mapping) -> Stood<'b> {
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
        match self.values.iter().any(|held| held.value == scalar.text) {
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
            "    values:\n",
            "      - {value: draft, role: initial}\n",
            "      - {value: current, role: live}\n",
            "      - {value: superseded, role: terminal-retained}\n",
            "      - {value: deprecated, role: terminal-retained}\n",
            "      - {value: discharged, role: terminal-retained}\n",
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

    /// The fold spec 3's rule needs, and the two arms that are neither live nor
    /// terminal. They are separate because `headwater-generate` releases a
    /// refused recording at the initial state and holds one whose role it
    /// cannot read.
    #[test]
    fn a_role_says_whether_a_state_is_live_terminal_or_the_one_a_document_opens_at() {
        let facet = StateFacet::of(&shape());
        assert_eq!(facet.standing("current"), Standing::Live);
        assert_eq!(facet.standing("superseded"), Standing::Terminal);
        assert_eq!(facet.standing("deprecated"), Standing::Terminal);
        assert_eq!(facet.standing("discharged"), Standing::Terminal);
        assert_eq!(facet.standing("draft"), Standing::Initial);
        // A value the facet does not admit has no role, and a value with no
        // role decides nothing either.
        assert_eq!(facet.standing("retired"), Standing::Neither);
    }

    /// A taxonomy may rename every state, and the roles are what survives. This
    /// is spec 2's worked overlay, read at the check layer.
    #[test]
    fn a_renamed_vocabulary_that_keeps_its_roles_reads_the_same() {
        let source = load(concat!(
            "facets:\n",
            "  phase:\n",
            "    role: state\n",
            "    values:\n",
            "      - {value: opened, role: initial}\n",
            "      - {value: ratified, role: live}\n",
            "      - {value: withdrawn, role: terminal-retained}\n",
        ));
        let facet = StateFacet::of(&Shape::read(&source).expect("a shape"));
        assert_eq!(facet.standing("ratified"), Standing::Live);
        assert_eq!(facet.standing("withdrawn"), Standing::Terminal);
        assert_eq!(facet.standing("opened"), Standing::Initial);
        assert_eq!(facet.standing("current"), Standing::Neither);
    }

    /// The two readings of "terminal", over the taxonomy this repository
    /// resolves.
    ///
    /// The role on a state value is what [`StateFacet::standing`] reads, and
    /// [`crate::dependency`] is the rule that reads it. The machine is what
    /// [`crate::shape::LifecycleRegime::terminal`] reads, and
    /// [`crate::retention`] and [`crate::transition`] are the rules that read
    /// that one. `lifecycle soundness` refuses a regime where the two differ
    /// over a state it reaches, so this holds for every taxonomy the resolver
    /// accepts and not for this corpus alone.
    ///
    /// It re-resolves rather than reading the committed lock, because a lock is
    /// an output of the rule this assertion rests on.
    #[test]
    fn the_role_and_the_machine_name_one_terminal_set_over_this_repository() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let repository = headwater_resolve::repository(&root).expect("this repository resolves");
        let shape = Shape::read(&repository.resolution.taxonomy).expect("a shape");
        let facet = StateFacet::of(&shape);

        let mut read: Vec<(&str, &str, bool, bool)> = Vec::new();
        for regime in &shape.lifecycle {
            for state in regime.states() {
                read.push((
                    regime.name.as_str(),
                    state,
                    regime.terminal(state),
                    facet.standing(state) == Standing::Terminal,
                ));
            }
        }

        let split: Vec<String> = read
            .iter()
            .filter(|(_, _, machine, role)| machine != role)
            .map(|(regime, state, machine, role)| {
                format!(
                    "`{regime}` reaches `{state}`: the machine says {machine} and the role \
                     says {role}"
                )
            })
            .collect();
        assert!(
            split.is_empty(),
            "two readings of terminal disagree over this repository:\n  {}",
            split.join("\n  ")
        );

        // The denominator, and both arms. The same assertion over a shape that
        // read no regime is silent, and one where every state answered the same
        // way would be silent for a second reason.
        assert_eq!(read.len(), 9, "the readings ran over {read:?}");
        assert!(read.iter().any(|(_, _, machine, _)| *machine));
        assert!(read.iter().any(|(_, _, machine, _)| !*machine));
    }

    /// The disagreement, built by hand, and what refuses it.
    ///
    /// This layer reads a lock and re-runs no resolver rule over one, so a
    /// shape built from source still holds two readings that differ. That is
    /// what `lifecycle soundness` buys and what nothing here would catch:
    /// `leaves` is terminal to the rule that reads the role and not to the one
    /// that reads the machine, and `sealed` is that the other way round.
    #[test]
    fn a_shape_built_by_hand_still_holds_two_readings_that_differ() {
        let source = load(concat!(
            "facets:\n",
            "  status:\n",
            "    role: state\n",
            "    values:\n",
            "      - {value: draft, role: initial}\n",
            "      - {value: current, role: live}\n",
            "      - {value: leaves, role: terminal-retained}\n",
            "      - {value: sealed}\n",
            "regimes:\n",
            "  lifecycle:\n",
            "    standard:\n",
            "      initial: draft\n",
            "      transitions: {draft: [current], current: [leaves, sealed], leaves: [sealed]}\n",
            "kinds:\n",
            "  decision: {lifecycle: standard}\n",
        ));
        let shape = Shape::read(&source).expect("a shape");
        let facet = StateFacet::of(&shape);
        let regime = shape.lifecycle_of("decision").expect("a regime");

        // The role names it terminal and the machine gives it an exit.
        assert_eq!(facet.standing("leaves"), Standing::Terminal);
        assert!(!regime.terminal("leaves"));

        // The machine gives it no exit and the value carries no role.
        assert!(regime.terminal("sealed"));
        assert_eq!(facet.standing("sealed"), Standing::Neither);
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
