// SPDX-License-Identifier: Apache-2.0
//! The finding, and the one order it is ever written in.
//!
//! [Spec 4](../../../../docs/spec/04-assurance-model.md#findings) fixes the
//! shape: a rule, a severity, an obligation, a path, a line, a message, a
//! remediation, and whether a fix is mechanical. Every field below is that
//! field, and one of them is an `Option` for a reason the next paragraph gives.
//!
//! # The obligation is optional here, and it says what a taxonomy declared
//!
//! Spec 4 says every finding names the obligation it serves, and
//! [spec 12](../../../../docs/spec/12-check-layer.md#the-plugin-interface)
//! makes `obligation()` required so that the check surface does not become the
//! place where a rule escapes the "every rule earns its place" discipline. An
//! obligation is **data**: a taxonomy declares `obligations` and `controls`,
//! and a rule reaches its obligation through the control that names the rule.
//! [`crate::register`] is that path, and the runner stamps the field.
//!
//! It stays an `Option` because a taxonomy may declare no control for a rule.
//! The value is then `None` rather than an invented identifier that no register
//! would recognize, and the run reports which rules those are.
//!
//! # The severity below is the check's, and the obligation carries another one
//!
//! Two scales share one word, and reading them as one is the mistake this
//! paragraph exists to prevent.
//! [Spec 12](../../../../docs/spec/12-check-layer.md#severity-is-the-checks-posture-is-the-controls)
//! rules that a check reports severity and that whether it blocks is the
//! control's business. That is the field below, and its values are `error`,
//! `warn` and `info`. An obligation also declares a severity, on the scale
//! `high`, `medium`, `low`, and that one says how much the invariant matters.
//! The coverage report reads it, because spec 4 asks what fraction of
//! obligations are verified *by severity*. Spec 4's own worked finding carries
//! `error` against `OB-014`, which carries `medium`, so the two were never one
//! field.
//!
//! # Fixability is the patch, and it stopped being a field
//!
//! Spec 4 lists fixability among the members and
//! [spec 12](../../../../docs/spec/12-check-layer.md#fixability) says what may
//! carry it: "A check may return a patch alongside a finding." Until `--fix`
//! there was no patch, so a `fixable: bool` stood in for one, and three files
//! said in a comment that a flag with nothing behind it claims a capability the
//! engine does not have.
//!
//! [`crate::Patch`] is the thing itself, and [`Finding::fixable`] reads it. A
//! second field beside it could disagree with it, and
//! [HW-OBL-0087](../../../../docs/obligations/0087-fixable-has-two-readings-inside-one-engine.md)
//! is the record of that disagreement already standing between two rules. The
//! reading this engine now carries is "the engine will fix it". The other
//! reading — that the defect has a mechanical remedy — is what the severity
//! already says, on the bar `CLAUDE.md` states.

use crate::paint::{ColorMode, Role};

/// What a check says about a finding. Whether it blocks is the control's
/// business ([spec 12](../../../../docs/spec/12-check-layer.md#severity-is-the-checks-posture-is-the-controls)).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Error,
    Warn,
    Info,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Severity::Error => "error",
            Severity::Warn => "warn",
            Severity::Info => "info",
        })
    }
}

/// One finding, from any origin.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    /// The rule that produced it, in the dotted form spec 4 writes.
    pub rule: &'static str,
    pub severity: Severity,
    /// The obligation this rule serves, and `None` when no control names the
    /// rule. See the module comment.
    pub obligation: Option<String>,
    /// Relative to the repository root, with `/` separators.
    pub path: String,
    /// One-based, and 0 where the finding is about a file rather than a line.
    pub line: usize,
    pub column: usize,
    pub message: String,
    /// What to do. Never empty: a finding that states no remedy is a complaint.
    pub remediation: String,
    /// The correction this engine will write under `check --fix`, and nothing
    /// where it will write none
    /// ([spec 12](../../../../docs/spec/12-check-layer.md#fixability)).
    pub patch: Option<crate::Patch>,
}

impl Finding {
    /// Whether a patch rides with this finding. See the module comment: this is
    /// read from the patch rather than declared beside it.
    pub fn fixable(&self) -> bool {
        self.patch.is_some()
    }

    /// The sort key [spec 12](../../../../docs/spec/12-check-layer.md#determinism-concretely)
    /// fixes: path, line, check id, message.
    ///
    /// The column joins it after the line, because two findings on one line are
    /// otherwise ordered by their message, which reads as an arbitrary order to
    /// somebody looking at the file.
    pub fn order(&self) -> (&str, usize, usize, &str, &str) {
        (&self.path, self.line, self.column, self.rule, &self.message)
    }

