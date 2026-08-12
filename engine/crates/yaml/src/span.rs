// SPDX-License-Identifier: Apache-2.0
//! Source positions, in file coordinates.
//!
//! Spec 12 requires that a finding anchor to a line. Everything here exists to
//! satisfy that, and every span this crate hands out is already in the
//! coordinates a reader's editor uses: lines and columns from one, byte offsets
//! into the source.

/// A position in the source.
///
/// The byte offset is the field that costs something to produce. The YAML
/// dependency reports a *character* index, and the two diverge on the first
/// non-ASCII byte. This specification's own prose is full of em dashes, so the
/// divergence is the normal case rather than an edge case, and a consumer that
/// slices the source on a character index cuts a multi-byte character in half.
/// [`crate::loader`] converts once, at the boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, PartialOrd, Ord)]
pub struct Position {
    /// 1-indexed line.
    pub line: usize,
    /// 1-indexed column.
    pub col: usize,
    /// Byte offset into the source.
    pub offset: usize,
}

impl Position {
    pub fn new(line: usize, col: usize, offset: usize) -> Self {
        Self { line, col, offset }
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}

/// A half-open range: `start` is inclusive and `end` is exclusive.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, PartialOrd, Ord)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Span {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    /// The source text this span covers, or `None` when the span does not lie
    /// inside `source`.
    pub fn slice<'a>(&self, source: &'a str) -> Option<&'a str> {
        source.get(self.start.offset..self.end.offset)
    }
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.start)
    }
}

/// A value that knows where it came from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spanned<T> {
    pub value: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(value: T, span: Span) -> Self {
        Self { value, span }
    }
}

impl<T> std::ops::Deref for Spanned<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}
