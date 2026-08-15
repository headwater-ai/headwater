// SPDX-License-Identifier: Apache-2.0
//! The resolved taxonomy written back out, in one canonical form.
//!
//! Two things need this. A round-trip fixture needs a text to load again, which
//! is how a resolver demonstrates that its result is a taxonomy source and not
//! an internal structure that happens to answer questions
//! ([spec 12](../../../../docs/spec/12-check-layer.md#the-correctness-roots)).
//! And the lock is this text with a hash over it, which is
//! [#51](https://github.com/headwater-ai/headwater/issues/51).
//!
//! Declaration order is kept, because [spec 2](../../../../docs/spec/02-taxonomy-model.md#customization-by-composition)
//! wants a resolution result that "is reproducible and reviewable in a diff",
//! and a writer that sorted keys would move half the file whenever an overlay
//! added a declaration in the middle.
//!
//! A scalar is written plain where every character allows it and double-quoted
//! otherwise. Nothing here decides a type: [Q2](../../../../docs/spec/09-decisions.md#q2--schema-format)
//! rules that types come from the meta-schema, so `no` is written back as `no`
//! and reads back as the string it was.

use headwater_yaml::{Mapping, Scalar, Value};

/// The resolved taxonomy as a source that loads to the same tree.
pub fn render(map: &Mapping) -> String {
    let mut out = String::new();
    mapping(map, 0, &mut out);
    out
}

fn mapping(map: &Mapping, indent: usize, out: &mut String) {
    for entry in map {
        let pad = " ".repeat(indent);
        out.push_str(&format!("{pad}{}:", scalar_text(&entry.key.value)));
        match &entry.value.value {
            Value::Scalar(scalar) => out.push_str(&format!(" {}\n", write_scalar(scalar))),
            Value::Map(inner) if inner.is_empty() => out.push_str(" {}\n"),
            Value::Seq(items) if items.is_empty() => out.push_str(" []\n"),
            Value::Map(inner) => {
                out.push('\n');
                mapping(inner, indent + 2, out);
            }
            Value::Seq(items) => {
                out.push('\n');
                sequence(items, indent + 2, out);
            }
        }
    }
}

fn sequence(items: &[headwater_yaml::Spanned<Value>], indent: usize, out: &mut String) {
    let pad = " ".repeat(indent);
    for item in items {
        match &item.value {
            Value::Scalar(scalar) => out.push_str(&format!("{pad}- {}\n", write_scalar(scalar))),
            Value::Map(inner) if inner.is_empty() => out.push_str(&format!("{pad}- {{}}\n")),
            Value::Seq(inner) if inner.is_empty() => out.push_str(&format!("{pad}- []\n")),
            Value::Map(inner) => {
                // The first member rides on the dash, which is what a reader of
                // a diff expects to see and what every hand-written source in
                // this repository does.
                let mut body = String::new();
                mapping(inner, indent + 2, &mut body);
                out.push_str(&hang(&body, indent));
            }
            Value::Seq(inner) => {
                let mut body = String::new();
                sequence(inner, indent + 2, &mut body);
                out.push_str(&hang(&body, indent));
            }
        }
    }
}

/// Put a `- ` in front of the first line of an already indented block.
fn hang(body: &str, indent: usize) -> String {
    let pad = " ".repeat(indent);
    match body.strip_prefix(&" ".repeat(indent + 2)) {
        Some(rest) => format!("{pad}- {rest}"),
        None => format!("{pad}-\n{body}"),
    }
}

fn write_scalar(scalar: &Scalar) -> String {
    scalar_text(&scalar.text)
}

fn scalar_text(text: &str) -> String {
    if is_plain(text) {
        return text.to_string();
    }
    let mut out = String::from("\"");
    for found in text.chars() {
        match found {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            _ => out.push(found),
        }
    }
    out.push('"');
    out
}

/// Whether a text may be written with no quotation marks.
///
/// The set is deliberately small. A writer that tried to keep every plain form
/// a hand-written source uses would have to reimplement the YAML resolver's
/// ambiguity rules, and the cost of quoting a value that did not need it is one
/// pair of characters in a diff.
fn is_plain(text: &str) -> bool {
    let mut characters = text.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    if !(first.is_ascii_alphanumeric() || first == '_') {
        return false;
    }
    text.chars()
        .all(|found| found.is_ascii_alphanumeric() || matches!(found, '_' | '-' | '.' | '/' | '+'))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::merge::same;

    fn round_trip(source: &str) {
        let loaded = headwater_yaml::load(source).expect("the fixture loads");
        let written = render(loaded.value.as_map().expect("a mapping"));
        let again =
            headwater_yaml::load(&written).unwrap_or_else(|errors| panic!("{written}\n{errors:?}"));
        assert!(
            same(&loaded.value, &again.value),
            "{source}\nbecame\n{written}"
        );
    }

    #[test]
    fn a_taxonomy_of_every_shape_survives_a_round_trip() {
        round_trip(
            "\
taxonomy: acme
version: 1.0.0
vocabularies:
  lifecycle_state:
    - {value: draft, role: initial}
    - {value: current, role: live}
facets:
  status: {role: state, required: true}
regimes:
  voice:
    narrative: {}
  language:
    default: {tag: en-US, controlled: none}
shelves:
  decisions: {path: \"docs/decisions/**\", homogeneous: true}
identifier_schemes:
  decision_id: {pattern: \"DR-{namespace}-{seq:04d}\", namespace: repo, allocation: minted-once}
core:
  requires:
    - facet_role: state
    - relation_family: succession
      lifecycle_sensitive: true
",
        );
    }

    #[test]
    fn a_value_the_yaml_resolver_would_have_typed_is_written_back_as_written() {
        round_trip("facets:\n  answer: {default: no, count: 013}\n");
    }

    #[test]
    fn a_multiline_string_survives_as_one_scalar() {
        round_trip("shelves:\n  a:\n    reason: >-\n      one line\n      and another\n");
    }

    #[test]
    fn an_empty_collection_keeps_its_form() {
        round_trip("regimes:\n  voice:\n    narrative: {}\nmappings: []\n");
    }
}
