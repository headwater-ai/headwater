// SPDX-License-Identifier: Apache-2.0
//! The loader itself: an event stream in, a spanned tree out, and the Q2
//! dialect enforced on the way through.
//!
//! The order of work is deliberate. A syntax error stops everything, because
//! there is no tree to say anything about. Every other rejection is collected,
//! so that an author who wrote three anchors reads three lines rather than
//! running the loader three times.

use crate::error::{ErrorKind, LoadError};
use crate::span::{Position, Span, Spanned};
use crate::value::{Entry, Mapping, Scalar, Style, Value};
use saphyr_parser::{Event, Parser, ScalarStyle, ScanError};

/// Load a taxonomy source.
///
/// The errors come back in source order, and the list is empty only when the
/// result is `Ok`.
pub fn load(source: &str) -> Result<Spanned<Value>, Vec<LoadError>> {
    let offsets = ByteOffsets::new(source);
    let mut events: Vec<(Ev, Span)> = Vec::new();

    for item in Parser::new_from_str(source) {
        let (event, span) = match item {
            Ok(pair) => pair,
            Err(error) => return Err(vec![syntax_error(&error, &offsets)]),
        };
        events.push((Ev::from(event), offsets.span(span)));
    }

    let mut errors = Vec::new();
    let root = build_document(&events, &mut errors);
    match root {
        Some(root) if errors.is_empty() => Ok(root),
        _ => {
            if errors.is_empty() {
                errors.push(LoadError::new(ErrorKind::Empty, Span::default()));
            }
            errors.sort_by_key(|e| e.span.start);
            Err(errors)
        }
    }
}

/// Char index to byte offset, and the 1-indexed column the crate does not give.
///
/// Both conversions are here rather than spread over the call sites, because
/// both are easy to get wrong once and then never notice: an off-by-one column
/// looks like a working editor jump, and a char index looks like a byte offset
/// on every ASCII fixture ever written.
struct ByteOffsets {
    /// Byte offset per char index, with one extra entry for the end.
    table: Vec<usize>,
}

impl ByteOffsets {
    fn new(source: &str) -> Self {
        let mut table: Vec<usize> = source.char_indices().map(|(byte, _)| byte).collect();
        table.push(source.len());
        Self { table }
    }

    fn offset(&self, char_index: usize) -> usize {
        self.table
            .get(char_index)
            .copied()
            .unwrap_or_else(|| self.table.last().copied().unwrap_or(0))
    }

    fn position(&self, marker: saphyr_parser::Marker) -> Position {
        // The crate documents `col` as 1-indexed and reports it 0-indexed. The
        // spike found this and `marker_columns_are_zero_indexed` below is the
        // standing test, so that a fixed upstream fails loudly rather than
        // shifting every column by one.
        Position::new(marker.line(), marker.col() + 1, self.offset(marker.index()))
    }

    fn span(&self, span: saphyr_parser::Span) -> Span {
        Span::new(self.position(span.start), self.position(span.end))
    }
}

fn syntax_error(error: &ScanError, offsets: &ByteOffsets) -> LoadError {
    let at = offsets.position(*error.marker());
    LoadError::new(
        ErrorKind::Syntax(error.info().to_string()),
        Span::new(at, at),
    )
}

/// Owned mirror of the parser's event, so the tree builder does not thread the
/// input lifetime through every frame.
enum Ev {
    DocStart,
    DocEnd,
    Alias,
    /// Text, style, whether an anchor was attached, and any explicit tag.
    Scalar(String, Style, bool, Option<String>),
    SeqStart(bool, Option<String>),
    SeqEnd,
    MapStart(bool, Option<String>),
    MapEnd,
    Other,
}

