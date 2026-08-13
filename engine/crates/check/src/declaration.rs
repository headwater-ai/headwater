// SPDX-License-Identifier: Apache-2.0
//! A Graph-origin check: a `relations:` block declares edges the graph can use.
//!
//! # The gap this closes
//!
//! [`crate::target`] closed the dangling edge, which is the one phase-A defect
//! of a relation that leaves an edge behind. Four others leave none:
//! a block that is not a mapping, a key no declaration holds, an entry that
//! names no target, and one triple written twice. Each is
//! [`headwater_graph::edges::Problem`], the graph printed each under its own
//! heading, and each reached no rule. So none carried an obligation or a
//! severity, and `headwater check --strict` exited 0 over a document whose
//! whole `relations:` block the build had thrown away.
//!
//! The fifth member of that enum, `SourceHasNoIdentifier`, is
//! [`crate::identity`]'s. The two rules partition the enum, and each one matches
//! it exhaustively, so a sixth member is a compile error in two files rather
//! than a defect that reaches a reader in neither.
//!
//! # Why this grain, and why the report is on the view
//!
//! None of the four is edge-scoped, because each one stops an edge from
//! existing and an edge-scoped check is instantiated per edge. The unit that
//! survives is the document that wrote the block, which is `Document` scope.
//!
//! What a document-scoped view carries is front matter, and a second reading of
//! that front matter here would be a second definition of each defect. Three of
//! the four would agree with the build. `RepeatedTriple` would not:
//! [Q4](../../../../docs/spec/09-decisions.md#q4--relation-storage) identifies
//! an edge by its **normalized** target, and two spellings of one anchor are one
//! target only after a resolver has said so. This corpus writes exactly that
//! case. So the rule reads what the build decided, through
//! [`crate::scope::DocumentView::phase_a`], and derives nothing of its own.
//!
//! The report arrives on the **view** rather than on the check. A check that
//! held the whole report and looked its own path up in it could look a
//! sibling's path up as easily, and scope enforcement is a
//! [correctness root](../../../../docs/spec/12-check-layer.md#the-correctness-roots).
//! The view carries the report for one document and holds none other.
//!
//! # The generation step
//!
//! Every kind that some declared relation admits at either end, read through
//! the `is_a` chain exactly as [`crate::endpoint`] reads it. A kind that no
//! relation admits anywhere cannot write an entry this rule could hold to
//! anything, and generating over it would take the coverage rule's failing
//! fixture away: a classified document that no rule reads is
//! [`crate::coverage`]'s finding, and a rule that instantiates over every kind
//! makes it unreachable.
//!
//! The instance exists for a clean block and for a document with no block at
//! all, on [`crate::target`]'s argument: a rule whose instances are only its
//! findings has no denominator, and coverage would then call a document with a
//! well-formed `relations:` block unchecked for this.
//!
//! # Severity and fixability
//!
//! Error, on the precedent of `relation.target.unresolved`: a declared relation
//! that produced no edge is a claim the corpus does not hold, and every query
//! and projection over that relation reads the corpus rather than the claim.
//!
//! No fix, on spec 12's own bar. Three of the four repairs are a rewrite of the
//! block. The fourth looks mechanical and is not: a repeated triple is two
//! entries, either of which may carry instance attributes the other does not,
//! and only the author knows which entry was the one they meant.

use crate::finding::{at, Finding, Severity};
use crate::instance::Outcome;
use crate::scope::{DocumentCheck, DocumentView};
use crate::shape::Shape;
use headwater_graph::declarations::Relation;
use headwater_graph::edges::{Problem, Reported};
use headwater_graph::Declarations;

pub const RULE: &str = "relation.declaration.unusable";

/// An instance built with no phase-A report, which the instantiation never
/// produces for a check that declared the input. Recorded rather than passed,
/// because a rule that returned `Passed` here would report a clean block for a
/// document nothing looked at.
const NO_REPORT: &str = "the view carries no phase-A report for this document";

/// The kinds that some declared relation admits at either end.
///
/// The generation step of this rule and of [`crate::identity`], written once.
/// The two rules are the two halves of one question — whether a declared
/// relation becomes an edge — so a kind that one of them generated over and the
/// other did not would leave half of that question unasked on the same
/// document, and no report would say which half.
pub struct EndpointKinds<'a> {
    declared: Vec<&'a Relation>,
    shape: &'a Shape,
}

impl<'a> EndpointKinds<'a> {
    pub fn of(declarations: &'a Declarations, shape: &'a Shape) -> Self {
        EndpointKinds {
            declared: declarations.relations.iter().collect(),
            shape,
        }
    }

    /// Whether a document of this kind may stand at either end of any declared
    /// relation. Read through the `is_a` chain, because no document is an
    /// abstract kind and a comparison of two strings would admit none of them.
    ///
    /// An endpoint set that names an anchor kind matches nothing here, which is
    /// right: a document is never an external anchor.
    pub fn admit(&self, kind: &str) -> bool {
        self.declared.iter().any(|relation| {
            relation
                .from
                .iter()
                .chain(relation.to.iter())
                .any(|allowed| self.shape.descends_from(kind, allowed))
        })
    }
}

/// The check, generated from the relation declarations and the kind hierarchy.
pub struct Unusable<'a> {
    kinds: EndpointKinds<'a>,
}

impl<'a> Unusable<'a> {
    pub fn over(declarations: &'a Declarations, shape: &'a Shape) -> Self {
        Unusable {
            kinds: EndpointKinds::of(declarations, shape),
        }
    }
}

