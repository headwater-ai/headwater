// SPDX-License-Identifier: Apache-2.0
//! A Graph-origin check: an instance of a relation whose declared
//! `invalid_when` condition holds at both ends.
//!
//! # The gap this closes
//!
//! [Spec 2](../../../../docs/spec/02-taxonomy-model.md) declares
//! `invalid_when: {both: {status: current}}` on `conflicts_with`, and lists
//! "two `current` decisions joined by `conflicts_with`" as an incoherent corpus
//! state that a check reports. The declaration existed and no rule read it:
//! `headwater_resolve` counted the facet as read for the relevance canon, and
//! a corpus with two live, contradictory decisions checked clean
//! ([HW-OBL-0042](../../../../docs/obligations/0042-a-declared-invalid-when-reaches-no-check-and-two-live.md),
//! [#1491](https://github.com/headwater-ai/headwater/issues/1491)).
//!
//! # The condition is read, and nothing is known
//!
//! [`headwater_graph::declarations::Relation::invalid_when`] carries the
//! `(facet, value)` pairs of `invalid_when.both`. The rule forms an instance
//! only for a relation that declares one, so the declaration gates it and the
//! relation's name does not. It reads the facet the condition names, and never
//! the literal `status` or the facet that carries the state role: a taxonomy
//! that names another facet is read unchanged. The condition holds where
//! **every** pair holds at **both** ends. A facet one end does not declare is a
//! condition that does not hold there, and that is a pass.
//!
//! The meta-schema admits `both` alone, with a `gap:` comment that spec 2
//! states no other form. This rule reads `both` alone and widens nothing.
//!
//! # One finding per declared entry
//!
//! The unit is [`EdgeUnit::Pair`], the Q4 triple. `conflicts_with` is
//! `reciprocal: symmetric` and declares no inverse, so `A conflicts_with B`
//! written in A and `B conflicts_with A` written in B are two triples and two
//! instances. Each instance reports on the file that wrote its entry, so a
//! conflict both ends declare gives two findings and a one-sided one gives
//! one. That is deliberate: each author learns of the conflict in their own
//! file, and an instance cannot see the other triple to de-duplicate against
//! it without reading outside the read set that keys it.
//!
//! # The severity is advisory, and the finding carries no patch
//!
//! The remedy is a judgment.
//! [Q18](../../../../docs/spec/09-decisions.md#q18--recording-adjudicated-disagreements)
//! rules that a human settles a live disagreement by writing a decision that
//! `overrides` or `supersedes` one side, and which side loses is not a thing
//! this engine can derive. A finding whose remedy is a rewrite is advisory in
//! this repository's rule, so the severity is a warning.

use crate::finding::{at, Finding, Severity};
use crate::instance::Outcome;
use crate::scope::{EdgeCheck, EdgeUnit, EdgeView};
use headwater_graph::declarations::Relation;
use headwater_graph::Declarations;
use headwater_yaml::Mapping;

pub const RULE: &str = "relation.pair.invalid";

/// The group carried no half at all, which the instantiation never produces.
/// Recorded rather than panicked on, for [`crate::target`]'s reason.
const NO_HALF: &str = "the entry carries no declared half";

/// The far end is not a document. `EdgeUnit::Pair` never groups one, so this is
/// unreachable rather than tolerated.
const NO_PAIR: &str = "this edge has no second document to read";

/// The check. It carries the relations that declare a condition.
pub struct InvalidPair<'a> {
    /// Every relation that declares `invalid_when.both`. Empty for a taxonomy
    /// that declares none, and then this rule generates no instance at all.
    conditioned: Vec<&'a Relation>,
}

impl<'a> InvalidPair<'a> {
    pub fn over(declarations: &'a Declarations) -> Self {
        InvalidPair {
            conditioned: declarations
                .relations
                .iter()
                .filter(|relation| !relation.invalid_when.is_empty())
                .collect(),
        }
    }
}

/// The scalar value one end declares for a facet, and nothing where it
/// declares none or declares a value that is not a scalar.
fn value<'m>(facets: &'m Mapping, facet: &str) -> Option<&'m str> {
    facets
        .get(facet)
        .and_then(|value| value.value.as_scalar())
        .map(|scalar| scalar.text.as_str())
}

/// The condition as an author reads it: "`status: current`", and the pairs
/// joined by "and".
fn condition(relation: &Relation) -> String {
    relation
        .invalid_when
        .iter()
        .map(|(facet, value)| format!("`{facet}: {value}`"))
        .collect::<Vec<_>>()
        .join(" and ")
}

impl EdgeCheck for InvalidPair<'_> {
    const RULE: &'static str = self::RULE;
    /// See [`crate::placement::Placement::VERSION`].
    const VERSION: u32 = 1;
    /// The Q4 pair. See the module comment: both ends have to be documents,
    /// because the rule reads a facet at each of them.
    const UNIT: EdgeUnit = EdgeUnit::Pair;

    fn instantiates(&self, relation: &str) -> bool {
        self.conditioned.iter().any(|known| known.name == relation)
    }

    fn evaluate(&self, view: &EdgeView<'_>) -> Outcome {
        // The declared half when a document wrote it, because a finding anchors
        // at the entry an author is looking at.
        let Some(half) = view.declared_half().or_else(|| view.inverse_half()) else {
            return Outcome::Skipped(NO_HALF.to_string());
        };
        let Some(relation) = self
            .conditioned
            .iter()
            .find(|known| known.name == view.relation())
        else {
            return Outcome::Skipped(NO_PAIR.to_string());
        };
        let Some((source, target)) = view.ends() else {
            return Outcome::Skipped(NO_PAIR.to_string());
        };
        let (Some(source_facets), Some(target_facets)) = (source.facets(), target.facets()) else {
            return Outcome::Skipped(
                "the census parsed no document at one end of this edge, so there is no facet to \
                 read there"
                    .to_string(),
            );
        };

        let holds = relation.invalid_when.iter().all(|(facet, wanted)| {
            value(source_facets, facet) == Some(wanted.as_str())
                && value(target_facets, facet) == Some(wanted.as_str())
        });
        if !holds {
            return Outcome::Passed;
        }

        let condition = condition(relation);
        let (line, column) = at(Some(half.span));
        Outcome::failed_with(Finding {
            rule: self::RULE,
            severity: Severity::Warn,
            obligation: None,
            path: half.source.path.clone(),
            line,
            column,
            message: format!(
                "`{}` declares `{}` to `{}`, and both ends hold {condition}: the taxonomy declares \
                 an instance of this relation invalid when both ends hold that, so the corpus \
                 states two things that cannot both stand",
                source.id, relation.name, target.id
            ),
            remediation: format!(
                "settle it in a decision that `overrides` or `supersedes` one of {} and {}, or \
                 change the facet at one end if the two no longer conflict",
                source.path, target.path
            ),
            // No patch. Which side loses is a judgment only an author can
            // make (Q18).
            patch: None,
        })
    }
}
