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
//! # Two classes, and a switch between them
//!
//! [Spec 5](../../../../docs/spec/05-ai-integration.md#what-the-server-may-do-and-the-axis-that-decides-it)
//! lists three classes: query, working-tree write, and landed write. This
//! module registers the first, registers the second where the operator asked
//! for it, and can register the third under no argument at all. Spec 5 says why
//! the registration rather than the annotation is what carries the property:
//!
//! > "The protocol lets a server declare that a tool only reads. It also states
//! > that a client must not treat that declaration from an untrusted server as
//! > a guarantee… Where a class of tool is off, the server does not register
//! > it, so no handler exists to call. A property that a caller reads off a
//! > tool list is a hint. A property with no code path behind it is a
//! > guarantee."
//!
//! [`QUERY_CLASS`] and [`WRITE_CLASS`] are those closed lists, and
//! [`registered`] is the whole of what a switch decides. The `readOnlyHint`
//! annotation is emitted correctly on both, because a correct annotation costs
//! nothing and a client may use it — it is simply not what carries the
//! property.
//!
//! # What replaced "this server registers no tool that writes"
//!
//! That sentence was a property of the code until the write class arrived, and
//! four narrower ones stand in its place. Each is a code path rather than a
//! promise, and each is asserted in `tests/mcp.rs`.
//!
//! 1. **A server with no switch is the server that was here before.** With
//!    [`Server::writing`] at `None`, [`registered`] returns the six reads, and
//!    no handler that writes exists to reach. The old property holds verbatim
//!    for the default shape, which is the shape a client reaches by connecting
//!    to a checkout nobody meant to change.
//! 2. **The landed-write class is reachable by no argument.** A commit, a push
//!    and a merge are refused by spec 5 rather than by a flag, and no table
//!    here holds one. That is the row that mattered for the disclosure
//!    argument: an injected instruction needs an actuator, and a server that
//!    cannot land a change lends it none.
//! 3. **Every write leaves a record in the tree a human commits.** `new`
//!    appends a capture-cost reading that names the protocol as the surface of
//!    the run, so the reading and the document land in one diff. `fix` writes
//!    only what a finding derived under the fixability bar, and answers with
//!    the run *after* the write, so a patch that produced a document the checks
//!    reject reports it in the same answer.
//! 4. **No write tool accepts a document.** `headwater new` writes no
//!    `accepted_by` and no warrant of `accepted`, so neither does this. [Spec
//!    3's stop rule 5](../../../../docs/spec/03-authoring-and-lifecycle.md#provenance-is-recorded-not-assumed)
//!    forbids an agent from stamping its own output, and
//!    [HW-OBL-0108](../../../../docs/obligations/0108-an-agent-writes-the-acceptance-stamp-of-every-document-in-this-corpus.md)
//!    measures how often an agent does it anyway. What this surface adds to
//!    that is nothing: there is no field for it in what the verb writes, so the
//!    stamp is still an edit a reader can see in a diff.
//!
//! # A write ends the session, because the walk behind every answer is gone
//!
//! The corpus is walked once, before the server accepts a message, and every
//! tool answers from that walk. A write tool ends the tree that walk described.
//! [HW-OBL-0028](../../../../docs/obligations/0028-a-run-cannot-report-the-corpus-tree-because-nothing-computes.md)
//! measured what a stale walk costs: a server started before an edit reported
//! 28 findings where a fresh run reported 30, and its two answers were
//! identical across the edit. A server that wrote and then kept answering would
//! be that measurement, caused by the server itself.
//!
//! So a call that moves a byte of the checkout spends the server.
//! [`Session::spent`] holds that one fact, and every later `tools/call` is
//! refused by name with the reason. The two halves of the rule matter equally.
//!
//! - **The seal follows the bytes and never the call.** A `fix` that found no
//!   patch to write, and a `new` that the scaffolder refused, leave the tree
//!   as the walk described it. Such a call is not a write, and the session
//!   stays open.
//! - **The writing call still answers from a fresh walk.** A write tool is a
//!   verb of the terminal, run in full, and the verb re-reads the tree it
//!   wrote before it reports. So the one answer that could have been stale is
//!   the one answer this server never gives from the old walk.
//!
//! [`Server`] therefore stays immutable, and [`Session`] holds the single bit
//! that is not. "A caller that wants another tree starts another server" is the
//! remedy spec 5 already states for the clock, and this is that sentence with a
//! code path under it.
//!
//! # `check` runs the check layer, and its three inputs arrive rather than
//! being chosen here
//!
//! `check` is the one tool of the six that runs the check layer rather than
//! reading the graph, so it needs a clock, a cache and a corpus walk. Every one
//! of those is a decision, and a tool that took them again would put a second
//! set of defaults behind a protocol. So none of the three is taken here.
//! [Spec 5](../../../../docs/spec/05-ai-integration.md#what-a-check-tool-decides-and-where-each-decision-is-taken)
//! is where they are decided, and this is what the decisions come to in code.
//!
//! **The clock arrives.** [`Server::now`] is read once, when the server starts,
//! by the same two lines `headwater check` reads it with: `--now` where the
//! operator gave one, and the system date otherwise. A server whose host cannot
//! say what day it is does not start. Every result states the value, in all
//! four formats, because it is an input to the verdict.
//!
//! **There is no cache.** [`headwater_check::Cache::at`] reads a store in the
//! checkout and [`headwater_check::Cache::write`] puts it back, and a tool that
//! did the second thing would be a tool that writes. A cache that read and
//! never wrote is a third mode that no other caller of this engine has, which
//! is a second set of defaults reached by another road. So the run is
//! [`headwater_check::Cache::disabled`], and the invariant that makes that free
//! of consequence is the one spec 12 states: a cached run and an uncached run
//! write the same bytes, so refusing the cache costs time and never an answer.
//!
//! **There is no corpus walk.** The corpus is walked once, before the server
//! starts, and [`Server::census`] and [`Server::graph`] are what that walk
//! produced. Every one of the six tools answers from it, so `check` costs no
//! walk that `route` does not. What it costs is one run of the check layer over
//! two structures already in memory.
//!
//! # The one argument is the format, and there is no default for it
//!
//! Spec 6 closes the set of output formats at four, and
//! [`headwater_adapter::render`] writes all four for the CLI and for this tool
//! alike. So the argument this tool takes is the one the CLI also takes at this
//! boundary, and the answer is byte-identical to `headwater check --format
//! <name>` over the same corpus at the same date. It is required rather than
//! defaulted: a default chosen here is a decision taken in a call site, which
//! is the thing this whole section exists to avoid.
//!
//! It takes no path. A run is over the whole corpus because coverage is
//! computed against the census as its denominator, and a report filtered to a
//! path would carry either that denominator with fewer findings under it or a
//! smaller one that no run evaluated. Both are a second answer about one
//! corpus.
//!
//! # A tool takes the arguments its verb takes, and no write decision is taken
//! here
//!
//! One string was enough for every read, and it is not enough for `new`, which
//! takes a kind, a title and a repeatable relation. So [`Tool`] carries a list
//! of [`Argument`] rather than one name, and a repeatable one arrives as an
//! array of strings. The member of each pair is written `<relation>=<identifier>`,
//! which is the form `headwater new --relates` takes, so a caller who read
//! spec 6 types the same string here.
//!
//! What the write tools do **not** take is the whole of the argument. `fix`
//! takes a format and nothing else: no clock, because the server holds one for
//! its life; no root, because the checkout is the one the server was started
//! over; and no path, for the reason `check` takes none. `new` takes no clock
//! either. Every one of those is a decision the CLI takes at its own boundary,
//! and a tool that took one again would be a second set of defaults behind a
//! protocol.
//!
//! [`Writing`] is how that stays true. It holds the two verbs as functions the
//! caller supplies, so the run behind a tool call is the run behind the
//! terminal invocation and not a second composition of the same parts. The
//! result comes back as [`Written`]: the two streams every verb of this engine
//! writes, held rather than printed, and a statement of whether a byte landed.
//! The account and the artifact reach a client as two content blocks, in that
//! order, so the artifact block is the bytes the terminal reads on standard
//! output.
//!
//! There is no cache in either class. A write tool has a root, so the reason
//! `check` gives — that a cache write is a write this server does not do — no
//! longer covers it. What covers it is the invariant spec 12 carries: a cached
//! run and an uncached run write the same bytes, so refusing the cache costs
//! time and never an answer. One rule for the whole server is worth more than a
//! second mode reached by a switch.
//!
//! # The transport
//!
//! One JSON object per line, in and out. That is the framing MCP's stdio
//! transport fixes, and it is the framing that makes a server testable without
//! a process: [`respond`] takes the text of a request and returns the text of a
//! response, and [`serve`] is the loop around it.

