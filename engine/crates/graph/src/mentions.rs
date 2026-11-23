// SPDX-License-Identifier: Apache-2.0
//! Prose mentions of an identifier, bound to the document that carries it.
//! None of them is an edge.
//!
//! [Spec 1](../../../../docs/spec/01-conceptual-model.md#prose-links-are-not-relations)
//! says no syntax promotes prose into a relation, and this module keeps that:
//! it records where a document's own prose names another document, and an edge
//! is still only what a `relations:` block declares. What the record is for is
//! the advisory check Q4 promised in [`crate::links`]: a document that cites
//! another and declares no relation to it, so that the author is offered the
//! relation and the graph does not lose the edge the author meant (#1977).
//!
//! # What counts as a mention
//!
//! A token of the author's own prose that is exactly an identifier the index
//! holds, and that is not the document's own. The prose is every run of text
//! that [`headwater_doc::Ownership::Authored`] marks, so a code span, a fenced
//! or indented block, raw HTML and a block quote never mention anything: each
//! is either not prose or another author's. Link text is prose, so
//! `[HW-DR-0089](…)` is a mention of `HW-DR-0089` wherever the link points;
//! whether the path agrees with the text is [`crate::links`]'s question.
//!
//! A token is what [`tokens`] reads, which is the same reading the comment
//! scanner gives a code comment: a maximal run of ASCII letters, digits and
//! hyphens, with a trailing hyphen trimmed. So `HW-DR-0089.` and `(HW-DR-0089)`
//! name `HW-DR-0089`, and `XHW-DR-0089` and `HW-DR-00890` name nothing.
//!
//! # One record for each pair
//!
//! A document that names one identifier three times owes the graph one edge,
//! not three. So the bind keeps the first mention of each (document, named
//! document) pair, at its span, and counts the rest.

use crate::index::Index;
use headwater_census::census::Census;
use headwater_doc::Ownership;
use headwater_yaml::{Position, Span};
use std::collections::{BTreeMap, HashMap};

/// The first prose mention of one document by another.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mention {
    pub source_path: String,
    pub source_id: String,
    /// The kind of the document that wrote the mention, where it has one.
    pub source_kind: Option<String>,
    /// The identifier the prose names.
    pub named: String,
    pub named_path: String,
    /// The kind of the named document, where it has one.
    pub named_kind: Option<String>,
    /// The first mention: the token itself where the run maps byte for byte
    /// onto the source, and otherwise the run that holds it.
    pub span: Span,
    /// How many times the source's prose names the identifier.
    pub times: usize,
}

/// Every identifier-shaped token of `text`, with its byte offset: a maximal
/// run of ASCII letters, digits and hyphens that opens on a letter or a digit,
/// with a trailing hyphen trimmed off so that a sentence's punctuation is not
/// read as part of the token that precedes it.
pub(crate) fn tokens(text: &str) -> Vec<(usize, &str)> {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if (bytes[i] as char).is_ascii_alphanumeric() {
            let start = i;
            while i < bytes.len()
                && ((bytes[i] as char).is_ascii_alphanumeric() || bytes[i] == b'-')
            {
                i += 1;
            }
            let mut end = i;
            while end > start && bytes[end - 1] == b'-' {
                end -= 1;
            }
            out.push((start, &text[start..end]));
        } else {
            i += 1;
        }
    }
    out
}

/// Bind every prose mention of every document that carries an identifier.
///
/// The result is sorted by source path and then by the order of first mention,
/// so a report that reads it is stable across runs.
pub fn bind(census: &Census, index: &Index) -> Vec<Mention> {
    let held: HashMap<&str, (&str, Option<&str>)> = index
        .paths
        .iter()
        .filter_map(|entry| {
            entry
                .id
                .as_deref()
                .map(|id| (id, (entry.path.as_str(), entry.kind.as_deref())))
        })
        .collect();

    let mut out = Vec::new();
    for row in &census.rows {
        let Some(document) = &row.document else {
            continue;
        };
        let Some(source) = index.by_path(&row.path) else {
            continue;
        };
        let Some(source_id) = source.id.as_deref() else {
            continue;
        };
        // Insertion order is the order of first mention.
        let mut found: Vec<Mention> = Vec::new();
        let mut at: BTreeMap<&str, usize> = BTreeMap::new();
        for block in document.body.authored() {
            for run in &block.runs {
                if run.ownership != Ownership::Authored {
                    continue;
                }
                for (offset, token) in tokens(&run.text) {
                    if token == source_id {
                        continue;
                    }
                    let Some((named_path, named_kind)) = held.get(token) else {
                        continue;
                    };
                    if let Some(&slot) = at.get(token) {
                        found[slot].times += 1;
                        continue;
                    }
                    at.insert(token, found.len());
                    found.push(Mention {
                        source_path: row.path.clone(),
                        source_id: source_id.to_string(),
                        source_kind: source.kind.clone(),
                        named: token.to_string(),
                        named_path: named_path.to_string(),
                        named_kind: named_kind.map(str::to_string),
                        span: narrow(run.span, &run.text, offset, token.len()),
                        times: 1,
                    });
                }
            }
        }
        out.extend(found);
    }
    out
}

/// The span of a token inside a run, where the run's text is the source's
/// bytes unchanged: one line, and as many bytes as the span covers. An escape,
/// an entity or a soft break makes the text differ from the source, and then
/// the run's own span is the honest answer rather than a column that points at
/// the wrong character.
fn narrow(span: Span, text: &str, offset: usize, len: usize) -> Span {
    let one_line = span.start.line == span.end.line && !text.contains('\n');
    if !one_line || span.end.offset.saturating_sub(span.start.offset) != text.len() {
        return span;
    }
    let col = span.start.col + text[..offset].chars().count();
    Span::new(
        Position::new(span.start.line, col, span.start.offset + offset),
        Position::new(span.start.line, col + len, span.start.offset + offset + len),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_token_is_trimmed_of_a_trailing_hyphen_and_split_at_punctuation() {
        let found: Vec<&str> = tokens("See HW-DR-0089. Then (HW-OBL-0001)- and x-y--")
            .into_iter()
            .map(|(_, token)| token)
            .collect();
        assert_eq!(
            found,
            ["See", "HW-DR-0089", "Then", "HW-OBL-0001", "and", "x-y"]
        );
    }

    #[test]
    fn a_token_carries_its_byte_offset() {
        let found = tokens("— HW-DR-0089");
        assert_eq!(found, [(4, "HW-DR-0089")]);
    }

    #[test]
    fn a_run_that_matches_the_source_narrows_to_the_token() {
        let run = Span::new(Position::new(3, 1, 100), Position::new(3, 21, 120));
        let got = narrow(run, "This follows HW-0001", 13, 7);
        assert_eq!(got.start, Position::new(3, 14, 113));
        assert_eq!(got.end, Position::new(3, 21, 120));
    }

    #[test]
    fn a_run_that_differs_from_the_source_keeps_its_own_span() {
        // Twenty-one bytes of source for twenty bytes of text: an escape.
        let run = Span::new(Position::new(3, 1, 100), Position::new(3, 22, 121));
        assert_eq!(narrow(run, "This follows HW-0001", 13, 7), run);
    }
}
