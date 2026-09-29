// SPDX-License-Identifier: Apache-2.0
//! A Graph-origin check: no relation entry names the document that declares it.
//!
//! # The gap this closes
//!
//! A document could draw on itself, supersede itself or govern itself, and no
//! rule reported it ([#1232](https://github.com/headwater-ai/headwater/issues/1232)).
//! The target resolved, so [`crate::target`] passed it. The relation admits the
//! document's kind at both ends, so [`crate::endpoint`] passed it too. The
//! graph kept an edge whose two ends are one node.
//!
//! # Which relations may point at their own document: association alone
//!
//! [Spec 2](../../../../docs/spec/02-taxonomy-model.md#behavior-at-the-limits)
//! rules it: "Self-reference stays invalid in every family except
//! association." So the rule is generated from every declared relation whose
//! family is not `association`, as [`crate::target`] is generated from every
//! relation, and it names no relation. A relation in any other family, or in
//! no declared family, that points at its own document asserts nothing (a
//! document that draws on, governs, constrains or traces to itself) or asserts
//! a contradiction (a document that supersedes itself). An association edge is
//! exempt because the family is the one spec 2 lets cycle, and a relation of
//! it has no instance here, so it is outside the denominator rather than a
//! pass inside it.
//!
//! OB-REL-7 in the standard package is the obligation this rule discharges, and
//! it states the same thing.
//!
//! # The unit is the authored entry
//!
//! For [`crate::target`]'s reason: one instance is one entry of one
//! `relations:` block. A document that writes both halves of a reciprocal onto
//! itself gets two findings, one for each line it wrote, and the inverse half
//! is read as well as the declared one.
//!
//! # What it is not
//!
//! It compares the resolved target with the declaring document, and only a
//! target that resolved to a document is compared. A target that resolved to
//! nothing is [`crate::target`]'s finding, so an unbound target passes here,
//! and so does a `Withheld` target. Every instance stays in the denominator,
//! whatever its verdict, for the reason [`crate::target`] gives.
//!
//! An anchor target passes too, and that includes an anchor onto the declaring
//! document's own file. A target written as a path, under a relation whose
//! endpoint admits a `code_path` anchor (in headwater/standard: `governs`,
//! `traces_to` and `examines`), binds as an anchor and not as an unresolved
//! identifier. Its pattern can match the file that declares it: a decision
//! that writes `traces_to:` with its own path binds onto itself, and this
//! rule does not report it. The reason is that an anchor is a pattern over
//! the tree and not a document identity
//! ([HW-DR-0074](../../../../docs/decisions/0074-a-code-path-anchor-is-a-pattern-over-the-tree-and-it-binds-when-the-pattern-matches-at-least-one-entry.md)).
//! A pattern that covers its own file is often correct, as when a decision
//! governs the directory it sits in. Only a literal single-path pattern equal
//! to the declaring file is plainly a self-reference, and spec 2 rules on
//! document targets and says nothing about anchors (#1335).
//!
//! What reopens this: an owner ruling that a literal anchor onto the declaring
//! file is a self-reference under spec 2's rule. That ruling comes first, and
//! a finding for that one case follows from it.

use crate::finding::{at, Finding, Severity};
use crate::instance::Outcome;
use crate::scope::{EdgeCheck, EdgeUnit, EdgeView};
use headwater_graph::declarations::Relation;
use headwater_graph::{Declarations, Target};

pub const RULE: &str = "relation.target.is_source";

/// A group with no half at all, which the instantiation never produces.
/// Recorded rather than panicked on, as in [`crate::target`].
const NO_HALF: &str = "the entry carries no declared half";

/// The one family spec 2 lets point at its own document.
pub(crate) const EXEMPT_FAMILY: &str = "association";

/// The check, generated from the relation declarations.
pub struct SelfTarget<'a> {
    /// Every declared relation outside the association family, because none of
    /// them admits a self-edge.
    declared: Vec<&'a Relation>,
}

impl<'a> SelfTarget<'a> {
    pub fn over(declarations: &'a Declarations) -> Self {
        SelfTarget {
            declared: declarations
                .relations
                .iter()
                .filter(|relation| relation.family.as_deref() != Some(EXEMPT_FAMILY))
                .collect(),
        }
    }
}

impl EdgeCheck for SelfTarget<'_> {
    const RULE: &'static str = self::RULE;
    /// See [`crate::placement::Placement::VERSION`].
    const VERSION: u32 = 1;
    /// See the module comment.
    const UNIT: EdgeUnit = EdgeUnit::Entry;

    fn instantiates(&self, relation: &str) -> bool {
        self.declared.iter().any(|known| known.name == relation)
    }

    fn evaluate(&self, view: &EdgeView<'_>) -> Outcome {
        // One entry is one half by construction, and either direction can be
        // the one an author wrote.
        let Some(edge) = view.declared_half().or_else(|| view.inverse_half()) else {
            return Outcome::Skipped(NO_HALF.to_string());
        };

        // The path is the identity of a node, so two ends that share
        // one path are one document.
        let Target::Document { path, .. } = &edge.target else {
            return Outcome::Passed;
        };
        if *path != edge.source.path {
            return Outcome::Passed;
        }

        let (line, column) = at(Some(edge.span));
        Outcome::failed_with(Finding {
            rule: self::RULE,
            severity: Severity::Error,
            obligation: None,
            path: edge.source.path.clone(),
            line,
            column,
            message: format!(
                "`{}` declares `{}: {}`, and that names the document that declares it",
                edge.source.id, edge.name, edge.raw_target
            ),
            remediation: format!(
                "name the document that `{}` actually means under `{}`, or delete the entry, because only an association relation may point at its own document",
                edge.source.id, edge.name
            ),
            // No fix: the author alone knows which document the entry meant,
            // or whether it meant none.
            patch: None,
        })
    }
}
