// SPDX-License-Identifier: Apache-2.0
//! A Graph-origin check: no document is the target of two relations that write
//! two different states onto it.
//!
//! # The gap this closes
//!
//! A relation may declare `on_target: {set_state: <state>}`, and an edge of it
//! then puts its target in that state. `headwater generate` reads the setter
//! edges into a document and writes the state onto its page. Where two edges of
//! two relations write two different states, the page can carry only one, and
//! generate takes the first in sorted order so that the answer is the same on
//! every run. Before this rule nothing told the author that the page was one of
//! two answers ([#1086](https://github.com/headwater-ai/headwater/issues/1086),
//! first recorded in [spec 13](../../../../docs/spec/13-open-obligations.md)
//! from #820).
//!
//! # The grain is a neighbourhood, centered on the target
//!
//! The defect is a property of two edges that meet at one document, so no
//! edge-scoped instance can see it: an edge view reads one relation instance
//! and its two ends. The unit is the target document and the edges one step
//! away from it, which is [`NeighbourhoodCheck`] at depth 1. A neighbour's
//! relation is the declared relation type and its end is the end of that
//! declared relation, whichever name the author wrote, so an edge written as
//! `retired_by` in the target itself is read as `retires` into the target, and
//! a document that wrote `retired_by` toward another one is the *source* of
//! that `retires` and is told nothing by it. Generate reads the same direction.
//!
//! # The instance exists only where a clash can
//!
//! One relation writes one state. So a clash needs at least two relations that
//! declare two different states, and a taxonomy with fewer generates no
//! instance of this rule. This repository's own taxonomy declares one setter
//! today (`supersedes`), and the rule costs its corpus nothing.
//!
//! # Two relations that agree are not a clash
//!
//! `supersedes` and a second relation that also writes `superseded` tell one
//! document one thing twice. The rule groups the setter edges by the state they
//! write and reports only where two states remain.
//!
//! # Neighbors this rule does not own
//!
//! [HW-OBL-0196](../../../../docs/obligations/0196-a-relation-writes-a-state-onto-a-kind-that-binds-no-lifecycle-regime-and-nothing-reads-that-pair.md)
//! and [#225](https://github.com/headwater-ai/headwater/issues/225) are about
//! one relation and the kind at its target end: whether that kind admits the
//! state the relation writes. That is a question about two declarations and
//! about no document, and it stays with them. [`crate::dependency`] reads the
//! state a setter wrote to exempt the edge that caused it, and it reads one edge
//! at a time.
//!
//! # The severity is a warning, and the finding carries no patch
//!
//! The page still gets a deterministic state, so nothing downstream is broken
//! in a way an error would have to stop. But the page states one of two answers
//! without saying so, and the author has to choose: remove one edge, change the
//! relation one of the sources used, or change what one relation declares. Only
//! the author knows which, so the remediation is not mechanical and no patch is
//! offered.
//!
//! The finding anchors at the state facet of the target when the target writes
//! one, which is the line a reader is looking at when the page disagrees with
//! it, and at the top of the file otherwise.

use crate::finding::{at, Finding, Severity};
use crate::instance::Outcome;
use crate::lifecycle_state::StateFacet;
use crate::scope::{End, NeighbourhoodCheck, NeighbourhoodView};
use crate::shape::Shape;
use headwater_graph::Declarations;

pub const RULE: &str = "lifecycle.state.set_twice";

/// The check. It carries every relation that writes a state onto its target,
/// with the state it writes, and the name of the state facet to anchor at.
pub struct StateSetTwice<'a> {
    /// `(relation, state)` for every relation that declares
    /// `on_target.set_state`.
    setters: Vec<(&'a str, &'a str)>,
    /// Whether at least two of those states differ. Without that no document
    /// can be told two states, and the rule generates no instance.
    can_clash: bool,
    facet: StateFacet,
}

impl<'a> StateSetTwice<'a> {
    pub fn over(declarations: &'a Declarations, shape: &Shape) -> Self {
        let setters: Vec<(&str, &str)> = declarations
            .relations
            .iter()
            .filter_map(|relation| {
                relation
                    .sets_target_state
                    .as_deref()
                    .map(|state| (relation.name.as_str(), state))
            })
            .collect();
        let can_clash = setters
            .iter()
            .any(|(_, state)| setters.first().is_some_and(|(_, first)| first != state));
        StateSetTwice {
            setters,
            can_clash,
            facet: StateFacet::of(shape),
        }
    }

    fn state_of(&self, relation: &str) -> Option<&'a str> {
        self.setters
            .iter()
            .find(|(name, _)| *name == relation)
            .map(|(_, state)| *state)
    }
}

impl NeighbourhoodCheck for StateSetTwice<'_> {
    const RULE: &'static str = self::RULE;
    /// See [`crate::placement::Placement::VERSION`].
    const VERSION: u32 = 1;

    fn instantiates(&self, _kind: &str) -> bool {
        self.can_clash
    }

    fn evaluate(&self, view: &NeighbourhoodView<'_>) -> Outcome {
        // Every edge into this document that writes a state, as
        // `(state, relation, source)`, sorted so the message is the same on
        // every run.
        let mut set: Vec<(&str, &str, &str)> = view
            .neighbours()
            .iter()
            .filter(|neighbour| neighbour.end == End::Target)
            .filter_map(|neighbour| {
                self.state_of(neighbour.relation)
                    .map(|state| (state, neighbour.relation, neighbour.id))
            })
            .collect();
        set.sort_unstable();
        set.dedup();

        let mut states: Vec<&str> = set.iter().map(|(state, _, _)| *state).collect();
        states.dedup();
        if states.len() < 2 {
            return Outcome::Passed;
        }

        let edges = set
            .iter()
            .map(|(state, relation, source)| format!("`{relation}` from `{source}` sets `{state}`"))
            .collect::<Vec<_>>()
            .join(", ");
        let written = states
            .iter()
            .map(|state| format!("`{state}`"))
            .collect::<Vec<_>>()
            .join(" and ");
        let entry = self
            .facet
            .name
            .as_deref()
            .and_then(|name| view.facets().entry(name));
        let (line, column) = at(entry.map(|entry| entry.key.span));
        Outcome::Failed(vec![Finding {
            rule: self::RULE,
            severity: Severity::Warn,
            obligation: None,
            path: view.path().to_string(),
            line,
            column,
            message: format!(
                "relations tell this document {} states, {written}: {edges}. A generated page \
                 shows only `{}`, the first in sorted order",
                states.len(),
                states[0]
            ),
            remediation: "remove one of the edges, have its source declare a relation that \
                          writes the same state, or change what one relation declares under \
                          `on_target.set_state`"
                .to_string(),
            patch: None,
        }])
    }
}