impl From<Event<'_>> for Ev {
    fn from(event: Event<'_>) -> Self {
        match event {
            Event::DocumentStart(..) => Ev::DocStart,
            Event::DocumentEnd => Ev::DocEnd,
            Event::Alias(..) => Ev::Alias,
            Event::Scalar(text, style, anchor, tag) => Ev::Scalar(
                text.into_owned(),
                style_of(style),
                anchor != 0,
                tag_name(tag),
            ),
            Event::SequenceStart(anchor, tag) => Ev::SeqStart(anchor != 0, tag_name(tag)),
            Event::SequenceEnd => Ev::SeqEnd,
            Event::MappingStart(anchor, tag) => Ev::MapStart(anchor != 0, tag_name(tag)),
            Event::MappingEnd => Ev::MapEnd,
            _ => Ev::Other,
        }
    }
}

fn style_of(style: ScalarStyle) -> Style {
    match style {
        ScalarStyle::Plain => Style::Plain,
        ScalarStyle::SingleQuoted => Style::SingleQuoted,
        ScalarStyle::DoubleQuoted => Style::DoubleQuoted,
        ScalarStyle::Literal => Style::Literal,
        ScalarStyle::Folded => Style::Folded,
    }
}

fn tag_name(tag: Option<std::borrow::Cow<'_, saphyr_parser::Tag>>) -> Option<String> {
    tag.map(|tag| format!("{}{}", tag.handle, tag.suffix))
}

/// Find the one document, and report any that follow it.
fn build_document(events: &[(Ev, Span)], errors: &mut Vec<LoadError>) -> Option<Spanned<Value>> {
    let mut cursor = 0usize;
    let mut root = None;
    while cursor < events.len() {
        match &events[cursor].0 {
            Ev::DocStart => {
                cursor += 1;
                if root.is_none() {
                    root = Some(build_value(events, &mut cursor, None, errors));
                } else {
                    errors.push(LoadError::new(
                        ErrorKind::SecondDocument,
                        events[cursor - 1].1,
                    ));
                    // Consume it, so that a third document reports once more
                    // rather than confusing the walk.
                    let _ = build_value(events, &mut cursor, None, errors);
                }
            }
            _ => cursor += 1,
        }
    }
    root
}

/// `declared_at` is the span of the key this value hangs off, when it has one.
///
/// It exists for one case. An anchor or a tag on a *collection* is written
/// before the collection starts, and the event span for a mapping begins at its
/// first key — on the next line. Reporting an anchor there points an author at
/// the line below the `&`. The key is the nearest position that is certainly on
/// the right line. A scalar has no such problem, so it keeps its own span.
fn build_value(
    events: &[(Ev, Span)],
    cursor: &mut usize,
    declared_at: Option<Span>,
    errors: &mut Vec<LoadError>,
) -> Spanned<Value> {
    let Some((event, span)) = events.get(*cursor) else {
        return Spanned::new(empty_scalar(), Span::default());
    };
    let span = *span;
    let attached = declared_at.unwrap_or(span);
    match event {
        Ev::Scalar(text, style, anchored, tag) => {
            *cursor += 1;
            report_node_errors(*anchored, tag.as_deref(), span, errors);
            Spanned::new(
                Value::Scalar(Scalar {
                    text: text.clone(),
                    style: *style,
                }),
                span,
            )
        }
        Ev::Alias => {
            *cursor += 1;
            errors.push(LoadError::new(ErrorKind::Alias, span));
            // An alias has no value of its own, and the load is already failing.
            // A placeholder keeps the surrounding shape intact so that the rest
            // of the source still reports its own problems.
            Spanned::new(empty_scalar(), span)
        }
        Ev::SeqStart(anchored, tag) => {
            *cursor += 1;
            report_node_errors(*anchored, tag.as_deref(), attached, errors);
            let mut items = Vec::new();
            loop {
                match events.get(*cursor) {
                    Some((Ev::SeqEnd, end)) => {
                        *cursor += 1;
                        return Spanned::new(Value::Seq(items), Span::new(span.start, end.end));
                    }
                    Some(_) => items.push(build_value(events, cursor, None, errors)),
                    None => return Spanned::new(Value::Seq(items), span),
                }
            }
        }
        Ev::MapStart(anchored, tag) => {
            *cursor += 1;
            report_node_errors(*anchored, tag.as_deref(), attached, errors);
            let (entries, end) = build_map(events, cursor, errors);
            Spanned::new(
                Value::Map(Mapping::new(entries)),
                Span::new(span.start, end.end),
            )
        }
        // A stray end or a stream marker inside a value position. The parser
        // does not produce one; consuming it keeps the walk finite if it ever
        // does.
        _ => {
            *cursor += 1;
            Spanned::new(empty_scalar(), span)
        }
    }
}

