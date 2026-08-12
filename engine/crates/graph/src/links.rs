// SPDX-License-Identifier: Apache-2.0
//! Prose links, bound to what they point at. None of them is an edge.
//!
//! [Q4](../../../../docs/spec/09-decisions.md#q4--relation-storage) cut the
//! annotated prose link, and [spec 1](../../../../docs/spec/01-conceptual-model.md#prose-links-are-not-relations)
//! states what is left: "The engine extracts every prose link and checks that
//! it resolves. No syntax promotes one into a relation." So this module
//! resolves, and it produces no edge whatever a link points at.
//!
//! # A quoted link belongs to another author
//!
//! `Link::quoted` marks a link inside a block quote. A quoted citation is
//! evidence of what a source said, not a claim this document makes. Binding one
//! would attribute a reference to the wrong document, and the advisory check
//! that Q4 promises — a prose link to a corpus document with no declared
//! relation — would then tell an author to declare a relation on somebody
//! else's sentence. Quoted links are counted and never bound.
//!
//! # A destination is a path, and it is bound as one
//!
//! Every link in this corpus is a relative path with a fragment. Q4 makes an
//! edge target an identifier and never a path, so a link cannot become an edge
//! by resolving: it resolves to a *document*, and the identifier of that
//! document is what a later check compares against the declared edges.
//!
//! # The fragment is kept and not resolved
//!
//! Whether `#q4--relation-storage` names a heading of the target is a Document
//! check ([#57](https://github.com/headwater-ai/headwater/issues/57)), and the
//! parser already hands over every heading it would read. This module keeps the
//! fragment on the record and stops there, because a graph build that started
//! checking headings would be the check layer with no scope declaration.

use crate::anchors::normalize;
use crate::index::Index;
use headwater_doc::LinkForm;
use headwater_yaml::Span;
use std::path::Path;

/// One prose link of one document, and what it points at.
#[derive(Clone, Debug)]
pub struct Link {
    pub source_path: String,
    /// The destination as the author wrote it.
    pub destination: String,
    /// The part after `#`, kept for the check that reads it.
    pub fragment: Option<String>,
    pub form: LinkForm,
    pub span: Span,
    pub binding: Binding,
}

/// What a prose link resolved to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Binding {
    /// A file the census walked. `class` is the census's own word for it, so a
    /// link into an excluded directory reads as excluded rather than as broken.
    Corpus {
        path: String,
        class: &'static str,
        id: Option<String>,
    },
    /// A file of the repository that no census row covers, because it lies
    /// outside the corpus root. `CLAUDE.md` and `tools/ste-lint.py` are this.
    Repository { path: String },
    /// The path normalizes and nothing is there. A defect in this link.
    Missing { path: String },
    /// The destination will not normalize into a repository path.
    Unnormalizable { why: String },
    /// A fragment with no path: a link inside this document.
    SameDocument,
    /// A URL or a mail address. Reaching it is a network call, and a check
    /// never makes one ([spec 0](../../../../docs/spec/00-vision-and-scope.md#non-negotiables)).
    External,
}

impl std::fmt::Display for Binding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Binding::Corpus { path, class, .. } => write!(f, "{path} ({class})"),
            Binding::Repository { path } => write!(f, "{path}, outside the corpus root"),
            Binding::Missing { path } => write!(f, "no `{path}` in the repository"),
            Binding::Unnormalizable { why } => write!(f, "{why}"),
            Binding::SameDocument => write!(f, "this document"),
            Binding::External => write!(f, "an address outside the repository"),
        }
    }
}

impl Binding {
    /// Whether this binding is a defect in the link.
    pub fn is_broken(&self) -> bool {
        matches!(
            self,
            Binding::Missing { .. } | Binding::Unnormalizable { .. }
        )
    }
}

