// SPDX-License-Identifier: Apache-2.0
//! A Graph-origin check, generated from the participation expectations a kind
//! declares. This is the rule that reads the clock.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#the-five-origins-of-a-check)
//! lists "windowed participation expectations" among the Graph-origin examples,
//! and [spec 12](../../../../docs/spec/12-check-layer.md#temporal-inputs-the-clock-and-the-prior-version)
//! says how one is evaluated: it "reads a declared origin date from the
//! document and compares it against `ctx.now`. There is no history walk and no
//! search of the repository history."
//!
//! Nothing below names a relation, a kind or a number of days. A kind declares
//! `relations.expect`, and a taxonomy that declares one more gets one more
//! instance and this file does not change.
//!
//! # Why the grain is a neighbourhood and not an edge
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#what-this-leaves-open)
//! suspects the `Neighbourhood(depth)` scope is unnecessary: "if no real check
//! needs depth > 1, the correct move is to cut it and to keep `Edge` as the
//! only relational scope." This rule is the case that sentence does not cover,
//! and the reason is one line long.
//!
//! **An `Edge` instance exists per edge, and this rule is about an edge that
//! nobody declared.** There is nothing to instantiate over. The unit is the
//! document that owes the edge, and the inputs are that document plus the kinds
//! at the far end of every relation it does have — depth exactly 1. So depth 1
//! is needed and `Edge` does not reach it, which is a sharpening of that open
//! question rather than an answer to it, and
//! [13 — Open obligations](../../../../docs/spec/13-open-obligations.md)
//! carries it.
//!
//! # Either half may be the one that is written
//!
//! An expectation names a relation the way an author would write it, and this
//! corpus writes `cited_by` on the cited document. That name is resolved to a
//! relation type and an **end** once, when the check is built. Matching then
//! runs on the resolved pair, so an expectation is satisfied by the half its
//! own document wrote and by the half the other document wrote.
//!
//! # What is a skip here, and what is a pass
//!
//! A document the expectation does not apply to is a pass: a draft evaluation
//! that nothing cites yet has broken no expectation. A window that has not
//! elapsed is a pass for the same reason.
//!
//! A declaration this engine cannot read is a **skip with a reason** — an
//! unknown severity, a window that is not a number of days, an origin no facet
//! carries, or a document whose origin date will not parse. Every one of those
//! is a fact about the taxonomy or the document rather than a verdict, and
//! [spec 4](../../../../docs/spec/04-assurance-model.md#no-silent-passes-every-document-is-accounted-for)
//! wants it visible rather than counted as a pass.

use crate::context::Date;
use crate::finding::{at, Finding};
use crate::instance::Outcome;
use crate::scope::{End, NeighbourhoodCheck, NeighbourhoodView};
use crate::shape::{Expectation, Shape};
use headwater_graph::{Declarations, Direction};

pub const RULE: &str = "relation.participation.overdue";

/// Unreachable: this check declares `NEEDS_CLOCK`, so the runner hands it one.
/// Recorded rather than assumed, because a check that unwrapped here would turn
/// one wiring mistake into a run that reports nothing.
const NO_CLOCK: &str = "the run injected no clock";

const NO_RELATION: &str = "the expectation names a relation that no declaration holds";
const NO_SEVERITY: &str = "the expectation states a severity this engine does not know";
const NO_WINDOW: &str = "the expectation states no window this engine can read";
const NO_ORIGIN_FACET: &str = "the expectation measures from a role that no facet carries";
const NO_ORIGIN_DATE: &str = "the document declares no readable date under the origin facet";

/// One expectation, with every name it uses resolved once.
struct Windowed {
    /// The kind that declares it.
    kind: String,
    expectation: Expectation,
    /// The relation type the declared name resolves to.
    relation: Option<String>,
    /// Which end of that relation the document declaring the expectation sits
    /// at. Read off the direction, so an inverse name does not change it.
    end: Option<End>,
    /// The facet that carries the role the window is measured from.
    since_facet: Option<String>,
}

/// The check, generated from the participation expectations of every kind.
pub struct Participation<'a> {
    windowed: Vec<Windowed>,
    shape: &'a Shape,
}

impl<'a> Participation<'a> {
    /// The generation step, in full. It reads the taxonomy and never the
    /// corpus.
    pub fn over(shape: &'a Shape, declarations: &Declarations) -> Self {
        let mut windowed = Vec::new();
        for kind in &shape.kinds {
            for expectation in &kind.expectations {
                let named = declarations.named(&expectation.relation);
                windowed.push(Windowed {
                    kind: kind.name.clone(),
                    relation: named.map(|named| named.relation.name.clone()),
                    // A document that writes a relation under its own name is
                    // the source of the edge it declares, and one that writes
                    // the inverse name is its target.
                    end: named.map(|named| match named.direction {
                        Direction::AsDeclared => End::Source,
                        Direction::Inverse => End::Target,
                    }),
                    since_facet: shape
                        .facet_in_role(&expectation.since_role)
                        .map(|facet| facet.name.clone()),
                    expectation: expectation.clone(),
                });
            }
        }
        Participation { windowed, shape }
    }

