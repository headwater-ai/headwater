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

/// How far apart, in bytes, two checkpoints of a line that is not ASCII stand.
///
/// A column on such a line is counted from the last checkpoint at or before
/// the offset, so one position costs at most this many bytes of counting, and
/// the checkpoints cost one pair of numbers for each this many bytes.
const CHECKPOINT: usize = 256;

/// The start offset of every line in a source.
///
/// A column used to be counted from the start of its line for every position,
/// so one line of many spans took time in proportion to its square (#1968).
/// Now a line of ASCII counts its column as a byte difference, and any other
/// line counts from the nearest checkpoint before the offset. The tables are a
/// small part of the source: one number and one flag for each line, and one
/// checkpoint for each [`CHECKPOINT`] bytes of a line that is not ASCII.
pub struct Lines {
    /// Byte offset of the first byte of each line. Always starts with 0.
    starts: Vec<usize>,
    /// Whether each line, its newline included, is ASCII only.
    ascii: Vec<bool>,
    /// A byte offset on a character boundary and the characters from the start
    /// of its line to it, in source order. Each line that is not ASCII has one
    /// at its start and one for each [`CHECKPOINT`] bytes after that.
    checkpoints: Vec<(usize, usize)>,
    len: usize,
}

impl Lines {
    pub fn new(source: &str) -> Self {
        let mut starts = vec![0usize];
        // A source that ends in a newline gets a final start equal to its
        // length. That line is empty, and it is the line a span that ends at
        // the end of the file reports.
        starts.extend(source.match_indices('\n').map(|(offset, _)| offset + 1));
        let mut ascii = Vec::with_capacity(starts.len());
        let mut checkpoints = Vec::new();
        for (index, &start) in starts.iter().enumerate() {
            let end = starts.get(index + 1).copied().unwrap_or(source.len());
            let line = &source[start..end];
            let plain = line.is_ascii();
            ascii.push(plain);
            if plain {
                continue;
            }
            checkpoints.push((start, 0));
            let mut last = start;
            for (count, (at, _)) in line.char_indices().enumerate() {
                let at = start + at;
                if at - last >= CHECKPOINT {
                    checkpoints.push((at, count));
                    last = at;
                }
            }
        }
        Self {
            starts,
            ascii,
            checkpoints,
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
        let col = if self.ascii[line_index] {
            offset - start
        } else if !source.is_char_boundary(offset) {
            // An offset inside a character names no column, and the count
            // from the start of the line read it as the start.
            0
        } else {
            // The line's own first checkpoint is at its start and at or before
            // the offset, so the last checkpoint at or before it is on this line.
            let found = self.checkpoints.partition_point(|&(at, _)| at <= offset);
            let (at, count) = self.checkpoints[found - 1];
            count + source[at..offset].chars().count()
        };
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

    /// A line of many characters that are not ASCII, longer than several
    /// checkpoints, with a column read at each character and at offsets inside
    /// one, against the count from the start of the line (#1968).
    #[test]
    fn a_column_past_many_checkpoints_counts_from_the_line_start() {
        let line: String = (0..2_000)
            .map(|index| if index % 3 == 0 { '\u{2014}' } else { 'a' })
            .collect();
        let source = format!("first\n{line}\nlast\n");
        let lines = Lines::new(&source);
        let start = source.find('\n').unwrap() + 1;
        for offset in start..start + line.len() + 1 {
            let at = lines.position(&source, offset);
            let col = source
                .get(start..offset)
                .map(|text| text.chars().count())
                .unwrap_or(0);
            assert_eq!((at.line, at.col, at.offset), (2, col + 1, offset));
        }
        let last = lines.position(&source, source.len() - 2);
        assert_eq!((last.line, last.col), (3, 4));
    }
}
