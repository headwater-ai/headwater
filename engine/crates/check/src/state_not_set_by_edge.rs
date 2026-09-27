// SPDX-License-Identifier: Apache-2.0
//! A Graph-origin check: an authored document that an edge declares retired
//! stands at the state the edge sets.
//!
//! # The gap this closes
//!
//! [Spec 2](../../../../docs/spec/02-taxonomy-model.md) states what a relation
//! that declares `on_target: {set_state: superseded}` does: it "sets the
//! target's state-entry date in the same operation. A state change with no
//! stamp is a defect regardless of what caused the change." The engine wrote
//! that state onto a **generated** target (`headwater_generate::derived`), and
//! [`crate::state_set_twice`] reads generated targets. Nothing read an
//! **authored** one. A decision whose successor declared `supersedes` at it
//! could stay `current` indefinitely, and the decision index went on listing
//! it as current.
//! [#1198](https://github.com/headwater-ai/headwater/issues/1198) is the
//! report, and this corpus held a live instance of it: HW-DR-0078 superseded
//! HW-DR-0061, and HW-DR-0061 still read `current`.
//!
//! # What reaches an instance
//!
//! A relation that declares `on_target.set_state`, and only where a facet
//! carries the `state` role. The unit is [`EdgeUnit::Pair`], so both states
//! are in the read set that keys the instance, as they are for
//! [`crate::dependency`]. The pair is read in the direction the relation
//! declares, whichever document wrote the line: a target that wrote its own
//! `superseded_by` half is the same edge as a successor that wrote
//! `supersedes`.
//!
//! # When the edge has not landed yet
//!
//! A source standing at a state whose role is `initial` retires nothing yet.
//! [HW-DR-0086](../../../../docs/decisions/0086-a-reciprocal-half-is-owed-once-its-writer-leaves-its-initial-state.md)
//! rules the same for a reciprocal half: a draft is still being argued over.
//! A terminal source still counts, because a successor that was itself
//! superseded later still superseded its own target.
//!
//! A target at its initial state is silent too. The base regimes give a
//! `draft` no movement to `superseded`, so the fix would write a transition
//! that `lifecycle.transition.not_permitted` refuses when it lands.
//! [HW-DR-0085](../../../../docs/decisions/0085-a-live-document-that-rests-on-a-draft-one-is-reported-over-every-relation-because-a-draft-leaves-no-record-to-cite.md)
//! names this case for `lifecycle.dependency.on_initial` and leaves it
//! unreported, and this rule leaves it unreported for the same reason. The
//! tutorial `docs/tutorials/your-first-governed-corpus.md` walks a reader
//! through it: a live decision supersedes one that is still a draft.
//!
//! # Which rule owns which case
//!
//! A generated target is this engine's to write, and `generate --check` and
//! [`crate::state_set_twice`] hold it. A target whose state is not a state,
//! or whose own regime does not name it, is [`crate::lifecycle_state`]'s and
//! `facet.value.not_permitted`'s. A target whose kind binds no regime that
//! names the set state is the pair
//! [HW-OBL-0196](../../../../docs/obligations/0196-a-relation-writes-a-state-onto-a-kind-that-binds-no-lifecycle-regime-and-nothing-reads-that-pair.md)
//! records, and this rule does not close it. Each of these is a skip that
//! names the owner.
//!
//! # The severity is an error where the target is live, and the patch is the state with its stamp
//!
//! The remedy is mechanical and total where the target is live: write the
//! state the edge declares, and write the state-entry date beside it. The
//! date is the successor's own `state_entered` value, which is the date the
//! supersession landed. It is read from the corpus and not from a clock, so
//! the fix derives it rather than inventing it, which is what spec 2 requires
//! of `--fix`. Both writes ride in one [`Patch::Facets`], so the state never
//! lands without its stamp.
//!
//! The finding carries no patch where the fix would need a judgment:
//!
//! - the source declares no state-entry date, so there is no date to derive;
//! - the target has no state-entry key to rewrite, or the taxonomy declares
//!   no facet in that role;
//! - the target is not live. A `deprecated` target is terminal already, and
//!   which terminal state it ends at is the author's call.
//!
//! **The last case is advisory, not an error.** Which of two terminal states
//! the target ends at is a judgment, so the remedy is neither mechanical nor
//! total, and a finding that stopped a strict run would stop it on a question
//! only an author can settle. The rule still reports it, at `warn`, with no
//! patch. The first two cases stay errors: the target is live, the state the
//! edge sets is the one correct outcome, and only the date is missing for the
//! engine to write it.
//!
//! One instance reads one pair. Two successors of one target each report it,
//! each with its own stamp. `set_facets` in `headwater_scaffold::fix` applies
//! the first such patch in patch order and skips any later patch that names a
//! facet this run has already written, so one run writes one stamp. The
//! `expect` guard does not decide that case: the skip comes first.