    /// One finding as text, in the form a person reads in a terminal.
    ///
    /// `mode` is the color decision the caller already made —
    /// [`crate::paint::stdout_color`]'s equivalent under `headwater-cli`, or
    /// [`ColorMode::Plain`] for a machine format or a recorded fixture. This
    /// function reads no stream itself, on the rule
    /// `crate::paint`'s module comment states.
    pub fn render(&self, mode: ColorMode) -> String {
        use std::fmt::Write;
        let mut out = String::new();
        let _ = writeln!(
            out,
            "{} {}",
            crate::paint::paint(Role::Path, &self.location(), mode),
            crate::paint::severity_word(self.severity, mode)
        );
        // The obligation rides beside the rule, because "why am I being made to
        // do this?" is the question spec 4 gives every rule an answer to, and a
        // reader who has to look the rule up in a register does not ask it.
        let _ = match &self.obligation {
            Some(obligation) => writeln!(
                out,
                "  {} ({}): {}",
                self.rule,
                crate::paint::paint(Role::Obligation, obligation, mode),
                self.message
            ),
            None => writeln!(out, "  {}: {}", self.rule, self.message),
        };
        let _ = writeln!(
            out,
            "  {}{}: {}",
            crate::paint::paint(Role::Verb, "fix", mode),
            if self.fixable() { " (mechanical)" } else { "" },
            self.remediation
        );
        out
    }

    fn location(&self) -> String {
        match self.line {
            0 => self.path.clone(),
            line => format!("{}:{line}:{}", self.path, self.column),
        }
    }
}

/// Where a finding anchors, from the span the parser kept.
pub fn at(span: Option<headwater_yaml::Span>) -> (usize, usize) {
    match span {
        Some(span) => (span.start.line, span.start.col),
        None => (0, 0),
    }
}

/// Every finding of a run, in the one order spec 12 fixes.
pub fn sorted(mut findings: Vec<Finding>) -> Vec<Finding> {
    findings.sort_by(|a, b| a.order().cmp(&b.order()));
    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    fn finding(path: &str, line: usize, rule: &'static str) -> Finding {
        Finding {
            rule,
            severity: Severity::Error,
            obligation: None,
            path: path.to_string(),
            line,
            column: 1,
            message: "m".into(),
            remediation: "r".into(),
            patch: None,
        }
    }

    #[test]
    fn findings_sort_by_path_then_line_then_rule() {
        let sorted = sorted(vec![
            finding("b.md", 1, "z.rule"),
            finding("a.md", 9, "a.rule"),
            finding("a.md", 2, "z.rule"),
            finding("a.md", 2, "a.rule"),
        ]);
        let order: Vec<(&str, usize, &str)> = sorted
            .iter()
            .map(|f| (f.path.as_str(), f.line, f.rule))
            .collect();
        assert_eq!(
            order,
            [
                ("a.md", 2, "a.rule"),
                ("a.md", 2, "z.rule"),
                ("a.md", 9, "a.rule"),
                ("b.md", 1, "z.rule"),
            ]
        );
    }

    /// A finding about a file rather than a line prints no line, because a
    /// `:0:0` suffix sends a reader to look for something that is not there.
    #[test]
    fn a_finding_with_no_line_prints_the_path_alone() {
        assert!(finding("a.md", 0, "r")
            .render(ColorMode::Plain)
            .starts_with("a.md ✗ error\n"));
    }

    /// `Plain` writes no escape sequence anywhere in the three lines, and
    /// carries the severity glyph instead. `Ansi` writes the path, the
    /// severity, the obligation and the `fix` label each in their own SGR
    /// pair, and changes no word.
    #[test]
    fn a_rendered_finding_carries_color_only_under_ansi() {
        let mut one = finding("a.md", 3, "r");
        one.obligation = Some("OB-001".to_string());

        let plain = one.render(ColorMode::Plain);
        assert!(!plain.contains('\x1b'), "{plain:?}");
        assert!(plain.contains("✗ error"), "{plain:?}");
        assert!(plain.contains("(OB-001)"), "{plain:?}");
        assert!(plain.contains("fix"), "{plain:?}");

        let ansi = one.render(ColorMode::Ansi);
        assert!(ansi.contains('\x1b'), "{ansi:?}");
        for word in ["a.md:3:1", "error", "OB-001", "fix", "r", "m"] {
            assert!(ansi.contains(word), "{ansi:?} is missing {word:?}");
        }
    }
}