use crate::{Budget, Resolved, Surface};
use headwater_adapter::{Format, Subject};
use headwater_census::census::Census;
use headwater_check::{Cache, Context, Declared};
use headwater_graph::Graph;
use headwater_yaml::json::Json;
use headwater_yaml::{Mapping, Value};

/// The protocol version this server implements.
///
/// Stated as a constant rather than echoed from the client's request. A server
/// that echoed whatever arrived would claim conformance to a revision it has
/// never seen, which is the same class of statement as a register that reports
/// an obligation as verified because a control names it.
const PROTOCOL: &str = "2024-11-05";

/// What one server answers from, fixed before it accepts a message.
///
/// Every field here is a value some caller decided at its own boundary and
/// handed over. A server that reached for one of them itself — a clock, a
/// cache, a second walk of the corpus — would be the second set of defaults the
/// module comment refuses. Nothing here is mutable, which is why two calls in
/// one session answer the same bytes.
pub struct Server<'a> {
    /// The graph reads: five of the six tools are a projection of this.
    pub surface: Surface<'a>,
    /// What the walk before start-up produced. The surface reads it too, and it
    /// is here as well because a check run takes it as its denominator.
    pub census: &'a Census,
    /// The edges that walk produced, for the same two readers.
    pub graph: &'a Graph,
    /// The declarations one run of the check layer reads, out of the committed
    /// lock.
    pub declared: Declared<'a>,
    /// The taxonomy package and its version, which every rendered report
    /// states beside the lock digest.
    pub package: &'a str,
    pub version: &'a str,
    /// The clock, read once when the server started, and reported in every
    /// result. A protocol call has no boundary of its own to read one at, and a
    /// tool argument would let a caller pick the date that answers the way it
    /// wanted.
    pub now: Context,
    /// The working-tree write class, where the operator asked for it.
    ///
    /// `None` is the default shape, and it is the whole of the old property: no
    /// tool that writes is registered, so no handler that writes exists to
    /// reach. Spec 5 fixes the default, because a client may connect to a
    /// checkout that the user did not intend to change.
    pub writing: Option<Writing<'a>>,
}

