// SPDX-License-Identifier: Apache-2.0
//! A Shape-origin check, generated from `identifier_schemes` and the kinds that
//! name them.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#the-five-origins-of-a-check)
//! lists "identifier pattern" third among the Shape-origin examples. A kind
//! declares `identifier: {scheme: spec_id}`, the scheme declares
//! `pattern: "SPEC-{namespace}-{slug}"` and `namespace: HW`, and the check is
//! that the document's identifier is a string the template admits. Nothing
//! below names a scheme or a kind.
//!
//! # A template, and not a glob
//!
//! [Spec 2](../../../../docs/spec/02-taxonomy-model.md) writes three
//! placeholder forms and no others: `{namespace}`, `{slug}` and `{seq:04d}`.
//! `headwater_meta::pattern::Pattern` is the glob language a *shelf path* is
//! written in, where `*` and `**` mean runs of path segments, so it is the
//! wrong matcher for this and this module carries its own.
//!
//! The three forms mean three different things, and only one of them is a
//! constraint the specification states.
//!
//! * `{namespace}` is the declared `namespace`, exactly. That is the one
//!   lexical requirement the invariant core makes: an adopter "may change
//!   identifier patterns except the namespace"
//!   ([spec 2](../../../../docs/spec/02-taxonomy-model.md#the-core-is-semantic-not-lexical)),
//!   because a namespace is cheap at minting and unrecoverable once the
//!   identifier is in somebody else's ticket.
//! * `{seq:04d}` is exactly four decimal digits, and `{seq:0Nd}` is exactly N.
//!   The width is in the declaration, so the check reads it rather than
//!   assuming one.
//! * `{slug}` is a free token, and the specification says nothing else about
//!   it. So this check requires that it is present and not empty, and it
//!   requires nothing about its alphabet. A rule that admitted only lower-case
//!   words would be this engine inventing a lexical constraint that no
//!   declaration makes, over the one part of an identifier spec 2 leaves to the
//!   author.
//!
//! Matching is left to right, and each placeholder is bounded by what follows
//! it. `{namespace}` and `{seq:0Nd}` have a known width or a known value, so
//! they need no lookahead. `{slug}` does not, so it is read only in terminal
//! position: a template that puts a free token before another segment does not
//! say where the token ends, and two readings of `SPEC-HW-a-b` would be equally
//! defensible. Such a template makes the instance skip with the reason, rather
//! than pick one.
//!
//! # The prefix is a literal, and that is a gap this check reports around
//!
//! `SPEC-` is a run of literal characters inside the `pattern` string, and no
//! field names it. So this check can say that an identifier does not match its
//! template, and it can say which segment of the template stopped the match,
//! and it cannot say that two schemes mint disjoint identifiers — that is
//! `taxonomy validate`'s identifier-integrity rule, which recovers the prefix
//! by scanning the pattern for its leading literal.
//! [13 — Open obligations](../../../../docs/spec/13-open-obligations.md)
//! carries the meta-schema change that would make the prefix a declared field.
//!
//! # What this check does not do
//!
//! **Allocation.** `minted-once` and `reconcile-first` say how an identifier is
//! issued, not what it looks like. Whether a `seq` collides, or a minted
//! identifier was reused, is a question about the whole corpus and about
//! documents that no longer exist. It is not this rule.
//!
//! **Absence.** A typed document that declares no identifier is already
//! [`headwater_graph::index::Defect::NoIdentifier`], reported where it costs
//! something: that document is neither end of any edge. This check skips it,
//! visibly, and the skip names the front-matter key that was read.
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

pub const RULE: &str = "identifier.pattern.not_met";

/// One piece of a template, in declaration order.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Segment {
    /// Characters that appear as written. The scheme's prefix is one of these.
    Literal(String),
    /// `{namespace}`: the declared namespace, and no other string.
    Namespace,
    /// `{seq:0Nd}`: exactly N decimal digits.
    Sequence { width: usize },
    /// `{slug}`: a free, non-empty token, read to the end of the identifier.
    Slug,
}

impl Segment {
    /// What a report calls this segment when it is the one that failed.
    fn describe(&self, namespace: &str) -> String {
        match self {
            Segment::Literal(text) => format!("the literal `{text}`"),
            Segment::Namespace => format!("the namespace `{namespace}`"),
            Segment::Sequence { width } => format!("{width} digits"),
            Segment::Slug => "a slug".to_string(),
        }
    }
}

/// A template this check cannot read, and the reason a skip states.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Unreadable(String);

/// A parsed `pattern`, with the namespace it was declared beside.
#[derive(Clone, Debug)]
struct Template {
    segments: Vec<Segment>,
    namespace: String,
}

