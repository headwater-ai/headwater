// SPDX-License-Identifier: Apache-2.0
//! A Document-origin check: the sections a kind's contract requires.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#the-five-origins-of-a-check)
//! lists the section contract beside voice among the Document-origin examples,
//! and this one is generated the way a Shape check is. Nothing here names a
//! kind or a heading: the set comes from
//! [`crate::shape::Shape::required_sections`], which reads `sections.require`
//! along the `is_a` chain.
//!
//! # Why a mis-parsed heading is the failure this rule is measured by
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#the-correctness-roots) uses
//! this rule as its example of a silent pass: "a mis-parsed heading lets a
//! section contract hold with no finding anywhere". The headings come from the
//! CommonMark parse rather than from a scan for `#`, so a `# Decision` inside a
//! fenced block satisfies nothing, and a setext heading over `-----` satisfies
//! what it says.
//!
//! # A contract is about a heading, not about a level
//!
//! `sections: {require: [Context, Decision, Consequences]}` names three
//! headings and no depth. So the match is on the text of any heading at any
//! level, case-insensitively, and a document that writes `## Decision` where a
//! sibling writes `### Decision` satisfies the same contract. The meta-schema
//! declares no level, and a rule that assumed one would report a document that
//! obeys every declaration a taxonomy made.
//!
//! # Severity, and the fix it does not offer
//!
//! An error, on the terms [`crate::facet_required`] states: a required section
//! is either there or it is not, and no judgment decides which. What judgment
//! decides is the *content*, and that is why no patch comes with the finding.
//! [Spec 12](../../../../docs/spec/12-check-layer.md#fixability) names this case
//! among the ones a fix may never be offered for: "to rewrite a section to
//! satisfy a contract" is not mechanical, and an empty heading inserted to
//! silence a check is worse than the finding.

use crate::finding::{Finding, Severity};
use crate::instance::Outcome;
use crate::scope::{DocumentCheck, DocumentView};
use crate::shape::Shape;
use headwater_doc::body::Body;

pub const RULE: &str = "section.required.missing";

/// The check, generated from the section contracts.
pub struct Sections {
    /// Each kind, and every section a document of it owes.
    owed: Vec<(String, Vec<String>)>,
}

impl Sections {
    /// The generation step, in full.
    pub fn over(shape: &Shape) -> Self {
        Sections {
            owed: shape
                .kinds
                .iter()
                .map(|kind| (kind.name.clone(), shape.required_sections(&kind.name)))
                .filter(|(_, sections)| !sections.is_empty())
                .collect(),
        }
    }

    fn owed_by(&self, kind: &str) -> &[String] {
        self.owed
            .iter()
            .find(|(known, _)| known == kind)
            .map(|(_, sections)| sections.as_slice())
            .unwrap_or_default()
    }
}

impl DocumentCheck for Sections {
    const RULE: &'static str = self::RULE;
    /// The first edition of this rule.
    const VERSION: u32 = 1;
    /// The headings are in the body, so the body is declared.
    const NEEDS_BODY: bool = true;

    /// A kind whose contract requires no section generates no instance, on the
    /// accounting [`crate::facet_required`] states.
    fn instantiates(&self, kind: &str) -> bool {
        !self.owed_by(kind).is_empty()
    }

    fn evaluate(&self, view: &DocumentView<'_>) -> Outcome {
        let Some(body) = view.body() else {
            return Outcome::Passed;
        };
        let kind = view.kind();
        let owed = self.owed_by(kind).to_vec();
        let findings = absent_from_body(body, &owed)
            .into_iter()
            .map(|section| Finding {
                rule: self::RULE,
                severity: Severity::Error,
                obligation: None,
                path: view.path().to_string(),
                // A heading that is absent has no line, the way a facet that is
                // absent has none. The document is the context.
                line: 0,
                column: 0,
                message: format!(
                    "`{kind}` requires the section `{section}`, and no heading of this document says so"
                ),
                remediation: format!(
                    "add a `{section}` heading to {}, with the content the kind is for",
                    view.path()
                ),
                patch: None,
            })
            .collect();
        Outcome::failed(findings)
    }
}

/// The sections a contract requires that a parsed body does not carry, in the
/// order the contract states them.
///
/// The one copy of the match. `evaluate` above asks it of a document the census
/// classified `Typed`, and
/// [`headwater_generate`](https://docs.rs/headwater-generate) asks it of a body
/// an emitter composed, before that body becomes a file. A generated document
/// reaches no document-scoped rule, so the emitter is the only reader it has;
/// two copies of this comparison would let the two readers disagree about what
/// a heading satisfies.
pub fn absent_from_body(body: &Body, required: &[String]) -> Vec<String> {
    let headings: Vec<String> = body
        .headings()
        .filter(|heading| heading.quote_depth == 0)
        .map(|heading| normalized(&heading.text()))
        .collect();
    required
        .iter()
        .filter(|section| !headings.contains(&normalized(section)))
        .cloned()
        .collect()
}

/// The same question, asked of the whole text of a Markdown file.
///
/// The front matter is split off first, for the reason `fragment`'s anchor
/// reader states: a `---` fence read as body gives the line above it a setext
/// heading, and a heading set that is too large is a heading set that confirms a
/// contract nothing wrote. A source with no front matter is scanned whole, which
/// is what the fallback arm is for.
pub fn absent_from_source(source: &str, required: &[String]) -> Vec<String> {
    let body = match headwater_doc::split::split(source) {
        Ok(split) => headwater_doc::body::scan(source, split.body, split.body_offset),
        Err(_) => headwater_doc::body::scan(source, source, 0),
    };
    absent_from_body(&body, required)
}

/// A heading, reduced to what a contract names.
///
/// Case and surrounding space are not part of a section name. Everything else
/// is: a contract that names `Decision` is not satisfied by `Decisions`, and a
/// rule that stemmed the word would decide for an author what their heading
/// means.
fn normalized(text: &str) -> String {
    text.trim().to_lowercase()
}
