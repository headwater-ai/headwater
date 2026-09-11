// SPDX-License-Identifier: Apache-2.0
//! Front-matter parsing that keeps a span for every key.
//!
//! **Item 1 of the spike.** The evaluation claimed that no convenient
//! `Deserialize` implementation gives source positions, so the front-matter
//! layer is custom work over a streaming event parser in either language. This
//! module is that claim, built.
//!
//! It is deliberately a thin layer. The YAML dependency supplies an event
//! stream and a span per event, and nothing else. Everything below is the tree
//! builder that a spanned document needs and that a deserializer would hide.

use crate::span::{Position, Span, Spanned};
use saphyr_parser::{Event, Parser, ScanError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Null,
    Scalar(String),
    Seq(Vec<Spanned<Value>>),
    Map(Vec<Entry>),
}

/// One key/value pair. The key carries its own span, which is the span a
/// finding about a *facet* should point at.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub key: Spanned<String>,
    pub value: Spanned<Value>,
}

#[derive(Clone, Debug, Default)]
pub struct FrontMatter {
    pub entries: Vec<Entry>,
    /// Span of the whole block, including both `---` fences. This is where a
    /// finding about a *missing* facet has to point, because a missing key has
    /// no span of its own.
    pub span: Span,
    /// Line count of the block, so the body parser can offset its own spans.
    pub end_line: usize,
}

impl FrontMatter {
    pub fn entry(&self, key: &str) -> Option<&Entry> {
        self.entries.iter().find(|e| e.key.value == key)
    }

    pub fn get(&self, key: &str) -> Option<&Spanned<Value>> {
        self.entry(key).map(|e| &e.value)
    }

    /// The span to anchor a finding about this key to.
    pub fn key_span(&self, key: &str) -> Option<Span> {
        self.entry(key).map(|e| e.key.span)
    }

    pub fn scalar(&self, key: &str) -> Option<&str> {
        match self.get(key).map(|v| &v.value) {
            Some(Value::Scalar(s)) => Some(s.as_str()),
            _ => None,
        }
    }

    /// A sequence of scalars, with the span of each element retained. Relation
    /// declarations are this shape, and a dangling-edge finding must point at
    /// the offending element rather than at the key.
    pub fn scalar_seq(&self, key: &str) -> Vec<Spanned<String>> {
        match self.get(key).map(|v| &v.value) {
            Some(Value::Seq(items)) => items
                .iter()
                .filter_map(|i| match &i.value {
                    Value::Scalar(s) => Some(Spanned::new(s.clone(), i.span)),
                    _ => None,
                })
                .collect(),
            Some(Value::Scalar(s)) => {
                let span = self.get(key).map(|v| v.span).unwrap_or_default();
                vec![Spanned::new(s.clone(), span)]
            }
            _ => Vec::new(),
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    Missing,
    Unterminated,
    Yaml(String),
    NotAMapping,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::Missing => write!(f, "no front matter"),
            ParseError::Unterminated => write!(f, "front matter is not terminated by ---"),
            ParseError::Yaml(m) => write!(f, "front matter is not valid YAML: {m}"),
            ParseError::NotAMapping => write!(f, "front matter is not a mapping"),
        }
    }
}

/// Split a document into its front-matter block and its body.
///
/// Returns the inner YAML text, the number of file lines before it, its byte
/// offset, and the body with its own line offset.
pub struct Split<'a> {
    pub yaml: &'a str,
    pub yaml_line_offset: usize,
    pub yaml_byte_offset: usize,
    pub body: &'a str,
    pub body_line_offset: usize,
    pub fence_end_line: usize,
}

pub fn split(source: &str) -> Result<Split<'_>, ParseError> {
    // CRLF is not a curiosity. A corpus authored on Windows has it throughout,
    // and a parser that only accepts LF reports "no front matter" for every
    // file, which reads as a corpus-wide classification failure.
    let rest = source
        .strip_prefix("---\r\n")
        .or_else(|| source.strip_prefix("---\n"))
        .ok_or(ParseError::Missing)?;
    let open_len = source.len() - rest.len();

    // Find a closing fence that sits at the start of a line.
    let mut idx = 0usize;
    let close_at = loop {
        let hay = &rest[idx..];
        let Some(found) = hay.find("\n---") else {
            return Err(ParseError::Unterminated);
        };
        let abs = idx + found + 1; // position of the '-' after the newline
        let after = &rest[abs + 3..];
        if after.is_empty() || after.starts_with('\n') || after.starts_with('\r') {
            break abs;
        }
        idx = abs + 3;
    };

    let yaml = &rest[..close_at];
    // Body starts after the closing fence line.
    let after_fence = &rest[close_at + 3..];
    let body = after_fence
        .strip_prefix("\r\n")
        .or_else(|| after_fence.strip_prefix('\n'))
        .unwrap_or(after_fence);

    // `yaml` ends with the newline that precedes the closing fence, so the
    // newline count *is* the line count. Adding one here shifts every body
    // position by a line, which is the kind of defect that shows up as an
    // editor jumping one line short of the finding.
    let yaml_lines = yaml.matches('\n').count();
    // opening fence + yaml lines + closing fence
    let lines_before_body = 1 + yaml_lines + 1;
    Ok(Split {
        yaml,
        // Line 1 is the opening `---`, so YAML line 1 is file line 2.
        yaml_line_offset: 1,
        yaml_byte_offset: open_len,
        body,
        body_line_offset: lines_before_body,
        fence_end_line: lines_before_body,
    })
}

