// SPDX-License-Identifier: Apache-2.0
//! A Graph-origin check: no relation entry names the document that declares it.
//!
//! # The gap this closes
//!
//! A document could draw on itself, supersede itself or conflict with itself,
//! and no rule reported it ([#1232](https://github.com/headwater-ai/headwater/issues/1232)).
//! The target resolved, so [`crate::target`] passed it. The relation admits the
//! document's kind at both ends, so [`crate::endpoint`] passed it too. The
//! graph kept an edge whose two ends are one node.
//!
//! # Which relations may point at their own document: none
//!
//! No declared relation admits a self-edge. For each relation the standard
//! package and this repository's overlay declare, an edge from a document to
//! itself asserts nothing (a document that draws on, governs, constrains,
//! traces to or examines itself) or asserts a contradiction (a document that
//! supersedes itself, or conflicts with itself). So the rule is generated from
//! every declared relation, as [`crate::target`] is, and it names none of them.
//! A relation that one day needs a self-edge gets a declared exemption then.
//! No exemption flag exists now, because no relation would set it.
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
//! It compares the resolved target with the declaring document. A target that
//! resolved to nothing is [`crate::target`]'s finding, and a path that spells
//! the document's own file is one of those, because an edge target is an
//! identifier and never a path. So an unbound target passes here, and so do an
//! anchor and a `Withheld` target. Every instance stays in the denominator,
//! whatever its verdict, for the reason [`crate::target`] gives.
//!
//! An anchor target is outside this rule. `governs` onto a code path names a
//! path in the tree and not a document, so it cannot name its own source.

use crate::finding::{at, Finding, Severity};
use crate::instance::Outcome;
use crate::scope::{EdgeCheck, EdgeUnit, EdgeView};
use headwater_graph::declarations::Relation;
use headwater_graph::{Declarations, Target};

pub const RULE: &str = "relation.target.is_source";

/// A group with no half at all, which the instantiation never produces.
/// Recorded rather than panicked on, as in [`crate::target`].
const NO_HALF: &str = "the entry carries no declared half";

/// The check, generated from the relation declarations.
pub struct SelfTarget<'a> {
    /// Every declared relation, because none of them admits a self-edge.
    declared: Vec<&'a Relation>,
}

impl<'a> SelfTarget<'a> {
    pub fn over(declarations: &'a Declarations) -> Self {
        SelfTarget {
            declared: declarations.relations.iter().collect(),
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
                "name the document that `{}` actually means under `{}`, or delete the entry, because no declared relation may point at its own document",
                edge.source.id, edge.name
            ),
            // No fix: the author alone knows which document the entry meant,
            // or whether it meant none.
            patch: None,
        })
    }
}
