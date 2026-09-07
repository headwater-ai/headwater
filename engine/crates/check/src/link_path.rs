// SPDX-License-Identifier: Apache-2.0
//! A Graph-origin check at corpus grain: a prose link resolves to a file of the
//! repository.
//!
//! # The fact was already derived, and nothing read it
//!
//! [`headwater_graph::links`] binds every prose link of every document the
//! census read, and [`headwater_graph::links::Binding::is_broken`] is its own
//! word for a link that resolved to nothing. The graph build has always
//! reported those: the report prints the count, and beside it the citing
//! document, the line, the column and the path that is not there. What it has
//! never printed is a rule name, and no rule read the set, so a corpus whose
//! every citation of one document died when that document was renamed passed
//! `check --strict` at exit 0.
//!
//! So this rule computes nothing. It takes the set the build already produced
//! and turns each member into a [`Finding`]. A second reading of the links here
//! would be a second definition of what a prose link is, and the build's is the
//! one the report already prints.
//!
//! # This is the path half, and [`crate::fragment`] is neither half of it
//!
//! [`crate::fragment`] selects a link whose destination begins with `#`, which
//! is a link into the document that wrote it. Its population is by construction
//! links whose path resolves, because there is no path. This rule's population
//! is links whose path does **not** resolve. The two sets are disjoint, and a
//! destination that carries both a path and a fragment — `../spec/glossary.md#relation`
//! — belongs to this rule when the path is gone and to neither when it is not.
//! Whether a fragment names a heading of *another* document is a third
//! question, and
//! [HW-OBL-0077](../../../../docs/obligations/0077-a-fragment-on-a-path-needs-a-grain-that-no-scope-supplies.md)
//! records that it needs a grain no scope of this engine supplies. Nothing here
//! reads a fragment.
//!
//! # Why the corpus grain, and why it is not a document rule
//!
//! A dead link is written in one document, and the temptation is to report it
//! at that document's grain. The verdict is not a function of that document's
//! bytes. `../glossary.md` is a defect exactly when no file stands at that
//! path, so the input is the corpus, and a document-scoped instance would key
//! on the citing file alone: rename the target and the citing file is
//! untouched, so its key would not move and a cached pass would outlive the
//! rename that made it false. That is [`crate::duplicate`]'s argument, and it
//! reaches the same grain for the same reason — the defect is a relation
//! between two files, and no one file holds it.
//!
//! One instance, and its findings name the citing document each time, so an
//! author still reads the defect at the line they wrote.
//!
//! # What the read set covers, and the one case it does not
//!
//! A corpus-scoped instance reads every row of the census that carries a
//! document, and that is what makes the common case sound. A link into
//! `docs/` whose target is deleted loses a row from the read set, and a link
//! into `docs/` whose missing target is created gains one, so neither
//! direction can survive on a cached verdict.
//!
//! A target that is **not** a typed document is the case the read set does not
//! reach: a file outside the corpus root, and an untyped or excluded file
//! inside it. `binding_of` answers for both by asking the tree whether the file
//! exists, and no row of the read set carries that answer, so creating or
//! deleting one of those files leaves this instance's key where it was.
//! Closing it means putting those paths in the read set of a corpus-scoped
//! instance, which is machinery four rules share and a change of its own. It is
//! recorded rather than shipped here, and it is a false negative in one
//! direction and a stale finding in the other — never a wrong verdict about a
//! link whose target is a document of this corpus.
//!
//! # Severity, and the two messages
//!
//! Error, on the ground [`crate::fragment`] and [`crate::target`] already
//! stand on: both are reference-class rules, both declare a control whose
//! posture is advisory, and both emit [`Severity::Error`], because a reference
//! that resolves to nothing costs the reader the thing it was written to
//! reach and no count in any report shows the loss.
//!
//! [`headwater_graph::links::Binding::is_broken`] is two bindings and they are
//! two different repairs. `Missing` is a path that normalized and nothing is
//! there, and the repair is to repoint the link or restore the file.
//! `Unnormalizable` is a destination that will not become a repository path at
//! all — `../../..` climbing out above the root — and the repair is to rewrite
//! the destination. One message each, so a reader is not told to look for a
//! file that was never namable.
//!
//! No patch. Where exactly one file of the corpus carries the basename the
//! dead link names, repointing derives without judgment, and that is a fix this
//! engine could write. It is a separate correctness argument — basename
//! uniqueness, and the relative path recomputed from the citing document — and
//! it is filed rather than built here.

use crate::finding::{Finding, Severity};
use crate::instance::Outcome;
use crate::scope::{CorpusCheck, CorpusView};
use headwater_graph::links::{Binding, Link};

pub const RULE: &str = "link.path.unresolved";

/// A run whose scope did not admit the links has nothing to read, and it says
/// so rather than passing. [`crate::duplicate`]'s `NO_REPORT`, at the same
/// grain and for the same reason.
const NO_LINKS: &str = "the view carries no bound prose links for this corpus";

/// The check. It reads no declaration of its own, on [`crate::fragment`]'s
/// terms: no member of the taxonomy language turns prose-link resolution on or
/// off, because
/// [spec 1](../../../../docs/spec/01-conceptual-model.md#prose-links-are-not-relations)
/// makes it a property of a corpus rather than of a kind.
pub struct Paths;

