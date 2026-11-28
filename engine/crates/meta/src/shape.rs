// SPDX-License-Identifier: Apache-2.0
//! The shape language that the meta-schema is written in.
//!
//! Eight forms and two modifiers, and the whole of it is in
//! `meta-schema.yml`'s header comment. This module is
//! the reader: it turns a loaded YAML node into a [`Shape`], and it refuses a
//! shape that names no form or more than one.
//!
//! The form set is small on purpose. Every form here is one the thirteen
//! declarations of [the taxonomy model](../../../../docs/spec/taxonomy-model.md#the-thirteen-declarations)
//! actually use, and a form nothing uses would be a shape language feature that
//! no taxonomy could exercise.

use crate::error::{SchemaError, SchemaErrorKind};
use headwater_yaml::{core_schema, Scalar, Span, Value};

/// A scalar type. Q2 rules that a scalar takes its type from here and never
/// from the YAML resolver, so this list is the whole of what a scalar may be.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScalarType {
    String,
    Integer,
    Boolean,
    Date,
    Duration,
    /// The absence of a value, which `extends:` admits and nothing else does.
    Null,
}

impl ScalarType {
    pub fn parse(name: &str) -> Option<Self> {
        Some(match name {
            "string" => ScalarType::String,
            "integer" => ScalarType::Integer,
            "boolean" => ScalarType::Boolean,
            "date" => ScalarType::Date,
            "duration" => ScalarType::Duration,
            "null" => ScalarType::Null,
            _ => return None,
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            ScalarType::String => "string",
            ScalarType::Integer => "integer",
            ScalarType::Boolean => "boolean",
            ScalarType::Date => "date",
            ScalarType::Duration => "duration",
            ScalarType::Null => "null",
        }
    }

    /// Whether a scalar's text is of this type.
    ///
    /// Q2 puts this decision here rather than in the loader, so every rule about
    /// what `no` or `013` means lives in one place: the core schema offers a
    /// reading, and a declared type is what asks for it. Two readers ask: the
    /// meta-schema, for a value a taxonomy writes, and the check layer's
    /// `facet.value.not_of_type`, for a value a document writes. One predicate
    /// serves both, because one declaration with two parsers is the defect that
    /// [`headwater_yaml::core_schema`] records.
    pub fn admits(self, scalar: &Scalar) -> bool {
        match self {
            // A key written with nothing after it is not a string. It is the
            // author leaving the value out, and a required member that is there
            // and empty would otherwise pass.
            ScalarType::String => !core_schema::as_null(scalar),
            ScalarType::Integer => core_schema::as_int(scalar).is_some(),
            ScalarType::Boolean => core_schema::as_bool(scalar).is_some(),
            ScalarType::Date => calendar_date(&scalar.text).is_some(),
            ScalarType::Duration => is_duration(&scalar.text),
            ScalarType::Null => core_schema::as_null(scalar),
        }
    }
}

/// `YYYY-MM-DD` read as a real date of the proleptic Gregorian calendar, which
/// is what YAML 1.2 and ISO 8601 both mean by a date, as `(year, month, day)`.
///
/// It is the one form [authoring and lifecycle](../../../../docs/spec/authoring-and-lifecycle.md)
/// writes. Until HW-OBL-0171 this crate read the digit positions only, so it
/// admitted `2026-02-30` and `0000-00-00`, which the check layer's date reader
/// refused. The check layer's `Date::parse` now calls this function, so a date
/// that one reader refuses the other refuses too.
pub fn calendar_date(text: &str) -> Option<(i64, u32, u32)> {
    let bytes = text.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return None;
    }
    if ![0, 1, 2, 3, 5, 6, 8, 9]
        .iter()
        .all(|index| bytes[*index].is_ascii_digit())
    {
        return None;
    }
    let year: i64 = text[0..4].parse().ok()?;
    let month: u32 = text[5..7].parse().ok()?;
    let day: u32 = text[8..10].parse().ok()?;
    if !(1..=12).contains(&month) || day < 1 || day > days_in_month(year, month) {
        return None;
    }
    Some((year, month, day))
}