/// How many links were skipped, and why. Counted so that a corpus cannot lose
/// links to a rule nobody sees.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Skipped {
    /// Inside a block quote, so another author wrote them.
    pub quoted: usize,
    /// An image is a reference to an asset rather than to a document. The
    /// binding rules above are about documents, and #57 owns whichever check
    /// reads an image path.
    pub images: usize,
}

/// Bind every prose link of every document the census read.
///
/// Every document, and not only the typed ones. A link in an untyped file still
/// points somewhere, and the file is in the corpus: leaving it out would make
/// the link report quietly smaller than the census.
pub fn bind(
    census: &headwater_census::census::Census,
    index: &Index,
    base: &Path,
) -> (Vec<Link>, Skipped) {
    let mut links = Vec::new();
    let mut skipped = Skipped::default();

    for row in &census.rows {
        let Some(document) = &row.document else {
            continue;
        };
        for link in &document.body.links {
            if link.quoted {
                skipped.quoted += 1;
                continue;
            }
            if link.image {
                skipped.images += 1;
                continue;
            }
            let (path, fragment) = split_fragment(&link.destination);
            links.push(Link {
                source_path: row.path.clone(),
                destination: link.destination.clone(),
                fragment,
                form: link.form,
                span: link.span,
                binding: binding_of(&row.path, path, index, base),
            });
        }
    }

    (links, skipped)
}

fn binding_of(source_path: &str, destination: &str, index: &Index, base: &Path) -> Binding {
    if destination.is_empty() {
        return Binding::SameDocument;
    }
    if is_external(destination) {
        return Binding::External;
    }

    // Relative to the directory of the document that wrote it, which is how a
    // reader's browser resolves it and therefore what the author meant.
    let directory = source_path
        .rsplit_once('/')
        .map(|(head, _)| head)
        .unwrap_or("");
    let joined = if directory.is_empty() {
        destination.to_string()
    } else {
        format!("{directory}/{destination}")
    };

    let path = match normalize(&joined) {
        Ok(path) => path,
        Err(why) => return Binding::Unnormalizable { why },
    };

    if let Some(entry) = index.by_path(&path) {
        return Binding::Corpus {
            path,
            class: entry.class,
            id: entry.id.clone(),
        };
    }
    if base.join(&path).exists() {
        return Binding::Repository { path };
    }
    Binding::Missing { path }
}

/// Split a destination into its path and its fragment.
fn split_fragment(destination: &str) -> (&str, Option<String>) {
    match destination.split_once('#') {
        Some((path, fragment)) => (path, Some(fragment.to_string())),
        None => (destination, None),
    }
}

/// Whether the destination leaves the repository.
///
/// A scheme is what decides it, rather than a list of known schemes: `mailto:`,
/// `https:` and a scheme nobody has invented yet all mean the same thing here,
/// which is that no file on this disk is the target.
fn is_external(destination: &str) -> bool {
    if destination.starts_with("//") {
        return true;
    }
    let Some((scheme, _)) = destination.split_once(':') else {
        return false;
    };
    !scheme.is_empty()
        && scheme
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.')
        && scheme.starts_with(|c: char| c.is_ascii_alphabetic())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fragment_is_split_off_and_kept() {
        assert_eq!(
            split_fragment("09-decisions.md#q4--relation-storage"),
            ("09-decisions.md", Some("q4--relation-storage".to_string()))
        );
        assert_eq!(split_fragment("#scope"), ("", Some("scope".to_string())));
    }

    #[test]
    fn a_scheme_is_what_makes_a_destination_external() {
        assert!(is_external("https://w3id.org/headwater/"));
        assert!(is_external("mailto:security@example.com"));
        assert!(is_external("//example.com/x"));
        assert!(!is_external("09-decisions.md"));
        assert!(!is_external("../evaluations/relation-storage.md"));
        // A Windows drive letter is one character, and a one-letter scheme is
        // legal, so this is the case where the rule is a judgment call. A path
        // is the reading that a corpus of Markdown wants.
        assert!(!is_external("./c:/x"));
    }
}
