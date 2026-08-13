// SPDX-License-Identifier: Apache-2.0
//! A Graph-origin check: a document a relation may name declares an identifier
//! that names it.
//!
//! # The silent pass this closes
//!
//! [Spec 4](../../../../docs/spec/04-assurance-model.md#no-silent-passes-every-document-is-accounted-for)
//! exists to stop a run from reporting nothing about a document it never
//! checked. This is that failure one level below the census. A typed document
//! with eight entries under `relations:` and no identifier contributes **zero**
//! edges to the graph, because an edge is identified by its source. Every check
//! over those edges then reports nothing rather than something, and the
//! coverage account is unmoved: the document itself is classified, checked and
//! green. The census accounted for the file. Nothing accounted for its edges.
//!
//! The graph said so twice and neither report reached a rule.
//! [`headwater_graph::index::Defect::NoIdentifier`] is the document's side —
//! nothing can name it — and
//! [`headwater_graph::edges::Problem::SourceHasNoIdentifier`] is the block's
//! side, that everything it names is lost.
//! [`headwater_graph::index::Defect::NotAScalar`] is the same defect written a
//! second way: a sequence of two identifiers is a document that has not decided
//! which one it is, and the index carries neither.
//!
//! # One finding, because there is one repair
//!
//! All three say: mint an identifier for this document. Spec 4 reports a defect
//! to the person who can act on it, once, so this rule reports once and names
//! both consequences in the sentence. Two rules over one edit would put one
//! author on two lines of a report and leave them to work out that the second
//! goes away with the first.
//!
//! # What does not come with it, and why
//!
//! [`headwater_graph::index::Defect::Duplicate`] is the third member of that
//! enum and it belongs to another rule. It is a fact about **two** documents:
//! the graph reports it against the second one in census order, and which
//! document is second is a function of path order over the whole corpus. A
//! document-scoped instance reads one file, so its cache key names one file,
//! and a verdict keyed that way survives every edit to the other document — the
//! one that would settle it. That is not a rule with a missing message. It is a
//! rule at another grain, and the grain is the corpus: [`crate::duplicate`].
//!
//! # The generation step, and the kind that is asked for nothing
//!
//! Every kind that some declared relation admits at either end, through
//! [`crate::declaration::EndpointKinds`]. That is the declaration that makes
//! the question meaningful: an identifier is what lets a relation name a
//! document, so a kind that no relation admits anywhere is a kind this rule has
//! nothing to ask about.
//!
//! It is deliberately **not** "every kind that mints under an identifier
//! scheme", which is [`crate::identifier`]'s step. The two rules ask two
//! questions. That one asks whether an identifier matches the scheme its kind
//! names, and it needs a scheme to compare against. This one asks whether the
//! document can be named at all, and a kind may be an edge endpoint while its
//! taxonomy declares no scheme — the base package's `specification` is exactly
//! that. A rule generated from the scheme would report nothing about it.
//!
//! # Severity and fixability
//!
//! Error. An unnamed endpoint loses every edge at both of its ends, and the
//! loss is invisible in every count a report prints.
//!
//! No fix, and the reason is [`crate::identifier`]'s: to choose an identifier
//! is to allocate one, which is the part of spec 2 no rule in this engine
//! implements. A `{seq:04d}` that this engine picked would be an allocation
//! made without reading the corpus that already spent them.

use crate::declaration::EndpointKinds;
use crate::finding::{at, Finding, Severity};
use crate::instance::Outcome;
use crate::scope::{DocumentCheck, DocumentView};
use crate::shape::Shape;
use headwater_graph::edges::Problem;
use headwater_graph::index::{Defect, Reported};
use headwater_graph::Declarations;

pub const RULE: &str = "identifier.unusable";

/// As [`crate::declaration`].
const NO_REPORT: &str = "the view carries no phase-A report for this document";

/// The check, generated from the relation declarations and the kind hierarchy.
pub struct Identity<'a> {
    kinds: EndpointKinds<'a>,
    /// The front-matter key an identifier is read from. The same parameter
    /// [`crate::identifier`] takes, from the same
    /// [`headwater_graph::Config`], so that a corpus whose identifiers live
    /// under another key gets one answer from the index and the same answer
    /// from every rule that names the key in a repair.
    facet: String,
}

impl<'a> Identity<'a> {
    pub fn over(declarations: &'a Declarations, shape: &'a Shape, facet: &str) -> Self {
        Identity {
            kinds: EndpointKinds::of(declarations, shape),
            facet: facet.to_string(),
        }
    }
}

