// SPDX-License-Identifier: Apache-2.0
//! The MCP surface, over the same fixture tree the reads are recorded on.
//!
//! The recorded file is `fixtures/query.mcp`: a session, request by response.
//! Three properties are asserted rather than recorded, because a recording
//! cannot carry any of them — that no registered tool writes, that a call to a
//! tool outside the query class reaches no handler at all, and that the `check`
//! tool answers the bytes the CLI answers over the same corpus at the same
//! date.
//!
//!     HEADWATER_BLESS=1 cargo test -p headwater-query --test mcp

use headwater_adapter::{Format, Subject};
use headwater_census::census::{self, Census};
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::register::Register;
use headwater_check::{Cache, Context, Date, Declared, Shape};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use headwater_query::mcp::{self, Server, TOOLS};
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
    fn server(&self, at: &str) -> Server<'_> {
        Server {
            surface: self.surface(),
            census: &self.census,
            graph: &self.graph,
            declared: self.declared(),
            package: "query-fixture",
            version: "0.0.0",
            now: Context::at(Date::parse(at).expect("a date")),
        }
    }
}

fn session(server: &Server<'_>) -> String {
    let mut out = String::new();
    for request in SESSION {
        out.push_str("→ ");
        out.push_str(request);
        out.push('\n');
        match mcp::respond(server, request) {
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

/// The text a `check` call returned, out of the JSON-RPC envelope.
///
/// The envelope is JSON, and the text inside it is a report with newlines in
/// it, so this reads the payload back through the loader rather than by cutting
/// the string.
fn tool_text(response: &str) -> String {
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
        .first()
        .expect("one block")
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
}

fn call_check(server: &Server<'_>, format: &str) -> String {
    let request = format!(
        r#"{{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{{"name":"check","arguments":{{"format":"{format}"}}}}}}"#
    );
    tool_text(&mcp::respond(server, &request).expect("a request takes a response"))
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

/// The server registers the query class and nothing else.
///
/// Spec 5 fixes the mechanism: "Where a class of tool is off, the server does
/// not register it, so no handler exists to call." So this asserts the closed
/// list, and then asserts that each write tool spec 5 names reaches an error
/// that says the server registers no tool that writes.
#[test]
fn no_tool_outside_the_query_class_is_registered() {
    let names: Vec<&str> = TOOLS.iter().map(|tool| tool.name).collect();
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
    // The working-tree write class of spec 5, and the landed write class it
    // refuses outright. `fix` is the one that has to stay refused now that
    // `check` is registered, because a fix tool is this check tool and a write.
    for tool in ["new", "fix", "commit", "push", "merge"] {
        let request = format!(
            r#"{{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{{"name":"{tool}","arguments":{{}}}}}}"#
        );
        let response = mcp::respond(&server, &request).expect("a request takes a response");
        assert!(
            response.contains("is not a tool this server registers"),
            "{tool}: {response}"
        );
        assert!(
            response.contains("it registers no tool that writes"),
            "{tool}: {response}"
        );
    }
}

/// A read answers the same bytes twice, which is what an agent depends on.
#[test]
fn one_request_answers_the_same_bytes_twice() {
    let built = fixture_tree();
    let server = built.server(RECORDED_AT);
    for request in SESSION {
        assert_eq!(
            mcp::respond(&server, request),
            mcp::respond(&server, request),
            "{request}"
        );
    }
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
        let response = mcp::respond(&server, request).expect("a request takes a response");
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
    for tool in TOOLS.iter() {
        let request = format!(
            r#"{{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{{"name":"{}","arguments":{{"{}":"{}"}}}}}}"#,
            tool.name,
            tool.argument,
            an_argument_for(tool.name)
        );
        let response = mcp::respond(&server, &request).expect("a response");
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