fn build_map(
    events: &[(Ev, Span)],
    cursor: &mut usize,
    errors: &mut Vec<LoadError>,
) -> (Vec<Entry>, Span) {
    let mut entries: Vec<Entry> = Vec::new();
    loop {
        let Some((event, span)) = events.get(*cursor) else {
            let end = entries.last().map(|e| e.value.span).unwrap_or_default();
            return (entries, end);
        };
        let span = *span;
        match event {
            Ev::MapEnd => {
                *cursor += 1;
                return (entries, span);
            }
            Ev::Scalar(text, style, anchored, tag) => {
                *cursor += 1;
                report_node_errors(*anchored, tag.as_deref(), span, errors);
                if text == "<<" && *style == Style::Plain {
                    errors.push(LoadError::new(ErrorKind::MergeKey, span));
                }
                if let Some(first) = entries.iter().find(|e| e.key.value == *text) {
                    errors.push(LoadError::new(
                        ErrorKind::DuplicateKey {
                            key: text.clone(),
                            first: first.key.span,
                        },
                        span,
                    ));
                }
                let key = Spanned::new(text.clone(), span);
                let value = build_value(events, cursor, Some(span), errors);
                entries.push(Entry { key, value });
            }
            // A sequence or a mapping used as a key. Legal YAML, and nothing in
            // an overlay address can name it.
            _ => {
                errors.push(LoadError::new(ErrorKind::ComplexKey, span));
                let _ = build_value(events, cursor, None, errors);
                let _ = build_value(events, cursor, None, errors);
            }
        }
    }
}

fn report_node_errors(anchored: bool, tag: Option<&str>, span: Span, errors: &mut Vec<LoadError>) {
    if anchored {
        errors.push(LoadError::new(ErrorKind::Anchor, span));
    }
    if let Some(tag) = tag {
        errors.push(LoadError::new(ErrorKind::Tag(tag.to_string()), span));
    }
}