/// The two verbs of the working-tree write class, as the caller runs them.
///
/// Held as functions rather than built here. A write needs a root, a
/// scaffolder, a clock and a walk after the write, and every one of those is a
/// decision the CLI takes at its own boundary. A module that reached for them
/// would be the second set of defaults this whole file exists to refuse.
pub struct Writing<'a> {
    /// `headwater new <kind> --title <text> [--relates <relation>=<identifier>]`.
    #[allow(clippy::type_complexity)]
    pub scaffold: &'a dyn Fn(&str, &str, &[(String, String)]) -> Result<Written, String>,
    /// `headwater check --fix --format <format>`.
    pub fix: &'a dyn Fn(Format) -> Result<Written, String>,
}

/// What a verb wrote, held rather than printed.
///
/// [Spec 5](../../../../docs/spec/05-ai-integration.md#the-hook-contract-and-what-a-hook-cannot-bind)
/// fixes two streams and a status for every verb of this engine: "Standard
/// output carries the one artifact… Standard error carries the account of what
/// the engine did." A protocol call has one result rather than two streams, so
/// the two arrive as two content blocks and keep their separation.
pub struct Written {
    /// What the terminal reads on standard error: what was written, and what
    /// refused the patch it was offered.
    pub account: String,
    /// What the terminal reads on standard output, byte for byte.
    pub artifact: String,
    /// Whether a byte of the checkout moved.
    ///
    /// The seal follows this rather than the call. A verb that refused, or that
    /// found nothing to write, leaves the tree as the startup walk described
    /// it, and such a call is not a write.
    pub landed: bool,
    /// The status, which is the third term of the same contract.
    ///
    /// It reaches a client as `isError`, so a run that wrote a document and
    /// failed to record its capture-cost reading is reported as the partial
    /// result it is, rather than as a success with a line of prose about it.
    pub ok: bool,
}