impl CorpusCheck for Paths {
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

/// One finding per broken link, and nothing for a link that resolved.
fn finding(link: &Link) -> Option<Finding> {
    let (message, remediation) = match &link.binding {
        Binding::Missing { path } => (
            format!(
                "`{}` names no file of this repository: nothing stands at `{path}`",
                link.destination
            ),
            format!(
                "point it at a file that exists, or restore `{path}`. A document that moved keeps \
                 its identifier, so `headwater explain` finds where it went"
            ),
        ),
        Binding::Unnormalizable { why } => (
            format!(
                "`{}` does not name a path of this repository at all: {why}",
                link.destination
            ),
            "rewrite the destination as a path relative to the document that writes it, or as an \
             absolute address with a scheme"
                .to_string(),
        ),
        _ => return None,
    };
    Some(Finding {
        rule: self::RULE,
        severity: Severity::Error,
        obligation: None,
        path: link.source_path.clone(),
        line: link.span.start.line,
        column: link.span.start.col,
        message,
        remediation,
        // See the module comment: the repair is derivable only where the
        // corpus carries exactly one file of the target's basename, and that
        // is a fix of its own.
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

    fn link(destination: &str, binding: Binding) -> Link {
        Link {
            source_path: "docs/spec/01-conceptual-model.md".to_string(),
            destination: destination.to_string(),
            fragment: None,
            form: LinkForm::Inline,
            span: span(30, 5),
            binding,
        }
    }

    /// The finding is written at the line and column of the link, in the
    /// document that wrote it, and it names the path that is not there.
    #[test]
    fn a_missing_target_is_an_error_at_the_citing_line() {
        let found = finding(&link(
            "glossary-moved.md",
            Binding::Missing {
                path: "docs/spec/glossary-moved.md".to_string(),
            },
        ))
        .expect("a finding");
        assert_eq!(found.rule, self::RULE);
        assert_eq!(found.severity, Severity::Error);
        assert_eq!(found.path, "docs/spec/01-conceptual-model.md");
        assert_eq!(found.line, 30);
        assert_eq!(found.column, 5);
        assert!(found.message.contains("glossary-moved.md"), "{found:#?}");
        assert!(
            found.message.contains("docs/spec/glossary-moved.md"),
            "{found:#?}"
        );
        assert!(!found.fixable(), "{found:#?}");
    }

    /// A destination that will not become a repository path is a different
    /// repair, so it is a different sentence. A reader told to restore a file
    /// would look for one that was never namable.
    #[test]
    fn an_unnormalizable_destination_says_something_else() {
        let found = finding(&link(
            "../../../../outside.md",
            Binding::Unnormalizable {
                why: "climbs above the repository root".to_string(),
            },
        ))
        .expect("a finding");
        assert_eq!(found.severity, Severity::Error);
        assert!(
            found.message.contains("climbs above the repository root"),
            "{found:#?}"
        );
        assert!(!found.remediation.contains("restore"), "{found:#?}");
    }

    /// Every binding that resolved is nothing to this rule, the four of them
    /// enumerated rather than sampled: a link into the corpus, a link to a file
    /// outside it, a same-document fragment, and an address with a scheme.
    #[test]
    fn a_link_that_resolved_is_not_this_rule() {
        for binding in [
            Binding::Corpus {
                path: "docs/spec/glossary.md".to_string(),
                class: "typed",
                id: None,
            },
            Binding::Repository {
                path: "CLAUDE.md".to_string(),
            },
            Binding::SameDocument,
            Binding::External,
        ] {
            assert!(finding(&link("x", binding)).is_none());
        }
    }

    /// A view that carries no links skips with the reason, rather than passing.
    /// A rule that returned `Passed` there would report a corpus it never read
    /// as clean.
    #[test]
    fn a_view_with_no_links_skips_rather_than_passes() {
        let view = CorpusView::only_links(None);
        assert!(matches!(Paths.evaluate(&view), Outcome::Skipped(why) if why == NO_LINKS));
    }

    /// The whole set, and one finding per broken member of it.
    #[test]
    fn the_broken_links_of_the_view_become_the_findings() {
        let links = vec![
            link(
                "glossary-moved.md",
                Binding::Missing {
                    path: "docs/spec/glossary-moved.md".to_string(),
                },
            ),
            link("https://example.com", Binding::External),
            link(
                "../../../../outside.md",
                Binding::Unnormalizable {
                    why: "climbs above the repository root".to_string(),
                },
            ),
        ];
        let view = CorpusView::only_links(Some(&links));
        let Outcome::Failed(found) = Paths.evaluate(&view) else {
            panic!("two broken links are two findings");
        };
        assert_eq!(found.len(), 2, "{found:#?}");
    }

    /// A corpus whose links all resolve passes, which is what lets this rule be
    /// an error at all.
    #[test]
    fn a_corpus_with_no_broken_link_passes() {
        let links = vec![link("https://example.com", Binding::External)];
        let view = CorpusView::only_links(Some(&links));
        assert!(matches!(Paths.evaluate(&view), Outcome::Passed));
    }
}
