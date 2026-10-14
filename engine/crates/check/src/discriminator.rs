// SPDX-License-Identifier: Apache-2.0
//! A Shape-origin check at corpus grain: a heterogeneous shelf reads a kind
//! from every document it claims.
//!
//! # The finding spec 2 states and nothing raised
//!
//! [Spec 2](../../../../docs/spec/02-taxonomy-model.md#kind-resolution), step
//! 3: on a heterogeneous shelf the kind comes from the discriminator facet,
//! and "a missing or unrecognized value is a finding". The census stops
//! resolution there and counts the document as untyped, which is correct. But
//! an untyped document has no kind, so no document-scoped template
//! instantiates over it, and [`crate::coverage`] reads typed rows alone. So a
//! review written as `doc_type: review_recrod` left every check, and a strict
//! run exited 0 over it (HW-OBL-0191).
//!
//! # Which stops this rule reads, and which it does not
//!
//! Three stops of step 3 are this finding: no discriminator, a value the shelf
//! does not admit, and a value that is not a scalar. A sequence names no kind,
//! so it is an unrecognized value in spec 2's words.
//!
//! The other stops are not this rule. A path that no shelf claims is an
//! ordinary state of a corpus, and this repository carries two. A file with no
//! front matter has no discriminator to read, and its census row says which
//! shelf claims it. A shelf tie and an abstract kind are defects of the
//! taxonomy rather than of the document, so a finding in the document would
//! send its author to repair a file that is not wrong.
//!
//! # Why the grain is the corpus
//!
//! An untyped document has no kind, and every document-scoped instance is
//! instantiated from a kind's template, so no document-grained instance
//! reaches it. The corpus grain does, on [`crate::duplicate`]'s terms: one
//! instance, routed to the corpus and not to a document, so the coverage
//! numbers do not move. The view hands this rule the stopped resolutions,
//! which the census already derived from the rows in the read set. See
//! [`crate::scope::CorpusCheck::NEEDS_UNRESOLVED`].
//!
//! # Severity, and the shelf that does not declare it
//!
//! Spec 2 says the shelf declares the severity of this finding. No shelf
//! schema gives that declaration a form, and this engine does not invent one,
//! for the reason `headwater_census::resolve` gives about step 4. So the
//! severity is fixed at error until a declaration gives it a form. Error is
//! the precedent of every other structural finding of phase A, and it is what
//! makes a strict run fail over a document that left every check.
//!
//! No fix. The correct kind is the author's judgment, and the census prints
//! the kinds the shelf admits.
//!
//! # Anchor
//!
//! A value that was read anchors at its own line. A missing discriminator has
//! no line in the document, and the derivation's span for it is the shelf's
//! span in the lock, so the finding anchors at the first line of the document.

use crate::finding::{at, Finding, Severity};
use crate::instance::Outcome;
use crate::scope::{CorpusCheck, CorpusView, Unresolved};
use headwater_census::resolve::Untyped;

pub const RULE: &str = "shelf.discriminator.unresolved";

const NO_ROWS: &str = "the view carries no stopped resolutions for this corpus";

/// The check. It reads no declaration of its own: the shelves and the kinds
/// they admit already decided the stop, and the census carries it.
pub struct Discriminator;

impl CorpusCheck for Discriminator {
    const RULE: &'static str = self::RULE;
    /// The first edition of this rule.
    const VERSION: u32 = 1;
    const NEEDS_UNRESOLVED: bool = true;

    fn evaluate(&self, view: &CorpusView<'_>) -> Outcome {
        let Some(unresolved) = view.unresolved() else {
            return Outcome::Skipped(NO_ROWS.to_string());
        };
        Outcome::failed(unresolved.iter().filter_map(finding).collect())
    }
}

