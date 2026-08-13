// SPDX-License-Identifier: Apache-2.0
//! The agent-facing surface of the same library: MCP over a line of JSON.
//!
//! [Spec 6](../../../../docs/spec/06-engine-architecture.md#mcp-server): "The
//! MCP server is the agent-facing surface of the same library." So the tools
//! here call the reads beside them and render with the same functions the CLI
//! renders with. Two renderings of one answer is the drift this repository
//! spends its comments on, and a client and a terminal reading different text
//! is that drift with a protocol in the middle.
//!
//! # Only the query class exists, and the reason is not an annotation
//!
//! [Spec 5](../../../../docs/spec/05-ai-integration.md#what-the-server-may-do-and-the-axis-that-decides-it)
//! lists three classes: query, working-tree write, and landed write. This
//! module registers the query class and no other, and spec 5 says why that is
//! the mechanism rather than a promise:
//!
//! > "The protocol lets a server declare that a tool only reads. It also states
//! > that a client must not treat that declaration from an untrusted server as
//! > a guarantee… Where a class of tool is off, the server does not register
//! > it, so no handler exists to call. A property that a caller reads off a
//! > tool list is a hint. A property with no code path behind it is a
//! > guarantee."
//!
//! [`TOOLS`] is that closed list. There is no branch in [`respond`] that writes
//! a byte to any file, and `tests/mcp.rs` asserts the absence rather than
//! trusting this comment. The `readOnlyHint` annotation is still emitted,
//! because a correct annotation costs nothing and a client may use it — it is
//! simply not what carries the property.
//!
//! # `check` is not here yet, and that is a scope statement
//!
//! Spec 5 puts `check` in the query class. It is the one tool of the six that
//! runs the check layer rather than reading the graph, so it needs a clock, a
//! cache and a corpus walk, and every one of those is a decision the CLI makes
//! at its own boundary. It waits for the surface that makes those injectable
//! rather than being reproduced here with different defaults.
//!
//! # The transport
//!
//! One JSON object per line, in and out. That is the framing MCP's stdio
//! transport fixes, and it is the framing that makes a server testable without
//! a process: [`respond`] takes the text of a request and returns the text of a
//! response, and [`serve`] is the loop around it.

use crate::{Budget, Resolved, Surface};
use headwater_yaml::json::Json;
use headwater_yaml::{Mapping, Value};

/// The protocol version this server implements.
///
/// Stated as a constant rather than echoed from the client's request. A server
/// that echoed whatever arrived would claim conformance to a revision it has
/// never seen, which is the same class of statement as a register that reports
/// an obligation as verified because a control names it.
const PROTOCOL: &str = "2024-11-05";

/// One tool of the query class.
pub struct Tool {
    pub name: &'static str,
    pub description: &'static str,
    /// The one argument each read takes, and what it is called.
    pub argument: &'static str,
    pub argument_description: &'static str,
}

/// The whole of what this server registers. Spec 5's query class, less `check`.
pub const TOOLS: [Tool; 5] = [
    Tool {
        name: "route",
        description: "Resolve a task description to the documents that govern it, as pointers: \
                      paths and one-line summaries, never content. Silent when nothing matches, \
                      because a wrong pointer costs more than a missing one.",
        argument: "task",
        argument_description: "What you are about to do, in your own words.",
    },
    Tool {
        name: "explain",
        description: "Why this document is the kind it is: the shelf that matched, the rule that \
                      fired, the purpose it serves, and what is consequently required of it.",
        argument: "target",
        argument_description: "A path in the corpus, or an identifier.",
    },
    Tool {
        name: "related",
        description: "What this document points at and what points at it, with the cue at each \
                      edge and which end governs the reading.",
        argument: "target",
        argument_description: "A path in the corpus, or an identifier.",
    },
    Tool {
        name: "resolve_identifier",
        description: "The document an identifier names, or the near miss where none carries it.",
        argument: "id",
        argument_description: "An identifier, as a document declares it.",
    },
    Tool {
        name: "governing_docs_for_path",
        description: "The documents that govern a path in the repository, through a declared \
                      governance edge onto that path.",
        argument: "path",
        argument_description: "A path in the repository, relative to its root.",
    },
];

