// SPDX-License-Identifier: Apache-2.0
//! Byte offset to line and column, over a whole file.
//!
//! The CommonMark dependency reports a byte range and nothing else, so
//! something has to turn a range into the coordinates a reader's editor uses.
//! The loader converts at its own boundary for the same reason, and both agree
//! on what a column is: a **character** count from the start of the line,
//! from one. A byte column would put the caret inside an em dash, and this
//! specification corpus is full of them.

use headwater_yaml::{Position, Span};

/// The start offset of every line in a source.
pub struct Lines {
    /// Byte offset of the first byte of each line. Always starts with 0.
    starts: Vec<usize>,
    len: usize,
}

impl Lines {
    pub fn new(source: &str) -> Self {
        let mut starts = vec![0usize];
        // A source that ends in a newline gets a final start equal to its
        // length. That line is empty, and it is the line a span that ends at
        // the end of the file reports.
        starts.extend(source.match_indices('\n').map(|(offset, _)| offset + 1));
        Self {
            starts,
            len: source.len(),
        }
    }

    /// The position of a byte offset. An offset past the end reports the end,
    /// because a span that runs off the source is a defect in the caller and a
    /// panic here would hide which caller.
    pub fn position(&self, source: &str, offset: usize) -> Position {
        let offset = offset.min(self.len);
        // The last line whose start is at or before the offset.
        let line_index = match self.starts.binary_search(&offset) {
            Ok(index) => index,
            Err(index) => index - 1,
        };
        let start = self.starts[line_index];
        let col = source
            .get(start..offset)
            .map(|text| text.chars().count())
            .unwrap_or(0);
        Position::new(line_index + 1, col + 1, offset)
    }

    pub fn span(&self, source: &str, range: std::ops::Range<usize>) -> Span {
        Span::new(
            self.position(source, range.start),
            self.position(source, range.end),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_column_counts_characters_and_an_offset_counts_bytes() {
        // Both em dashes are three bytes and one character. A byte column would
        // report 12 for the `s` of `spans` and point four columns to its right.
        let source = "a — b — spans\nsecond\n";
        let lines = Lines::new(source);
        let at = lines.position(source, source.find("spans").unwrap());
        assert_eq!((at.line, at.col), (1, 9));
        assert_eq!(at.offset, 12);
    }

    #[test]
    fn the_second_line_starts_at_column_one() {
        let source = "first\nsecond\n";
        let lines = Lines::new(source);
        let at = lines.position(source, 6);
        assert_eq!((at.line, at.col, at.offset), (2, 1, 6));
    }

    #[test]
    fn an_offset_past_the_end_reports_the_end() {
        let source = "one\n";
        let lines = Lines::new(source);
        assert_eq!(lines.position(source, 99).offset, 4);
    }
}
