// SPDX-License-Identifier: Apache-2.0
//! A Document-origin check: the source form a language regime fixes.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#the-five-origins-of-a-check)
//! separates Document checks from the generated ones because "the body is not
//! in the graph". This rule reads something one step further out still: not the
//! body, but the shape of the file the body was written into.
//!
//! # The rule this file moves, and where it lived
//!
//! `CLAUDE.md` states it: "Do not hard-wrap Markdown source. Write each
//! paragraph, list item, and blockquote paragraph as one logical line ... A
//! deliberate hard break inside a paragraph (rare) uses a trailing backslash."
//! `tools/ste-lint.py` enforced it by counting lines, and this rule replaces
//! that script.
//!
//! # A source form is not prose, and the parser is what makes it checkable
//!
//! A paragraph that a later commit reflows reads the same and is written
//! differently, so nothing that reads sentences can report a wrap. Nor can a
//! rule count the lines a block covers: the range the parser reports for a
//! paragraph runs to the start of the next block, so its first and last line
//! differ for every paragraph in the corpus, and a backslash hard break is one
//! more thing a line count cannot tell from a wrap.
//!
//! [`headwater_doc::body::Block::soft_breaks`] is the fact instead. The parser
//! records one span per CommonMark soft break and none for a hard break, which
//! is the distinction `CLAUDE.md` asks for, held where spec 12 already holds
//! the parser as a correctness root.
//!
//! # The set of forms is closed, and this rule therefore never skips
//!
//! [`crate::language`] skips an instance whose `controlled` value this engine
//! has no rules for, because the meta-schema leaves that member open on stated
//! grounds: a closed set would refuse a source that is already committed.
//! `source_form` has no such history. The meta-schema declares
//! `{enum: [unconstrained, one_line_per_block]}`, so a value outside the set
//! never reaches this file, and a regime that fixes no form generates no
//! instance rather than a passing one.
//!
//! # Severity, and why this one is an error
//!
//! An error, and it is the [fixability](../../../../docs/spec/12-check-layer.md#fixability)
//! bar rather than a judgment about how much a wrap matters. The remediation is
//! to join two lines. That is mechanical and total, which is the same ground on
//! which [`crate::language`] makes a contraction an error and leaves a sentence
//! past the word limit advisory.

use crate::finding::{Finding, Severity};
use crate::instance::Outcome;
use crate::scope::{DocumentCheck, DocumentView, OutsideCheck};
use crate::shape::{LanguageRegime, Shape};
use headwater_doc::body::BlockKind;

pub const RULE: &str = "language.source_form.not_met";

/// The form this engine reports on. The meta-schema's other value,
/// `unconstrained`, is the absence of this rule rather than a second form.
const ONE_LINE_PER_BLOCK: &str = "one_line_per_block";

/// The check, generated from the language regimes and the kinds that bind them.
pub struct SourceForm {
    bound: Vec<Bound>,
    /// One binding per regime that lists paths outside the corpus root, with
    /// an empty `kind`.
    outside: Vec<Bound>,
}

struct Bound {
    kind: String,
    regime: String,
}

impl SourceForm {
    /// The generation step, in full.
    ///
    /// A regime that lists paths outside the corpus root binds those paths on
    /// the same terms (HW-DR-0084 clause 4), in a second list keyed on the
    /// regime. See [`crate::scope::over_outside_root`].
    pub fn over(shape: &Shape) -> Self {
        let bound = shape
            .kinds
            .iter()
            .filter_map(|kind| bind(&kind.name, shape.language_of(&kind.name)?))
            .collect();
        let outside = shape
            .language
            .iter()
            .filter(|regime| !regime.outside_root.is_empty())
            .filter_map(|regime| bind("", regime))
            .collect();
        SourceForm { bound, outside }
    }

    fn bound_to(&self, kind: &str) -> Option<&Bound> {
        self.bound.iter().find(|bound| bound.kind == kind)
    }

    /// The binding a view is read under: its regime for a path outside the
    /// corpus root, and its kind for every other document.
    fn bound_for(&self, view: &DocumentView<'_>) -> Option<&Bound> {
        match view.outside_regime() {
            Some(regime) => self.outside.iter().find(|bound| bound.regime == regime),
            None => self.bound_to(view.kind()),
        }
    }
}

/// One binding of the rule to a regime, and nothing for a regime that fixes no
/// source form.
fn bind(kind: &str, regime: &LanguageRegime) -> Option<Bound> {
    if regime.source_form.as_deref() != Some(ONE_LINE_PER_BLOCK) {
        return None;
    }
    Some(Bound {
        kind: kind.to_string(),
        regime: regime.name.clone(),
    })
}

impl OutsideCheck for SourceForm {
    fn binds_regime(&self, regime: &str) -> bool {
        self.outside.iter().any(|bound| bound.regime == regime)
    }
}

impl DocumentCheck for SourceForm {
    const RULE: &'static str = self::RULE;
    /// The first edition of this rule.
    const VERSION: u32 = 1;
    /// The body, because the soft breaks are on the blocks the scan produced.
    /// Nothing here reads a sentence.
    const NEEDS_BODY: bool = true;

    fn instantiates(&self, kind: &str) -> bool {
        self.bound_to(kind).is_some()
    }

    fn evaluate(&self, view: &DocumentView<'_>) -> Outcome {
        let Some(bound) = self.bound_for(view) else {
            return Outcome::Passed;
        };
        let Some(body) = view.body() else {
            return Outcome::Passed;
        };

        let mut findings = Vec::new();
        for block in &body.blocks {
            // A fenced or indented block is verbatim and an HTML block is not
            // ours to reflow. `CLAUDE.md` says the same: "fenced/indented code
            // kept verbatim".
            if matches!(block.kind, BlockKind::Code | BlockKind::Html) {
                continue;
            }
            // A quoted block is another author's words and this repository's own
            // file. The rule is about the file, so the quote is in scope, which
            // is where `CLAUDE.md` puts it: "one logical line per paragraph,
            // list item, or blockquote paragraph".
            for at in &block.soft_breaks {
                findings.push(Finding {
                    rule: self::RULE,
                    severity: Severity::Error,
                    obligation: None,
                    path: view.path().to_string(),
                    line: at.end.line,
                    column: at.end.col,
                    message: format!(
                        "`{}` writes one {} on one line, and this line continues the {} above",
                        bound.regime,
                        block.kind.name(),
                        block.kind.name()
                    ),
                    remediation: format!(
                        "join this line to the line above, or end the {} there",
                        block.kind.name()
                    ),
                    // Mechanical and total, and still no patch rides along:
                    // `check --fix` is #71, and a `fixable` flag with nothing
                    // behind it claims a capability the engine does not have.
                    patch: None,
                });
            }
        }
        findings.sort_by_key(|finding| (finding.line, finding.column));
        Outcome::failed(findings)
    }
}
