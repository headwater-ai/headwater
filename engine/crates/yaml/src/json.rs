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

/// One member of a JSON document, addressed by a path of keys.
///
/// `field(payload, ["tool_input", "file_path"])` reads `.tool_input.file_path`.
/// The answer is the text of a scalar: a string as it stands with its escapes
/// resolved, a number as it was written, and `true` or `false` for a boolean,
/// which is what a caller reading a flag off a wire compares against.
///
/// `None` is every way the read does not reach a scalar, and a caller that
/// distinguished them would be a caller acting on the shape of a message it
/// did not write. The document will not parse, a step of the path is not a
/// mapping, a key is absent, the member is an array or an object, or it
/// resolves to null. A hook treats all six as "the harness said nothing", and
/// the [hook contract](../../../../docs/spec/05-ai-integration.md#the-hook-contract-and-what-a-hook-cannot-bind)
/// makes that silence an outcome rather than a failure.
pub fn field(document: &str, path: &[String]) -> Option<String> {
    let scalar = walk(document, path)?.value.as_scalar()?.clone();
    match crate::core_schema::as_null(&scalar) {
        true => None,
        false => Some(scalar.text),
    }
}

/// How many elements the array or the mapping at that path holds.
///
/// A caller reads this to tell an empty collection from one with something in
/// it, which is a question `field` cannot answer and a caller cannot ask by
/// indexing: an empty array and an absent member both give nothing back, and
/// they are different facts about the message. `None` where the path reaches
/// no collection, on the same six grounds as [`field`].
pub fn count(document: &str, path: &[String]) -> Option<usize> {
    let node = walk(document, path)?;
    match &node.value {
        crate::value::Value::Seq(items) => Some(items.len()),
        crate::value::Value::Map(members) => Some(members.len()),
        crate::value::Value::Scalar(_) => None,
    }
}

/// The node a path of keys reaches, or `None`.
///
/// The reader is [`crate::load`], because JSON is a subset of the YAML 1.2 core
/// schema that the loader already implements. This function is therefore the
/// whole of the JSON reading this crate adds, and the escape table is
/// `saphyr-parser`'s rather than one written here.
fn walk(document: &str, path: &[String]) -> Option<crate::span::Spanned<crate::value::Value>> {
    let mut node = crate::load(document).ok()?;
    for key in path {
        let next = node.value.as_map()?.get(key)?.clone();
        node = next;
    }
    Some(node)
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

    /// The payload a harness puts on a hook's standard input, in the three
    /// shapes the three positions read.
    const PAYLOAD: &str = r#"{
      "session_id": "abc",
      "hook_event_name": "PreToolUse",
      "stop_hook_active": true,
      "prompt": "why is the hook reading JSON with an interpreter",
      "tool_input": {
        "file_path": "/home/a/docs/spec/05-ai-integration.md",
        "command": "*** Begin Patch\n*** Update File: docs/spec/05-ai-integration.md\n"
      },
      "pointers": [],
      "matched": [{"purpose": "rationale"}],
      "waiting_on": null
    }"#;

    fn path(keys: &[&str]) -> Vec<String> {
        keys.iter().map(|key| key.to_string()).collect()
    }

    #[test]
    fn a_field_is_read_at_the_top_level_and_under_a_nested_member() {
        assert_eq!(
            field(PAYLOAD, &path(&["hook_event_name"])).as_deref(),
            Some("PreToolUse")
        );
        assert_eq!(
            field(PAYLOAD, &path(&["tool_input", "file_path"])).as_deref(),
            Some("/home/a/docs/spec/05-ai-integration.md")
        );
    }

    /// The flag the review position reads. The core schema resolves it, so the
    /// caller compares against `true` rather than against a quoted literal.
    #[test]
    fn a_boolean_reads_as_the_word_a_caller_compares_against() {
        assert_eq!(
            field(PAYLOAD, &path(&["stop_hook_active"])).as_deref(),
            Some("true")
        );
    }

    /// An escape the harness wrote is resolved by the loader, so the value the
    /// caller reads is the value the harness sent. The write position reads a
    /// patch out of this member and its header lines are separated by one.
    #[test]
    fn an_escape_is_resolved_rather_than_handed_back() {
        let command = field(PAYLOAD, &path(&["tool_input", "command"])).expect("it reads");
        assert!(command.contains('\n'), "the newline is a newline");
        assert!(command.contains("*** Update File: docs/spec/05-ai-integration.md"));
    }

    /// The six ways a read reaches no scalar, which a caller treats alike.
    #[test]
    fn every_way_a_read_reaches_no_scalar_is_one_answer() {
        assert_eq!(field(PAYLOAD, &path(&["absent"])), None);
        assert_eq!(field(PAYLOAD, &path(&["tool_input", "absent"])), None);
        // A step of the path that is not a mapping.
        assert_eq!(field(PAYLOAD, &path(&["session_id", "deeper"])), None);
        // A member that is a collection rather than a scalar.
        assert_eq!(field(PAYLOAD, &path(&["tool_input"])), None);
        assert_eq!(field(PAYLOAD, &path(&["pointers"])), None);
        // Null is absent, and not the word `null`.
        assert_eq!(field(PAYLOAD, &path(&["waiting_on"])), None);
        // A document that will not parse.
        assert_eq!(field("{not json", &path(&["hook_event_name"])), None);
    }

    /// An empty array and an absent member are different facts about a message,
    /// and the count is what tells them apart.
    #[test]
    fn a_count_tells_an_empty_collection_from_an_absent_one() {
        assert_eq!(count(PAYLOAD, &path(&["pointers"])), Some(0));
        assert_eq!(count(PAYLOAD, &path(&["matched"])), Some(1));
        assert_eq!(count(PAYLOAD, &path(&["tool_input"])), Some(2));
        assert_eq!(count(PAYLOAD, &path(&["absent"])), None);
        assert_eq!(count(PAYLOAD, &path(&["session_id"])), None);
    }

    /// The writer and the reader are the two halves the hooks use, and a value
    /// that survives both is a value a hook may put on the wire.
    #[test]
    fn a_quoted_value_reads_back_as_itself() {
        let reason =
            "`docs/a.md` is a new document — run `headwater new`.\n\tIt \"refuses\" first.";
        let message = Json::object([("reason", Json::string(reason))]).render();
        assert_eq!(field(&message, &path(&["reason"])).as_deref(), Some(reason));
    }
}
