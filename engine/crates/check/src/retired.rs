// SPDX-License-Identifier: Apache-2.0
//! A Document-origin check: the terms a language regime retired.
//!
//! # This rule is a closed decision, implemented
//!
//! [Q21](../../../../docs/spec/09-decisions.md) settled where a retirement
//! lives: "It belongs to the language regime and never to a voice regime, and
//! the reason is scope. A voice regime binds per kind, and its `narrative`
//! value exempts a kind entirely. A retired term is retired in a proposal as
//! much as in a specification."
//!
//! [Spec 2](../../../../docs/spec/02-taxonomy-model.md#the-language-regime-carries-the-terms-that-the-corpus-retired)
//! states the shape and the reason for it: "A judgment that a corpus no longer
//! uses a term is a promise about the text, on the same terms as a spelling
//! lexicon. Today that judgment exists only as prose and a diff, so no
//! mechanism inherits it. `retired_terms` is where it becomes data."
//!
//! # The replacement decides the posture, and spec 2 decides that
//!
//! "With one, the fix is a substitution, which meets the mechanical-and-total
//! bar of [spec 12](../../../../docs/spec/12-check-layer.md#fixability), and the
//! check offers a patch. Without one, the finding carries remediation prose and
//! no patch. A retired term is the usual case for the first shape. A retired
//! *framing* is the usual case for the second, and no lexicon repairs it for
//! the author."
//!
//! So severity here is read off the entry rather than fixed by the rule. An
//! entry with a replacement is an error, because the remediation is a
//! substitution. An entry without one is advisory, because the remediation is a
//! rewrite. Neither is a judgment this file makes about a particular word.
//!
//! # The list this rule moved
//!
//! `CLAUDE.md` carried the stock-phrase list in prose and `tools/ste-lint.py`
//! carried it as ten regular expressions. This repository's overlay now carries
//! it as data, which is the whole of what the move buys: a house that retires
//! one more phrase edits its taxonomy and no code.
//!
//! # What it reads
//!
//! [`headwater_doc::Sentence::authored`] and nothing else, which is the text
//! the parser says this author wrote as prose. A term inside a code span or a
//! block quote is somebody else's word, and
//! [spec 3](../../../../docs/spec/03-authoring-and-lifecycle.md#what-a-lexical-rule-gets-wrong-and-where-posture-comes-from)
//! puts both outside a lexical rule by construction.

use crate::finding::{Finding, Severity};
use crate::instance::Outcome;
use crate::scope::{DocumentCheck, DocumentView};
use crate::shape::Shape;

pub const RULE: &str = "language.retired_term.used";

/// The check, generated from the language regimes and the kinds that bind them.
pub struct Retired {
    bound: Vec<Bound>,
    /// The facet in the `scent` role, resolved once from the shape. A retired
    /// term is retired wherever this document's author wrote it, and
    /// [`crate::frontmatter`] states why the population is a role.
    scent: Option<String>,
}

struct Bound {
    kind: String,
    regime: String,
    terms: Vec<Term>,
}

#[derive(Clone)]
struct Term {
    /// Matched in lower case, at word boundaries.
    term: String,
    reason: String,
    replacement: Option<String>,
}

impl Retired {
    /// The generation step, in full.
    ///
    /// A regime that retires nothing generates no instance. That is the same
    /// absence [`crate::voice`] reports for a regime that forbids nothing: an
    /// instance over one would count a document as checked by a rule with
    /// nothing to check, and [spec 12](../../../../docs/spec/12-check-layer.md)
    /// makes coverage mean the opposite of that.
    pub fn over(shape: &Shape) -> Self {
        let mut bound = Vec::new();
        for kind in &shape.kinds {
            let Some(regime) = shape.language_of(&kind.name) else {
                continue;
            };
            if regime.retired_terms.is_empty() {
                continue;
            }
            bound.push(Bound {
                kind: kind.name.clone(),
                regime: regime.name.clone(),
                terms: regime
                    .retired_terms
                    .iter()
                    .map(|entry| Term {
                        term: entry.term.to_lowercase(),
                        reason: entry.reason.clone(),
                        replacement: entry.replacement.clone(),
                    })
                    .collect(),
            });
        }
        Retired {
            bound,
            scent: crate::frontmatter::scent_facet(shape),
        }
    }

