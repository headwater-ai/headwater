// SPDX-License-Identifier: Apache-2.0
//! A Graph-origin check: `headwater generate` does not write one of two states
//! onto a document without saying so.
//!
//! # The gap this closes
//!
//! A relation may declare `on_target: {set_state: <state>}`, and an edge of it
//! then puts its target in that state. For a **generated** document, whose
//! facets no author writes, `headwater generate` reads the setter edges into it
//! and writes the state onto the page (`derived::set_state` in the generate
//! crate). Where two edges of two relations write two different states, the page
//! can carry only one, and generate takes the first in sorted order so that the
//! answer is the same on every run. Before this rule nothing told the author
//! that the page states one of two answers
//! ([#1086](https://github.com/headwater-ai/headwater/issues/1086), first
//! recorded in [spec 13](../../../../docs/spec/13-open-obligations.md) from
//! #820).
//!
//! # The documents it reads, and the edges
//!
//! The rule decides exactly the documents generate writes a state onto, from
//! exactly the edges generate reads, so that every finding is a statement about
//! a page that exists.
//!
//! - **The documents** are the rows the census classified as generated and
//!   resolved a kind for, where that kind requires the facet in the `state`
//!   role. Generate answers a required facet and no other. An authored document
//!   is not one of them: its author writes its state, and
//!   [`crate::lifecycle_state`] and [`crate::dependency`] read that value.
//! - **The edges** are the edges whose written target is that document's
//!   identifier, that were written under the declared name of their relation,
//!   and that the document did not write itself. That is `derived::incoming`
//!   clause by clause, and it reads the raw target for generate's reason: the
//!   answer does not depend on whether the index resolved the name. An edge written from the inverse name points the other way, so the
//!   state it sets is not this document's. An edge the output wrote would make
//!   the page a function of its own last version.
//!
//! # The grain is the corpus, and why not a neighbourhood
//!
//! A neighbourhood instance centers on a typed document, and a generated one is
//! not typed, so that grain never reaches the one document this rule is about.
//! A neighbour also carries no direction, and the direction is half of what
//! generate reads. The corpus view carries the edges and the generated rows
//! when a rule declares [`crate::scope::CorpusCheck::NEEDS_GRAPH`], and every
//! document either is derived from is already in its read set.
//!
//! # The instance exists only where a clash can
//!
//! One relation writes one state. So a clash needs at least two relations that
//! declare two different states, and a taxonomy with fewer runs no instance of
//! this rule: [`StateSetTwice::can_clash`] is the gate, and the runner reads it
//! before it builds the one corpus instance. This is the one corpus-scoped rule
//! with a gate, and the gate is a declaration rather than a property of any
//! corpus, which is the same generation step every other grain takes. This
//! repository's own taxonomy declares one setter today (`supersedes`), so the
//! rule costs its corpus nothing.
//!
//! # Neighbors this rule does not own
//!
//! [HW-OBL-0196](../../../../docs/obligations/0196-a-relation-writes-a-state-onto-a-kind-that-binds-no-lifecycle-regime-and-nothing-reads-that-pair.md)
//! and [#225](https://github.com/headwater-ai/headwater/issues/225) are about
//! one relation and the kind at its target end: whether that kind admits the
//! state the relation writes. That is a question about two declarations and
//! about no document, and it stays with them. Generate refuses such a state on
//! its own.
//!
//! # The severity is a warning, and the finding carries no patch
//!
//! The page still gets a deterministic state, so nothing downstream is broken in
//! a way an error would have to stop. But the page states one of two answers
//! without saying so, and the author has to choose: remove one edge, change the
//! relation one of the sources used, or change what one relation declares. Only
//! the author knows which, so the remediation is not mechanical and no patch is
//! offered. The finding anchors at the state facet the last run of generate
//! wrote, which is the line that states the chosen answer.

use crate::finding::{at, Finding, Severity};
use crate::instance::Outcome;
use crate::scope::{CorpusCheck, CorpusView};
use crate::shape::Shape;
use headwater_graph::{Declarations, Direction};

pub const RULE: &str = "lifecycle.state.set_twice";

