// SPDX-License-Identifier: Apache-2.0
//! A change that adds a file inside the governed scope, and no `governance`
//! edge reaches the file (#1573).
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
//! with. The rule calls the same function, so the figure and the rule cannot
//! disagree about whether one path is governed.
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
use headwater_graph::declarations::Declarations;

pub const RULE: &str = "relation.scope.ungoverned";

/// The check. It carries the relation declarations because the reach test
/// reads which relations are of the `governance` family.
pub struct Ungoverned<'a> {
    declarations: &'a Declarations,
}

impl<'a> Ungoverned<'a> {
    pub fn over(declarations: &'a Declarations) -> Self {
        Ungoverned { declarations }
    }
}

impl CorpusCheck for Ungoverned<'_> {
    const RULE: &'static str = self::RULE;
    /// The first edition of this rule.
    ///
    /// Raise the rule version when a rule's behavior changes.
    const VERSION: u32 = 1;
    /// The paths the change adds. A run with no change has none to read, and
    /// the runner skips the instance with the reason spec 12 fixes.
    const NEEDS_PRIOR: bool = true;
    /// The edges, for the reach test, and the scope the graph build walked.
    const NEEDS_GRAPH: bool = true;

    fn evaluate(&self, view: &CorpusView<'_>) -> Outcome {
        let (Some(edges), Some(scope)) = (view.edges(), view.scope()) else {
            return Outcome::Passed;
        };
        // The reach of each anchor kind that declares a pattern, once, rather
        // than one walk of every edge for each added path.
        let reached: std::collections::BTreeMap<&str, std::collections::BTreeSet<&str>> = scope
            .iter()
            .map(|reach| {
                (
                    reach.anchor_kind.as_str(),
                    headwater_graph::scope::governed(edges, self.declarations, &reach.anchor_kind),
                )
            })
            .collect();
        let mut findings = Vec::new();
        for path in view.added() {
            // Every pattern that admits the path and whose anchor kind no
            // governance edge reaches it through. A path that one pattern
            // admits and an edge reaches is governed, as the audit counts it.
            let admitting: Vec<&headwater_graph::scope::Reach> = scope
                .iter()
                .filter(|reach| {
                    reach
                        .entries
                        .binary_search_by(|entry| entry.as_str().cmp(path))
                        .is_ok()
                })
                .collect();
            if admitting.is_empty() {
                continue;
            }
            let governed = admitting.iter().any(|reach| {
                reached
                    .get(reach.anchor_kind.as_str())
                    .is_some_and(|set| set.contains(path))
            });
            if governed {
                continue;
            }
            let patterns = admitting
                .iter()
                .map(|reach| format!("`{}`", reach.pattern))
                .collect::<Vec<_>>()
                .join(", ");
            let (line, column) = at(None);
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
        Outcome::failed(findings)
    }
}
