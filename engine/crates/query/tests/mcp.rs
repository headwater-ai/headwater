// SPDX-License-Identifier: Apache-2.0
//! The MCP read surface, over the same fixture tree the reads are recorded on.
//!
//! The recorded file is `fixtures/query.mcp`: a session, request by response.
//! Two properties are asserted rather than recorded, because a recording cannot
//! carry either one — that no registered tool writes, and that a call to a tool
//! outside the query class reaches no handler at all.
//!
//!     HEADWATER_BLESS=1 cargo test -p headwater-query --test mcp

use headwater_census::census::{self, Census};
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::Shape;
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use headwater_query::mcp::{self, TOOLS};
use headwater_query::Surface;
use std::path::{Path, PathBuf};

/// One session, in the order a client opens one.
const SESSION: [&str; 8] = [
    r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05"}}"#,
    r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
    r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#,
    r#"{"jsonrpc":"2.0","id":"3","method":"tools/call","params":{"name":"route","arguments":{"task":"why is throttling applied at the edge"}}}"#,
    r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"resolve_identifier","arguments":{"id":"DR-FIX-0031"}}}"#,
    r#"{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"related","arguments":{"target":"query/specs/api-design.md"}}}"#,
    r#"{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"new","arguments":{"kind":"decision"}}}"#,
    r#"{"jsonrpc":"2.0","id":7,"method":"resources/read","params":{"uri":"file:///etc/passwd"}}"#,
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
        )
    }
}

fn session(surface: &Surface<'_>) -> String {
    let mut out = String::new();
    for request in SESSION {
        out.push_str("→ ");
        out.push_str(request);
        out.push('\n');
        match mcp::respond(surface, request) {
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

#[test]
fn the_session_runs_to_the_recorded_transcript() {
    let built = fixture_tree();
    let actual = session(&built.surface());
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
            "governing_docs_for_path"
        ]
    );

    let built = fixture_tree();
    let surface = built.surface();
    // The working-tree write class of spec 5, and the landed write class it
    // refuses outright.
    for tool in ["new", "fix", "commit", "push", "merge"] {
        let request = format!(
            r#"{{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{{"name":"{tool}","arguments":{{}}}}}}"#
        );
        let response = mcp::respond(&surface, &request).expect("a request takes a response");
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
    let surface = built.surface();
    for request in SESSION {
        assert_eq!(
            mcp::respond(&surface, request),
            mcp::respond(&surface, request),
            "{request}"
        );
    }
}

/// The tree the server ran over is the tree it found.
///
/// The read-only property is a claim about the filesystem, so it is asserted
/// against the filesystem: every path under the fixture tree, and its bytes,
/// before and after a session that calls every registered tool.
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
    let surface = built.surface();
    for tool in TOOLS.iter() {
        let request = format!(
            r#"{{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{{"name":"{}","arguments":{{"{}":"query/specs/api-design.md"}}}}}}"#,
            tool.name, tool.argument
        );
        mcp::respond(&surface, &request).expect("a response");
    }
    assert_eq!(before, snapshot(&fixtures_dir()));
}
