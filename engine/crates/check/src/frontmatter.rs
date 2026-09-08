// SPDX-License-Identifier: Apache-2.0
//! The prose a document writes in its front matter, for the rules that read it.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#the-five-origins-of-a-check)
//! held the Document origin to the body for two editions, and a `summary` is
//! prose that `headwater generate` publishes verbatim onto a shelf index. So the
//! most-read line of a document answered to no rule.
//!
//! # The population is a role, and one role
//!
//! `summary` is this repository's spelling of the facet in the `scent` role, and
//! an adopter picks another name for it. [`Shape::facet_in_role`] is the lookup,
//! and `headwater-meta` already refuses two facets in one role, so the name
//! resolves once at construction and no new declaration is needed.
//!
//! Every other role stays out. A facet in the `name` role is a title a reader
//! scans, it holds no verb, and a sentence-length rule over one reports a defect
//! whose only repair is a rename. Spec 12 states the whole ruling and the
//! condition that reopens it.
//!
//! # Every position is the position of the value
//!
//! A finding anchors at the start of the scalar the author wrote, and never at
//! an offset inside it. The loader hands over the text after it resolved escapes
//! and folded a block scalar, so an offset into that text is an offset into the
//! source exactly when neither happened. A line and a column of the front matter
//! is what a reader needs, and it is the part that survives both.
//!
//! # Nothing here writes a patch
//!
//! [`crate::patch::substitution`] replaces a byte range of a body under a
//! read-back that compares two parses of a body. No shape of this engine edits a
//! mapping, which [HW-OBL-0103](../../../../docs/obligations/0103-the-front-matter-half-of-a-patch-has-no-writer.md)
//! records. A caller of this module reports remediation prose and no patch.

use crate::scope::DocumentView;
use crate::shape::Shape;
use headwater_doc::Sentence;
use headwater_yaml::Span;

/// The role whose facet carries a sentence about the document.
pub const SCENT: &str = "scent";

/// The prose of one front-matter facet, and where the author wrote it.
pub struct Scent {
    /// The facet name, for the message that names where the defect is.
    pub facet: String,
    /// The span of the value, which every finding over it anchors at.
    pub span: Span,
    pub sentences: Vec<Sentence>,
}

/// The facet a shape puts in the `scent` role, and nothing when none does.
///
/// A corpus that declares no such facet loses nothing: a rule that resolves
/// `None` here reads the body alone, exactly as it did before.
pub fn scent_facet(shape: &Shape) -> Option<String> {
    shape.facet_in_role(SCENT).map(|facet| facet.name.clone())
}

impl Scent {
    /// The sentences of one document's scent facet.
    ///
    /// Nothing for a document that declares no such key, and nothing for one
    /// whose value is a sequence or a mapping: what that document owes is a
    /// scalar, and `facet.value.*` is where a defect of shape is reported.
    pub fn of(view: &DocumentView<'_>, facet: &str) -> Option<Scent> {
        let entry = view.facets().get(facet)?;
        let text = entry.value.as_scalar()?.text.as_str();
        if text.trim().is_empty() {
            return None;
        }
        // The same parse the body gets, so a code span in a summary is code
        // here too and the author-owned text is the parser's answer rather than
        // a second one. The offset is zero because no position of this parse
        // survives: see the module comment.
        let parsed = headwater_doc::body::scan(text, text, 0);
        Some(Scent {
            facet: facet.to_string(),
            span: entry.span,
            sentences: parsed.sentences(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shape::{Facet, Shape};
    use headwater_yaml::Position;

    fn facet(name: &str, role: Option<&str>) -> Facet {
        Facet {
            name: name.to_string(),
            role: role.map(str::to_string),
            value_type: None,
            required: false,
            values: Vec::new(),
            stale_after_days: None,
            span: Span::new(Position::new(1, 1, 0), Position::new(1, 1, 0)),
        }
    }

    /// The separation the ruling rests on. `summary` is read and `title` is not,
    /// and neither name is written into a rule.
    #[test]
    fn the_scent_role_names_the_facet_and_the_name_role_does_not() {
        let shape = Shape {
            facets: vec![facet("title", Some("name")), facet("summary", Some(SCENT))],
            ..Shape::default()
        };
        assert_eq!(scent_facet(&shape).as_deref(), Some("summary"));
    }

    /// A corpus that declares no facet in the role. The rules then read the body
    /// alone, which is what they did before this module existed.
    #[test]
    fn a_shape_with_no_scent_facet_names_nothing() {
        let shape = Shape {
            facets: vec![facet("title", Some("name")), facet("owner", None)],
            ..Shape::default()
        };
        assert_eq!(scent_facet(&shape), None);
    }
}