use crate::finding::{at, Finding, Severity};
use crate::instance::Outcome;
use crate::lifecycle_state::{Standing, StateFacet, Stood};
use crate::patch::Patch;
use crate::scope::{EdgeCheck, EdgeUnit, EdgeView};
use crate::shape::Shape;
use headwater_graph::declarations::Relation;
use headwater_graph::Declarations;
use headwater_yaml::Mapping;

pub const RULE: &str = "lifecycle.state.not_set_by_edge";

/// The group carried no half at all, which the instantiation never produces.
const NO_HALF: &str = "the entry carries no declared half";

/// The far end is not a document. `EdgeUnit::Pair` never groups one.
const NO_PAIR: &str = "this edge has no second document to read";

/// The check. It carries the relations that write a state onto their target
/// and the reading of the two facets it needs, all of which are declared.
pub struct NotSetByEdge<'a> {
    /// Every relation that declares `on_target.set_state`.
    setting: Vec<&'a Relation>,
    shape: &'a Shape,
    facet: StateFacet,
    /// The name of the facet in the `state_entered` role, and nothing where
    /// the taxonomy declares none. Then no finding carries a patch.
    entered: Option<String>,
}

impl<'a> NotSetByEdge<'a> {
    pub fn over(declarations: &'a Declarations, shape: &'a Shape) -> Self {
        NotSetByEdge {
            setting: declarations
                .relations
                .iter()
                .filter(|relation| relation.sets_target_state.is_some())
                .collect(),
            shape,
            facet: StateFacet::of(shape),
            entered: shape
                .facet_in_role("state_entered")
                .map(|facet| facet.name.clone()),
        }
    }

    /// The names of the states the regime of a kind names, and nothing for a
    /// kind that binds no regime.
    fn regime_states(&self, kind: &str) -> Option<Vec<&str>> {
        self.shape.lifecycle_of(kind).map(|regime| regime.states())
    }
}

/// A scalar facet as written, and nothing where the key is absent or holds
/// something other than a scalar.
fn scalar<'m>(facets: &'m Mapping, name: &str) -> Option<&'m str> {
    facets
        .entry(name)?
        .value
        .value
        .as_scalar()
        .map(|scalar| scalar.text.as_str())
        .filter(|text| !text.is_empty())
}

