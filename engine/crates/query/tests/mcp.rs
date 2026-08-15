// SPDX-License-Identifier: Apache-2.0
//! The MCP surface, over the same fixture tree the reads are recorded on.
//!
//! The recorded file is `fixtures/query.mcp`: a session, request by response.
//! Everything a recording cannot carry is asserted instead — that a server with
//! no switch registers no tool that writes, that the landed-write class reaches
//! no handler under any switch, that the `check` tool answers the bytes the CLI
//! answers over the same corpus at the same date, and that a call which moved a
//! byte ends the session.
//!
//!     HEADWATER_BLESS=1 cargo test -p headwater-query --test mcp
//!
//! # The write class is a pair of functions here, as it is in the CLI
//!
//! [`headwater_query::mcp::Writing`] holds the two verbs rather than the parts
//! of them, so this suite states what a verb did rather than running a
//! scaffolder. That is the point of the shape: a test can hand the server a
//! verb that writes a stated file and a verb that writes nothing, and then
//! assert the one rule the server itself owns — that the seal follows the bytes
//! and never the call.

use headwater_adapter::{Format, Subject};
use headwater_census::census::{self, Census};
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::register::Register;
use headwater_check::{Cache, Context, Date, Declared, Shape};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use headwater_query::mcp::{self, Server, Session, Writing, Written, QUERY_CLASS, WRITE_CLASS};
use headwater_query::Surface;
use std::path::{Path, PathBuf};

/// The date the recorded session is answered at.
///
/// Stated rather than read, because the report a `check` call returns names the
/// date it evaluated against and a recording of it would otherwise change every
/// midnight.
///
/// The value is inside one document's participation window and past another's,
/// which is what [`LATER`] is chosen against.
const RECORDED_AT: &str = "2026-03-15";

/// A date after every participation window of the fixture tree has elapsed.
///
/// A server holds the clock it started with, so two servers over one corpus
/// differ in what they report and in what they find. This is the second of the
/// two, and the finding count between them is the difference.
const LATER: &str = "2027-03-01";

/// The lock digest this fixture answers under.
///
/// The fixture taxonomy is not a resolved package, so no lock file states one.
/// The value is still an input to every report, so it is a constant here rather
/// than an empty string that a reader would take for a missing field.
const LOCK: &str = "sha256:0000000000000000000000000000000000000000000000000000000000000000";

/// One session, in the order a client opens one.
const SESSION: [&str; 10] = [
    r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05"}}"#,
    r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
    r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#,
    r#"{"jsonrpc":"2.0","id":"3","method":"tools/call","params":{"name":"route","arguments":{"task":"why is throttling applied at the edge"}}}"#,
    r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"resolve_identifier","arguments":{"id":"DR-FIX-0031"}}}"#,
    r#"{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"related","arguments":{"target":"query/specs/api-design.md"}}}"#,
    r#"{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"check","arguments":{"format":"markdown"}}}"#,
    // A format nobody declared. The refusal names the closed set, which is the
    // list `check --format` names when a terminal asks the same wrong question.
    r#"{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"check","arguments":{"format":"yaml"}}}"#,
    r#"{"jsonrpc":"2.0","id":8,"method":"tools/call","params":{"name":"new","arguments":{"kind":"decision"}}}"#,
    r#"{"jsonrpc":"2.0","id":9,"method":"resources/read","params":{"uri":"file:///etc/passwd"}}"#,
];

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

struct Built {
    census: Census,
    graph: Graph,
    shape: Shape,
    taxonomy: Taxonomy,
    relations: Declarations,
    register: Register,
    config: Config,
}

fn fixture_tree() -> Built {
    let corpus = Corpus::new(fixtures_dir(), "query");
    let source = std::fs::read_to_string(fixtures_dir().join("query.taxonomy.yml"))
        .expect("the fixture taxonomy");
    let root = headwater_yaml::load(&source)
        .expect("it loads")
        .value
        .as_map()
        .expect("a mapping")
        .clone();
    let taxonomy = Taxonomy::read(&root).expect("the taxonomy reads");
    let relations = Declarations::read(&root).expect("the declarations read");
    let shape = Shape::read(&root).expect("the shape reads");
    let register = Register::read(&root).expect("the register reads");
    let census = census::take(&corpus, &taxonomy);
    let graph = Graph::build(
        &census,
        &relations,
        &Resolvers::over(&corpus),
        &corpus,
        &Config::default(),
    );
    Built {
        census,
        graph,
        shape,
        taxonomy,
        relations,
        register,
        config: Config::default(),
    }
}

