// SPDX-License-Identifier: Apache-2.0
//! A Shape-origin check, generated from `identifier_schemes` and the kinds that
//! name them.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#the-five-origins-of-a-check)
//! lists "identifier pattern" third among the Shape-origin examples. A kind
//! declares `identifier: {scheme: spec_id}`, the scheme declares
//! `pattern: "{namespace}-SPEC-{slug}"` and `namespace: HW`, and the check is
//! that the document's identifier is a string the template admits. Nothing
//! below names a scheme or a kind.
//!
//! # The grammar is [`headwater_meta::identifier`], and there is one of it
//!
//! The three placeholder forms, what each one admits, and why a `{slug}` is
//! terminal are all stated there. This module reads a pattern through that one
//! reader and adds nothing of its own, which is what makes the shape a document
//! is held to and the shape `headwater new` mints one statement rather than two.
//! `taxonomy validate` reads the same module, and its identifier-integrity rule
//! decides disjointness out of the same segments.
//!
//! # What this check does not do
//!
//! **Allocation.** `minted-once` and `reconcile-first` say how an identifier is
//! issued, not what it looks like. Whether a `seq` collides, or a minted
//! identifier was reused, is a question about the whole corpus and about
//! documents that no longer exist. It is not this rule.
//!
//! **Disjointness.** Whether two schemes admit one string is a question about
//! the taxonomy and not about a document, and `taxonomy validate` answers it
//! before a lock is written. This rule holds one document to one scheme.
//!
//! **Absence.** A typed document that declares no identifier is
//! [`crate::identity`]'s, which reports it where it costs something: that
//! document is neither end of any edge. This check skips it, visibly, and the
//! skip names the front-matter key that was read. The two rules ask two
//! questions and read two declarations. This one needs a scheme to compare an
//! identifier against, and it generates over the kinds that name one. That one
//! asks whether a document can be named at all, and it generates over the kinds
//! that a relation admits.
//!
//! That second one is the whole of what this check does about the guess in
//! [`headwater_graph::Config`]. The key that holds an identifier is a parameter
//! and not a declaration, so this check takes the same parameter the graph
//! takes rather than writing `id` into a rule. Where the parameter finds
//! nothing, the outcome is a skip that quotes the key, and never a finding: a
//! rule that told an author their identifier was wrong, having looked under a
//! key nobody declared, would report the engine's assumption as the author's
//! defect.
//!
//! # Where the finding reports
//!
//! At the identifier's own key, which is the line an author edits. The
//! [`crate::scope::DocumentView`] carries the front matter with spans, so the
//! finding lands on the key rather than on the file.
//!
//! No fix is offered. The template says the shape of a replacement and never
//! its content: the correct `{slug}` for a document is a name somebody chooses,
//! and a `{seq:04d}` this engine picked would be an allocation, which is the
//! part of the specification this rule just said it does not implement.

use crate::finding::{at, Finding, Severity};
use crate::instance::Outcome;
use crate::scope::{DocumentCheck, DocumentView};
use crate::shape::Shape;
use headwater_meta::identifier::{Template, Unreadable};

pub const RULE: &str = "identifier.pattern.not_met";

/// The check, generated from `identifier_schemes` and the kinds that name them.
pub struct Identifier {
    /// Each kind that mints under a scheme, and what the grammar made of that
    /// scheme's pattern. Computed once: the generation step reads the taxonomy
    /// and the evaluation step reads one document.
    schemes: Vec<(String, String, Result<Template, Unreadable>)>,
    /// The front-matter key an identifier is read from. A parameter, taken from
    /// the same [`headwater_graph::Config`] the graph index takes it from. See
    /// the module comment for why this check does not write `id` into a rule.
    facet: String,
}

impl Identifier {
    /// The generation step, in full.
    pub fn over(shape: &Shape, facet: &str) -> Self {
        Identifier {
            schemes: shape
                .kinds
                .iter()
                .filter_map(|kind| {
                    let scheme = shape.identifier_scheme_of(&kind.name)?;
                    Some((
                        kind.name.clone(),
                        scheme.name.clone(),
                        Template::parse(&scheme.pattern, &scheme.namespace),
                    ))
                })
                .collect(),
            facet: facet.to_string(),
        }
    }

    fn scheme_of(&self, kind: &str) -> Option<&(String, String, Result<Template, Unreadable>)> {
        self.schemes.iter().find(|(known, _, _)| known == kind)
    }
}

impl DocumentCheck for Identifier {
    const RULE: &'static str = self::RULE;
    /// The first edition of this rule.
    const VERSION: u32 = 1;

    /// A kind that mints under no scheme generates no instance, on the terms
    /// [`crate::facet_required`] states. `specification` in the base package is
    /// one: it declares no `identifier`, so no document of it owes a shape.
    fn instantiates(&self, kind: &str) -> bool {
        self.scheme_of(kind).is_some()
    }

    fn evaluate(&self, view: &DocumentView<'_>) -> Outcome {
        let Some((_, name, template)) = self.scheme_of(view.kind()) else {
            return Outcome::Passed;
        };
        let template = match template {
            Ok(template) => template,
            Err(Unreadable(reason)) => {
                return Outcome::Skipped(format!("scheme `{name}`: {reason}"))
            }
        };

        let facet = &self.facet;
        let Some(entry) = view.facets().get(facet) else {
            return Outcome::Skipped(format!(
                "no `{facet}` facet, which is the key this engine reads an identifier from and which no declaration states"
            ));
        };
        let Some(scalar) = entry.value.as_scalar() else {
            return Outcome::Skipped(format!(
                "the `{facet}` facet is {}, and an identifier is a word",
                entry.value.kind_name()
            ));
        };
        let identifier = scalar.text.trim();

        let Some(reason) = template.refusal(identifier) else {
            return Outcome::Passed;
        };

        let (line, column) = at(view.facets().key_span(facet));
        Outcome::failed_with(Finding {
            rule: self::RULE,
            severity: Severity::Error,
            obligation: None,
            path: view.path().to_string(),
            line,
            column,
            message: format!(
                "`{identifier}` does not match `{}`, which scheme `{name}` declares: {reason}",
                template.render()
            ),
            remediation: format!(
                "write the identifier of {} in the form `{}`",
                view.path(),
                template.render()
            ),
            patch: None,
        })
    }
}