/// One finding for a discriminator stop, and nothing for any other stop.
fn finding(row: &Unresolved<'_>) -> Option<Finding> {
    let (facet, permitted, span) = match row.reason {
        Untyped::MissingDiscriminator { facet } => (facet, None, None),
        Untyped::UnknownDiscriminator {
            facet, permitted, ..
        } => (facet, Some(permitted), row.span),
        Untyped::DiscriminatorNotAScalar { facet } => (facet, None, row.span),
        Untyped::NoShelf | Untyped::AmbiguousShelf { .. } | Untyped::AbstractKind { .. } => {
            return None
        }
    };
    let (line, column) = match span {
        Some(_) => at(span),
        None => (1, 1),
    };
    // The census reason's own sentence, so that the census row and the
    // finding cannot say two different things about one document.
    let message = format!(
        "{}, so no kind resolves and no check of a kind reaches this document",
        row.reason
    );
    let remediation = match permitted {
        Some(kinds) => format!(
            "set `{facet}` to one of the kinds this shelf admits: {}",
            kinds.join(", ")
        ),
        None => format!(
            "set `{facet}` to one kind this shelf admits, as one word; `headwater explain {}` \
             prints the shelf and its kinds",
            row.path
        ),
    };
    Some(Finding {
        rule: self::RULE,
        severity: Severity::Error,
        obligation: None,
        path: row.path.to_string(),
        line,
        column,
        message,
        remediation,
        patch: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use headwater_yaml::{Position, Span};

    fn span(line: usize) -> Option<Span> {
        Some(Span {
            start: Position {
                line,
                col: 1,
                offset: 0,
            },
            end: Position {
                line,
                col: 9,
                offset: 8,
            },
        })
    }

    fn row(reason: &Untyped, at: Option<Span>) -> Unresolved<'_> {
        Unresolved {
            path: "docs/reviews/probe.md",
            reason,
            span: at,
        }
    }

    fn evaluate(rows: Vec<Unresolved<'_>>) -> Vec<Finding> {
        match Discriminator.evaluate(&CorpusView::only_unresolved(Some(rows))) {
            Outcome::Failed(findings) => findings,
            Outcome::Passed => Vec::new(),
            other => panic!("{other:?}"),
        }
    }

    /// The reproduction of HW-OBL-0191 at the predicate: an unknown value is
    /// one error at its own line, and it names the value and the kinds.
    #[test]
    fn an_unknown_value_is_an_error_at_its_line() {
        let reason = Untyped::UnknownDiscriminator {
            facet: "doc_type".to_string(),
            value: "review_recrod".to_string(),
            permitted: vec!["review_prompt".to_string(), "review_record".to_string()],
        };
        let found = evaluate(vec![row(&reason, span(2))]);
        assert_eq!(found.len(), 1, "{found:#?}");
        assert_eq!(found[0].severity, Severity::Error);
        assert_eq!(found[0].line, 2);
        assert!(!found[0].fixable());
        assert!(found[0].message.contains(&reason.to_string()));
        assert!(found[0]
            .remediation
            .contains("review_prompt, review_record"));
    }

    /// A missing discriminator anchors at line 1, not at the shelf's line in
    /// the lock that the derivation recorded.
    #[test]
    fn a_missing_discriminator_anchors_at_the_first_line() {
        let reason = Untyped::MissingDiscriminator {
            facet: "doc_type".to_string(),
        };
        let found = evaluate(vec![row(&reason, span(40))]);
        assert_eq!(found.len(), 1, "{found:#?}");
        assert_eq!((found[0].line, found[0].column), (1, 1));
    }

    #[test]
    fn a_value_that_is_not_a_scalar_is_an_error() {
        let reason = Untyped::DiscriminatorNotAScalar {
            facet: "doc_type".to_string(),
        };
        let found = evaluate(vec![row(&reason, span(3))]);
        assert_eq!(found.len(), 1, "{found:#?}");
        assert_eq!(found[0].line, 3);
    }

    /// The stops that are not about the document's discriminator raise
    /// nothing from this rule.
    #[test]
    fn the_other_stops_are_not_this_rule() {
        for reason in [
            Untyped::NoShelf,
            Untyped::AmbiguousShelf {
                shelves: vec!["a".to_string(), "b".to_string()],
            },
            Untyped::AbstractKind {
                kind: "record".to_string(),
            },
        ] {
            assert!(evaluate(vec![row(&reason, span(2))]).is_empty(), "{reason}");
        }
    }

    /// A view that carries no stopped resolutions is a skip, not a pass.
    #[test]
    fn a_view_without_the_rows_skips() {
        assert!(matches!(
            Discriminator.evaluate(&CorpusView::only_unresolved(None)),
            Outcome::Skipped(_)
        ));
    }
}