/// The view carried no graph, which the runner never builds for this rule.
const NO_GRAPH: &str = "the view carries no edges and no generated documents";

/// The check. It carries every relation that writes a state onto its target,
/// with the state it writes, and the name of the facet in the `state` role.
pub struct StateSetTwice<'a> {
    /// `(relation, state)` for every relation that declares
    /// `on_target.set_state`.
    setters: Vec<(&'a str, &'a str)>,
    shape: &'a Shape,
    state_facet: Option<String>,
}

impl<'a> StateSetTwice<'a> {
    pub fn over(declarations: &'a Declarations, shape: &'a Shape) -> Self {
        StateSetTwice {
            setters: declarations
                .relations
                .iter()
                .filter_map(|relation| {
                    relation
                        .sets_target_state
                        .as_deref()
                        .map(|state| (relation.name.as_str(), state))
                })
                .collect(),
            shape,
            state_facet: shape
                .facet_in_role("state")
                .map(|facet| facet.name.clone()),
        }
    }

    /// Whether any document can be told two states: at least two relations
    /// declare `on_target.set_state` and not all of them name one state, and
    /// the taxonomy has a facet in the `state` role for generate to write.
    pub fn can_clash(&self) -> bool {
        self.state_facet.is_some()
            && self
                .setters
                .iter()
                .any(|(_, state)| self.setters.first().is_some_and(|(_, first)| first != state))
    }

    fn state_of(&self, relation: &str) -> Option<&'a str> {
        self.setters
            .iter()
            .find(|(name, _)| *name == relation)
            .map(|(_, state)| *state)
    }
}

impl CorpusCheck for StateSetTwice<'_> {
    const RULE: &'static str = self::RULE;
    /// See [`crate::placement::Placement::VERSION`].
    const VERSION: u32 = 1;
    const NEEDS_GRAPH: bool = true;

    fn evaluate(&self, view: &CorpusView<'_>) -> Outcome {
        let (Some(edges), Some(generated)) = (view.edges(), view.generated()) else {
            return Outcome::Skipped(NO_GRAPH.to_string());
        };
        let Some(state_facet) = self.state_facet.as_deref() else {
            return Outcome::Passed;
        };

        let mut findings = Vec::new();
        for page in generated {
            if !self
                .shape
                .required_facets(page.kind)
                .iter()
                .any(|facet| facet == state_facet)
            {
                continue;
            }
            // `derived::incoming`, clause by clause: into this document, as
            // declared, and not written by it.
            let mut set: Vec<(&str, &str, &str)> = edges
                .iter()
                .filter(|edge| edge.source.path != page.path)
                .filter(|edge| edge.direction == Direction::AsDeclared)
                .filter(|edge| edge.raw_target.trim() == page.id)
                .filter_map(|edge| {
                    self.state_of(&edge.declared)
                        .map(|state| (state, edge.declared.as_str(), edge.source.id.as_str()))
                })
                .collect();
            set.sort_unstable();
            set.dedup();

            let mut states: Vec<&str> = set.iter().map(|(state, _, _)| *state).collect();
            states.dedup();
            if states.len() < 2 {
                continue;
            }

            let edges_named = set
                .iter()
                .map(|(state, relation, source)| {
                    format!("`{relation}` from `{source}` sets `{state}`")
                })
                .collect::<Vec<_>>()
                .join(", ");
            let (line, column) = at(page.facets.entry(state_facet).map(|entry| entry.key.span));
            findings.push(Finding {
                rule: self::RULE,
                severity: Severity::Warn,
                obligation: None,
                path: page.path.to_string(),
                line,
                column,
                message: format!(
                    "`headwater generate` writes `{}` onto this page, the first in sorted order of \
                     the {} states its incoming relations set: {edges_named}",
                    states[0],
                    states.len()
                ),
                remediation: "in the documents that declare these edges, remove one edge or use a \
                              relation that sets the same state, or change what one relation \
                              declares under `on_target.set_state`. This file is generated, so \
                              an edit here does not last"
                    .to_string(),
                patch: None,
            });
        }

        match findings.is_empty() {
            true => Outcome::Passed,
            false => Outcome::Failed(findings),
        }
    }
}