/// The one fact about a server that is not fixed before it accepts a message.
///
/// [`Server`] is immutable, which is why two reads in one session answer the
/// same bytes. This is the exception, and it is one bit: whether a call has
/// moved a byte of the checkout, and which one did.
#[derive(Default)]
pub struct Session {
    spent: Option<&'static str>,
}

impl Session {
    /// The tool that ended this session, where one has.
    pub fn spent(&self) -> Option<&'static str> {
        self.spent
    }
}

impl Server<'_> {
    /// What every report of this server states about the run behind it.
    fn subject<'a>(&'a self, now: &'a str) -> Subject<'a> {
        Subject {
            package: self.package,
            version: self.version,
            lock: self.declared.lock,
            now,
        }
    }
}

/// One argument of one tool.
pub struct Argument {
    pub name: &'static str,
    pub description: &'static str,
    /// A call that omits it is refused by name. Every read takes exactly one
    /// required argument, which is what makes the schema of a read what it was.
    pub required: bool,
    /// It arrives as an array of strings rather than as one string.
    ///
    /// A caller that sent a bare string is refused with the shape named. Two
    /// accepted shapes for one argument is a second reading of one request, and
    /// the engine refuses those everywhere else.
    pub repeatable: bool,
}

impl Argument {
    /// One string, and a call without it is refused.
    pub const fn required(name: &'static str, description: &'static str) -> Argument {
        Argument {
            name,
            description,
            required: true,
            repeatable: false,
        }
    }

    /// A list of strings, and an absent list is an empty one.
    pub const fn repeatable(name: &'static str, description: &'static str) -> Argument {
        Argument {
            name,
            description,
            required: false,
            repeatable: true,
        }
    }
}

/// One tool of one class.
pub struct Tool {
    pub name: &'static str,
    pub description: &'static str,
    pub arguments: &'static [Argument],
    /// Whether a call may move a byte of the checkout. It decides the
    /// annotations, and nothing else reads it: what a handler does is what the
    /// handler does.
    pub writes: bool,
}

impl Tool {
    /// The one argument a read takes, for a caller that holds a tool and wants
    /// the name to put a value under.
    pub fn only(&self) -> &'static Argument {
        &self.arguments[0]
    }
}

/// Spec 5's query class: it changes nothing, and it is always registered.
pub const QUERY_CLASS: [Tool; 6] = [
    Tool {
        name: "route",
        description: "Resolve a task description to the documents that govern it, as pointers: \
                      paths and one-line summaries, never content. Silent when nothing matches, \
                      because a wrong pointer costs more than a missing one.",
        arguments: &[Argument::required(
            "task",
            "What you are about to do, in your own words.",
        )],
        writes: false,
    },
    Tool {
        name: "explain",
        description: "Why this document is the kind it is: the shelf that matched, the rule that \
                      fired, the purpose it serves, and what is consequently required of it.",
        arguments: &[Argument::required(
            "target",
            "A path in the corpus, or an identifier.",
        )],
        writes: false,
    },
    Tool {
        name: "related",
        description: "What this document points at and what points at it, with the cue at each \
                      edge and which end governs the reading.",
        arguments: &[Argument::required(
            "target",
            "A path in the corpus, or an identifier.",
        )],
        writes: false,
    },
    Tool {
        name: "resolve_identifier",
        description: "The document an identifier names, or the near miss where none carries it.",
        arguments: &[Argument::required(
            "id",
            "An identifier, as a document declares it.",
        )],
        writes: false,
    },
    Tool {
        name: "governing_docs_for_path",
        description: "The documents that govern a path in the repository, through a declared \
                      governance edge onto that path.",
        arguments: &[Argument::required(
            "path",
            "A path in the repository, relative to its root.",
        )],
        writes: false,
    },
    Tool {
        name: "check",
        description: "Run every check over the whole corpus and report what this run found: the \
                      taxonomy it read, the date it evaluated against, the coverage, and each \
                      finding with its remediation. The bytes are those of `headwater check \
                      --format <format>` over the same corpus at the same date. It writes \
                      nothing, and it uses no cache.",
        arguments: &[Argument::required(
            "format",
            "One of `text`, `json`, `sarif`, `markdown`. `text` is the whole report in the \
             engine's own words, `markdown` is the findings as a job summary, `sarif` is a check \
             run, and `json` is the finding shape itself. There is no default: the vocabulary is \
             the caller's to choose.",
        )],
        writes: false,
    },
];

