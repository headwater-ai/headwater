// SPDX-License-Identifier: Apache-2.0
//! A Document-origin check: a fragment this document writes into itself.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#the-five-origins-of-a-check)
//! lists prose-link resolution among the Document-origin examples, and
//! [`headwater_graph::links`] left this half deliberately undone: "whether
//! `#q4--relation-storage` names a heading of the target is a Document check
//! ... a graph build that started checking headings would be the check layer
//! with no scope declaration."
//!
//! # One half of that sentence is reachable at this grain, and one is not
//!
//! A fragment with no path names a heading of **this** document, and this
//! document is what a `Document` scope carries. So that half is here, whole.
//!
//! A fragment on a path names a heading of **another** document, and no scope
//! this engine has carries one. `Document` carries one document. `Edge` carries
//! a relation instance, and a prose link is not a relation
//! ([Q4](../../../../docs/spec/09-decisions.md#q4--relation-storage)).
//! `Neighbourhood` carries the documents one *relation* away, and it carries
//! their identity rather than their body. The grain that would reach it is one
//! document and the documents its prose links reach, and inventing a fifth
//! grain is not this issue's work.
//!
//! [13 — Open obligations](../../../../docs/spec/13-open-obligations.md) carries
//! the rest, with the number: 1519 links of this corpus carry a fragment on a
//! path, against 277 that carry one alone.
//!
//! # Why the anchors are computed rather than read
//!
//! A Markdown heading has no anchor in the source. The anchor is what a
//! renderer derives from the heading text, so a check that resolves a fragment
//! has to derive it the same way, and two renderers do not agree. The rule
//! below is the one GitHub applies, because that is where this corpus is read,
//! and it is stated in one function so that an adopter reading a wrong verdict
//! finds one place to look.
//!
//! # Every instance says what it read
//!
//! A document that writes no fragment into itself **skips with that reason**
//! rather than passing. A pass would count the document as checked by a rule
//! that had nothing to check, and `coverage.document_unchecked` would then be
//! unable to fire anywhere: this rule generates over every kind, so a vacuous
//! pass here would route every classified document to a check before anything
//! was read.

use crate::finding::{Finding, Severity};
use crate::instance::Outcome;
use crate::scope::{DocumentCheck, DocumentView};

pub const RULE: &str = "link.fragment.unresolved";

/// The check. It reads no declaration, and the module comment says why: the
/// taxonomy language has no member that turns prose-link resolution on or off,
/// because [spec 1](../../../../docs/spec/01-conceptual-model.md#prose-links-are-not-relations)
/// makes it a property of a corpus rather than of a kind.
pub struct Fragments;

impl DocumentCheck for Fragments {
    const RULE: &'static str = self::RULE;
    /// The first edition of this rule.
    const VERSION: u32 = 1;
    const NEEDS_BODY: bool = true;

    fn evaluate(&self, view: &DocumentView<'_>) -> Outcome {
        let Some(body) = view.body() else {
            return Outcome::Passed;
        };
        // A quoted link belongs to another author, and an image is a reference
        // to an asset. Both are the graph build's rules, held to here so that
        // one corpus has one answer about what a prose link is.
        let fragments: Vec<&headwater_doc::Link> = body
            .links
            .iter()
            .filter(|link| !link.quoted && !link.image)
            .filter(|link| link.destination.starts_with('#') && link.destination.len() > 1)
            .collect();
        if fragments.is_empty() {
            return Outcome::Skipped("this document writes no fragment into itself".to_string());
        }

        let anchors = anchors(body);
        let findings = fragments
            .iter()
            .filter(|link| !anchors.contains(&link.destination[1..].to_lowercase()))
            .map(|link| Finding {
                rule: self::RULE,
                severity: Severity::Error,
                obligation: None,
                path: view.path().to_string(),
                line: link.span.start.line,
                column: link.span.start.col,
                message: format!(
                    "`{}` names no heading of this document",
                    link.destination
                ),
                remediation: format!(
                    "point it at a heading of {}, or write the heading it names",
                    view.path()
                ),
                // The heading an author meant is a guess among the headings
                // this document has, and spec 12 admits a fix only where one
                // outcome is derivable without judgment.
                fixable: false,
            })
            .collect();
        Outcome::failed(findings)
    }
}

/// Every anchor a renderer gives this document, in heading order.
///
/// A repeated heading takes a numeric suffix, first occurrence bare. Two
/// `## Consequences` headings therefore give `consequences` and
/// `consequences-1`, and a link to the second resolves.
fn anchors(body: &headwater_doc::Body) -> Vec<String> {
    let mut anchors: Vec<String> = Vec::new();
    for heading in body.headings() {
        let slug = slug(&heading.text());
        let seen = anchors
            .iter()
            .filter(|known| **known == slug || known.starts_with(&format!("{slug}-")))
            .count();
        anchors.push(match seen {
            0 => slug,
            n => format!("{slug}-{n}"),
        });
    }
    anchors
}

/// The anchor a renderer derives from a heading.
///
/// Lower case; every character that is not a letter, a digit, a space or a
/// hyphen removed; then each space becomes a hyphen. An em dash therefore
/// leaves the two spaces around it, and `Q5 — Voice checking depth` resolves as
/// `q5--voice-checking-depth`, which is what this corpus writes.
fn slug(text: &str) -> String {
    text.trim()
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_')
        .map(|c| if c == ' ' { '-' } else { c })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use headwater_doc::body::scan;

    #[test]
    fn an_em_dash_leaves_the_spaces_around_it() {
        assert_eq!(slug("Q5 — Voice checking depth"), "q5--voice-checking-depth");
        assert_eq!(slug("`$package.optional`"), "packageoptional");
        assert_eq!(slug("Two phases, and why the order matters"), "two-phases-and-why-the-order-matters");
    }

    #[test]
    fn a_repeated_heading_takes_a_numeric_suffix() {
        let body = scan("# One\n\n# One\n\n# One\n", "# One\n\n# One\n\n# One\n", 0);
        assert_eq!(anchors(&body), ["one", "one-1", "one-2"]);
    }
}
