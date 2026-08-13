// SPDX-License-Identifier: Apache-2.0
//! A Shape-origin check, generated from the facet and kind declarations.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#the-five-origins-of-a-check)
//! puts "required facet" first among the Shape-origin examples and rules that
//! Shape checks are "generated, not written": a new facet in the taxonomy
//! produces its checks with no code. Nothing below names a facet or a kind. The
//! set a document owes is [`crate::shape::Shape::required_facets`], and a
//! taxonomy that requires one more gets one more finding and this file does not
//! change.
//!
//! # One instance per document, and a list of findings inside it
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#instances-and-why-coverage-needs-them)
//! sets the grain in its own example: "`facet_required` is not one check but
//! 412 instances", against a corpus of 412 documents. So the instance is the
//! document and not the pair of a document and a facet.
//!
//! A document can omit four facets, and each omission is a separate line for
//! its author to write. Spec 12 gives a check the signature
//! `check(view, ctx) -> [Finding]` for exactly this, and
//! [`crate::instance::Outcome::Failed`] carries the list. A verdict that named
//! the first omission would make the other three cost one run each to find.
//!
//! # Where the finding reports, and why it offers no fix
//!
//! Against the document, with no line. There is no line: the finding is that a
//! key is absent, and the front matter block it is absent from is the whole
//! file's worth of context an author needs.
//!
//! No fix is offered, and that is [spec 12](../../../../docs/spec/12-check-layer.md#fixability)'s
//! bar rather than a gap. A fix is mechanical only when there is "one correct
//! outcome, derivable without judgment". The value of a missing `summary` is a
//! sentence somebody has to write, and a `status` this engine picked would be a
//! claim about the document that nobody made.
//!
//! # The scope
//!
//! [`DocumentCheck`] is the whole declaration: one document, and the front
//! matter without the body. The required set is computed once per kind when the
//! check is built, which is the generation step reading the taxonomy, and the
//! evaluation step then reads one document.

use crate::finding::{Finding, Severity};
use crate::instance::Outcome;
use crate::scope::{DocumentCheck, DocumentView, ExportTargets};
use crate::shape::Shape;

pub const RULE: &str = "facet.required.missing";

/// The check, generated from the facet and kind declarations.
pub struct Required {
    /// Each kind, and every facet a document of it owes. Computed once, because
    /// the generation step reads the taxonomy and the evaluation step reads one
    /// document.
    owed: Vec<(String, Vec<String>)>,
}

impl Required {
    /// The generation step, in full.
    pub fn over(shape: &Shape) -> Self {
        Required {
            owed: shape
                .kinds
                .iter()
                .map(|kind| (kind.name.clone(), shape.required_facets(&kind.name)))
                .filter(|(_, facets)| !facets.is_empty())
                .collect(),
        }
    }

    fn owed_by(&self, kind: &str) -> &[String] {
        self.owed
            .iter()
            .find(|(known, _)| known == kind)
            .map(|(_, facets)| facets.as_slice())
            .unwrap_or_default()
    }
}

impl DocumentCheck for Required {
    const RULE: &'static str = self::RULE;
    /// The first edition of this rule. Raise it when what the rule decides
    /// changes, because that is what invalidates the cached verdicts of the
    /// edition before it ([`crate::cache`]).
    const VERSION: u32 = 1;
    /// A required facet becomes a member of the `required` array of the kind
    /// that owes it, and a validator reports one error for each member that a
    /// document does not state. That is the same document and the same facet,
    /// and `engine/crates/generate/tests/differential.rs` measures it in both
    /// directions over a corpus that breaks the rule on purpose.
    const EXPORTABLE_AS: ExportTargets = &["jsonschema"];

    /// A kind that owes no facet generates no instance. An instance that could
    /// only ever pass counts a document as checked by a rule that had nothing
    /// to check, which is the accounting [`crate::placement`] states in full.
    fn instantiates(&self, kind: &str) -> bool {
        !self.owed_by(kind).is_empty()
    }

    fn evaluate(&self, view: &DocumentView<'_>) -> Outcome {
        let kind = view.kind();
        let findings = self
            .owed_by(kind)
            .iter()
            .filter(|facet| view.facets().entry(facet).is_none())
            .map(|facet| Finding {
                rule: self::RULE,
                severity: Severity::Error,
                obligation: None,
                path: view.path().to_string(),
                // The finding is that a key is absent, so there is no line it
                // is at. `0` is what the finding renderer prints as the path
                // alone, rather than sending a reader to a line that is not
                // the one they want.
                line: 0,
                column: 0,
                message: format!("`{kind}` requires the facet `{facet}`, and it is not declared"),
                remediation: format!(
                    "add `{facet}:` to the front matter of {}, with the value this document has",
                    view.path()
                ),
                fixable: false,
            })
            .collect();
        Outcome::failed(findings)
    }
}
