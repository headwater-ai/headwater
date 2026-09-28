// SPDX-License-Identifier: Apache-2.0
//! A Graph-origin check at corpus grain: a prose link whose text is an
//! identifier reaches the document that carries that identifier.
//!
//! # The defect the other two link rules cannot see
//!
//! `[HW-DR-0089](0090-….md)` names one decision and reaches another. The path
//! resolves, so [`crate::link_path`] is silent, and it carries no fragment, so
//! [`crate::fragment`] is silent too. A reader who follows the link reads the
//! wrong decision and has no signal that it is wrong. The text says what the
//! author meant and the destination says where the reader goes, and nothing
//! compared the two.
//!
//! # This rule computes nothing either
//!
//! [`headwater_graph::links::bind`] keeps the text of each link and asks the
//! index whether that text is exactly an identifier the corpus holds. It
//! records the answer as [`Link::names`]. The binding already carries the
//! identifier of the file the path reaches. This rule compares the two, and
//! nothing else: no second pattern for the identifier schemes lives here, so
//! a scheme the taxonomy adds is read the day it is declared.
//!
//! # What it does not read
//!
//! Text that holds an identifier among other words (`HW-DR-0089 clause 3`),
//! text with the shape of an identifier that names no document, and a target
//! that carries no identifier. Each is a different question with a different
//! false-positive rate, and none is read here. A link inside a block quote is
//! skipped by the bind, as it is for every link rule.
//!
//! A link into the same document (`#context`) reaches the document that wrote
//! it, so it is compared against that document's own identifier. The form of
//! the link does not matter: an inline link and a reference-style link bind
//! the same way.
//!
//! # Why the corpus grain
//!
//! [`crate::link_path`]'s argument, unchanged: a renumbering of the target
//! changes no byte of the citing document, so a document-scoped key would keep
//! a cached pass that the renumbering made false.
//!
//! # Severity, and why there is no patch
//!
//! Error, on the ground [`crate::link_path`] stands on: a reference that
//! reaches the wrong thing costs the reader what it was written to reach, and
//! no count shows the loss. No patch, because which side is wrong, the text or
//! the path, is a judgment the engine cannot make.

use crate::finding::{Finding, Severity};
use crate::instance::Outcome;
use crate::scope::{CorpusCheck, CorpusView};
use headwater_graph::links::{Binding, Link};

pub const RULE: &str = "link.identifier.mismatch";

/// [`crate::link_path`]'s `NO_LINKS`, at the same grain and for the same
/// reason.
const NO_LINKS: &str = "the view carries no bound prose links for this corpus";

/// The check. It reads no declaration of its own, on [`crate::link_path`]'s
/// terms.
pub struct Identifiers;

impl CorpusCheck for Identifiers {
    const RULE: &'static str = self::RULE;
    /// The first edition of this rule.
    const VERSION: u32 = 1;
    const NEEDS_LINKS: bool = true;

    fn evaluate(&self, view: &CorpusView<'_>) -> Outcome {
        let Some(links) = view.links() else {
            return Outcome::Skipped(NO_LINKS.to_string());
        };
        Outcome::failed(links.iter().filter_map(finding).collect())
    }
}