/// Answer one request, or `None` where the message is a notification.
///
/// A notification carries no `id` and takes no response, which is the protocol's
/// rule and not a shortcut: a server that answered one would put a message on
/// the wire that no client is reading.
pub fn respond(surface: &Surface<'_>, request: &str) -> Option<String> {
    let parsed = headwater_yaml::load(request).ok()?;
    let message = parsed.value.as_map()?;
    let method = scalar(message, "method").unwrap_or_default();
    // A message with no `id` is a notification, and the `?` is the return.
    // `notifications/initialized` is the one this server receives, and any
    // other is ignored on the same rule.
    let id = raw(&message.get("id")?.value);

    let result = match method.as_str() {
        "initialize" => Ok(Json::object([
            ("protocolVersion", Json::string(PROTOCOL)),
            (
                "capabilities",
                Json::object([("tools", Json::object([("listChanged", Json::Bool(false))]))]),
            ),
            (
                "serverInfo",
                Json::object([
                    ("name", Json::string("headwater")),
                    ("version", Json::string(env!("CARGO_PKG_VERSION"))),
                ]),
            ),
        ])),
        "tools/list" => Ok(Json::object([("tools", tools())])),
        "tools/call" => call(surface, message),
        // The protocol's own code for a method a server does not implement.
        // Every write tool of spec 5's other two classes arrives here, because
        // no handler for one exists to reach.
        other => Err(Failure {
            code: -32601,
            message: format!("`{other}` is not a method this server implements"),
        }),
    };

    Some(match result {
        Ok(result) => Json::object([
            ("jsonrpc", Json::string("2.0")),
            ("id", Json::Raw(id)),
            ("result", result),
        ])
        .render(),
        Err(failure) => Json::object([
            ("jsonrpc", Json::string("2.0")),
            ("id", Json::Raw(id)),
            (
                "error",
                Json::object([
                    ("code", Json::Raw(failure.code.to_string())),
                    ("message", Json::string(failure.message)),
                ]),
            ),
        ])
        .render(),
    })
}

/// A JSON-RPC error, at the protocol's own codes.
struct Failure {
    code: i32,
    message: String,
}

fn tools() -> Json {
    Json::Array(
        TOOLS
            .iter()
            .map(|tool| {
                Json::object([
                    ("name", Json::string(tool.name)),
                    ("description", Json::string(tool.description)),
                    (
                        "inputSchema",
                        Json::object([
                            ("type", Json::string("object")),
                            (
                                "properties",
                                Json::Object(vec![(
                                    tool.argument.to_string(),
                                    Json::object([
                                        ("type", Json::string("string")),
                                        ("description", Json::string(tool.argument_description)),
                                    ]),
                                )]),
                            ),
                            ("required", Json::Array(vec![Json::string(tool.argument)])),
                        ]),
                    ),
                    (
                        "annotations",
                        Json::object([
                            ("readOnlyHint", Json::Bool(true)),
                            ("destructiveHint", Json::Bool(false)),
                        ]),
                    ),
                ])
            })
            .collect(),
    )
}

