// SPDX-License-Identifier: Apache-2.0
//! A Graph-origin check: the target of a declared edge resolves to something.
//!
//! # The gap this closes
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#two-phases-and-why-the-order-matters)
//! lists "a dangling edge" among the structural findings of phase A, beside an
//! unparseable file, an unclassifiable path and an ambiguous shelf match. The
//! graph detected one and printed it under its own heading, and it reached no
//! rule. So it carried no obligation and no severity, `--strict` exited 0 over
//! it, and a gate went green across a corpus whose edges pointed at nothing.
//!
//! The other three structural outcomes do not have that hole. Each one is a
//! census row, and [`crate::coverage`] reads the census: a file that did not
//! classify is a document with no instance, and that is a finding. A dangling
//! edge is the one member of the list with no row of its own, because a census
//! row is a file and an edge is not one.
//!
//! # The unit is the authored entry, not the Q4 pair
//!
//! [`crate::scope::EdgeUnit`] states the general rule and this is the check
//! that needed it. A pair is a source, a relation and a **target document**, so
//! a pair exists only where the target already resolved. An edge that resolves
//! to nothing has no far end, nothing at that end can have written a second
//! half, and grouping would drop exactly the edges this rule is about.
//!
//! So one instance covers one entry of one `relations:` block. Both halves of a
//! bound reciprocal pair are two instances here, which is right: they are two
//! strings written by two authors in two files, and each one is correct or
//! broken on its own.
//!
//! # Every declared relation, and what that buys
//!
//! The generation step reads the relation declarations and nothing else, so a
//! new relation in a taxonomy produces its instances with no code. Unlike
//! [`crate::endpoint`], this rule needs neither endpoint set: a relation that
//! names no `from` and no `to` still names a target, and a target that resolves
//! to nothing is a defect whatever the ends admit.
//!
//! The instance exists for a bound target as well as for an unbound one. A rule
//! whose instances are only its findings reports a count that reads as its own
//! denominator, and coverage would then say that a document with fifty resolved
//! edges was never checked for a dangling one.
//!
//! # What it is not
//!
//! It is not the endpoint rule. Whether the target *resolved* and whether the
//! two kinds are ones the relation *permits* are two questions, and the second
//! one can only be asked after the first is answered.
//!
//! It is also not every phase-A defect of a `relations:` block. A block that is
//! not a mapping, an unknown relation name, an entry with no `to`, a repeated
//! triple and a source with no identifier are `headwater_graph::Problem`, and
//! each one stops an edge from existing at all. No edge means no instance, so
//! no rule here can reach them. They are reported under the graph heading and
//! they answer to nothing, which is the same shape of gap one level further
//! back.
//!
//! # The scope
//!
//! [`EdgeCheck`] is the whole declaration. The view carries the halves that
//! declared the edge, and the read set is the declaring document. An unbound
//! target contributes no second document to read, because there is no document
//! at the other end to hash.

use crate::finding::{at, Finding, Severity};
use crate::instance::Outcome;
use crate::scope::{EdgeCheck, EdgeUnit, EdgeView};
use headwater_graph::declarations::Relation;
use headwater_graph::{Declarations, Target, Unbound};

pub const RULE: &str = "relation.target.unresolved";

/// A group with no half at all, which the instantiation never produces.
/// Recorded rather than panicked on, because a panic inside a check is a run
/// that reports nothing about the rest of the corpus.
const NO_HALF: &str = "the entry carries no declared half";

/// The check, generated from the relation declarations.
pub struct Targets<'a> {
    /// Every declared relation. A relation is here whether or not it states its
    /// endpoint kinds, because a target string resolves or does not resolve
    /// without reference to either end.
    declared: Vec<&'a Relation>,
}

impl<'a> Targets<'a> {
    pub fn over(declarations: &'a Declarations) -> Self {
        Targets {
            declared: declarations.relations.iter().collect(),
        }
    }
}

