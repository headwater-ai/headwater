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
//! **A list holds when it contains the value.** An end that writes
//! `status: [current]` holds `status: current`, and an end that writes
//! `[draft, superseded]` does not. A mapping holds nothing. The other reading,
//! that a list never holds, was measured and refused (#1542): no other rule
//! reports a list-valued state facet, so two live decisions joined by
//! `conflicts_with` that wrote their status as a list checked clean, which is
//! the state HW-OBL-0042 discharged.
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
//! A relation that declares an inverse groups both halves into one instance.
//! That instance reports once, on the declared half where the source end wrote
//! one, and on the inverse half where only the target end did.
//!
//! # The severity is advisory, and the finding carries no patch
//!
//! The remedy is a judgment.
//! [Q18](../../../../docs/spec/09-decisions.md#q18--recording-adjudicated-disagreements)
//! rules that a human settles a live disagreement by writing a decision that
//! `overrides` or `supersedes` one side, and which side loses is not a thing
//! this engine can derive. A finding whose remedy is a rewrite is advisory in
//! this repository's rule, so the severity is a warning.
//!
//! The remediation names only the edits that clear the finding and end there.
//! One is an end moved off the condition. The other is the entry removed, where
//! the two documents no longer conflict. It does not name `overrides`. Under
//! Q18 an overridden decision stays `current`, so the condition still holds and
//! the finding stays, and a taxonomy can declare no `overrides` relation at all.
//!
//! **Where the condition names the facet in the `state` role, the remedy moves
//! that facet by supersession alone.** The facet is read by role, through
//! [`StateFacet`], and not as the literal `status`. Spec 2's condition is
//! `status: current`, so the states left to move to are a terminal one and the
//! initial one. The initial one is not an exit: a live document joined to a
//! document at the initial state is `lifecycle.dependency.on_initial`'s
//! finding, and that rule's remedy is to promote the draft, which restores this
//! finding (#1542). So the remedy names no changed state facet and no initial
//! value. A facet of the condition without the state role is still named as a
//! facet to change at one end.

use crate::finding::{at, Finding, Severity};
use crate::instance::Outcome;
use crate::lifecycle_state::StateFacet;
use crate::scope::{EdgeCheck, EdgeUnit, EdgeView};
use crate::shape::Shape;
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
    /// The name of the facet in the `state` role, read by role and never as
    /// the literal `status`. It decides the remedy and never the verdict.
    state: Option<String>,
}

impl<'a> InvalidPair<'a> {
    pub fn over(declarations: &'a Declarations, shape: &Shape) -> Self {
        InvalidPair {
            conditioned: declarations
                .relations
                .iter()
                .filter(|relation| !relation.invalid_when.is_empty())
                .collect(),
            state: StateFacet::of(shape).name,
        }
    }

    /// The edits that clear the finding and end there. See the module comment
    /// for why a changed state facet is not one of them.
    fn remedy(
        &self,
        relation: &Relation,
        condition: &str,
        (source, target): (&str, &str),
        entry: &str,
    ) -> String {
        let stated = relation
            .invalid_when
            .iter()
            .any(|(facet, _)| Some(facet.as_str()) == self.state.as_deref());
        let others = relation
            .invalid_when
            .iter()
            .filter(|(facet, _)| Some(facet.as_str()) != self.state.as_deref())
            .map(|(facet, _)| format!("`{facet}`"))
            .collect::<Vec<_>>()
            .join(" and ");
        let moves = match (stated, others.is_empty()) {
            (true, true) => "supersede it".to_string(),
            (true, false) => format!("supersede it, or change {others} at one end"),
            (false, _) => {
                format!("change {others} at one end, or supersede it where that changes {others}")
            }
        };
        format!(
            "decide which of {source} and {target} stands and move the other off {condition}: \
             {moves}. If the two no longer conflict, remove the `{}` entry from {entry}",
            relation.name
        )
    }
}

/// Whether one end holds `wanted` for a facet. A scalar holds when its text is
/// `wanted`, and a list holds when one of its scalar items is. An absent facet
/// and a mapping do not hold. See the module comment for why a list is read.
fn holds(facets: &Mapping, facet: &str, wanted: &str) -> bool {
    let Some(value) = facets.get(facet) else {
        return false;
    };
    if let Some(scalar) = value.value.as_scalar() {
        return scalar.text == wanted;
    }
    value.value.as_seq().is_some_and(|items| {
        items
            .iter()
            .filter_map(|item| item.value.as_scalar())
            .any(|scalar| scalar.text == wanted)
    })
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
    const VERSION: u32 = 2;
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

        let held = relation.invalid_when.iter().all(|(facet, wanted)| {
            holds(source_facets, facet, wanted) && holds(target_facets, facet, wanted)
        });
        if !held {
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
            remediation: self.remedy(
                relation,
                &condition,
                (&source.path, &target.path),
                &half.source.path,
            ),
            // No patch. Which side loses is a judgment only an author can
            // make (Q18).
            patch: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A taxonomy whose state facet is `phase`, not `status`, so a remedy that
    /// knew the state facet by its name in spec 2 reads it as an ordinary facet
    /// and offers to change it.
    const RENAMED: &str = "\
facets:
  phase:
    role: state
    values:
      - {value: proposed, role: initial}
      - {value: adopted,  role: live}
      - {value: replaced, role: terminal-retained}
relations:
  conflicts_with:
    family: association
    from: [decision]
    to:   [decision]
    reciprocal: symmetric
    invalid_when: {both: {phase: adopted}}
    created_by: author
";

    #[test]
    fn the_state_facet_is_read_by_its_role_and_not_its_name() {
        let root = headwater_yaml::load(RENAMED).expect("the source loads");
        let root = root.value.as_map().expect("a mapping");
        let shape = Shape::read(root).expect("the shape reads");
        let declarations = Declarations::read(root).expect("the declarations read");
        let rule = InvalidPair::over(&declarations, &shape);
        let relation = rule.conditioned[0];
        let remedy = rule.remedy(relation, &condition(relation), ("a.md", "b.md"), "a.md");
        assert!(
            remedy.contains("off `phase: adopted`: supersede it. "),
            "{remedy}"
        );
        assert!(!remedy.contains("change `phase`"), "{remedy}");
        assert!(!remedy.contains("proposed"), "{remedy}");
        assert!(
            remedy.contains("remove the `conflicts_with` entry from a.md"),
            "{remedy}"
        );
    }
}
