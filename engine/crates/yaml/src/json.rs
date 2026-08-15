// SPDX-License-Identifier: Apache-2.0
//! Just enough JSON to speak a protocol, written out rather than depended on.
//!
//! The engine carries two third-party crates, and both are parsers for formats
//! it did not invent. A serializer for a format this narrow is a smaller thing
//! than the dependency would be: the writer below emits strings, numbers,
//! arrays and objects, and the reader is [`crate::load`], because JSON is a
//! subset of the YAML 1.2 core schema that the loader already implements.
//!
//! The writer sits beside that reader for one reason. Three crates emit JSON —
//! a protocol message, a generated descriptor and a capture-cost reading — and
//! a writer that lived in any one of them would be a dependency the other two
//! take on a crate they need for nothing else.
//!
//! # What that reader does and does not accept
//!
//! It accepts what a protocol message is made of: flow mappings, flow
//! sequences, double-quoted strings, and plain scalars for numbers. It refuses
//! a duplicate key, which JSON permits and no protocol message needs, and it
//! carries no `null` or boolean type of its own
//! ([Q2](../../../../docs/spec/09-decisions.md#q2--schema-format)), so a caller
//! reads those through [`crate::core_schema`]. Both limits are stated
//! here because they are the whole of the difference from a JSON library.

/// A JSON value this crate writes.
///
/// Deliberately small. Every message this crate emits is built from these four
/// shapes, and a value the protocol does not need is a value nothing tests.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Json {
    /// Written as it stands, which is how a request identifier is echoed back
    /// in the type it arrived in.
    Raw(String),
    String(String),
    Bool(bool),
    Array(Vec<Json>),
    /// In insertion order, because a recorded response is read by a person and
    /// a map that reordered itself would be a diff on every run.
    Object(Vec<(String, Json)>),
}

impl Json {
    pub fn string(text: impl Into<String>) -> Json {
        Json::String(text.into())
    }

    pub fn object(members: impl IntoIterator<Item = (&'static str, Json)>) -> Json {
        Json::Object(
            members
                .into_iter()
                .map(|(key, value)| (key.to_string(), value))
                .collect(),
        )
    }

    pub fn render(&self) -> String {
        let mut out = String::new();
        self.write(&mut out);
        out
    }

    /// The same value, indented, and ending in a newline.
    ///
    /// A protocol message is one line, because the client that reads it is a
    /// program and a newline inside a framed message buys nothing. A generated
    /// file is committed and read in a diff, so a change to one member has to
    /// arrive as a change to one line. Two renderings, one value, and the
    /// escaping below is shared rather than copied.
    ///
    /// An empty array or object stays on its own line. A two-line form for
    /// nothing is noise, and it is also a member that a diff would show moving
    /// when the thing it holds is still empty.
    pub fn render_pretty(&self) -> String {
        let mut out = String::new();
        self.write_pretty(0, &mut out);
        out.push('\n');
        out
    }

    fn write_pretty(&self, depth: usize, out: &mut String) {
        match self {
            Json::Array(items) if !items.is_empty() => {
                out.push_str("[\n");
                for (at, item) in items.iter().enumerate() {
                    indent(depth + 1, out);
                    item.write_pretty(depth + 1, out);
                    out.push_str(match at + 1 < items.len() {
                        true => ",\n",
                        false => "\n",
                    });
                }
                indent(depth, out);
                out.push(']');
            }
            Json::Object(members) if !members.is_empty() => {
                out.push_str("{\n");
                for (at, (key, value)) in members.iter().enumerate() {
                    indent(depth + 1, out);
                    escape(key, out);
                    out.push_str(": ");
                    value.write_pretty(depth + 1, out);
                    out.push_str(match at + 1 < members.len() {
                        true => ",\n",
                        false => "\n",
                    });
                }
                indent(depth, out);
                out.push('}');
            }
            other => other.write(out),
        }
    }

    fn write(&self, out: &mut String) {
        match self {
            Json::Raw(text) => out.push_str(text),
            Json::Bool(value) => out.push_str(match value {
                true => "true",
                false => "false",
            }),
            Json::String(text) => escape(text, out),
            Json::Array(items) => {
                out.push('[');
                for (at, item) in items.iter().enumerate() {
                    if at > 0 {
                        out.push(',');
                    }
                    item.write(out);
                }
                out.push(']');
            }
            Json::Object(members) => {
                out.push('{');
                for (at, (key, value)) in members.iter().enumerate() {
                    if at > 0 {
                        out.push(',');
                    }
                    escape(key, out);
                    out.push(':');
                    value.write(out);
                }
                out.push('}');
            }
        }
    }
}

fn indent(depth: usize, out: &mut String) {
    for _ in 0..depth {
        out.push_str("  ");
    }
}

/// A JSON string literal.
///
/// Every control character below `0x20` is escaped, because a summary is prose
/// an author wrote and a tab in it would otherwise produce a message no client
/// can parse. The two-character forms are used where JSON has one, and `\u00XX`
/// otherwise.
fn escape(text: &str, out: &mut String) {
    use std::fmt::Write;
    out.push('"');
    for character in text.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            control if (control as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", control as u32);
            }
            other => out.push(other),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_string_carries_every_character_a_summary_may_hold() {
        assert_eq!(
            Json::string("a \"quoted\" line\nand a tab\there").render(),
            "\"a \\\"quoted\\\" line\\nand a tab\\there\""
        );
        assert_eq!(Json::string("\u{1}").render(), "\"\\u0001\"");
        // An em dash is a character a rendered pointer holds, and it needs no
        // escape: JSON is Unicode, and a writer that escaped it would produce a
        // longer message that says the same thing.
        assert_eq!(Json::string("a — b").render(), "\"a — b\"");
    }

    #[test]
    fn an_object_keeps_the_order_it_was_written_in() {
        let value = Json::object([
            ("b", Json::Raw("1".to_string())),
            ("a", Json::Array(vec![Json::Bool(true)])),
        ]);
        assert_eq!(value.render(), "{\"b\":1,\"a\":[true]}");
    }

    /// The loader reads back what this writer emits, which is the property that
    /// makes one crate both halves of a protocol.
    #[test]
    fn the_yaml_loader_reads_what_this_writer_emits() {
        let message = Json::object([
            ("jsonrpc", Json::string("2.0")),
            ("id", Json::Raw("7".to_string())),
            (
                "params",
                Json::object([("task", Json::string("why is it this way"))]),
            ),
        ])
        .render();
        let read = crate::load(&message).expect("it reads");
        let map = read.value.as_map().expect("a mapping");
        assert_eq!(
            map.get("id")
                .and_then(|node| node.value.as_scalar())
                .map(|scalar| scalar.text.as_str()),
            Some("7")
        );
    }
}
