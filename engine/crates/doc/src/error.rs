// SPDX-License-Identifier: Apache-2.0
//! What a document can be rejected for.
//!
//! Every rejection carries a span, on the same rule the loader states: a
//! rejection an author cannot locate costs the same as no message. The three
//! reasons that are not the loader's are all about the block rather than about
//! its contents, so all three point at the top of the file, which is where the
//! block either is or should be.

use headwater_yaml::{ErrorKind, LoadError, Position, Span};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    pub reason: Reason,
    pub span: Span,
}

impl ParseError {
    pub fn new(reason: Reason, span: Span) -> Self {
        Self { reason, span }
    }

    /// A rejection about the file as a whole, reported at its first character.
    pub fn at_start(reason: Reason) -> Self {
        let at = Position::new(1, 1, 0);
        Self::new(reason, Span::new(at, at))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reason {
    /// The file does not open with a `---` fence.
    NoFrontMatter,
    /// It opens with one and never closes it.
    UnterminatedFrontMatter,
    /// The block holds a sequence or a scalar. Facets are named, so the block
    /// is a mapping, and this is the one shape rule the loader does not own:
    /// the loader knows the dialect and nothing about the meaning.
    NotAMapping(&'static str),
    /// The loader refused the block.
    Yaml(ErrorKind),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: ", self.span.start)?;
        match &self.reason {
            Reason::NoFrontMatter => write!(
                f,
                "no front matter; a document declares its facets in a `---` block \
                 on the first line"
            ),
            Reason::UnterminatedFrontMatter => {
                write!(f, "the front-matter block is never closed by `---`")
            }
            Reason::NotAMapping(found) => write!(
                f,
                "front matter is {found}, and a facet has a name, so the block is a mapping"
            ),
            // The loader owns this text. Restating it here would be a second
            // wording of the same rule, and the two drift.
            Reason::Yaml(kind) => write!(f, "{kind}"),
        }
    }
}

impl std::error::Error for ParseError {}

impl From<LoadError> for ParseError {
    fn from(error: LoadError) -> Self {
        ParseError::new(Reason::Yaml(error.kind), error.span)
    }
}

/// Render a rejection the way the fixtures record it: one line per error, in
/// source order.
pub fn render(errors: &[ParseError]) -> String {
    let mut out = String::new();
    for error in errors {
        out.push_str(&error.to_string());
        out.push('\n');
    }
    out
}