/// The outcome of matching one identifier against one template.
enum Match {
    Admitted,
    /// The segment that stopped the match, and what was left of the identifier
    /// when it did.
    Refused {
        at: Segment,
        rest: String,
    },
    /// Every segment matched and characters were left over.
    Trailing {
        rest: String,
    },
}

impl Template {
    /// Read a `pattern` into segments, or say why it cannot be read.
    fn parse(pattern: &str, namespace: &str) -> Result<Self, Unreadable> {
        let mut segments: Vec<Segment> = Vec::new();
        let mut literal = String::new();
        let mut rest = pattern;

        while let Some(open) = rest.find('{') {
            literal.push_str(&rest[..open]);
            let after = &rest[open + 1..];
            let Some(close) = after.find('}') else {
                return Err(Unreadable(format!(
                    "the pattern `{pattern}` opens a placeholder that it does not close"
                )));
            };
            let name = &after[..close];
            if !literal.is_empty() {
                segments.push(Segment::Literal(std::mem::take(&mut literal)));
            }
            segments.push(read_placeholder(name, pattern)?);
            rest = &after[close + 1..];
        }
        literal.push_str(rest);
        if !literal.is_empty() {
            segments.push(Segment::Literal(literal));
        }

        // A free token is bounded only by the end of the identifier. See the
        // module comment: a template that puts one before another segment does
        // not state where it ends.
        if let Some(position) = segments.iter().position(|part| part == &Segment::Slug) {
            if position + 1 != segments.len() {
                return Err(Unreadable(format!(
                    "the pattern `{pattern}` writes `{{slug}}` before another segment, and a free token has no stated end"
                )));
            }
        }
        if segments.is_empty() {
            return Err(Unreadable(
                "the scheme declares no `pattern`, so nothing states the shape of its identifiers"
                    .to_string(),
            ));
        }
        if namespace.is_empty() {
            return Err(Unreadable(
                "the scheme declares no `namespace`, and the namespace is the one part of a pattern that an overlay may not change".to_string(),
            ));
        }
        Ok(Template {
            segments,
            namespace: namespace.to_string(),
        })
    }

    /// The template as a person reads it, for the remediation line.
    fn render(&self) -> String {
        self.segments
            .iter()
            .map(|segment| match segment {
                Segment::Literal(text) => text.clone(),
                Segment::Namespace => self.namespace.clone(),
                Segment::Sequence { width } => "0".repeat(*width),
                Segment::Slug => "<slug>".to_string(),
            })
            .collect()
    }

    fn matches(&self, identifier: &str) -> Match {
        let mut rest = identifier;
        for segment in &self.segments {
            let consumed = match segment {
                Segment::Literal(text) => rest.strip_prefix(text.as_str()),
                Segment::Namespace => rest.strip_prefix(self.namespace.as_str()),
                Segment::Sequence { width } => digits(rest, *width),
                // Terminal by construction, and a free token is not empty.
                Segment::Slug => match rest.is_empty() {
                    true => None,
                    false => Some(""),
                },
            };
            match consumed {
                Some(remainder) => rest = remainder,
                None => {
                    return Match::Refused {
                        at: segment.clone(),
                        rest: rest.to_string(),
                    }
                }
            }
        }
        match rest.is_empty() {
            true => Match::Admitted,
            false => Match::Trailing {
                rest: rest.to_string(),
            },
        }
    }
}

/// Exactly `width` decimal digits at the front, and the rest.
///
/// Fixed width rather than "at least one digit", because that is what a
/// declaration writing `{seq:04d}` states. `DR-HW-42` is not the identifier
/// `DR-HW-0042` written short: it is a second string, and an index that held
/// both would resolve one name to two documents.
fn digits(text: &str, width: usize) -> Option<&str> {
    let head = text.get(..width)?;
    match head.bytes().all(|byte| byte.is_ascii_digit()) {
        true => Some(&text[width..]),
        false => None,
    }
}

/// One placeholder, in the three forms spec 2 writes.
fn read_placeholder(name: &str, pattern: &str) -> Result<Segment, Unreadable> {
    if name == "namespace" {
        return Ok(Segment::Namespace);
    }
    if name == "slug" {
        return Ok(Segment::Slug);
    }
    // `seq:04d`, and any width. The `d` is Python's format language, which is
    // what spec 2 borrowed the form from.
    if let Some(spec) = name.strip_prefix("seq:0") {
        if let Some(width) = spec.strip_suffix('d').and_then(|w| w.parse::<usize>().ok()) {
            if width > 0 {
                return Ok(Segment::Sequence { width });
            }
        }
    }
    Err(Unreadable(format!(
        "the pattern `{pattern}` writes the placeholder `{{{name}}}`, and spec 2 states `{{namespace}}`, `{{slug}}` and `{{seq:0Nd}}`"
    )))
}