/// Owned mirror of the parser's event, so that the tree builder does not have
/// to thread the input lifetime through every frame.
enum Ev {
    MapStart,
    MapEnd,
    SeqStart,
    SeqEnd,
    /// The style is not decoration. `key:` with nothing after it arrives as a
    /// *plain* `~`, while `key: ""` arrives as a double-quoted empty string.
    /// Only the style separates "the author left this blank" from "the author
    /// wrote an empty string", and a required-facet check has to tell them
    /// apart. A deserializer collapses both to `Some("")`.
    Scalar(String, bool),
    Other,
}

/// YAML 1.2 core-schema null, but only when written plainly.
fn is_null(value: &str, plain: bool) -> bool {
    plain && matches!(value, "" | "~" | "null" | "Null" | "NULL")
}

pub fn parse(source: &str) -> Result<FrontMatter, ParseError> {
    let split = split(source)?;
    let mut events: Vec<(Ev, Span)> = Vec::new();

    for item in Parser::new_from_str(split.yaml) {
        let (event, span): (Event, saphyr_parser::Span) =
            item.map_err(|e: ScanError| ParseError::Yaml(e.to_string()))?;
        let span = Span::new(
            Position::new(span.start.line(), span.start.col(), span.start.index()),
            Position::new(span.end.line(), span.end.col(), span.end.index()),
        )
        .shift(split.yaml_line_offset, split.yaml_byte_offset);

        let ev = match event {
            Event::MappingStart(..) => Ev::MapStart,
            Event::MappingEnd => Ev::MapEnd,
            Event::SequenceStart(..) => Ev::SeqStart,
            Event::SequenceEnd => Ev::SeqEnd,
            Event::Scalar(v, style, ..) => {
                Ev::Scalar(v.into_owned(), style == saphyr_parser::ScalarStyle::Plain)
            }
            _ => Ev::Other,
        };
        events.push((ev, span));
    }

    // Find the outermost mapping and build from there.
    let start = events
        .iter()
        .position(|(e, _)| matches!(e, Ev::MapStart))
        .ok_or(ParseError::NotAMapping)?;

    let mut cursor = start + 1;
    let entries = build_map(&events, &mut cursor)?;

    let end_line = split.fence_end_line;
    let span = Span::new(
        Position::new(1, 1, 0),
        Position::new(end_line, 4, split.yaml_byte_offset + split.yaml.len() + 4),
    );

    Ok(FrontMatter {
        entries,
        span,
        end_line,
    })
}

fn build_map(events: &[(Ev, Span)], cursor: &mut usize) -> Result<Vec<Entry>, ParseError> {
    let mut entries = Vec::new();
    loop {
        let Some((ev, span)) = events.get(*cursor) else {
            return Err(ParseError::Yaml("unexpected end of events".into()));
        };
        match ev {
            Ev::MapEnd => {
                *cursor += 1;
                return Ok(entries);
            }
            Ev::Scalar(k, _) => {
                let key = Spanned::new(k.clone(), *span);
                *cursor += 1;
                let value = build_value(events, cursor)?;
                entries.push(Entry { key, value });
            }
            // A non-scalar key. Legal YAML, meaningless as a facet name.
            _ => {
                skip_value(events, cursor)?;
                skip_value(events, cursor)?;
            }
        }
    }
}

fn build_value(events: &[(Ev, Span)], cursor: &mut usize) -> Result<Spanned<Value>, ParseError> {
    let Some((ev, span)) = events.get(*cursor) else {
        return Err(ParseError::Yaml("unexpected end of events".into()));
    };
    let span = *span;
    match ev {
        Ev::Scalar(v, plain) => {
            *cursor += 1;
            let value = if is_null(v, *plain) {
                Value::Null
            } else {
                Value::Scalar(v.clone())
            };
            Ok(Spanned::new(value, span))
        }
        Ev::SeqStart => {
            *cursor += 1;
            let mut items = Vec::new();
            loop {
                match events.get(*cursor) {
                    Some((Ev::SeqEnd, end)) => {
                        *cursor += 1;
                        let full = Span::new(span.start, end.end);
                        return Ok(Spanned::new(Value::Seq(items), full));
                    }
                    Some(_) => items.push(build_value(events, cursor)?),
                    None => return Err(ParseError::Yaml("unterminated sequence".into())),
                }
            }
        }
        Ev::MapStart => {
            *cursor += 1;
            let entries = build_map(events, cursor)?;
            let end = events
                .get(cursor.saturating_sub(1))
                .map(|(_, s)| s.end)
                .unwrap_or(span.end);
            Ok(Spanned::new(Value::Map(entries), Span::new(span.start, end)))
        }
        _ => {
            *cursor += 1;
            Ok(Spanned::new(Value::Null, span))
        }
    }
}

fn skip_value(events: &[(Ev, Span)], cursor: &mut usize) -> Result<(), ParseError> {
    build_value(events, cursor).map(|_| ())
}
