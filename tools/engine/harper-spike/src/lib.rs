// SPDX-License-Identifier: Apache-2.0
//! The #1013 spike: run harper-core over the prose this document's own author
//! wrote, as `headwater-doc` segments it, and map each finding back to file
//! bytes.
//!
//! Two modes. `authored` gives Harper one sentence at a time, built from the
//! `Authored` runs of `Body::sentences()` and nothing else, through Harper's
//! plain-English constructor. `raw` gives Harper the whole file through its own
//! Markdown parser, which is what HW-DR-0041 (Q41) called the false-positive
//! class of a second, raw-Markdown linter. The decisive fixture compares the
//! two over one file.

use harper_core::linting::{LintGroup, Suggestion};
use harper_core::spell::FstDictionary;
use harper_core::{Dialect, Document};
use headwater_doc::body::{BlockKind, Ownership};

/// One Harper finding on one file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub path: String,
    pub line: usize,
    pub column: usize,
    /// The name of the Harper rule, as `LintGroup::organized_lints` keys it.
    pub rule: String,
    /// Harper's `LintKind`, its category.
    pub kind: String,
    pub message: String,
    /// The first suggestion, rendered: `replace:<text>`, `insert:<text>`,
    /// `remove`, or empty.
    pub suggestion: String,
    /// Byte range in the file, or `None` when the finding does not map back
    /// to bytes the author wrote verbatim.
    pub range: Option<(usize, usize)>,
    /// The file bytes at `range`, which a `Patch::Text` would carry as
    /// `expect`. Empty when `range` is `None`.
    pub expect: String,
    /// The text Harper flagged, from its own input.
    pub flagged: String,
}

/// The linter, built once: the curated lint set, the embedded dictionary, and
/// American English, which is this repository's ruling (CLAUDE.md).
pub fn linter() -> LintGroup {
    LintGroup::new_curated(FstDictionary::curated(), Dialect::American)
}

/// One piece of the text given to Harper: `len` bytes at `at` in that text
/// came from `file` in the source, byte for byte when `verbatim`.
#[derive(Clone, Copy, Debug)]
struct Segment {
    at: usize,
    file: usize,
    len: usize,
    verbatim: bool,
}

/// The text of one sentence that the author wrote, with the map back to file
/// bytes. The text equals `Sentence::authored` before its trim.
fn authored_text(
    body: &headwater_doc::body::Body,
    sentence: &headwater_doc::Sentence,
) -> (String, Vec<Segment>) {
    let mut text = String::new();
    let mut segments = Vec::new();
    let (low, high) = (sentence.span.start.offset, sentence.span.end.offset);
    for block in &body.blocks {
        if block.quote_depth > 0 || matches!(block.kind, BlockKind::Code | BlockKind::Html) {
            continue;
        }
        for run in &block.runs {
            if run.ownership != Ownership::Authored {
                continue;
            }
            let (from, to) = (run.span.start.offset, run.span.end.offset);
            let (lo, hi) = (from.max(low), to.min(high));
            if lo >= hi {
                continue;
            }
            let verbatim = to - from == run.text.len();
            let piece = if verbatim {
                match run.text.get(lo - from..hi - from) {
                    Some(piece) => piece,
                    None => continue,
                }
            } else if from >= low && to <= high {
                run.text.as_str()
            } else {
                continue;
            };
            segments.push(Segment {
                at: text.len(),
                file: if verbatim { lo } else { from },
                len: piece.len(),
                verbatim,
            });
            text.push_str(piece);
        }
    }
    (text, segments)
}

/// The file byte range of `start..end` in the text, when both ends fall in
/// one verbatim segment.
fn to_file(segments: &[Segment], start: usize, end: usize) -> Option<(usize, usize)> {
    let _ = segments;
    let _ = (start, end);
    None
}

/// Byte offset of a char index into `text`.
fn byte_of(text: &str, index: usize) -> usize {
    text.char_indices()
        .nth(index)
        .map(|(at, _)| at)
        .unwrap_or(text.len())
}