/// The check, generated from `identifier_schemes` and the kinds that name them.
pub struct Identifier {
    /// Each kind that mints under a scheme, and what this module made of that
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

        let reason = match template.matches(identifier) {
            Match::Admitted => return Outcome::Passed,
            Match::Refused { at, rest } => match rest.is_empty() {
                true => format!("it ends before {}", at.describe(&template.namespace)),
                false => format!(
                    "`{rest}` is where {} was expected",
                    at.describe(&template.namespace)
                ),
            },
            Match::Trailing { rest } => format!("`{rest}` is left over at the end"),
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
            fixable: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn admits(pattern: &str, namespace: &str, identifier: &str) -> bool {
        let template = Template::parse(pattern, namespace).expect("a readable pattern");
        matches!(template.matches(identifier), Match::Admitted)
    }

    #[test]
    fn a_namespace_placeholder_is_the_declared_namespace_and_no_other_string() {
        assert!(admits("SPEC-{namespace}-{slug}", "HW", "SPEC-HW-glossary"));
        // The one lexical requirement the invariant core makes.
        assert!(!admits("SPEC-{namespace}-{slug}", "HW", "SPEC-XX-glossary"));
        assert!(!admits("SPEC-{namespace}-{slug}", "HW", "SPEC--glossary"));
    }

    #[test]
    fn a_sequence_placeholder_is_the_width_the_declaration_writes() {
        assert!(admits("DR-{namespace}-{seq:04d}", "ACME", "DR-ACME-0042"));
        // Not the same identifier written short. See `digits`.
        assert!(!admits("DR-{namespace}-{seq:04d}", "ACME", "DR-ACME-42"));
        assert!(!admits("DR-{namespace}-{seq:04d}", "ACME", "DR-ACME-00042"));
        assert!(!admits("DR-{namespace}-{seq:04d}", "ACME", "DR-ACME-004x"));
        assert!(admits("DR-{namespace}-{seq:03d}", "ACME", "DR-ACME-042"));
    }

    #[test]
    fn a_slug_is_a_free_token_and_this_check_states_nothing_about_its_alphabet() {
        assert!(admits("SPEC-{namespace}-{slug}", "HW", "SPEC-HW-two-words"));
        assert!(admits(
            "SPEC-{namespace}-{slug}",
            "HW",
            "SPEC-HW-Mixed_Case9"
        ));
        // Present and not empty is the whole of the requirement.
        assert!(!admits("SPEC-{namespace}-{slug}", "HW", "SPEC-HW-"));
    }

    #[test]
    fn the_literal_prefix_has_to_be_there() {
        assert!(!admits("SPEC-{namespace}-{slug}", "HW", "REG-HW-glossary"));
        assert!(!admits("SPEC-{namespace}-{slug}", "HW", "HW-glossary"));
    }

    /// A pattern this module cannot read is a skip with the reason in it, and
    /// never a finding against a document.
    #[test]
    fn a_pattern_this_check_cannot_read_says_so() {
        for (pattern, namespace) in [
            // A free token before another segment: two readings, and the
            // declaration does not choose.
            ("SPEC-{slug}-{namespace}", "HW"),
            // A placeholder spec 2 does not write.
            ("SPEC-{namespace}-{uuid}", "HW"),
            // A width the format language does not give.
            ("DR-{namespace}-{seq:d}", "HW"),
            ("DR-{namespace}-{seq:00d}", "HW"),
            // An unclosed placeholder.
            ("SPEC-{namespace", "HW"),
            // The namespace is required, and a scheme without one states
            // nothing this check can hold an identifier to.
            ("SPEC-{namespace}-{slug}", ""),
            // No pattern at all.
            ("", "HW"),
        ] {
            assert!(
                Template::parse(pattern, namespace).is_err(),
                "`{pattern}` read as a template"
            );
        }
    }

    /// The report quotes the template back with the namespace filled in, so a
    /// reader compares two strings rather than a string and a form.
    #[test]
    fn a_rendered_template_is_the_shape_an_identifier_takes() {
        let template = Template::parse("SPEC-{namespace}-{slug}", "HW").expect("a template");
        assert_eq!(template.render(), "SPEC-HW-<slug>");
        let template = Template::parse("DR-{namespace}-{seq:04d}", "ACME").expect("a template");
        assert_eq!(template.render(), "DR-ACME-0000");
    }
}