/// The length of a month of the proleptic Gregorian calendar.
fn days_in_month(year: i64, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ => match (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
            true => 29,
            false => 28,
        },
    }
}

/// A count and a unit, as `within: 90d` writes it.
///
/// The unit set is a guess in the direction Q2 settles: `d`, `w`, `m` and `y`
/// are what a participation window is written in, and nothing states the set.
fn is_duration(text: &str) -> bool {
    let Some(unit) = text.chars().last() else {
        return false;
    };
    let count = &text[..text.len() - unit.len_utf8()];
    matches!(unit, 'd' | 'w' | 'm' | 'y')
        && !count.is_empty()
        && count.bytes().all(|byte| byte.is_ascii_digit())
}

/// One member of a closed mapping: a shape, plus whether it must be there.
#[derive(Clone, Debug)]
pub struct Member {
    pub name: String,
    pub shape: Shape,
    pub required: bool,
}

/// A form, plus the modifiers that apply wherever the form stands.
#[derive(Clone, Debug)]
pub struct Shape {
    pub form: Form,
    /// Whether a `$`-reference may stand here instead of a literal. The default
    /// refuses one, which is what keeps a reference out of the position that a
    /// reference reads: a vocabulary entry is not `reference: allowed`, so
    /// `vocabularies.a: $vocabularies.b` fails on shape rather than on a rule
    /// that a resolver would have to carry.
    pub reference: bool,
    /// Whether the keys of a mapping are address segments. They are, everywhere
    /// but one place, and `addressable: false` is how the meta-schema says that
    /// a mapping's keys are not reachable by an overlay.
    pub addressable: bool,
}

#[derive(Clone, Debug)]
pub enum Form {
    Scalar(ScalarType),
    Enum(Vec<String>),
    Seq(Box<Shape>),
    Map(Box<Shape>),
    Members(Vec<Member>),
    Block(String),
    OneOf(Vec<Shape>),
    /// A node the meta-schema does not constrain, and the reason it does not.
    /// Every use is a gap that [Open obligations](../../../../docs/spec/open-obligations.md)
    /// carries, and holding the reason here is what lets the crate count them.
    Free(String),
}

impl Form {
    pub fn name(&self) -> &'static str {
        match self {
            Form::Scalar(_) => "scalar",
            Form::Enum(_) => "enum",
            Form::Seq(_) => "seq",
            Form::Map(_) => "map",
            Form::Members(_) => "members",
            Form::Block(_) => "block",
            Form::OneOf(_) => "one_of",
            Form::Free(_) => "free",
        }
    }
}

/// The eight form keys, in the order the header comment lists them.
const FORMS: [&str; 8] = [
    "scalar", "enum", "seq", "map", "members", "block", "one_of", "free",
];
const MODIFIERS: [&str; 3] = ["required", "reference", "addressable"];