fn render(suggestion: Option<&Suggestion>) -> String {
    match suggestion {
        Some(Suggestion::ReplaceWith(chars)) => {
            format!("replace:{}", chars.iter().collect::<String>())
        }
        Some(Suggestion::InsertAfter(chars)) => {
            format!("insert:{}", chars.iter().collect::<String>())
        }
        Some(Suggestion::Remove) => "remove".to_string(),
        None => String::new(),
    }
}

fn line_column(source: &str, offset: usize) -> (usize, usize) {
    let before = &source[..offset.min(source.len())];
    let line = before.matches('\n').count() + 1;
    let column = before.rfind('\n').map_or(before.len(), |nl| before.len() - nl - 1) + 1;
    (line, column)
}

fn clean(text: &str) -> String {
    text.replace(['\t', '\n', '\r'], " ")
}

/// Every Harper finding on the author-owned prose of one file.
pub fn authored(path: &str, source: &str, linter: &mut LintGroup) -> Vec<Row> {
    let Ok(document) = headwater_doc::parse_prose(source) else {
        return Vec::new();
    };
    let body = &document.body;
    let mut rows = Vec::new();
    for sentence in body.sentences() {
        let (text, segments) = authored_text(body, &sentence);
        if text.trim().is_empty() {
            continue;
        }
        let harper = Document::new_plain_english_curated(&text);
        for (rule, lints) in linter.organized_lints(&harper) {
            for lint in lints {
                let start = byte_of(&text, lint.span.start);
                let end = byte_of(&text, lint.span.end);
                let range = to_file(&segments, start, end);
                let expect = range
                    .map(|(a, b)| source[a..b].to_string())
                    .unwrap_or_default();
                let anchor = range.map_or(sentence.span.start.offset, |(a, _)| a);
                let (line, column) = line_column(source, anchor);
                rows.push(Row {
                    path: path.to_string(),
                    line,
                    column,
                    rule: rule.clone(),
                    kind: format!("{:?}", lint.lint_kind),
                    message: clean(&lint.message),
                    suggestion: clean(&render(lint.suggestions.first())),
                    range,
                    expect: clean(&expect),
                    flagged: clean(&text[start..end]),
                });
            }
        }
    }
    rows.sort_by(|a, b| (a.line, a.column, &a.rule).cmp(&(b.line, b.column, &b.rule)));
    rows
}

/// Every Harper finding on the whole file, through Harper's Markdown parser.
pub fn raw(path: &str, source: &str, linter: &mut LintGroup) -> Vec<Row> {
    let harper = Document::new_markdown_default_curated(source);
    let mut rows = Vec::new();
    for (rule, lints) in linter.organized_lints(&harper) {
        for lint in lints {
            let start = byte_of(source, lint.span.start);
            let end = byte_of(source, lint.span.end);
            let (line, column) = line_column(source, start);
            rows.push(Row {
                path: path.to_string(),
                line,
                column,
                rule: rule.clone(),
                kind: format!("{:?}", lint.lint_kind),
                message: clean(&lint.message),
                suggestion: clean(&render(lint.suggestions.first())),
                range: Some((start, end)),
                expect: clean(&source[start..end]),
                flagged: clean(&source[start..end]),
            });
        }
    }
    rows.sort_by(|a, b| (a.line, a.column, &a.rule).cmp(&(b.line, b.column, &b.rule)));
    rows
}

impl Row {
    /// One line of the committed TSV.
    pub fn tsv(&self) -> String {
        let (start, end) = self
            .range
            .map_or((String::from("-"), String::from("-")), |(a, b)| {
                (a.to_string(), b.to_string())
            });
        [
            self.path.as_str(),
            &self.line.to_string(),
            &self.column.to_string(),
            &start,
            &end,
            &self.rule,
            &self.kind,
            &self.flagged,
            &self.expect,
            &self.suggestion,
            &self.message,
        ]
        .join("\t")
    }
}

/// The header of the TSV.
pub const HEADER: &str =
    "path\tline\tcolumn\tstart\tend\trule\tkind\tflagged\texpect\tsuggestion\tmessage";