impl EdgeCheck for Targets<'_> {
    const RULE: &'static str = self::RULE;
    /// See [`crate::placement::Placement::VERSION`].
    const VERSION: u32 = 1;
    /// See the module comment: a pair cannot carry an edge that bound to
    /// nothing, and those are the edges this rule exists for.
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

        // Three of the four bound targets are a resolution that succeeded. The
        // fourth is `Withheld`, which an export filter produced deliberately,
        // and spec 7 rules that it is never a defect and never counted as one.
        let Target::Unbound(unbound) = &edge.target else {
            return Outcome::Passed;
        };

        let (line, column) = at(Some(edge.span));
        Outcome::failed_with(Finding {
            rule: self::RULE,
            severity: Severity::Error,
            obligation: None,
            path: edge.source.path.clone(),
            line,
            column,
            message: format!(
                "`{}` declares `{}: {}`, and that target {unbound}",
                edge.source.id, edge.name, edge.raw_target
            ),
            remediation: remediation(unbound, &edge.name, &edge.raw_target),
            // No fix, and the reason is spec 12's own bar rather than a
            // judgment about effort. A target that names nothing has no one
            // correct outcome: the repair is another identifier, a new
            // document, a kind on a document that has none, or the deletion of
            // the entry, and only the author knows which of the four was meant.
            fixable: false,
        })
    }
}

/// What to do about it, which is a different sentence for each way a target can
/// bind to nothing. The variants send an author to three different files, and
/// one sentence for all five would send every author to the link.
fn remediation(unbound: &Unbound, name: &str, raw: &str) -> String {
    match unbound {
        Unbound::NoSuchTarget { .. } => format!(
            "write the identifier of a document that exists under `{name}`, or mint `{raw}` on the document this entry means"
        ),
        Unbound::NotTyped { path, .. } => format!(
            "give {path} a kind, because both ends of a declared relation are kinds and only a typed document is a node"
        ),
        Unbound::NoResolver {
            anchor_kind,
            resolver,
        } => format!(
            "supply the resolver `{resolver}` that `{anchor_kind}` names, or declare an anchor kind whose resolver this run has"
        ),
        Unbound::AnchorUnresolved { anchor_kind, .. } => format!(
            "correct `{raw}` so that the resolver for `{anchor_kind}` admits it"
        ),
        Unbound::AmbiguousAnchor { anchor_kinds } => format!(
            "narrow the anchor kinds that claim `{raw}`, because {} both admit it and the target has two identities",
            anchor_kinds.join(" and ")
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every way a target can bind to nothing has a remedy of its own, and each
    /// one names the file that holds the repair.
    ///
    /// Two of the five have a fixture in `fixtures/check/`, because that tree
    /// declares no anchor kind and the other three need one. The table is
    /// asserted here instead, so that a new `Unbound` variant cannot reach a
    /// reader as somebody else's sentence.
    #[test]
    fn each_way_a_target_binds_to_nothing_sends_its_author_to_a_different_file() {
        let cases = [
            (
                Unbound::NoSuchTarget {
                    also_tried: Vec::new(),
                },
                "mint `DR-FIX-0001`",
            ),
            (
                Unbound::NotTyped {
                    path: "docs/notes/loose.md".to_string(),
                    class: "untyped",
                    detail: "no front matter".to_string(),
                },
                "give docs/notes/loose.md a kind",
            ),
            (
                Unbound::NoResolver {
                    anchor_kind: "code_path".to_string(),
                    resolver: "source-tree".to_string(),
                },
                "supply the resolver `source-tree`",
            ),
            (
                Unbound::AnchorUnresolved {
                    anchor_kind: "code_path".to_string(),
                    why: "the path climbs above the repository".to_string(),
                },
                "correct `DR-FIX-0001`",
            ),
            (
                Unbound::AmbiguousAnchor {
                    anchor_kinds: vec!["code_path".to_string(), "work_item".to_string()],
                },
                "code_path and work_item",
            ),
        ];

        let mut seen: Vec<String> = Vec::new();
        for (unbound, expected) in cases {
            let text = remediation(&unbound, "traces_to", "DR-FIX-0001");
            assert!(text.contains(expected), "{text}");
            assert!(
                !seen.contains(&text),
                "two variants share one remedy: {text}"
            );
            seen.push(text);
        }
    }
}