impl Built {
    fn surface(&self) -> Surface<'_> {
        Surface::over(
            &self.census,
            &self.graph,
            &self.shape,
            &self.taxonomy,
            &self.relations,
            &self.config,
        )
    }

    fn declared(&self) -> Declared<'_> {
        Declared {
            lock: LOCK,
            taxonomy: &self.taxonomy,
            shape: &self.shape,
            relations: &self.relations,
            config: &self.config,
            register: &self.register,
            adoption: None,
            source: "query.taxonomy.yml",
        }
    }

    /// A server at a stated date, which is the only way one is built here: the
    /// clock is fixed before a message arrives, and a test that read the system
    /// clock would record a different report every day.
    ///
    /// The write class is off, which is the default shape and the one the
    /// recorded session runs over.
    fn server(&self, at: &str) -> Server<'_> {
        Server {
            surface: self.surface(),
            census: &self.census,
            graph: &self.graph,
            declared: self.declared(),
            package: "query-fixture",
            version: "0.0.0",
            now: Context::at(Date::parse(at).expect("a date")),
            writing: None,
        }
    }

    /// The same server with the working-tree write class registered, over the
    /// two verbs a caller supplied.
    fn writing<'a>(&'a self, at: &str, writing: Writing<'a>) -> Server<'a> {
        Server {
            writing: Some(writing),
            ..self.server(at)
        }
    }
}

/// The two verbs of the write class, as a caller supplies them.
struct Verbs {
    #[allow(clippy::type_complexity)]
    scaffold: Box<dyn Fn(&str, &str, &[(String, String)]) -> Result<Written, String>>,
    fix: Box<dyn Fn(Format) -> Result<Written, String>>,
}

impl Verbs {
    fn writing(&self) -> Writing<'_> {
        Writing {
            scaffold: &*self.scaffold,
            fix: &*self.fix,
        }
    }
}

/// A `Writing` whose two verbs state what they did rather than doing it.
///
/// The server owns one rule about a write and it is the seal, so what the
/// suite needs is control over the one fact the seal reads.
fn verbs(landed: bool) -> Verbs {
    let scaffold = move |kind: &str, title: &str, relates: &[(String, String)]| {
        Ok(Written {
            account: String::new(),
            artifact: format!(
                "wrote a {kind} called {title}, with {} edge(s)\n",
                relates.len()
            ),
            landed,
            ok: true,
        })
    };
    let fix = move |format: Format| {
        Ok(Written {
            account: match landed {
                true => "headwater: fixed one file (1 patch)\n".to_string(),
                false => "headwater: no finding of this run carries a patch\n".to_string(),
            },
            artifact: format!("the report, in {}\n", format.name()),
            landed,
            ok: true,
        })
    };
    Verbs {
        scaffold: Box::new(scaffold),
        fix: Box::new(fix),
    }
}

fn session(server: &Server<'_>) -> String {
    let mut held = Session::default();
    let mut out = String::new();
    for request in SESSION {
        out.push_str("→ ");
        out.push_str(request);
        out.push('\n');
        match mcp::respond(server, &mut held, request) {
            Some(response) => {
                out.push_str("← ");
                out.push_str(&response);
                out.push('\n');
            }
            None => out.push_str("← (a notification takes no response)\n"),
        }
    }
    out
}

/// One request, answered by a session nobody else is holding.
fn once(server: &Server<'_>, request: &str) -> String {
    mcp::respond(server, &mut Session::default(), request).expect("a request takes a response")
}

/// The request that calls one tool with the arguments given.
fn calling(tool: &str, arguments: &str) -> String {
    format!(
        r#"{{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{{"name":"{tool}","arguments":{arguments}}}}}"#
    )
}

/// Every content block a call returned, out of the JSON-RPC envelope.
///
/// The envelope is JSON, and the text inside it is a report with newlines in
/// it, so this reads the payload back through the loader rather than by cutting
/// the string.
fn content(response: &str) -> Vec<String> {
    headwater_yaml::load(response)
        .expect("a response is JSON")
        .value
        .as_map()
        .expect("an object")
        .get("result")
        .expect("a result rather than an error")
        .value
        .as_map()
        .expect("an object")
        .get("content")
        .expect("content")
        .value
        .as_seq()
        .expect("a list")
        .iter()
        .map(|block| {
            block
                .value
                .as_map()
                .expect("an object")
                .get("text")
                .expect("text")
                .value
                .as_scalar()
                .expect("a scalar")
                .text
                .clone()
        })
        .collect()
}