/// Spec 5's working-tree write class, registered only where the operator asked
/// for it.
///
/// Both tools are the verb of the same name, run in full over the checkout the
/// server was started in. Neither takes a clock, a root or a path, for the
/// reasons the module comment states. The third class of spec 5 — a commit, a
/// push, a merge — has no table here and no switch that would reach one.
pub const WRITE_CLASS: [Tool; 2] = [
    Tool {
        name: "new",
        description: "Scaffold a document of a kind into this working tree: the placement its \
                      shelf dictates, the front matter its facets require, the sections its \
                      contract requires, an identifier under its scheme, and the edges the \
                      taxonomy assigns to a scaffold. It writes what `headwater new` writes. It \
                      decides everything before it writes anything, it never overwrites a \
                      document, and it refuses rather than guessing. It writes no acceptance: no \
                      field it fills records that a human accepted the result, because acceptance \
                      is a human act. The run appends one capture-cost reading naming this \
                      protocol as its surface, so the reading and the document land in one diff. \
                      A call that writes a document ends this server.",
        arguments: &[
            Argument::required(
                "kind",
                "The kind to scaffold, as the taxonomy names it. Call `explain` on a document of \
                 the kind, or read the refusal, which names every kind this taxonomy declares.",
            ),
            Argument::required(
                "title",
                "The document's own name. The file name and the facet in the `name` role both \
                 come from it, and this engine invents neither.",
            ),
            Argument::repeatable(
                "relates",
                "An edge to declare, written `<relation>=<identifier>`, which is the form \
                 `headwater new --relates` takes. A list, and an absent list is no edge. The verb \
                 refuses a relation the taxonomy assigns to another creator, an end the relation \
                 forbids, and a target that resolves to nothing.",
            ),
        ],
        writes: true,
    },
    Tool {
        name: "fix",
        description: "Write the patch that rides with each finding of a run, in this working \
                      tree, then report the run after the write. A finding carries a patch only \
                      where the remedy is mechanical and total, and a finding an author \
                      suppressed carries none. Every patch is held against the bytes it names and \
                      every result is read back, so a document whose shape this engine guessed \
                      wrong is refused with nothing written for that file. The bytes are those of \
                      `headwater check --fix --format <format>`, and the account of what was \
                      written is the first content block. A call that wrote a file ends this \
                      server; a call that found no patch to write does not.",
        arguments: &[Argument::required(
            "format",
            "One of `text`, `json`, `sarif`, `markdown`, on the same terms `check` takes it. \
             There is no default.",
        )],
        writes: true,
    },
];