    /// Whether the document carries the facet values the expectation applies
    /// to. An expectation with no `when` applies to every document of the kind.
    fn applies(&self, expectation: &Expectation, view: &NeighbourhoodView<'_>) -> bool {
        expectation.when.iter().all(|(facet, value)| {
            view.facets()
                .get(facet)
                .and_then(|node| node.value.as_scalar())
                .is_some_and(|scalar| &scalar.text == value)
        })
    }

    /// Whether a neighbour satisfies the expectation: the right relation, the
    /// right end, and a kind that descends from the one it names.
    fn satisfied(
        &self,
        expectation: &Expectation,
        relation: &str,
        end: End,
        view: &NeighbourhoodView<'_>,
    ) -> bool {
        view.neighbours().iter().any(|neighbour| {
            neighbour.relation == relation
                && neighbour.end == end
                && match &expectation.to_kind {
                    None => true,
                    Some(kind) => self.shape.descends_from(neighbour.kind, kind),
                }
        })
    }
}

impl NeighbourhoodCheck for Participation<'_> {
    const RULE: &'static str = self::RULE;
    /// See [`crate::placement::Placement::VERSION`].
    const VERSION: u32 = 1;
    /// The declaration that admits `ctx.now` to the view, and the same one that
    /// puts the date in the cache key. See [`crate::scope`] and [`crate::cache`]
    /// for why those cannot be two decisions.
    const NEEDS_CLOCK: bool = true;

    fn instantiates(&self, kind: &str) -> bool {
        self.windowed.iter().any(|windowed| windowed.kind == kind)
    }

    fn evaluate(&self, view: &NeighbourhoodView<'_>) -> Outcome {
        let Some(now) = view.now() else {
            return Outcome::Skipped(NO_CLOCK);
        };

        let mut findings = Vec::new();
        // The first reason a declaration could not be read. One instance
        // reports one reason, and the reasons are counted per rule rather than
        // per expectation, so the first is the one a reader acts on.
        let mut unreadable: Option<&'static str> = None;
        let mut note = |reason| {
            unreadable.get_or_insert(reason);
        };

        for windowed in self
            .windowed
            .iter()
            .filter(|windowed| windowed.kind == view.kind())
        {
            let expectation = &windowed.expectation;
            if !self.applies(expectation, view) {
                continue;
            }
            let (Some(relation), Some(end)) = (&windowed.relation, windowed.end) else {
                note(NO_RELATION);
                continue;
            };
            let Some(severity) = expectation.severity else {
                note(NO_SEVERITY);
                continue;
            };
            let Some(within) = expectation.within_days else {
                note(NO_WINDOW);
                continue;
            };
            let Some(since) = &windowed.since_facet else {
                note(NO_ORIGIN_FACET);
                continue;
            };
            if self.satisfied(expectation, relation, end, view) {
                continue;
            }

            let entry = view.facets().entry(since);
            let origin = entry
                .and_then(|entry| entry.value.value.as_scalar())
                .and_then(|scalar| Date::parse(&scalar.text));
            let Some(origin) = origin else {
                note(NO_ORIGIN_DATE);
                continue;
            };
            let elapsed = now.days_since(origin);
            if elapsed <= within {
                // The window is still open, and a document inside it owes
                // nothing yet. A negative count is a date in the future, which
                // is the same answer for the same reason.
                continue;
            }

            // At the origin facet, which is the line that started the window.
            // A finding about an edge nobody wrote has no line of its own, and
            // this is the nearest line that explains the count.
            let (line, column) = at(entry.map(|entry| entry.key.span));
            let owed = match &expectation.to_kind {
                Some(kind) => format!("`{relation}` to a `{kind}`"),
                None => format!("`{relation}`"),
            };
            findings.push(Finding {
                rule: self::RULE,
                severity,
                obligation: None,
                path: view.path().to_string(),
                line,
                column,
                message: format!(
                    "`{}`: {elapsed} days since `{since}`, and this `{}` reaches no {owed} \
                     inside the {within} days the taxonomy allows",
                    expectation.id,
                    view.kind()
                ),
                remediation: match &expectation.rationale {
                    Some(rationale) => format!(
                        "declare {owed} under `relations:` in {}, or record why it is not owed: {rationale}",
                        view.path()
                    ),
                    None => format!("declare {owed} under `relations:` in {}", view.path()),
                },
                fixable: false,
            });
        }

        match (findings.is_empty(), unreadable) {
            (false, _) => Outcome::Failed(findings),
            (true, Some(reason)) => Outcome::Skipped(reason),
            (true, None) => Outcome::Passed,
        }
    }
}