impl DocumentCheck for Identity<'_> {
    const RULE: &'static str = self::RULE;
    /// See [`crate::placement::Placement::VERSION`].
    const VERSION: u32 = 1;
    /// See [`crate::declaration`]: the report the build wrote, for this
    /// document and no other.
    const NEEDS_PHASE_A: bool = true;

    fn instantiates(&self, kind: &str) -> bool {
        self.kinds.admit(kind)
    }

    fn evaluate(&self, view: &DocumentView<'_>) -> Outcome {
        let Some(trouble) = view.phase_a() else {
            return Outcome::Skipped(NO_REPORT.to_string());
        };

        // Whether this document's own `relations:` block lost its source. It is
        // the cost of the same defect rather than a second one, so it changes
        // the sentence and never the number of findings.
        let sourceless = trouble
            .relations
            .iter()
            .any(|reported| matches!(reported.problem, Problem::SourceHasNoIdentifier));

        let mut findings: Vec<Finding> = trouble
            .identity
            .iter()
            .filter_map(|reported| finding(reported, sourceless, &self.facet))
            .collect();

        // The build reports `SourceHasNoIdentifier` only where the index also
        // reported why, so the list above already carries it. If that ever
        // stops holding, the member is reported here rather than dropped: a
        // defect that two components each expect the other to report is the
        // silent pass this rule exists to close.
        if sourceless && findings.is_empty() {
            findings.push(Finding {
                rule: self::RULE,
                severity: Severity::Error,
                obligation: None,
                path: view.path().to_string(),
                line: 0,
                column: 0,
                message: Problem::SourceHasNoIdentifier.to_string(),
                remediation: self.mint(),
                fixable: false,
            });
        }

        Outcome::failed(findings)
    }
}

impl Identity<'_> {
    /// What to do, which is one sentence for all three defects because it is
    /// one edit.
    fn mint(&self) -> String {
        format!(
            "write one identifier of this document under the `{}` key, because a relation names a \
             document by its identifier and a document with none can stand at neither end of one",
            self.facet
        )
    }
}

/// One reported defect as a finding, or nothing for the one this rule does not
/// own.
fn finding(reported: &Reported, sourceless: bool, facet: &str) -> Option<Finding> {
    match &reported.defect {
        Defect::NoIdentifier { .. } | Defect::NotAScalar => {}
        // See the module comment: a duplicate is a fact about two documents,
        // this instance read one of them, and [`crate::duplicate`] reads both.
        Defect::Duplicate { .. } => return None,
    }

    let (line, column) = at(reported.span);
    Some(Finding {
        rule: self::RULE,
        severity: Severity::Error,
        obligation: None,
        path: reported.path.clone(),
        line,
        column,
        message: format!("{}{}", reported.defect, cost(sourceless)),
        remediation: format!(
            "write one identifier of this document under the `{facet}` key, because a relation \
             names a document by its identifier and a document with none can stand at neither end \
             of one"
        ),
        fixable: false,
    })
}

/// What the document lost beside its name, where it declared edges.
///
/// The clause is here rather than in a second finding, because the second
/// report would go away with the first edit and a reader cannot tell that from
/// two defects.
fn cost(sourceless: bool) -> &'static str {
    match sourceless {
        true => {
            ", and every entry of its `relations:` block is lost with it, because an edge is \
                 identified by its source"
        }
        false => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use headwater_yaml::{Position, Span};

    fn reported(defect: Defect) -> Reported {
        Reported {
            path: "docs/spec/03-anonymous.md".to_string(),
            span: Some(Span {
                start: Position {
                    line: 2,
                    col: 1,
                    offset: 0,
                },
                end: Position {
                    line: 2,
                    col: 3,
                    offset: 2,
                },
            }),
            defect,
        }
    }

    /// The two defects this rule owns, the one it does not, and what the block
    /// side adds to the sentence.
    #[test]
    fn a_document_that_cannot_be_named_is_one_finding_that_states_what_it_costs() {
        for defect in [
            Defect::NoIdentifier {
                kind: "design_spec".to_string(),
            },
            Defect::NotAScalar,
        ] {
            let bare = finding(&reported(defect.clone()), false, "id").expect("a finding");
            assert_eq!(bare.severity, Severity::Error, "{bare:#?}");
            assert!(!bare.fixable, "{bare:#?}");
            assert_eq!(bare.line, 2, "{bare:#?}");
            assert!(bare.remediation.contains("`id` key"), "{bare:#?}");
            assert!(!bare.message.contains("relations"), "{bare:#?}");

            // The same defect on a document that declared edges says what the
            // edges cost, in one finding rather than two.
            let laden = finding(&reported(defect), true, "id").expect("a finding");
            assert!(
                laden
                    .message
                    .contains("every entry of its `relations:` block is lost"),
                "{laden:#?}"
            );
            assert_eq!(laden.remediation, bare.remediation);
        }

        // A duplicate belongs to another grain, and the module comment says
        // why. `crate::duplicate` is the rule that reads both claimants.
        assert!(finding(
            &reported(Defect::Duplicate {
                id: "SPEC-FIX-first".to_string(),
                other: "docs/spec/00-first.md".to_string(),
                other_span: None,
            }),
            false,
            "id"
        )
        .is_none());
    }

    /// The repair names the key this engine reads an identifier from, and the
    /// key is a parameter rather than a word written into this rule.
    #[test]
    fn the_repair_names_the_configured_identifier_key() {
        let found = finding(&reported(Defect::NotAScalar), false, "identifier").expect("a finding");
        assert!(found.remediation.contains("`identifier` key"), "{found:#?}");
    }
}