/// What this server registers, which is the whole of what a switch decides.
///
/// The query class always, and the write class where the operator started the
/// server with it. A tool outside both reaches no handler, and there is no
/// third list for it to be in.
pub fn registered(server: &Server<'_>) -> Vec<&'static Tool> {
    let mut out: Vec<&'static Tool> = QUERY_CLASS.iter().collect();
    if server.writing.is_some() {
        out.extend(WRITE_CLASS.iter());
    }
    out
}

/// Answer one request, or `None` where the message is a notification.
///
/// A notification carries no `id` and takes no response, which is the protocol's
/// rule and not a shortcut: a server that answered one would put a message on
/// the wire that no client is reading.
pub fn respond(server: &Server<'_>, session: &mut Session, request: &str) -> Option<String> {
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
        // The table is a fact about the registration and not about the tree, so
        // a spent server still answers it. What it may not do is call one.
        "tools/list" => Ok(Json::object([("tools", tools(server))])),
        "tools/call" => match session.spent {
            Some(tool) => Err(spent(tool)),
            None => call(server, message).map(|answer| {
                if answer.landed {
                    session.spent = Some(answer.tool);
                }
                answer.result
            }),
        },
        // The protocol's own code for a method a server does not implement.
        // Every tool of spec 5's landed-write class arrives here or at the tool
        // refusal below, because no handler for one exists to reach.
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

/// The refusal a spent server answers every call with.
///
/// `-32000` is the first of the range JSON-RPC reserves for a server to define,
/// which is what this is: the request is well formed and the server can no
/// longer answer it. The message names the tool that ended the session, because
/// a caller that reads only this line is owed the cause.
fn spent(tool: &str) -> Failure {
    Failure {
        code: -32000,
        message: format!(
            "this server walked the corpus once, before it accepted a message, and `{tool}` then \
             moved a byte of that tree. Every answer it could still give is about a tree that is \
             gone, so it answers none. Start another server over the tree as it is now"
        ),
    }
}

/// What one tool call produced.
///
/// The tool name rides with the result because the seal names the call that
/// ended the session, and a caller of a spent server reads that name.
struct Answer {
    tool: &'static str,
    result: Json,
    landed: bool,
}

fn tools(server: &Server<'_>) -> Json {
    Json::Array(
        registered(server)
            .into_iter()
            .map(|tool| {
                let required: Vec<Json> = tool
                    .arguments
                    .iter()
                    .filter(|argument| argument.required)
                    .map(|argument| Json::string(argument.name))
                    .collect();
                Json::object([
                    ("name", Json::string(tool.name)),
                    ("description", Json::string(tool.description)),
                    (
                        "inputSchema",
                        Json::object([
                            ("type", Json::string("object")),
                            (
                                "properties",
                                Json::Object(
                                    tool.arguments
                                        .iter()
                                        .map(|argument| {
                                            let shape = match argument.repeatable {
                                                false => vec![("type", Json::string("string"))],
                                                true => vec![
                                                    ("type", Json::string("array")),
                                                    (
                                                        "items",
                                                        Json::object([(
                                                            "type",
                                                            Json::string("string"),
                                                        )]),
                                                    ),
                                                ],
                                            };
                                            let mut members = shape;
                                            members.push((
                                                "description",
                                                Json::string(argument.description),
                                            ));
                                            (argument.name.to_string(), Json::object(members))
                                        })
                                        .collect(),
                                ),
                            ),
                            ("required", Json::Array(required)),
                        ]),
                    ),
                    (
                        // Correct, and not what carries the property. Spec 5: a
                        // client must not treat this declaration from an
                        // untrusted server as a guarantee. The registration
                        // above is what a caller may rely on.
                        "annotations",
                        Json::object([
                            ("readOnlyHint", Json::Bool(!tool.writes)),
                            ("destructiveHint", Json::Bool(tool.writes)),
                        ]),
                    ),
                ])
            })
            .collect(),
    )
}

fn call(server: &Server<'_>, message: &Mapping) -> Result<Answer, Failure> {
    let surface = &server.surface;
    let params = message
        .get("params")
        .and_then(|node| node.value.as_map())
        .ok_or_else(|| Failure {
            code: -32602,
            message: "a call names a tool and its arguments".to_string(),
        })?;
    let name = scalar(params, "name").unwrap_or_default();
    let arguments = params.get("arguments").and_then(|node| node.value.as_map());

    let known = registered(server);
    let tool = known
        .iter()
        .find(|tool| tool.name == name)
        .ok_or_else(|| {
            // The message names the tools that exist, because a client that
            // asked for a tool this server does not carry is owed the reason
            // rather than a bare refusal. There are two reasons, and they are
            // different facts: a switch the operator did not give, and a class
            // spec 5 refuses outright.
            let names: Vec<&str> = known.iter().map(|tool| tool.name).collect();
            let why = match server.writing.is_some() {
                false => {
                    "and it registers no tool that writes. The working-tree write class is off, \
                     and `headwater mcp --write` is what registers `new` and `fix`. A tool that \
                     lands a change — a commit, a push, a merge — is registered by no switch at \
                     all, because acceptance is a human act"
                }
                true => {
                    "and that is every tool it has. A tool that lands a change — a commit, a \
                     push, a merge — is registered by no switch, because acceptance is a human \
                     act: this server writes into a working tree that a person reviews and \
                     commits"
                }
            };
            Failure {
                code: -32602,
                message: format!(
                    "`{name}` is not a tool this server registers. It registers {}, {why}",
                    names.join(", ")
                ),
            }
        })
        .copied()?;
    // Every declared argument, read in the shape the tool declares for it. A
    // required one that is absent is a refusal naming it, and a repeatable one
    // that arrived as a bare string is a refusal naming the shape.
    let mut given: Vec<(&'static str, String)> = Vec::new();
    let mut listed: Vec<(&'static str, Vec<String>)> = Vec::new();
    for argument in tool.arguments {
        match argument.repeatable {
            false => match arguments.and_then(|map| scalar(map, argument.name)) {
                Some(value) => given.push((argument.name, value)),
                None if argument.required => {
                    return Err(Failure {
                        code: -32602,
                        message: format!(
                            "`{}` takes `{}`, and none was given",
                            tool.name, argument.name
                        ),
                    })
                }
                None => {}
            },
            true => listed.push((argument.name, repeated(arguments, tool.name, argument)?)),
        }
    }
    let held = |name: &str| -> String {
        given
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, value)| value.clone())
            .unwrap_or_default()
    };
    let repeated_held = |name: &str| -> Vec<String> {
        listed
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, values)| values.clone())
            .unwrap_or_default()
    };

    if tool.writes {
        return write(server, tool, &held, &repeated_held);
    }
    let argument = held(tool.only().name);

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
        "check" => check(server, &argument)?,
        // Unreachable: `tool` came out of `registered`. Answered rather than
        // panicked, because a panic inside a server is a dropped connection and
        // this is a sentence.
        other => format!("`{other}` is registered and not implemented\n"),
    };

    Ok(Answer {
        tool: tool.name,
        result: blocks(&[text], true),
        // A read moves no byte. That is the property `registered` carries for
        // the query class, and this is the line that spends nothing.
        landed: false,
    })
}

