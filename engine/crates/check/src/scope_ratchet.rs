// SPDX-License-Identifier: Apache-2.0
//! A change that adds a file inside the governed scope, or leaves one there
//! with no `governance` edge, and no edge reaches the file (#1573).
//!
//! A corpus declares its governed scope as `anchors.<kind>.scope`, and
//! `taxonomy audit` counts, per pattern, the entries a `governance` edge
//! reaches. That figure is a reading and not a bar: nothing failed when it
//! went down. This rule is the bar, and the baseline it holds a change to is
//! the change itself.
//!
//! # The baseline is the change, so nothing is stored
//!
//! A ratchet needs a figure to compare against. A floor committed per pattern
//! is a corpus-wide fold that every branch rewrites, and
//! [HW-DR-0049](../../../../docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md)
//! rules that such a fold is derived and never stored. A figure computed on
//! the base branch is an input that spec 12 keeps out of a check. The change
//! already names what it adds, so the rule reads that: a path the manifest
//! names on an `added` line, inside a scope pattern, that no `governance`
//! edge onto the pattern's anchor kind matches, is new debt. A file that
//! already stood ungoverned is old debt, and an edit to it is not reported. A
//! corpus parks old debt in the `adoption` block of its lock, and this rule
//! does not reach it.
//!
//! # A removal is new debt too
//!
//! A change can lower the governed count without adding a file: it drops or
//! narrows a `governs` entry, or it deletes the document that held one. The
//! change names each document it carries, and the manifest supplies the
//! version that stood before it. So the rule reads the governance entries of
//! each prior version, of a document a row still holds
//! ([`crate::change::Change::committed`]) and of one that left
//! ([`crate::change::Change::departed`]). It takes every target written in
//! the declared direction onto an anchor kind that declares a scope, matches
//! it against the entries that kind's scope admits now, and reports each
//! entry it matched that no edge reaches now. The prior entry is read by
//! [`headwater_graph::edges::declared_targets`], the reader the graph build
//! calls, and matched lexically by the pattern language the resolver uses,
//! so no resolver runs over a version that is not in the tree. A file the
//! change deletes is in no scope now, so it is never a candidate. A file that
//! stood ungoverned and that no prior entry reached is old debt still. A path
//! the change adds is the added half's finding, and it is reported once.
//!
//! # A change to the scope itself is not read
//!
//! A new or wider pattern under `anchors.<kind>.scope` admits files that
//! already stood, and some of them can have no edge. The rule does not report
//! them. A wider scope adds no code and lowers no governed count, `taxonomy
//! audit` reports the edit, and a gate on it would make a wider scope cost a
//! backfill in the same change, which pushes a corpus to keep its scope
//! narrow. The question reopens when an adopter reports a scope edit that hid
//! new debt.
//!
//! # An edit is not an addition, and the bind keeps them apart
//!
//! A source file has no census row in a corpus that walks only its documents.
//! An `added` line and a `prior` line for such a path both bind to no row,
//! and a source file's prior bytes are no document, so the two once bound to
//! one shape. [`crate::change::Change::added`] reads the line's verb, which
//! the bind now keeps, so an edit to one of the files that already stood
//! ungoverned is never read as an addition.
//!
//! # One reach test
//!
//! [`headwater_graph::scope::governed`] is the test `taxonomy audit` counts
//! with. The rule calls the same function for both halves, so the figure and
//! the rule cannot disagree about whether one path is governed.
//!
//! # The severity is a warning, and posture decides whether it blocks
//!
//! The remedy is a rewrite: write a `governs` edge from the document that
//! states what the file does, or narrow the scope. Neither is mechanical, so
//! the finding is a warning and carries no patch. A control at `posture:
//! blocking` makes a strict run fail on it (#2015), and that is how a corpus
//! turns the ratchet on. The standard package ships the control advisory.

use crate::finding::{at, Finding, Severity};
use crate::instance::Outcome;
use crate::scope::{CorpusCheck, CorpusView};
use headwater_graph::declarations::{Declarations, Direction};
use headwater_graph::scope::Reach;
use headwater_meta::pattern::Pattern;
use std::collections::{BTreeMap, BTreeSet};

pub const RULE: &str = "relation.scope.ungoverned";

/// The check. It carries the relation declarations because the reach test
/// reads which relations are of the `governance` family, and the name of the
/// relations block because the removal half reads that block of a prior
/// version.
pub struct Ungoverned<'a> {
    declarations: &'a Declarations,
    relations_facet: &'a str,
}

impl<'a> Ungoverned<'a> {
    pub fn over(declarations: &'a Declarations, relations_facet: &'a str) -> Self {
        Ungoverned {
            declarations,
            relations_facet,
        }
    }

