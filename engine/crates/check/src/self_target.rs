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
//! # An anchor onto the declaring document's own file
//!
//! A target written as a path, under a relation whose endpoint admits a
//! `code_path` anchor (in headwater/standard: `governs`, `traces_to` and
//! `examines`), binds as an anchor and not as a document. This rule reports
//! one such anchor: a `source-tree` anchor that holds exactly one pattern,
//! where that pattern has no wildcard and is the declaring document's own
//! path. A decision that writes `traces_to:` with its own path governs itself,
//! and the owner ruled that this exact case is a self-reference under spec 2's
//! rule (#1350, 2026-09-29: "Report it, exact own file").
//!
//! Every other anchor passes, because the ruling reaches the exact own file and
//! nothing wider. An anchor is a pattern over the tree and not a document
//! identity
//! ([HW-DR-0074](../../../../docs/decisions/0074-a-code-path-anchor-is-a-pattern-over-the-tree-and-it-binds-when-the-pattern-matches-at-least-one-entry.md)).
//! A pattern with a wildcard that matches the own file passes, as when a
//! decision governs the directory it sits in, and that is often correct. A
//! list that holds the own file among other paths passes. An anchor of another
//! resolver passes, because its pattern can name a path in another repository
//! that has the same spelling.

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
    const VERSION: u32 = 2;
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

        let (message, remediation) = match &edge.target {
            // The path is the identity of a node, so two ends that share
            // one path are one document.
            Target::Document { path, .. } if *path == edge.source.path => (
                format!(
                    "`{}` declares `{}: {}`, and that names the document that declares it",
                    edge.source.id, edge.name, edge.raw_target
                ),
                format!(
                    "name the document that `{}` actually means under `{}`, or delete the entry, because only an association relation may point at its own document",
                    edge.source.id, edge.name
                ),
            ),
            target if names_own_file(target, &edge.source.path) => (
                format!(
                    "`{}` declares `{}: {}`, and that names its own file `{}`",
                    edge.source.id, edge.name, edge.raw_target, edge.source.path
                ),
                format!(
                    "name the file that `{}` actually means under `{}`, or delete the entry, because only an association relation may point at its own document",
                    edge.source.id, edge.name
                ),
            ),
            _ => return Outcome::Passed,
        };

        let (line, column) = at(Some(edge.span));
        Outcome::failed_with(Finding {
            rule: self::RULE,
            severity: Severity::Error,
            obligation: None,
            path: edge.source.path.clone(),
            line,
            column,
            message,
            remediation,
            // No fix: the author alone knows which document or file the entry
            // meant, or whether it meant none.
            patch: None,
        })
    }
}

/// Whether an anchor target is exactly the declaring document's own file: a
/// `source-tree` anchor with one pattern, that pattern literal, and equal to
/// the source's path. Both sides are normalized repository-relative paths, so
/// string equality is the comparison. A wildcard, a list, another resolver and
/// every non-anchor target answer no. See the module comment.
fn names_own_file(target: &Target, source_path: &str) -> bool {
    let Target::Anchor {
        resolver, patterns, ..
    } = target
    else {
        return false;
    };
    let [only] = patterns.as_slice() else {
        return false;
    };
    resolver == SOURCE_TREE
        && headwater_meta::Pattern::new(&only.pattern).is_literal()
        && only.pattern == source_path
}

/// The one resolver whose patterns are paths in the tree that holds the
/// declaring document. Another resolver's pattern can name a path in another
/// repository that happens to share the spelling.
const SOURCE_TREE: &str = "source-tree";

#[cfg(test)]
mod tests {
    use super::*;
    use headwater_graph::edges::PatternMember;

    fn anchor(resolver: &str, patterns: &[&str]) -> Target {
        Target::Anchor {
            anchor_kind: "code_path".to_string(),
            resolver: resolver.to_string(),
            normalized: patterns.join(", "),
            excluded_by: None,
            revision: headwater_graph::anchors::Revision::known(None),
            patterns: patterns
                .iter()
                .map(|pattern| PatternMember {
                    pattern: pattern.to_string(),
                    matched: vec![pattern.to_string()],
                    revision: headwater_graph::anchors::Revision::known(None),
                })
                .collect(),
        }
    }

    #[test]
    fn a_single_literal_source_tree_pattern_equal_to_the_source_is_its_own_file() {
        assert!(names_own_file(
            &anchor("source-tree", &["docs/a.md"]),
            "docs/a.md"
        ));
    }

    #[test]
    fn another_file_a_list_a_wildcard_and_another_resolver_are_not() {
        assert!(!names_own_file(
            &anchor("source-tree", &["docs/b.md"]),
            "docs/a.md"
        ));
        assert!(!names_own_file(
            &anchor("source-tree", &["docs/a.md", "docs/b.md"]),
            "docs/a.md"
        ));
        assert!(!names_own_file(
            &anchor("snapshot", &["docs/a.md"]),
            "docs/a.md"
        ));
        // A file whose own name holds a wildcard character: the pattern that
        // spells it is a glob over more than that file, so it is wider than
        // the exact own file.
        assert!(!names_own_file(
            &anchor("source-tree", &["docs/a?.md"]),
            "docs/a?.md"
        ));
    }

    /// The comparison is equality of whole paths. A literal that is the end
    /// of the own path names another file (a `docs/README.md` that governs
    /// the root `README.md`), and a literal that is its start names the
    /// directory it sits in, which is wider than the exact own file.
    #[test]
    fn a_literal_that_is_only_the_end_or_the_start_of_the_own_path_is_not() {
        assert!(!names_own_file(
            &anchor("source-tree", &["a.md"]),
            "docs/a.md"
        ));
        assert!(!names_own_file(
            &anchor("source-tree", &["docs"]),
            "docs/a.md"
        ));
        assert!(!names_own_file(
            &anchor("source-tree", &["docs/a.md"]),
            "other/docs/a.md"
        ));
    }
}