impl Shape {
    /// Read a shape out of the meta-schema source.
    ///
    /// `at` is the dotted place inside the meta-schema, which is what a
    /// rejection names. A malformed meta-schema is a defect in the engine's own
    /// shipped file rather than in an adopter's taxonomy, so the message is
    /// written for whoever edits this crate.
    pub fn read(node: &Value, span: Span, at: &str) -> Result<Self, SchemaError> {
        let map = node
            .as_map()
            .ok_or_else(|| SchemaError::new(SchemaErrorKind::ShapeNotAMapping, at, span))?;

        let mut found: Option<(&str, &headwater_yaml::Entry)> = None;
        for entry in map {
            let key = entry.key.value.as_str();
            if MODIFIERS.contains(&key) {
                continue;
            }
            let Some(form) = FORMS.iter().find(|name| **name == key) else {
                return Err(SchemaError::new(
                    SchemaErrorKind::UnknownForm(key.to_string()),
                    at,
                    entry.key.span,
                ));
            };
            if let Some((first, _)) = found {
                return Err(SchemaError::new(
                    SchemaErrorKind::TwoForms {
                        first: first.to_string(),
                        second: key.to_string(),
                    },
                    at,
                    entry.key.span,
                ));
            }
            found = Some((form, entry));
        }
        let Some((name, entry)) = found else {
            return Err(SchemaError::new(SchemaErrorKind::NoForm, at, span));
        };

        let value = &entry.value.value;
        let value_span = entry.value.span;
        let form = match name {
            "scalar" => {
                let text = text_of(value, at, value_span)?;
                Form::Scalar(ScalarType::parse(&text).ok_or_else(|| {
                    SchemaError::new(SchemaErrorKind::UnknownScalarType(text), at, value_span)
                })?)
            }
            "enum" => Form::Enum(strings_of(value, at, value_span)?),
            "seq" => Form::Seq(Box::new(Shape::read(
                value,
                value_span,
                &format!("{at}.seq"),
            )?)),
            "map" => Form::Map(Box::new(Shape::read(
                value,
                value_span,
                &format!("{at}.map"),
            )?)),
            "members" => {
                let members = value.as_map().ok_or_else(|| {
                    SchemaError::new(SchemaErrorKind::ShapeNotAMapping, at, value_span)
                })?;
                let mut read = Vec::new();
                for member in members {
                    let name = member.key.value.clone();
                    let where_ = format!("{at}.{name}");
                    let shape = Shape::read(&member.value.value, member.value.span, &where_)?;
                    let required = flag(&member.value.value, "required");
                    read.push(Member {
                        name,
                        shape,
                        required,
                    });
                }
                Form::Members(read)
            }
            "block" => Form::Block(text_of(value, at, value_span)?),
            "one_of" => {
                let items = value.as_seq().ok_or_else(|| {
                    SchemaError::new(SchemaErrorKind::ShapeNotASequence, at, value_span)
                })?;
                let mut read = Vec::new();
                for (index, item) in items.iter().enumerate() {
                    read.push(Shape::read(
                        &item.value,
                        item.span,
                        &format!("{at}.one_of.{index}"),
                    )?);
                }
                Form::OneOf(read)
            }
            "free" => Form::Free(text_of(value, at, value_span)?),
            _ => unreachable!("the form name came from FORMS"),
        };

        Ok(Shape {
            form,
            reference: reference_allowed(node),
            // Keys are segments unless the meta-schema says otherwise, because
            // an overlay that cannot name a declaration cannot change it.
            addressable: !matches!(named(node, "addressable"), Some(text) if text == "false"),
        })
    }
}

/// Whether a shape carries `reference: allowed`.
///
/// Anything else under the key is refused, because a modifier with a value
/// nobody reads is a rule that looks declared and is not.
fn reference_allowed(node: &Value) -> bool {
    named(node, "reference").is_some_and(|text| text == "allowed")
}

fn named(node: &Value, key: &str) -> Option<String> {
    node.as_map()
        .and_then(|map| map.get(key))
        .and_then(|value| value.value.as_scalar())
        .map(|scalar| scalar.text.clone())
}

fn flag(node: &Value, key: &str) -> bool {
    node.as_map()
        .and_then(|map| headwater_yaml::core_schema::flag(map, key))
        .unwrap_or(false)
}

fn text_of(node: &Value, at: &str, span: Span) -> Result<String, SchemaError> {
    node.as_scalar()
        .map(|scalar| scalar.text.clone())
        .ok_or_else(|| SchemaError::new(SchemaErrorKind::ShapeNotAScalar, at, span))
}

fn strings_of(node: &Value, at: &str, span: Span) -> Result<Vec<String>, SchemaError> {
    let items = node
        .as_seq()
        .ok_or_else(|| SchemaError::new(SchemaErrorKind::ShapeNotASequence, at, span))?;
    items
        .iter()
        .map(|item| text_of(&item.value, at, item.span))
        .collect()
}