/// The one block a read answers with.
fn tool_text(response: &str) -> String {
    let blocks = content(response);
    assert_eq!(blocks.len(), 1, "a read answers with one block");
    blocks.into_iter().next().expect("one block")
}

fn call_check(server: &Server<'_>, format: &str) -> String {
    tool_text(&once(
        server,
        &calling("check", &format!(r#"{{"format":"{format}"}}"#)),
    ))
}

#[test]
fn the_session_runs_to_the_recorded_transcript() {
    let built = fixture_tree();
    let actual = session(&built.server(RECORDED_AT));
    let recorded = fixtures_dir().join("query.mcp");
    if std::env::var_os("HEADWATER_BLESS").is_some() {
        std::fs::write(&recorded, &actual).expect("cannot write the expectation");
        return;
    }
    let expected = std::fs::read_to_string(&recorded)
        .unwrap_or_else(|e| panic!("{}: {e}. Run with HEADWATER_BLESS=1", recorded.display()));
    assert_eq!(expected, actual);
}

/// A server with no switch registers the query class and nothing else.
///
/// This is the whole of the property the write class replaced, held for the
/// default shape. Spec 5 fixes the mechanism: "Where a class of tool is off,
/// the server does not register it, so no handler exists to call." So this
/// asserts the closed list, and then asserts that each write tool spec 5 names
/// reaches an error that says the server registers no tool that writes.
#[test]
fn no_tool_outside_the_query_class_is_registered_without_the_switch() {
    let names: Vec<&str> = QUERY_CLASS.iter().map(|tool| tool.name).collect();
    assert_eq!(
        names,
        vec![
            "route",
            "explain",
            "related",
            "resolve_identifier",
            "governing_docs_for_path",
            "check"
        ]
    );

    let built = fixture_tree();
    let server = built.server(RECORDED_AT);
    assert!(mcp::registered(&server).iter().all(|tool| !tool.writes));
    // The working-tree write class of spec 5, and the landed write class it
    // refuses outright. `fix` is the one that has to stay refused, because a
    // fix tool is the check tool and a write.
    for tool in ["new", "fix", "commit", "push", "merge"] {
        let response = once(&server, &calling(tool, "{}"));
        assert!(
            response.contains("is not a tool this server registers"),
            "{tool}: {response}"
        );
        assert!(
            response.contains("it registers no tool that writes"),
            "{tool}: {response}"
        );
        // The refusal names the switch, because a client that asked for a write
        // tool is owed the reason rather than a bare no.
        assert!(
            response.contains("headwater mcp --write"),
            "{tool}: {response}"
        );
    }
}

/// The switch registers the working-tree write class and reaches no further.
///
/// Spec 5's third row is a refusal rather than a deferral, and no argument of
/// this engine turns it into anything. So a commit, a push and a merge are
/// refused by the server that can write, on a different sentence from the one
/// the server that cannot write gives.
#[test]
fn the_switch_registers_two_tools_and_no_landed_write() {
    let names: Vec<&str> = WRITE_CLASS.iter().map(|tool| tool.name).collect();
    assert_eq!(names, vec!["new", "fix"]);

    let built = fixture_tree();
    let supplied = verbs(false);
    let server = built.writing(RECORDED_AT, supplied.writing());
    let registered: Vec<&str> = mcp::registered(&server)
        .iter()
        .map(|tool| tool.name)
        .collect();
    assert_eq!(
        registered,
        vec![
            "route",
            "explain",
            "related",
            "resolve_identifier",
            "governing_docs_for_path",
            "check",
            "new",
            "fix"
        ]
    );
    for tool in ["commit", "push", "merge", "branch", "propose"] {
        let response = once(&server, &calling(tool, "{}"));
        assert!(
            response.contains("is not a tool this server registers"),
            "{tool}: {response}"
        );
        assert!(
            response.contains("acceptance is a human act"),
            "{tool}: {response}"
        );
    }
}

/// A tool that writes says so in the annotation, and a read says the opposite.
///
/// The annotation is not what carries the property, and it is still a fact
/// about the tool. A server that registered a write tool and annotated it
/// `readOnlyHint: true` would be telling a client something false about itself.
#[test]
fn every_tool_annotates_what_it_does() {
    let built = fixture_tree();
    let supplied = verbs(false);
    let server = built.writing(RECORDED_AT, supplied.writing());
    let listed = once(&server, r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#);
    for tool in &QUERY_CLASS {
        assert!(!tool.writes, "{}", tool.name);
    }
    for tool in &WRITE_CLASS {
        assert!(tool.writes, "{}", tool.name);
    }
    // Six reads and two writes, and the two annotations are the two values.
    assert_eq!(listed.matches(r#""readOnlyHint":true"#).count(), 6);
    assert_eq!(listed.matches(r#""readOnlyHint":false"#).count(), 2);
    assert_eq!(listed.matches(r#""destructiveHint":true"#).count(), 2);
    assert_eq!(listed.matches(r#""destructiveHint":false"#).count(), 6);
}

/// A read answers the same bytes twice, which is what an agent depends on.
#[test]
fn one_request_answers_the_same_bytes_twice() {
    let built = fixture_tree();
    let server = built.server(RECORDED_AT);
    let mut held = Session::default();
    for request in SESSION {
        assert_eq!(
            mcp::respond(&server, &mut held, request),
            mcp::respond(&server, &mut held, request),
            "{request}"
        );
    }
}

/// A call that moved a byte ends the session, and a call that moved none does
/// not.
///
/// This is the answer to what a session looks like after a write, and it is a
/// code path rather than a sentence in a specification. The server walked the
/// corpus once, so an answer after a write would be about a tree that is gone —
/// which HW-OBL-0028 measured at two findings over this repository. The seal
/// follows the bytes: a `fix` that found no patch, and a scaffolder that
/// refused, leave the tree as the walk described it.
#[test]
fn a_call_that_moved_a_byte_ends_the_session() {
    let built = fixture_tree();

    // Nothing landed: every later call is answered, including a second write.
    let supplied = verbs(false);
    let quiet = built.writing(RECORDED_AT, supplied.writing());
    let mut held = Session::default();
    for request in [
        calling("fix", r#"{"format":"text"}"#),
        calling("fix", r#"{"format":"text"}"#),
        calling("route", r#"{"task":"throttling"}"#),
    ] {
        let response = mcp::respond(&quiet, &mut held, &request).expect("a response");
        assert!(!response.contains(r#""error""#), "{request}: {response}");
    }
    assert_eq!(held.spent(), None);

    // A byte landed: the writing call is answered in full, and every call after
    // it is refused by name with the reason.
    let supplied = verbs(true);
    let writing = built.writing(RECORDED_AT, supplied.writing());
    let mut held = Session::default();
    let first = mcp::respond(&writing, &mut held, &calling("fix", r#"{"format":"text"}"#))
        .expect("a response");
    assert!(!first.contains(r#""error""#), "{first}");
    assert!(first.contains("the report, in text"), "{first}");
    assert_eq!(held.spent(), Some("fix"));

    for request in [
        calling("fix", r#"{"format":"text"}"#),
        calling("new", r#"{"kind":"decision","title":"A decision"}"#),
        calling("route", r#"{"task":"throttling"}"#),
        calling("check", r#"{"format":"text"}"#),
    ] {
        let response = mcp::respond(&writing, &mut held, &request).expect("a response");
        assert!(response.contains("-32000"), "{request}: {response}");
        assert!(
            response.contains("`fix` then moved a byte of that tree"),
            "{request}: {response}"
        );
        assert!(
            response.contains("Start another server"),
            "{request}: {response}"
        );
    }
    // The table is a fact about the registration rather than about the tree, so
    // a spent server still answers what it has.
    let listed = mcp::respond(
        &writing,
        &mut held,
        r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#,
    )
    .expect("a response");
    assert!(!listed.contains(r#""error""#), "{listed}");
}

/// A write tool answers with two content blocks, and the second is the
/// artifact.
///
/// Spec 5 fixes two streams for every verb of this engine. A protocol call has
/// one result, so the account and the artifact arrive as two blocks and keep
/// their separation. A client that read only the last block reads what a
/// terminal reads on standard output.
#[test]
fn a_write_answers_with_the_account_and_then_the_artifact() {
    let built = fixture_tree();
    let supplied = verbs(true);
    let server = built.writing(RECORDED_AT, supplied.writing());
    let response = once(&server, &calling("fix", r#"{"format":"markdown"}"#));
    let blocks = content(&response);
    assert_eq!(
        blocks,
        vec![
            "headwater: fixed one file (1 patch)\n".to_string(),
            "the report, in markdown\n".to_string()
        ]
    );

    // A verb with nothing to say on the account leaves one block, which is what
    // every read answers with.
    let supplied = verbs(false);
    let quiet = built.writing(RECORDED_AT, supplied.writing());
    let response = once(
        &quiet,
        &calling(
            "new",
            r#"{"kind":"decision","title":"A decision","relates":["supersedes=DR-FIX-0031"]}"#,
        ),
    );
    assert_eq!(
        content(&response),
        vec!["wrote a decision called A decision, with 1 edge(s)\n".to_string()]
    );
}

/// `new` takes three arguments, and each one is refused in its own words.
///
/// One string was enough for every read and it is not enough here. A required
/// argument that is absent is named, and a repeatable one that arrived as a
/// bare string is refused with the shape stated rather than read as a single
/// item.
#[test]
fn the_write_tools_take_the_arguments_their_verbs_take() {
    let built = fixture_tree();
    let supplied = verbs(false);
    let server = built.writing(RECORDED_AT, supplied.writing());

    let response = once(&server, &calling("new", r#"{"title":"A decision"}"#));
    assert!(response.contains("`new` takes `kind`"), "{response}");
    let response = once(&server, &calling("new", r#"{"kind":"decision"}"#));
    assert!(response.contains("`new` takes `title`"), "{response}");
    // `relates` is optional, so a call without one runs.
    let response = once(
        &server,
        &calling("new", r#"{"kind":"decision","title":"A decision"}"#),
    );
    assert!(!response.contains(r#""error""#), "{response}");
    assert!(response.contains("with 0 edge(s)"), "{response}");

    let response = once(
        &server,
        &calling(
            "new",
            r#"{"kind":"decision","title":"A decision","relates":"supersedes=DR-FIX-0031"}"#,
        ),
    );
    assert!(response.contains("as a list of strings"), "{response}");
    let response = once(
        &server,
        &calling(
            "new",
            r#"{"kind":"decision","title":"A decision","relates":["supersedes"]}"#,
        ),
    );
    assert!(
        response.contains("is not `<relation>=<identifier>`"),
        "{response}"
    );

    // `fix` takes a format and nothing else: no clock, no root, no path.
    assert_eq!(
        WRITE_CLASS
            .iter()
            .find(|tool| tool.name == "fix")
            .expect("the fix tool")
            .arguments
            .iter()
            .map(|argument| argument.name)
            .collect::<Vec<&str>>(),
        vec!["format"]
    );
    let response = once(&server, &calling("fix", r#"{"format":"yaml"}"#));
    assert!(
        response.contains("`fix` takes a format, one of text, json, sarif, markdown"),
        "{response}"
    );
}

/// The tool answers what the CLI answers, in all four formats.
///
/// This is the whole of the "second set of defaults" question, asserted rather
/// than argued. The run on the right is built here from the clock, the
/// declarations, the census and the graph, and rendered through the dispatcher
/// `headwater check` renders through. A tool that reached for a clock of its
/// own, opened a cache, or walked the corpus again would answer something else
/// in at least one format.
#[test]
fn the_check_tool_answers_the_bytes_the_cli_answers() {
    let built = fixture_tree();
    let server = built.server(RECORDED_AT);

    let mut cache = Cache::disabled();
    let run = headwater_check::run(
        &built.census,
        &built.graph,
        &built.declared(),
        &Context::at(Date::parse(RECORDED_AT).expect("a date")),
        &mut cache,
    );
    let subject = Subject {
        package: "query-fixture",
        version: "0.0.0",
        lock: LOCK,
        now: RECORDED_AT,
    };
    for format in Format::ALL {
        let expected =
            headwater_adapter::render(&run, &built.census, &built.graph, &subject, format);
        assert_eq!(
            expected,
            call_check(&server, format.name()),
            "the {} format",
            format.name()
        );
    }
}

/// Every format states the date the server was started at, and no other.
///
/// The clock is fixed once, for the life of the server, and reported in every
/// result. Two servers over one corpus differ in exactly that value, and a tool
/// that read the system clock per call would answer today's date from both.
#[test]
fn the_answer_states_the_date_the_server_fixed() {
    let built = fixture_tree();
    let early = built.server(RECORDED_AT);
    let late = built.server(LATER);
    for format in Format::ALL {
        let from_early = call_check(&early, format.name());
        let from_late = call_check(&late, format.name());
        assert!(
            from_early.contains(RECORDED_AT) && !from_early.contains(LATER),
            "the {} format states no date, or the wrong one",
            format.name()
        );
        assert!(
            from_late.contains(LATER) && !from_late.contains(RECORDED_AT),
            "the {} format states no date, or the wrong one",
            format.name()
        );
        assert_ne!(from_early, from_late, "the {} format", format.name());
    }
}

/// A verdict moves with the clock the server was started at, and not with today.
///
/// The date is not decoration on the report. `relation.participation.overdue`
/// is windowed, so a specification inside its window on one date is past it on
/// a later one, and the two servers below differ in the finding rather than
/// only in the line that states the date. A tool that ignored its server's
/// clock and read the host's would answer the same thing from both.
#[test]
fn a_verdict_moves_with_the_clock_the_server_holds() {
    let built = fixture_tree();
    // One row of the job summary per finding, which is the coarsest grain that
    // counts findings rather than mentions of a rule: the text report names
    // every rule it ran in a list, whether or not the rule found anything.
    let overdue = |at: &str| -> usize {
        call_check(&built.server(at), "markdown")
            .lines()
            .filter(|line| line.contains("`relation.participation.overdue`"))
            .count()
    };
    assert_eq!(
        overdue(RECORDED_AT),
        2,
        "two specifications are past their window at {RECORDED_AT}"
    );
    assert_eq!(
        overdue(LATER),
        3,
        "a third one is past it at {LATER}, and nothing else moved"
    );
}

/// A format nobody declared is refused, and the refusal names the closed set.
///
/// The tool takes no default. A caller that named nothing, or named something
/// outside spec 6's four, gets the list rather than a report in a vocabulary
/// this engine picked for it.
#[test]
fn the_check_tool_takes_a_format_and_defaults_to_none() {
    let built = fixture_tree();
    let server = built.server(RECORDED_AT);
    for request in [
        r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"check","arguments":{}}}"#,
        r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"check","arguments":{"format":"yaml"}}}"#,
        r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"check","arguments":{"format":""}}}"#,
    ] {
        let response = once(&server, request);
        assert!(response.contains("error"), "{request}: {response}");
        assert!(
            response.contains("text, json, sarif, markdown") || response.contains("takes `format`"),
            "{request}: {response}"
        );
    }
}

/// An argument each tool accepts, so that a call reaches the read behind it.
///
/// A `match` rather than one string for all six: `check` refuses a path where a
/// format belongs, and a call that a tool refused proves nothing about what the
/// tool does when it runs. A tool this list does not name fails the suite,
/// which is what makes the next tool's author supply one.
fn an_argument_for(tool: &str) -> &'static str {
    match tool {
        "route" => "throttling",
        "explain" | "related" | "governing_docs_for_path" => "query/specs/api-design.md",
        "resolve_identifier" => "DR-FIX-0031",
        "check" => "text",
        other => panic!("{other} is registered and this suite has no argument for it"),
    }
}

/// The tree the server ran over is the tree it found.
///
/// The read-only property is a claim about the filesystem, so it is asserted
/// against the filesystem: every path under the fixture tree, and its bytes,
/// before and after a session that calls every registered tool.
///
/// The `check` tool is why this snapshot covers a directory rather than the
/// documents. A run with a cache writes its store into the checkout, and the
/// store would land here as a new path under the fixture tree. So this test is
/// what holds the module's claim that the tool uses no cache, and it is the
/// only thing that holds it.
#[test]
fn a_session_writes_nothing_to_the_corpus_it_reads() {
    fn snapshot(at: &Path) -> Vec<(PathBuf, Vec<u8>)> {
        let mut found = Vec::new();
        let mut stack = vec![at.to_path_buf()];
        while let Some(path) = stack.pop() {
            for entry in std::fs::read_dir(&path).expect("the fixture tree") {
                let entry = entry.expect("an entry").path();
                match entry.is_dir() {
                    true => stack.push(entry),
                    false => {
                        let bytes = std::fs::read(&entry).expect("a fixture");
                        found.push((entry, bytes));
                    }
                }
            }
        }
        found.sort();
        found
    }

    let before = snapshot(&fixtures_dir());
    let built = fixture_tree();
    let server = built.server(RECORDED_AT);
    for tool in &QUERY_CLASS {
        let request = calling(
            tool.name,
            &format!(
                r#"{{"{}":"{}"}}"#,
                tool.only().name,
                an_argument_for(tool.name)
            ),
        );
        let response = once(&server, &request);
        // A refused call reads nothing and therefore writes nothing, which
        // would make this test pass for the wrong reason.
        assert!(
            !response.contains(r#""error""#),
            "{}: {response}",
            tool.name
        );
    }
    assert_eq!(before, snapshot(&fixtures_dir()));
}