fn empty_scalar() -> Value {
    Value::Scalar(Scalar {
        text: String::new(),
        style: Style::Plain,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The two conventions this crate corrects, asserted against the dependency
    /// rather than assumed. Either one changing upstream silently moves every
    /// span, so both are tested where the correction is made.
    #[test]
    fn marker_columns_are_zero_indexed() {
        let source = "id: DR-0001\n";
        let (_, span) = Parser::new_from_str(source)
            .filter_map(Result::ok)
            .find(|(event, _)| matches!(event, Event::Scalar(..)))
            .expect("a scalar event");
        assert_eq!(span.start.col(), 0, "the crate documents this as 1-indexed");
        assert_eq!(span.start.line(), 1);
    }

    #[test]
    fn marker_index_counts_characters_and_the_loader_reports_bytes() {
        // The em dash is three bytes and one char, so the two indices diverge
        // at the second key. Every specification file in this repository has
        // one, which is why this cannot wait for a later milestone.
        let source = "title: a — dash\nid: DR-0001\n";
        let root = load(source).expect("loads");
        let map = root.as_map().expect("a mapping");
        let span = map.key_span("id").expect("the id key");
        assert_eq!(span.start.line, 2);
        assert_eq!(span.start.col, 1);
        assert_eq!(&source[span.start.offset..span.end.offset], "id");
    }

    fn kinds(source: &str) -> Vec<ErrorKind> {
        load(source)
            .expect_err("expected a rejection")
            .into_iter()
            .map(|e| e.kind)
            .collect()
    }

    #[test]
    fn a_plain_mapping_loads() {
        let root = load("kinds:\n  - decision\n  - guide\nversion: 1\n").expect("loads");
        let map = root.as_map().expect("a mapping");
        assert_eq!(map.len(), 2);
        let seq = map.get("kinds").unwrap().as_seq().expect("a sequence");
        assert_eq!(seq.len(), 2);
        assert_eq!(seq[0].as_scalar().unwrap().text, "decision");
    }

    #[test]
    fn a_blank_value_and_an_empty_string_stay_different() {
        let root = load("blank:\nempty: \"\"\n").expect("loads");
        let map = root.as_map().unwrap();
        let blank = map.get("blank").unwrap().as_scalar().unwrap();
        let empty = map.get("empty").unwrap().as_scalar().unwrap();
        assert_eq!((blank.text.as_str(), blank.style), ("~", Style::Plain));
        assert_eq!((empty.text.as_str(), empty.style), ("", Style::DoubleQuoted));
    }

    #[test]
    fn duplicate_keys_are_an_error_and_name_the_first() {
        let errors = load("a: 1\nb: 2\na: 3\n").expect_err("rejected");
        assert_eq!(errors.len(), 1);
        match &errors[0].kind {
            ErrorKind::DuplicateKey { key, first } => {
                assert_eq!(key, "a");
                assert_eq!(first.start.line, 1);
                assert_eq!(errors[0].span.start.line, 3);
            }
            other => panic!("wrong error: {other:?}"),
        }
    }

    #[test]
    fn anchors_aliases_and_merge_keys_are_forbidden() {
        assert_eq!(kinds("base: &b\n  a: 1\nuse: *b\n"), {
            vec![ErrorKind::Anchor, ErrorKind::Alias]
        });
        // A merge key with an inline mapping needs no alias to reach the rule.
        assert_eq!(kinds("a: 1\n<<:\n  b: 2\n"), vec![ErrorKind::MergeKey]);
    }

    #[test]
    fn an_anchor_on_a_mapping_reports_at_the_line_that_carries_it() {
        // The mapping's own event starts at `abstract` on line 2, so the naive
        // span points an author one line past the `&`.
        let errors = load("defaults: &defaults\n  abstract: false\n").expect_err("rejected");
        assert_eq!(errors[0].kind, ErrorKind::Anchor);
        assert_eq!((errors[0].span.start.line, errors[0].span.start.col), (1, 1));
    }

    #[test]
    fn crlf_sources_load() {
        // A corpus authored on Windows has CRLF throughout. A loader that only
        // accepts LF rejects every file, which reads as a corpus-wide failure
        // rather than as one defect here.
        let root = load("taxonomy:\r\n  package: headwater/standard\r\n").expect("loads");
        let map = root.as_map().expect("a mapping");
        let inner = map.get("taxonomy").unwrap().as_map().expect("a mapping");
        assert_eq!(
            inner.get("package").unwrap().as_scalar().unwrap().text,
            "headwater/standard"
        );
    }

    #[test]
    fn an_explicit_tag_is_forbidden() {
        assert_eq!(
            kinds("version: !!str 1\n"),
            vec![ErrorKind::Tag("tag:yaml.org,2002:str".into())]
        );
    }

    #[test]
    fn a_complex_key_is_forbidden() {
        assert_eq!(kinds("? [a, b]\n: 1\n"), vec![ErrorKind::ComplexKey]);
    }

    #[test]
    fn a_second_document_is_an_error() {
        assert_eq!(kinds("a: 1\n---\nb: 2\n"), vec![ErrorKind::SecondDocument]);
    }

    #[test]
    fn an_empty_source_is_an_error() {
        assert_eq!(kinds(""), vec![ErrorKind::Empty]);
    }

    #[test]
    fn every_rejection_is_reported_in_source_order() {
        let errors = load("a: &x 1\nb: *x\na: 2\n").expect_err("rejected");
        let lines: Vec<usize> = errors.iter().map(|e| e.span.start.line).collect();
        assert_eq!(lines, vec![1, 2, 3]);
    }

    #[test]
    fn a_syntax_error_stops_at_one() {
        let errors = load("a: [1, 2\n").expect_err("rejected");
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0].kind, ErrorKind::Syntax(_)));
    }
}