    /// The patterns that admit `path`, where no governance edge onto any of
    /// their anchor kinds reaches it, and `None` where the path is in no
    /// scope or an edge reaches it. A path that one pattern admits and an
    /// edge reaches is governed, as the audit counts it.
    fn ungoverned<'s>(
        path: &str,
        scope: &'s [Reach],
        reached: &BTreeMap<&str, BTreeSet<&str>>,
    ) -> Option<Vec<&'s Reach>> {
        let admitting: Vec<&Reach> = scope
            .iter()
            .filter(|reach| {
                reach
                    .entries
                    .binary_search_by(|entry| entry.as_str().cmp(path))
                    .is_ok()
            })
            .collect();
        if admitting.is_empty() {
            return None;
        }
        let governed = admitting.iter().any(|reach| {
            reached
                .get(reach.anchor_kind.as_str())
                .is_some_and(|set| set.contains(path))
        });
        (!governed).then_some(admitting)
    }

    /// Each in-scope entry that a governance target of a prior version
    /// matched, with the documents whose prior version matched it.
    ///
    /// A target is read only where it was written in the declared direction
    /// onto an anchor kind that declares a scope. A target that does not
    /// normalize matches nothing: the commit that wrote it already failed a
    /// relation rule.
    fn reached_before<'v>(
        &self,
        view: &CorpusView<'v>,
        scope: &'v [Reach],
    ) -> BTreeMap<&'v str, BTreeSet<&'v str>> {
        let priors = view
            .committed()
            .iter()
            .map(|entry| (entry.path, entry.facets))
            .chain(
                view.departed()
                    .iter()
                    .map(|entry| (entry.path, entry.facets)),
            );
        let mut candidates: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
        for (document, facets) in priors {
            let declared = headwater_graph::edges::declared_targets(
                facets,
                self.relations_facet,
                self.declarations,
            );
            for entry in declared {
                if entry.direction != Direction::AsDeclared
                    || entry.relation.family.as_deref() != Some("governance")
                {
                    continue;
                }
                for raw in &entry.raws {
                    let Ok(normalized) = headwater_graph::anchors::normalize(raw) else {
                        continue;
                    };
                    let pattern = Pattern::new(&normalized);
                    for reach in scope
                        .iter()
                        .filter(|reach| entry.relation.to.contains(&reach.anchor_kind))
                    {
                        for path in reach.entries.iter().filter(|path| pattern.matches(path)) {
                            candidates
                                .entry(path.as_str())
                                .or_default()
                                .insert(document);
                        }
                    }
                }
            }
        }
        candidates
    }
}

fn patterns_of(admitting: &[&Reach]) -> String {
    admitting
        .iter()
        .map(|reach| format!("`{}`", reach.pattern))
        .collect::<Vec<_>>()
        .join(", ")
}

impl CorpusCheck for Ungoverned<'_> {
    const RULE: &'static str = self::RULE;
    /// The second edition: the rule reads the prior version of each document
    /// the change carries, and reports a file the change left with no
    /// governance edge. A verdict cached by the first edition over a change
    /// that narrowed a document did not read that prior.
    ///
    /// Raise the rule version when a rule's behavior changes.
    const VERSION: u32 = 2;
    /// The paths the change adds, and the prior version of each document it
    /// carries. A run with no change has neither, and the runner skips the
    /// instance with the reason spec 12 fixes.
    const NEEDS_PRIOR: bool = true;
    /// The edges, for the reach test, and the scope the graph build walked.
    const NEEDS_GRAPH: bool = true;

    fn evaluate(&self, view: &CorpusView<'_>) -> Outcome {
        let (Some(edges), Some(scope)) = (view.edges(), view.scope()) else {
            return Outcome::Passed;
        };
        // The reach of each anchor kind that declares a pattern, once, rather
        // than one walk of every edge for each path.
        let reached: BTreeMap<&str, BTreeSet<&str>> = scope
            .iter()
            .map(|reach| {
                (
                    reach.anchor_kind.as_str(),
                    headwater_graph::scope::governed(edges, self.declarations, &reach.anchor_kind),
                )
            })
            .collect();
        let mut findings = Vec::new();
        let (line, column) = at(None);
        for path in view.added() {
            let Some(admitting) = Self::ungoverned(path, scope, &reached) else {
                continue;
            };
            let patterns = patterns_of(&admitting);
            findings.push(Finding {
                rule: RULE,
                severity: Severity::Warn,
                obligation: None,
                path: (*path).to_string(),
                line,
                column,
                message: format!(
                    "This change adds `{path}`, which the governed scope {patterns} admits, and \
                     no governance edge reaches it"
                ),
                remediation: format!(
                    "Add a `governs` entry that names `{path}`, or a pattern that matches it, to \
                     the document that states what it does. If the file is outside what the \
                     corpus means to govern, narrow the scope pattern instead"
                ),
                patch: None,
            });
        }
        // The removal half. A path the change adds did not stand before it,
        // and the loop above has already reported it if no edge reaches it.
        let added: BTreeSet<&str> = view.added().iter().copied().collect();
        for (path, documents) in self.reached_before(view, scope) {
            if added.contains(path) {
                continue;
            }
            let Some(admitting) = Self::ungoverned(path, scope, &reached) else {
                continue;
            };
            let patterns = patterns_of(&admitting);
            let documents = documents
                .iter()
                .map(|document| format!("`{document}`"))
                .collect::<Vec<_>>()
                .join(", ");
            findings.push(Finding {
                rule: RULE,
                severity: Severity::Warn,
                obligation: None,
                path: path.to_string(),
                line,
                column,
                message: format!(
                    "This change leaves `{path}`, which the governed scope {patterns} admits, \
                     with no governance edge: {documents} reached it before the change"
                ),
                remediation: format!(
                    "Restore a `governs` entry that names `{path}`, or a pattern that matches \
                     it, in the document that states what it does. If the file is outside what \
                     the corpus means to govern, narrow the scope pattern instead"
                ),
                patch: None,
            });
        }
        Outcome::failed(findings)
    }
}