/// One finding for a link whose text names one identifier and whose path
/// reaches a document that carries another.
fn finding(link: &Link) -> Option<Finding> {
    let named = link.names.as_deref()?;
    // A link into the same document reaches the document that wrote it, so
    // the identifier it reaches is the citing document's own.
    let (path, reached) = match &link.binding {
        Binding::Corpus {
            path,
            id: Some(reached),
            ..
        } => (path.as_str(), reached.as_str()),
        Binding::SameDocument => (link.source_path.as_str(), link.source_id.as_deref()?),
        _ => return None,
    };
    if named == reached {
        return None;
    }
    Some(Finding {
        rule: self::RULE,
        severity: Severity::Error,
        obligation: None,
        path: link.source_path.clone(),
        line: link.span.start.line,
        column: link.span.start.col,
        message: format!(
            "the link text names `{named}`, but `{}` reaches `{path}`, which is `{reached}`",
            link.destination
        ),
        remediation: format!(
            "point the link at the document that carries `{named}`, or change the text to \
             `{reached}`. `headwater explain {named}` names where `{named}` lives"
        ),
        // See the module comment: which side is wrong is a judgment.
        patch: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use headwater_doc::LinkForm;
    use headwater_yaml::{Position, Span};

    fn span(line: usize, col: usize) -> Span {
        Span {
            start: Position {
                line,
                col,
                offset: 0,
            },
            end: Position {
                line,
                col: col + 1,
                offset: 1,
            },
        }
    }

    fn link(destination: &str, text: &str, names: Option<&str>, binding: Binding) -> Link {
        Link {
            source_path: "docs/spec/12-check-layer.md".to_string(),
            destination: destination.to_string(),
            fragment: destination.split_once('#').map(|(_, f)| f.to_string()),
            text: text.to_string(),
            names: names.map(str::to_string),
            source_id: Some("HW-DR-0002".to_string()),
            form: LinkForm::Inline,
            span: span(40, 3),
            binding,
        }
    }

    fn reaches(id: Option<&str>) -> Binding {
        Binding::Corpus {
            path: "docs/decisions/0090-other.md".to_string(),
            class: "typed",
            id: id.map(str::to_string),
        }
    }

    /// The case the issue names: the text says one decision and the path
    /// reaches another. One error, at the link, naming both identifiers and
    /// the path.
    #[test]
    fn text_naming_one_identifier_and_a_path_reaching_another_is_an_error() {
        let found = finding(&link(
            "../decisions/0090-other.md",
            "HW-DR-0089",
            Some("HW-DR-0089"),
            reaches(Some("HW-DR-0090")),
        ))
        .expect("a finding");
        assert_eq!(found.rule, self::RULE);
        assert_eq!(found.severity, Severity::Error);
        assert_eq!(found.path, "docs/spec/12-check-layer.md");
        assert_eq!(found.line, 40);
        assert_eq!(found.column, 3);
        assert!(found.message.contains("HW-DR-0089"), "{found:#?}");
        assert!(found.message.contains("HW-DR-0090"), "{found:#?}");
        assert!(
            found.message.contains("docs/decisions/0090-other.md"),
            "{found:#?}"
        );
        assert!(!found.fixable(), "{found:#?}");
    }

    /// A fragment on the destination does not change what the path reaches.
    #[test]
    fn a_fragment_on_a_mismatched_link_is_still_an_error() {
        assert!(finding(&link(
            "../decisions/0090-other.md#decision",
            "HW-DR-0089",
            Some("HW-DR-0089"),
            reaches(Some("HW-DR-0090")),
        ))
        .is_some());
    }

    /// Text and target agree.
    #[test]
    fn text_naming_the_identifier_it_reaches_is_nothing() {
        assert!(finding(&link(
            "../decisions/0090-other.md",
            "HW-DR-0090",
            Some("HW-DR-0090"),
            reaches(Some("HW-DR-0090")),
        ))
        .is_none());
    }

    /// Text that is not an identifier is not read.
    #[test]
    fn text_that_names_no_identifier_is_nothing() {
        assert!(finding(&link(
            "../decisions/0090-other.md",
            "the decision",
            None,
            reaches(Some("HW-DR-0090")),
        ))
        .is_none());
    }

    /// Every binding that carries no identifier is nothing to this rule: a
    /// missing path is `link.path.unresolved`'s, and the others reach nothing
    /// that has an identifier to compare.
    #[test]
    fn a_same_document_link_reaches_the_citing_documents_own_identifier() {
        // The helper writes the link in HW-DR-0002.
        let found = finding(&link(
            "#context",
            "HW-DR-0001",
            Some("HW-DR-0001"),
            Binding::SameDocument,
        ))
        .expect("text naming another document on a link into this one");
        assert!(found.message.contains("HW-DR-0001"), "{found:#?}");
        assert!(found.message.contains("HW-DR-0002"), "{found:#?}");
        assert!(finding(&link(
            "#context",
            "HW-DR-0002",
            Some("HW-DR-0002"),
            Binding::SameDocument,
        ))
        .is_none());
        // A citing document with no identifier has nothing to compare.
        let mut unnamed = link(
            "#context",
            "HW-DR-0001",
            Some("HW-DR-0001"),
            Binding::SameDocument,
        );
        unnamed.source_id = None;
        assert!(finding(&unnamed).is_none());
    }

    #[test]
    fn a_target_with_no_identifier_is_nothing() {
        for binding in [
            Binding::Missing {
                path: "docs/decisions/0089-gone.md".to_string(),
            },
            reaches(None),
            Binding::Repository {
                path: "CLAUDE.md".to_string(),
            },
            Binding::External,
            Binding::Unnormalizable {
                why: "climbs above the repository root".to_string(),
            },
        ] {
            assert!(finding(&link("x", "HW-DR-0089", Some("HW-DR-0089"), binding)).is_none());
        }
    }

    /// A view that carries no links skips with the reason, rather than passing.
    #[test]
    fn a_view_with_no_links_skips_rather_than_passes() {
        let view = CorpusView::only_links(None);
        assert!(matches!(Identifiers.evaluate(&view), Outcome::Skipped(why) if why == NO_LINKS));
    }

    /// The whole set, and one finding per mismatched member of it.
    #[test]
    fn the_mismatched_links_of_the_view_become_the_findings() {
        let links = vec![
            link(
                "a.md",
                "HW-DR-0089",
                Some("HW-DR-0089"),
                reaches(Some("HW-DR-0090")),
            ),
            link(
                "b.md",
                "HW-DR-0090",
                Some("HW-DR-0090"),
                reaches(Some("HW-DR-0090")),
            ),
            link(
                "c.md",
                "HW-DR-0001",
                Some("HW-DR-0001"),
                reaches(Some("HW-DR-0002")),
            ),
        ];
        let view = CorpusView::only_links(Some(&links));
        let Outcome::Failed(found) = Identifiers.evaluate(&view) else {
            panic!("two mismatched links are two findings");
        };
        assert_eq!(found.len(), 2, "{found:#?}");
    }

    /// A corpus whose links all agree passes.
    #[test]
    fn a_corpus_with_no_mismatch_passes() {
        let links = vec![link(
            "b.md",
            "HW-DR-0090",
            Some("HW-DR-0090"),
            reaches(Some("HW-DR-0090")),
        )];
        let view = CorpusView::only_links(Some(&links));
        assert!(matches!(Identifiers.evaluate(&view), Outcome::Passed));
    }
}