/// One or two content blocks, in the shape a tool result takes.
fn blocks(texts: &[String], ok: bool) -> Json {
    Json::object([
        (
            "content",
            Json::Array(
                texts
                    .iter()
                    .filter(|text| !text.is_empty())
                    .map(|text| {
                        Json::object([
                            ("type", Json::string("text")),
                            ("text", Json::string(text.clone())),
                        ])
                    })
                    .collect(),
            ),
        ),
        ("isError", Json::Bool(!ok)),
    ])
}

/// A repeatable argument, which arrives as a list of strings or not at all.
fn repeated(
    arguments: Option<&Mapping>,
    tool: &str,
    argument: &Argument,
) -> Result<Vec<String>, Failure> {
    let Some(node) = arguments.and_then(|map| map.get(argument.name)) else {
        return Ok(Vec::new());
    };
    let refuse = || Failure {
        code: -32602,
        message: format!(
            "`{tool}` takes `{}` as a list of strings, and one arrived in another shape. Write \
             `[\"{}\"]` rather than a bare string",
            argument.name,
            match argument.name {
                "relates" => "supersedes=HW-DR-0007",
                other => other,
            }
        ),
    };
    let items = node.value.as_seq().ok_or_else(refuse)?;
    items
        .iter()
        .map(|item| {
            item.value
                .as_scalar()
                .map(|scalar| scalar.text.clone())
                .ok_or_else(refuse)
        })
        .collect()
}