    fn bound_to(&self, kind: &str) -> Option<&Bound> {
        self.bound.iter().find(|bound| bound.kind == kind)
    }
}

impl DocumentCheck for Retired {
    const RULE: &'static str = self::RULE;
    /// Edition two widens the population to the facet in the `scent` role. The
    /// list itself is data, so a taxonomy that retires one more term does not
    /// raise this: the resolved taxonomy is already part of what the cache keys
    /// on. The population is not data, and a warm cache would serve edition
    /// one's verdict over every document this rule already read.
    const VERSION: u32 = 2;
    const NEEDS_BODY: bool = true;

    fn instantiates(&self, kind: &str) -> bool {
        self.bound_to(kind).is_some()
    }

    fn evaluate(&self, view: &DocumentView<'_>) -> Outcome {
        let Some(bound) = self.bound_to(view.kind()) else {
            return Outcome::Passed;
        };
        let body = view.body();
        let scent = self
            .scent
            .as_deref()
            .and_then(|facet| crate::frontmatter::Scent::of(view, facet));

        let mut findings = Vec::new();
        let prose = body
            .into_iter()
            .flat_map(|body| body.sentences().into_iter().map(move |s| (s, Some(body))))
            .chain(
                scent
                    .iter()
                    .flat_map(|scent| scent.sentences.iter().cloned().map(|s| (s, None))),
            );
        for (sentence, in_body) in prose {
            // Where the finding points, and what it calls the text it read.
            // Front matter anchors at the start of the value the author wrote,
            // for the reason `crate::frontmatter` states.
            let (line, column, subject) = match (in_body, &scent) {
                (Some(_), _) => (
                    sentence.span.start.line,
                    sentence.span.start.col,
                    "this sentence".to_string(),
                ),
                (None, Some(scent)) => (
                    scent.span.start.line,
                    scent.span.start.col,
                    format!("the `{}` facet", scent.facet),
                ),
                // Unreachable: a sentence outside the body comes from the scent.
                (None, None) => (0, 0, "this sentence".to_string()),
            };
            let text = sentence.authored.to_lowercase();
            for term in &bound.terms {
                if !crate::voice::contains_word(&text, &term.term) {
                    continue;
                }
                // The term as the author wrote it, which is what a patch has to
                // replace and what decides the case the replacement takes. The
                // match above was over the lowered text, so an offset into it
                // is an offset into the authored text exactly when lowering
                // moved no byte. It does move one for a handful of characters
                // outside this corpus, and a patch is the irreversible half, so
                // the guard is a length rather than an assumption.
                let written = match text.len() == sentence.authored.len() {
                    false => None,
                    true => crate::voice::word_at(&text, &term.term)
                        .and_then(|at| sentence.authored.get(at..at + term.term.len()))
                        .map(str::to_string),
                };
                // A sentence of front matter reaches no patch shape at all. The
                // one this rule places replaces a byte range of a body, and
                // HW-OBL-0103 records that nothing of this engine edits a
                // mapping.
                let patch = match (&written, &term.replacement, in_body) {
                    (Some(written), Some(replacement), Some(body)) => crate::patch::substitution(
                        view.path(),
                        body,
                        &sentence,
                        written,
                        &crate::patch::matching_case(written, replacement),
                    ),
                    _ => None,
                };
                findings.push(Finding {
                    rule: self::RULE,
                    severity: match term.replacement {
                        Some(_) => Severity::Error,
                        None => Severity::Warn,
                    },
                    obligation: None,
                    path: view.path().to_string(),
                    line,
                    column,
                    message: format!(
                        "`{}` retires `{}`, and {subject} writes it: {}",
                        bound.regime, term.term, term.reason
                    ),
                    remediation: match (&term.replacement, in_body) {
                        (Some(replacement), Some(_)) => format!("write `{replacement}`"),
                        (Some(replacement), None) => format!(
                            "write `{replacement}`, by hand: no patch of this engine edits front matter"
                        ),
                        (None, _) => format!("rewrite the sentence without `{}`", term.term),
                    },
                    // Spec 2 decides this, and it is the one place in the engine
                    // where a taxonomy entry rather than a rule says whether the
                    // fix is mechanical. An entry with a replacement whose term
                    // this engine cannot place in the source carries the same
                    // remediation prose and no patch.
                    patch,
                });
            }
        }
        findings.sort_by_key(|finding| (finding.line, finding.column));
        Outcome::failed(findings)
    }
}
