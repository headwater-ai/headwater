//! The finding shape from spec 4, reduced to what the spike needs.

use crate::span::Span;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

impl Severity {
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Info => "info",
            Severity::Warning => "warn",
            Severity::Error => "error",
        }
    }
}

/// A mechanical and total fix, in the sense spec 12 defines. Anything needing
/// judgment carries remediation prose instead and no patch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fix {
    pub description: String,
    pub replacement: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    pub check: &'static str,
    pub obligation: &'static str,
    pub severity: Severity,
    pub path: String,
    /// The position the finding anchors to. Item 1 is the question of whether
    /// this survives from the YAML parser all the way to rendered output.
    pub span: Span,
    pub message: String,
    pub remediation: String,
    pub fix: Option<Fix>,
}

impl Finding {
    /// Spec 12's stable ordering: path, line, check id, message.
    pub fn sort_key(&self) -> (&str, usize, usize, &'static str, &str) {
        (
            self.path.as_str(),
            self.span.start.line,
            self.span.start.col,
            self.check,
            self.message.as_str(),
        )
    }

    /// One line, in the shape an editor and a human both read.
    pub fn render(&self) -> String {
        format!(
            "{}:{}:{}: [{}] {}: {}",
            self.path,
            self.span.start.line,
            self.span.start.col,
            self.severity.as_str(),
            self.check,
            self.message
        )
    }
}

pub fn sort(findings: &mut [Finding]) {
    findings.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
}
