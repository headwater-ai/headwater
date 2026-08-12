// SPDX-License-Identifier: Apache-2.0
//! What a taxonomy source can be rejected for.
//!
//! Each variant is one of the Q2 loader rulings, or the syntax error the YAML
//! dependency raised. Every one carries a span, because a rejection an author
//! cannot locate costs the same as no message.

use crate::span::Span;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadError {
    pub kind: ErrorKind,
    pub span: Span,
}

impl LoadError {
    pub fn new(kind: ErrorKind, span: Span) -> Self {
        Self { kind, span }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    /// The YAML dependency could not scan the source. Nothing below ran.
    Syntax(String),
    /// An anchor definition, `&name`.
    Anchor,
    /// An alias reference, `*name`.
    Alias,
    /// A merge key, `<<`.
    MergeKey,
    /// An explicit tag, `!!str` or `!Local`.
    Tag(String),
    /// The same key twice in one mapping.
    DuplicateKey { key: String, first: Span },
    /// A key that is not a scalar: a sequence or a mapping used as a key.
    ComplexKey,
    /// A second document in the stream.
    SecondDocument,
    /// A source with no document in it.
    Empty,
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // A default span means there is no position to give, which is true of
        // exactly one rejection: a source with no document in it. Printing
        // `0:0` there would name a line that no file has.
        if self.span != Span::default() {
            write!(f, "{}: ", self.span.start)?;
        }
        match &self.kind {
            ErrorKind::Syntax(message) => write!(f, "{message}"),
            ErrorKind::Anchor => write!(
                f,
                "an anchor is not allowed in a taxonomy source; \
                 the $-reference is the reuse mechanism an overlay can address"
            ),
            ErrorKind::Alias => write!(
                f,
                "an alias is not allowed in a taxonomy source; \
                 the $-reference is the reuse mechanism an overlay can address"
            ),
            ErrorKind::MergeKey => write!(
                f,
                "a merge key is not allowed in a taxonomy source; \
                 write the keys out, or use a $-reference"
            ),
            ErrorKind::Tag(tag) => write!(
                f,
                "the tag `{tag}` is not allowed in a taxonomy source; \
                 a scalar takes its type from the meta-schema"
            ),
            ErrorKind::DuplicateKey { key, first } => write!(
                f,
                "duplicate key `{key}`, first declared at {}",
                first.start
            ),
            ErrorKind::ComplexKey => write!(
                f,
                "a key must be a scalar; nothing can address a key that is not one"
            ),
            ErrorKind::SecondDocument => write!(
                f,
                "a taxonomy source holds one document, and this is the second"
            ),
            ErrorKind::Empty => write!(f, "the source holds no document"),
        }
    }
}

impl std::error::Error for LoadError {}

/// Render a rejection the way the fixtures record it: one line per error, in
/// source order.
pub fn render(errors: &[LoadError]) -> String {
    let mut out = String::new();
    for error in errors {
        out.push_str(&error.to_string());
        out.push('\n');
    }
    out
}