impl EdgeCheck for NotSetByEdge<'_> {
    const RULE: &'static str = self::RULE;
    /// See [`crate::placement::Placement::VERSION`].
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
        let target_state = match self.facet.stood(target_facets) {
            Stood::At(state) => state,
            Stood::Undeclared => {
                return Outcome::Skipped(crate::dependency::undeclared("target", target.id))
            }
            Stood::NotAState(value) => {
                return Outcome::Skipped(crate::dependency::not_a_state("target", target.id, value))
            }
        };

        // A successor still at its initial state retires nothing yet.
        if self.facet.standing(source_state) == Standing::Initial {
            return Outcome::Passed;
        }
        // A target still at its initial state has no movement to the set state
        // in the base regimes, so no author can follow the fix. See the module
        // comment: this is the silent case HW-DR-0085 names.
        if self.facet.standing(target_state) == Standing::Initial {
            return Outcome::Passed;
        }
        if target_state == set {
            return Outcome::Passed;
        }
        // The pair HW-OBL-0196 records: the relation writes a state that the
        // target's kind has no machine to hold.
        let states = self.regime_states(target.kind);
        if !states.as_ref().is_some_and(|states| states.contains(&set)) {
            return Outcome::Skipped(format!(
                "`{}` declares `{set}` onto a `{}`, and no lifecycle regime of that kind names \
                 `{set}`. HW-OBL-0196 records that pair, and this rule does not read it",
                relation.name, target.kind
            ));
        }
        if !states
            .as_ref()
            .is_some_and(|states| states.contains(&target_state))
        {
            return Outcome::Skipped(format!(
                "`{}` stands at `{target_state}`, which the lifecycle regime of a `{}` does not \
                 name, and `{}` reports that",
                target.id,
                target.kind,
                crate::lifecycle_state::RULE
            ));
        }

        let state_name = self.facet.name.as_deref().unwrap_or_default();
        let anchor = target_facets.entry(state_name).map(|entry| entry.key.span);
        let (line, column) = at(anchor);

        // The patch, and the reason there is none where there is none.
        let stamp = self
            .entered
            .as_deref()
            .and_then(|name| scalar(source_facets, name));
        let written = self
            .entered
            .as_deref()
            .and_then(|name| scalar(target_facets, name).map(|value| (name, value)));
        // A live target has one correct outcome, the state the edge sets. A
        // target that is terminal already asks which terminal state it ends
        // at, and that is a judgment. See the module comment.
        let severity = match self.facet.standing(target_state) {
            Standing::Live => Severity::Error,
            _ => Severity::Warn,
        };
        let (patch, why_not) = match (self.facet.standing(target_state), stamp, written) {
            (Standing::Live, Some(stamp), Some((entered, was))) => (
                Some(Patch::Facets {
                    path: target.path.to_string(),
                    set: vec![
                        (
                            state_name.to_string(),
                            target_state.to_string(),
                            set.to_string(),
                        ),
                        (entered.to_string(), was.to_string(), stamp.to_string()),
                    ],
                }),
                None,
            ),
            (Standing::Live, None, _) => (
                None,
                Some(format!(
                    "{} declares no {}, so there is no landing date to copy, and a fix does not \
                     invent one",
                    source.path,
                    self.entered.as_deref().unwrap_or("state-entry date")
                )),
            ),
            (Standing::Live, Some(_), None) => (
                None,
                Some(format!(
                    "{} has no {} to rewrite, so write it beside the state",
                    target.path,
                    self.entered.as_deref().unwrap_or("state-entry date")
                )),
            ),
            _ => (
                None,
                Some(format!(
                    "{} is not live, and which terminal state it ends at is the author's call",
                    target.path
                )),
            ),
        };
        let stamp_prose = match stamp {
            Some(stamp) => format!(" and its state-entry date to `{stamp}`"),
            None => String::new(),
        };
        let mut remediation = format!(
            "set the state of {} to `{set}`{stamp_prose}, the date the supersession by `{}` \
             landed; or remove the edge if `{}` does not replace it",
            target.path, source.id, source.id
        );
        if let Some(why_not) = why_not {
            remediation.push_str(&format!(". No fix is offered: {why_not}"));
        }

        Outcome::failed_with(Finding {
            rule: self::RULE,
            severity,
            obligation: None,
            path: target.path.to_string(),
            line,
            column,
            message: format!(
                "`{}` stands at `{target_state}`, and `{}` (at `{source_state}`) declares \
                 `{}` to it, which sets its target's state to `{set}`",
                target.id, source.id, relation.name
            ),
            remediation,
            patch,
        })
    }
}
