//! Source positions, in file coordinates.
//!
//! Spec 12 requires that a finding anchor to a line, and that the parser retain
//! spans for front-matter keys, headings and links. Everything here exists to
//! satisfy that, and item 1 of the spike is the test of whether it survives all
//! the way to rendered output.

/// A position in the *file*, not in the front-matter block.
///
/// The distinction is the whole difficulty. A YAML parser reports positions
/// relative to the text it was given, and the text it was given starts after
/// the opening `---`. Every position crossing out of `frontmatter` is
/// translated once, at the boundary, so that nothing downstream has to know
/// that the translation happened.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, PartialOrd, Ord)]
pub struct Position {
    /// 1-indexed line in the file.
    pub line: usize,
    /// 1-indexed column.
    pub col: usize,
    /// Byte offset into the file.
    pub offset: usize,
}

impl Position {
    pub fn new(line: usize, col: usize, offset: usize) -> Self {
        Self { line, col, offset }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, PartialOrd, Ord)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Span {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    /// Translate a span reported against an embedded block into file
    /// coordinates. `line_offset` is the number of file lines that precede the
    /// block, and `byte_offset` the number of bytes.
    pub fn shift(self, line_offset: usize, byte_offset: usize) -> Self {
        Self {
            start: Position::new(
                self.start.line + line_offset,
                self.start.col + 1,
                self.start.offset + byte_offset,
            ),
            end: Position::new(
                self.end.line + line_offset,
                self.end.col + 1,
                self.end.offset + byte_offset,
            ),
        }
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
