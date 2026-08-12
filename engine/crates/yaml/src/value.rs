// SPDX-License-Identifier: Apache-2.0
//! The loaded tree.
//!
//! There is no `Null` variant, and no `Bool` or `Int` either. That is the whole
//! shape of [Q2](../../../../docs/spec/09-decisions.md#q2--schema-format):
//! *scalar types come from the meta-schema and never from the YAML resolver*.
//! A loader that decided here which scalars are null would be the YAML resolver
//! under another name, and the meta-schema would inherit a typing it never
//! asked for.
//!
//! So a scalar keeps two things: the text as written, and the style it was
//! written in. [`crate::core_schema`] turns that pair into a type, and it runs
//! only where the meta-schema names a type.

use crate::span::{Span, Spanned};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Scalar(Scalar),
    Seq(Vec<Spanned<Value>>),
    Map(Mapping),
}

impl Value {
    pub fn as_scalar(&self) -> Option<&Scalar> {
        match self {
            Value::Scalar(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_seq(&self) -> Option<&[Spanned<Value>]> {
        match self {
            Value::Seq(items) => Some(items),
            _ => None,
        }
    }

    pub fn as_map(&self) -> Option<&Mapping> {
        match self {
            Value::Map(m) => Some(m),
            _ => None,
        }
    }

    /// The name this crate uses for the value in a message.
    pub fn kind_name(&self) -> &'static str {
        match self {
            Value::Scalar(_) => "a scalar",
            Value::Seq(_) => "a sequence",
            Value::Map(_) => "a mapping",
        }
    }
}

/// A scalar, as written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scalar {
    /// The text after YAML has resolved escapes and folded block scalars, and
    /// before anything has decided what type it is.
    pub text: String,
    pub style: Style,
}

impl Scalar {
    /// Whether the core schema is even allowed to look at this scalar.
    ///
    /// Only a plain scalar carries a type. `"true"` is the string `true` in
    /// every YAML version, and that is the escape hatch an author reaches for
    /// when a value happens to look like something else.
    pub fn is_plain(&self) -> bool {
        self.style == Style::Plain
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    Plain,
    SingleQuoted,
    DoubleQuoted,
    Literal,
    Folded,
}

impl Style {
    pub fn name(self) -> &'static str {
        match self {
            Style::Plain => "plain",
            Style::SingleQuoted => "single-quoted",
            Style::DoubleQuoted => "double-quoted",
            Style::Literal => "literal",
            Style::Folded => "folded",
        }
    }
}

/// An ordered mapping.
///
/// Order is kept because the loader is the thing that reproduces a source
/// faithfully, and because a diff over a resolved artifact is only readable if
/// nothing reorders silently. Lookup is linear, which is right at the size of a
/// taxonomy source and wrong at the size of a corpus; the day that changes, the
/// index belongs here rather than at every call site.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Mapping {
    entries: Vec<Entry>,
}

/// One key/value pair. The key carries its own span, which is the span a
/// finding about the *declaration* points at, as opposed to its value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub key: Spanned<String>,
    pub value: Spanned<Value>,
}

impl Mapping {
    pub fn new(entries: Vec<Entry>) -> Self {
        Self { entries }
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Entry> {
        self.entries.iter()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

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
}

impl<'a> IntoIterator for &'a Mapping {
    type Item = &'a Entry;
    type IntoIter = std::slice::Iter<'a, Entry>;
    fn into_iter(self) -> Self::IntoIter {
        self.entries.iter()
    }
}