fn call(surface: &Surface<'_>, message: &Mapping) -> Result<Json, Failure> {
    let params = message
        .get("params")
        .and_then(|node| node.value.as_map())
        .ok_or_else(|| Failure {
            code: -32602,
            message: "a call names a tool and its arguments".to_string(),
        })?;
    let name = scalar(params, "name").unwrap_or_default();
    let arguments = params.get("arguments").and_then(|node| node.value.as_map());

    let tool = TOOLS.iter().find(|tool| tool.name == name).ok_or_else(|| {
        // The message names the tools that exist, because a client that asked
        // for a write tool is owed the reason rather than a bare refusal.
        let names: Vec<&str> = TOOLS.iter().map(|tool| tool.name).collect();
        Failure {
            code: -32602,
            message: format!(
                "`{name}` is not a tool this server registers. It registers {}, and it registers \
                 no tool that writes",
                names.join(", ")
            ),
        }
    })?;
    let argument = arguments
        .and_then(|map| scalar(map, tool.argument))
        .ok_or_else(|| Failure {
            code: -32602,
            message: format!(
                "`{}` takes `{}`, and none was given",
                tool.name, tool.argument
            ),
        })?;

    let text = match tool.name {
        "route" => surface.route(&argument, Budget::default()).render(),
        "explain" => match surface.explain(&argument) {
            Some(explanation) => explanation.render(),
            None => format!("{argument} is not a document of this corpus\n"),
        },
        "related" => match surface.find(&argument) {
            Some(document) => {
                let mut out = String::new();
                for neighbour in surface.related(&document) {
                    out.push_str(&neighbour.render());
                    out.push('\n');
                }
                match out.is_empty() {
                    true => format!("{argument} declares no relation, and none names it\n"),
                    false => out,
                }
            }
            None => format!("{argument} is not a document of this corpus\n"),
        },
        "resolve_identifier" => match surface.resolve_identifier(&argument) {
            Resolved::Document(pointer) => format!("{}\n", pointer.render()),
            Resolved::NearMiss(near) => {
                format!("no document carries {argument}, and one carries {near}\n")
            }
            Resolved::Nothing => format!("no document carries {argument}\n"),
        },
        "governing_docs_for_path" => {
            let governing = surface.governing_docs_for_path(&argument);
            match governing.is_empty() {
                true => format!("no document governs {argument}\n"),
                false => {
                    let mut out = String::new();
                    for pointer in &governing {
                        out.push_str(&pointer.render());
                        out.push('\n');
                    }
                    out
                }
            }
        }
        // Unreachable: `tool` came out of `TOOLS`. Answered rather than
        // panicked, because a panic inside a server is a dropped connection and
        // this is a sentence.
        other => format!("`{other}` is registered and not implemented\n"),
    };

    Ok(Json::object([
        (
            "content",
            Json::Array(vec![Json::object([
                ("type", Json::string("text")),
                ("text", Json::string(text)),
            ])]),
        ),
        ("isError", Json::Bool(false)),
    ]))
}

/// A scalar member of a mapping, as text.
fn scalar(map: &Mapping, key: &str) -> Option<String> {
    map.get(key)
        .and_then(|node| node.value.as_scalar())
        .map(|scalar| scalar.text.clone())
}

/// A value echoed back in the JSON type it arrived in.
///
/// A request identifier is a number or a string, and a response has to carry
/// back the one the client sent. The loader gives the text and the style it was
/// written in, and a plain scalar that reads as an integer is the number case.
fn raw(value: &Value) -> String {
    let Some(scalar) = value.as_scalar() else {
        return "null".to_string();
    };
    match headwater_yaml::core_schema::as_int(scalar).is_some() {
        true => scalar.text.clone(),
        false => Json::string(scalar.text.clone()).render(),
    }
}

/// The stdio loop: one JSON object per line, in and out.
///
/// It answers until standard input closes. A line that is not a message is
/// skipped rather than answered, because a response to an unparsed message
/// carries no identifier that any client could match it to.
pub fn serve(surface: &Surface<'_>, input: impl std::io::BufRead, mut output: impl std::io::Write) {
    for line in input.lines() {
        let Ok(line) = line else { return };
        if line.trim().is_empty() {
            continue;
        }
        let Some(response) = respond(surface, &line) else {
            continue;
        };
        if writeln!(output, "{response}").is_err() {
            return;
        }
        let _ = output.flush();
    }
}