impl DocumentCheck for Unusable<'_> {
    const RULE: &'static str = self::RULE;
    /// See [`crate::placement::Placement::VERSION`].
    const VERSION: u32 = 1;
    /// The whole input of this rule. See the module comment for why it reaches
    /// the check through the view rather than beside it.
    const NEEDS_PHASE_A: bool = true;

    fn instantiates(&self, kind: &str) -> bool {
        self.kinds.admit(kind)
    }

    fn evaluate(&self, view: &DocumentView<'_>) -> Outcome {
        let Some(trouble) = view.phase_a() else {
            return Outcome::Skipped(NO_REPORT.to_string());
        };
        let findings = trouble
            .relations
            .iter()
            .filter_map(|reported| finding(reported))
            .collect();
        Outcome::failed(findings)
    }
}

/// One reported problem as a finding, or nothing where another rule answers.
///
/// The message is [`Problem`]'s own `Display`, so the graph's report and this
/// finding say one sentence about one defect. What differs per variant is the
/// repair, and the four repairs send an author to four different lines.
fn finding(reported: &Reported) -> Option<Finding> {
    let remediation = match &reported.problem {
        Problem::BlockNotAMapping(_) => {
            "write `relations:` as a mapping whose keys are relation names, because Q4 puts every \
             relation instance under one key and the relation is that key"
                .to_string()
        }
        Problem::UnknownRelation { name } => format!(
            "write a relation this taxonomy declares, or the inverse of one, in place of `{name}`, \
             or declare `{name}` in the taxonomy"
        ),
        Problem::EntryNotUsable { relation, .. } => format!(
            "write each entry of `{relation}` as a target reference, or as a mapping with `to:` \
             and the instance attributes beside it"
        ),
        Problem::NoTarget { relation } => format!(
            "give this entry of `{relation}` a `to:`, or delete it, because an entry that names no \
             target declares no edge and the attributes on it reach nothing"
        ),
        Problem::RepeatedTriple { relation, target } => format!(
            "delete one of the two entries of `{relation}` that name {target}. Q4 identifies an \
             edge by its source, its relation and its normalized target, so the second declares no \
             second edge, and the instance attributes on it are lost with it"
        ),
        // `crate::identity` answers for this one, because the repair is the
        // identity of the document rather than the block, and reporting it here
        // as well would send one author to two rules for one edit.
        Problem::SourceHasNoIdentifier => return None,
    };

    let (line, column) = at(reported.span);
    Some(Finding {
        rule: self::RULE,
        severity: Severity::Error,
        obligation: None,
        path: reported.path.clone(),
        line,
        column,
        message: reported.problem.to_string(),
        remediation,
        // See the module comment: three of the four repairs are a rewrite, and
        // the fourth is a choice between two entries that only the author can
        // make.
        fixable: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use headwater_yaml::{Position, Span};

    fn span() -> Option<Span> {
        Some(Span {
            start: Position {
                line: 7,
                col: 3,
                offset: 0,
            },
            end: Position {
                line: 7,
                col: 9,
                offset: 6,
            },
        })
    }

    fn reported(problem: Problem) -> Reported {
        Reported {
            path: "docs/spec/00-first.md".to_string(),
            span: span(),
            problem,
        }
    }

    /// Every way a `relations:` block fails has a repair of its own, and the
    /// one member another rule owns produces nothing here.
    ///
    /// The match in [`finding`] is exhaustive, so a new `Problem` variant is a
    /// compile error. This is the other half: that no two variants share a
    /// sentence, and that the partition with [`crate::identity`] is the one the
    /// module comment states.
    #[test]
    fn each_way_a_relations_block_fails_sends_its_author_to_a_different_line() {
        let cases = [
            (
                Problem::BlockNotAMapping("a sequence"),
                "write `relations:` as a mapping",
            ),
            (
                Problem::UnknownRelation {
                    name: "invented_relation".to_string(),
                },
                "in place of `invented_relation`",
            ),
            (
                Problem::EntryNotUsable {
                    relation: "traces_to".to_string(),
                    found: "a sequence",
                },
                "write each entry of `traces_to`",
            ),
            (
                Problem::NoTarget {
                    relation: "cites_evidence".to_string(),
                },
                "give this entry of `cites_evidence` a `to:`",
            ),
            (
                Problem::RepeatedTriple {
                    relation: "governs".to_string(),
                    target: "docs/spec/01-second.md".to_string(),
                },
                "delete one of the two entries of `governs`",
            ),
        ];

        let mut seen: Vec<String> = Vec::new();
        for (problem, expected) in cases {
            let text = reported(problem);
            let found = finding(&text).expect("a finding");
            assert!(found.remediation.contains(expected), "{found:#?}");
            assert_eq!(found.severity, Severity::Error, "{found:#?}");
            assert!(!found.fixable, "{found:#?}");
            // At the entry that declared it, which is the line an author edits.
            assert_eq!(found.line, 7, "{found:#?}");
            assert!(
                !seen.contains(&found.remediation),
                "two variants share one repair: {}",
                found.remediation
            );
            seen.push(found.remediation);
        }

        // The one the identity rule owns. A finding here as well would send one
        // author to two rules for one edit.
        assert!(finding(&reported(Problem::SourceHasNoIdentifier)).is_none());
    }

    /// The message is the graph's own sentence, so a reader who saw the build's
    /// report and a reader who saw the finding read one statement.
    #[test]
    fn the_message_is_the_sentence_the_graph_build_already_wrote() {
        let problem = Problem::UnknownRelation {
            name: "invented_relation".to_string(),
        };
        let text = problem.to_string();
        let found = finding(&reported(problem)).expect("a finding");
        assert_eq!(found.message, text);
    }
}