/// One call of the working-tree write class.
///
/// Both arms are the verb the caller supplied, run whole. Nothing about a run
/// is decided here: this function reads the arguments the tool declared, hands
/// them over, and turns the two streams that come back into two content blocks.
fn write(
    server: &Server<'_>,
    tool: &'static Tool,
    held: &dyn Fn(&str) -> String,
    listed: &dyn Fn(&str) -> Vec<String>,
) -> Result<Answer, Failure> {
    // Unreachable: `registered` lists the write class only where this is
    // `Some`, and the tool came out of `registered`. Answered rather than
    // unwrapped, for the reason the arm above is answered.
    let Some(writing) = &server.writing else {
        return Err(Failure {
            code: -32603,
            message: format!("`{}` is registered and this server cannot write", tool.name),
        });
    };

    let written = match tool.name {
        "new" => {
            let mut relates = Vec::new();
            for pair in listed("relates") {
                // The form is the CLI's form, and a value that is not in it is
                // refused rather than guessed at. A relation name with no
                // target is an edge to nothing.
                let Some((relation, target)) = pair.split_once('=') else {
                    return Err(Failure {
                        code: -32602,
                        message: format!(
                            "`{pair}` is not `<relation>=<identifier>`, which is the form \
                             `relates` takes"
                        ),
                    });
                };
                relates.push((relation.to_string(), target.to_string()));
            }
            (writing.scaffold)(&held("kind"), &held("title"), &relates).map_err(|why| Failure {
                // Every refusal of the scaffolder is about the request: an
                // unknown kind, an end a relation forbids, a target that
                // resolves to nothing, a document that already exists. It
                // decides everything before it writes anything, so a refusal
                // wrote nothing.
                code: -32602,
                message: format!("nothing was written. {why}"),
            })?
        }
        "fix" => {
            let named = held("format");
            let Some(format) = Format::parse(&named) else {
                let names: Vec<&str> = Format::ALL.iter().map(|format| format.name()).collect();
                return Err(Failure {
                    code: -32602,
                    message: format!("`fix` takes a format, one of {}", names.join(", ")),
                });
            };
            (writing.fix)(format).map_err(|why| Failure {
                // A refusal here is a file this engine would not write, which
                // is a fact about the tree rather than about the request.
                code: -32000,
                message: why,
            })?
        }
        other => {
            return Err(Failure {
                code: -32603,
                message: format!("`{other}` is registered and not implemented"),
            })
        }
    };

    Ok(Answer {
        tool: tool.name,
        result: blocks(&[written.account, written.artifact], written.ok),
        landed: written.landed,
    })
}

/// One run of the check layer, in the format the caller named.
///
/// The three inputs the module comment settles are all read off [`Server`]
/// rather than decided here, and the fourth is this function's argument. What
/// is left is the call the CLI makes with the same five values, so the bytes
/// are the CLI's bytes.
fn check(server: &Server<'_>, format: &str) -> Result<String, Failure> {
    let Some(format) = Format::parse(format) else {
        let names: Vec<&str> = Format::ALL.iter().map(|format| format.name()).collect();
        return Err(Failure {
            code: -32602,
            // The same list the CLI refuses an unknown `--format` with, because
            // a caller told two different things about one closed set has to
            // find out which one is current.
            message: format!("`check` takes a format, one of {}", names.join(", ")),
        });
    };
    // No cache, and the run is therefore complete every time. See the module
    // comment: one rule for the whole server, and spec 12's invariant is what
    // makes it cost time rather than an answer.
    let mut cache = Cache::disabled();
    let run = headwater_check::run(
        server.census,
        server.graph,
        &server.declared,
        &server.now,
        &mut cache,
    );
    let artifact = headwater_adapter::render(
        &run,
        server.census,
        server.graph,
        &server.subject(&server.now.now().render()),
        format,
    );
    // The audit the CLI fails a run on, which a caller here is less able to
    // perform for itself: it holds one artifact rather than a terminal, and a
    // finding that reached no output is invisible to it. A defective adapter is
    // an engine defect rather than a fact about the corpus, so it reaches the
    // protocol as an internal error and never as a report with a hole in it.
    let audited = headwater_adapter::census(&run, &artifact);
    match audited.is_defective() {
        false => Ok(artifact),
        true => Err(Failure {
            code: -32603,
            message: format!(
                "the {} adapter dropped {} of {} findings with no declared loss reason: {}",
                format.name(),
                audited.unaccounted.len(),
                audited.findings,
                audited.unaccounted.join(", ")
            ),
        }),
    }
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
///
/// The session outlives no loop: a server that wrote keeps answering the
/// refusal until the client closes the pipe, rather than exiting under a client
/// that is still talking to it.
pub fn serve(server: &Server<'_>, input: impl std::io::BufRead, mut output: impl std::io::Write) {
    let mut session = Session::default();
    for line in input.lines() {
        let Ok(line) = line else { return };
        if line.trim().is_empty() {
            continue;
        }
        let Some(response) = respond(server, &mut session, &line) else {
            continue;
        };
        if writeln!(output, "{response}").is_err() {
            return;
        }
        let _ = output.flush();
    }
}
