// SPDX-License-Identifier: Apache-2.0
//! A Document-origin check: the text the scaffolder writes for a person to
//! replace, still in the document.
//!
//! `headwater new` and the MCP `new` tool write a document that passes every
//! other rule, because the headings a kind requires are there and every facet
//! carries a value. Where the scaffolder cannot determine the content, it
//! writes a prompt instead: [`SECTION_PROMPT`] under each heading, and
//! [`SCENT_PROMPT`] or [`facet_prompt`] as a facet value. Before this rule, a
//! document that still held those prompts passed `headwater check --strict`
//! with no finding that named the file
//! ([HW-OBL-0113](../../../../docs/obligations/0113-every-check-passes-a-document-that-is-still-the-scaffolder-s-placeholder.md)).
//!
//! # One source for the strings
//!
//! The scaffolder reads the three prompts from this module, because the
//! scaffold crate depends on this one and not the other way round. A prompt
//! that the scaffolder writes and this rule does not know would bring back
//! the silent pass, and `headwater-scaffold`'s pipeline test asserts that the
//! rule names each prompt a scaffolded document carries.
//!
//! # What matches, and what does not
//!
//! The match is exact, so detection is total and no judgment decides it:
//!
//! - a paragraph whose whole text, trimmed, is [`SECTION_PROMPT`], and every
//!   run of which the document's own author wrote. The prompt inside a fenced
//!   block, an inline code span or a block quotation is a quotation of the
//!   prompt. A paragraph that goes on after the prompt, such as
//!   `TODO write this section. The pool holds eight connections.`, is a
//!   section somebody started to write. A list item or a heading that reads
//!   the prompt is not where the scaffolder writes it. None of these is
//!   reported.
//! - a top-level scalar facet whose value is [`SCENT_PROMPT`], or is
//!   [`facet_prompt`] of its own key.
//!
//! The neighbor rule that this one is not is "a paragraph that starts with
//! `TODO`". An authored `TODO: pick the pool size before launch.` is an
//! author's note, and this rule does not read it.
//!
//! # Every lifecycle state, `draft` included
//!
//! The measured case in HW-OBL-0113 is a draft that the MCP `new` tool wrote.
//! A rule that skipped `draft` would skip the case it exists for.
//!
//! # A warning, with no fix
//!
//! [Spec 12](../../../../docs/spec/check-layer.md#fixability) makes a defect
//! an error when its remedy is mechanical. The remedy here is to write the
//! section or the summary, and that is a rewrite, so the finding is advisory.
//! [`crate::sections`] reads the opposite way, and the difference is the
//! remedy and not the detection: a missing heading is an error because adding
//! it is the one repair, and the content under it is a person's to write. An
//! adopter who wants a placeholder to fail a strict run binds this rule with a
//! control whose posture is blocking.

use crate::finding::{at, Finding, Severity};
use crate::instance::Outcome;
use crate::scope::{DocumentCheck, DocumentView};
use crate::shape::Shape;
use headwater_doc::body::{BlockKind, Ownership};

pub const RULE: &str = "scaffold.placeholder.unwritten";

/// The text the scaffolder writes under each heading it adds.
pub const SECTION_PROMPT: &str = "TODO write this section.";

/// The value the scaffolder writes for a facet whose role is `scent`, when
/// nothing determines it.
pub const SCENT_PROMPT: &str =
    "TODO one sentence: what a reader learns here, which is what routing reads";

/// The value the scaffolder writes for any other facet it cannot determine.
pub fn facet_prompt(facet: &str) -> String {
    format!("TODO {facet}")
}

/// The check, generated from the kinds a scaffolder can write a prompt into.
///
/// The scaffolder writes a prompt under each heading a kind requires and into
/// each facet a kind requires that nothing determines, and nowhere else. So a
/// kind that requires neither a section nor a facet has no instance, on the
/// accounting [`crate::facet_required`] states: an instance that could only
/// ever pass counts a document as checked by a rule that had nothing to check.
/// The match itself reads every top-level facet and every paragraph, because
/// a prompt an author copied into a facet the kind does not require is still
/// a prompt.
pub struct Placeholder {
    kinds: Vec<String>,
}

impl Placeholder {
    /// The generation step, in full.
    pub fn over(shape: &Shape) -> Self {
        Placeholder {
            kinds: shape
                .kinds
                .iter()
                .filter(|kind| {
                    !shape.required_sections(&kind.name).is_empty()
                        || !shape.required_facets(&kind.name).is_empty()
                })
                .map(|kind| kind.name.clone())
                .collect(),
        }
    }
}

impl DocumentCheck for Placeholder {
    const RULE: &'static str = self::RULE;
    /// The first edition of this rule.
    ///
    /// Raise the rule version when a rule's behavior changes.
    const VERSION: u32 = 1;
    /// The section prompt is in the body.
    const NEEDS_BODY: bool = true;

    fn instantiates(&self, kind: &str) -> bool {
        self.kinds.iter().any(|known| known == kind)
    }

    fn evaluate(&self, view: &DocumentView<'_>) -> Outcome {
        let path = view.path();
        let mut findings: Vec<Finding> = view
            .facets()
            .iter()
            .filter(|entry| {
                entry.value.value.as_scalar().is_some_and(|scalar| {
                    scalar.text == SCENT_PROMPT || scalar.text == facet_prompt(&entry.key.value)
                })
            })
            .map(|entry| {
                let facet = &entry.key.value;
                let (line, column) = at(Some(entry.value.span));
                Finding {
                    rule: self::RULE,
                    severity: Severity::Warn,
                    obligation: None,
                    path: path.to_string(),
                    line,
                    column,
                    message: format!(
                        "`{facet}` still holds the prompt the scaffolder wrote for a person to replace"
                    ),
                    remediation: format!("write the value of `{facet}` into {path}"),
                    patch: None,
                }
            })
            .collect();
        if let Some(body) = view.body() {
            findings.extend(
                // One test of ownership covers both exclusions: a run inside a
                // block quotation is `Quoted` and a run in a code block or a
                // code span is `Code`, so the parser decides each of them once.
                body.blocks
                    .iter()
                    .filter(|block| block.kind == BlockKind::Paragraph)
                    .filter(|block| {
                        block
                            .runs
                            .iter()
                            .all(|run| run.ownership == Ownership::Authored)
                    })
                    .filter(|block| block.text().trim() == SECTION_PROMPT)
                    .map(|block| {
                        let (line, column) = at(Some(block.span));
                        Finding {
                            rule: self::RULE,
                            severity: Severity::Warn,
                            obligation: None,
                            path: path.to_string(),
                            line,
                            column,
                            message: format!(
                                "this section still reads `{SECTION_PROMPT}`, the prompt the scaffolder wrote for a person to replace"
                            ),
                            remediation: format!(
                                "write the section in {path}, in place of the prompt"
                            ),
                            patch: None,
                        }
                    }),
            );
        }
        Outcome::failed(findings)
    }
}
