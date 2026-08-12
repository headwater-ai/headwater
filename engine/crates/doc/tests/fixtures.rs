// SPDX-License-Identifier: Apache-2.0
//! The parser's conformance corpus.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#the-correctness-roots) asks
//! for this by name: "a parser conformance corpus is part of the engine's own
//! test surface". The reason is the failure mode. A parser defect does not
//! crash a run, it moves a span or loses a block, and every check downstream
//! then reports a correct result about the wrong text.
//!
//! Every case is a pair of files under `fixtures/`: the document, and what the
//! parser is expected to make of it. `accept/` records the front-matter tree and
//! the body, span by span. `reject/` records the text an author would read.
//! Both are recorded files rather than assertions in Rust, so that a rule
//! survives the replacement of the code under it.
//!
//!     HEADWATER_BLESS=1 cargo test -p headwater-doc --test fixtures
//!
//! Read the diff before committing it. A blessed fixture is the change.

use headwater_doc::body::Body;
use headwater_doc::error::render;
use headwater_doc::{parse, Document, Mapping, Span, Value};
use std::path::{Path, PathBuf};

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root")
}

fn blessing() -> bool {
    std::env::var_os("HEADWATER_BLESS").is_some()
}

/// Every `.md` under `dir`, in name order, so a run reports the same way twice.
fn cases(dir: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .map(|entry| entry.expect("a directory entry").path())
        .filter(|path| path.extension().is_some_and(|e| e == "md"))
        .collect();
    found.sort();
    assert!(!found.is_empty(), "no fixtures in {}", dir.display());
    found
}

fn compare(source_path: &Path, expected_extension: &str, actual: &str) {
    let expected_path = source_path.with_extension(expected_extension);
    if blessing() {
        std::fs::write(&expected_path, actual).expect("cannot write the expectation");
        return;
    }
    let expected = std::fs::read_to_string(&expected_path).unwrap_or_else(|e| {
        panic!(
            "{}: {e}. Run with HEADWATER_BLESS=1 to record it.",
            expected_path.display()
        )
    });
    assert_eq!(
        expected,
        actual,
        "\n{} does not match {}",
        source_path.display(),
        expected_path.display()
    );
}

#[test]
fn accepted_documents_parse_to_the_recorded_shape() {
    for path in cases(&fixtures_dir().join("accept")) {
        let source = std::fs::read_to_string(&path).expect("cannot read the fixture");
        match parse(&source) {
            Ok(doc) => compare(&path, "parse", &render_document(&doc)),
            Err(errors) => panic!("{} was rejected:\n{}", path.display(), render(&errors)),
        }
    }
}

#[test]
fn rejected_documents_report_the_recorded_errors() {
    for path in cases(&fixtures_dir().join("reject")) {
        let source = std::fs::read_to_string(&path).expect("cannot read the fixture");
        match parse(&source) {
            Ok(_) => panic!(
                "{} was accepted, and the fixture says it must not be",
                path.display()
            ),
            Err(errors) => compare(&path, "errors", &render(&errors)),
        }
    }
}

/// Every document this repository has, parsed as it stands.
///
/// M1 exists to put the real corpus in front of the code while the design is
/// still cheap to change. A constructed fixture cannot fail the way a file
/// somebody wrote for another purpose can, and this corpus was written before
/// there was a parser to please.
///
/// A file the parser refuses is not a test failure here, because the parser is
/// not the component that decides what a corpus should contain. An untyped file
/// is an ordinary finding of the census ([#44](https://github.com/headwater-ai/headwater/issues/44)),
/// and turning it into a red test would move that judgment into the wrong
/// layer. So the refusals are *recorded*, in `fixtures/corpus.exceptions`.
///
/// Only the exceptions are recorded, and never the files that parse. Adding a
/// well-formed document to this repository therefore changes nothing here,
/// while adding one the engine cannot read changes a committed file and asks a
/// human to look. That is the property a full census would lose: it would churn
/// on every commit and nobody would read the diff.
#[test]
fn the_corpus_parses_except_where_this_records_otherwise() {
    let root = repository_root();
    let docs = root.join("docs");
    let mut exceptions = String::new();
    let mut parsed = 0usize;
    let mut span_failures: Vec<String> = Vec::new();

    for path in markdown_under(&docs) {
        let source = std::fs::read_to_string(&path).expect("cannot read a document");
        let shown = path
            .strip_prefix(&root)
            .unwrap_or(&path)
            .display()
            .to_string();
        match parse(&source) {
            Ok(doc) => {
                parsed += 1;
                // The property no single fixture proves and every consumer
                // assumes: a span is a byte range into the file it came from.
                // A character index passes every ASCII test and cuts an em dash
                // in half, and this corpus is full of them.
                for block in &doc.body.blocks {
                    for run in &block.runs {
                        if run.span.slice(&source).is_none() {
                            span_failures.push(format!("{shown}: a run span is not in the source"));
                        }
                    }
                }
            }
            Err(errors) => {
                exceptions.push_str(&format!("{shown}\n"));
                for error in &errors {
                    exceptions.push_str(&format!("  {error}\n"));
                }
            }
        }
    }

    assert!(parsed > 0, "no documents found under {}", docs.display());
    // A span defect *is* a failure. Nothing downstream can recover from it, and
    // no corpus fact makes it acceptable.
    assert!(span_failures.is_empty(), "{}", span_failures.join("\n"));

    let recorded = fixtures_dir().join("corpus.exceptions");
    if blessing() {
        std::fs::write(&recorded, &exceptions).expect("cannot write the expectation");
        return;
    }
    let expected = std::fs::read_to_string(&recorded).unwrap_or_else(|e| {
        panic!(
            "{}: {e}. Run with HEADWATER_BLESS=1 to record it.",
            recorded.display()
        )
    });
    assert_eq!(
        expected,
        exceptions,
        "\nthe corpus no longer matches {}",
        recorded.display()
    );
}

fn markdown_under(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        for entry in std::fs::read_dir(&current).expect("cannot read a directory") {
            let path = entry.expect("a directory entry").path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "md") {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

/// The recorded shape: the front matter as a tree, then the body.
///
/// Positions read `line:col(byte)`. The byte offset is here because it is the
/// field with no visible symptom when it is wrong: a column error moves a
/// caret, and a byte error corrupts whatever slices the source.
fn render_document(doc: &Document) -> String {
    let mut out = String::new();
    out.push_str(&format!("front matter {}\n", place(doc.block)));
    render_map(&doc.facets, 1, &mut out);
    render_body(&doc.body, &mut out);
    out
}

fn render_body(body: &Body, out: &mut String) {
    out.push_str("body\n");
    for block in &body.blocks {
        let level = match block.level() {
            Some(level) => format!(" h{level}"),
            None => String::new(),
        };
        let quote = match block.quote_depth {
            0 => String::new(),
            depth => format!(" quoted×{depth}"),
        };
        out.push_str(&format!(
            "  {}{level}{quote} {}\n",
            block.kind.name(),
            place(block.span)
        ));
        for run in &block.runs {
            out.push_str(&format!(
                "    {} {:?} {}\n",
                run.ownership.name(),
                run.text,
                place(run.span)
            ));
        }
    }
    if !body.links.is_empty() {
        out.push_str("links\n");
        for link in &body.links {
            let image = if link.image { " image" } else { "" };
            let quoted = if link.quoted { " quoted" } else { "" };
            out.push_str(&format!(
                "  {:?}{image}{quoted} -> {:?} {:?} {}\n",
                link.form,
                link.destination,
                link.text,
                place(link.span)
            ));
        }
    }
}

fn render_map(map: &Mapping, depth: usize, out: &mut String) {
    let pad = "  ".repeat(depth);
    for entry in map {
        out.push_str(&format!(
            "{pad}key {:?} {}\n",
            entry.key.value,
            place(entry.key.span)
        ));
        render_value(&entry.value.value, entry.value.span, depth + 1, out);
    }
}

fn render_value(value: &Value, span: Span, depth: usize, out: &mut String) {
    let pad = "  ".repeat(depth);
    match value {
        Value::Scalar(scalar) => out.push_str(&format!(
            "{pad}scalar {} {:?} {}\n",
            scalar.style.name(),
            scalar.text,
            place(span)
        )),
        Value::Seq(items) => {
            out.push_str(&format!("{pad}seq {}\n", place(span)));
            for item in items {
                render_value(&item.value, item.span, depth + 1, out);
            }
        }
        Value::Map(map) => {
            out.push_str(&format!("{pad}map {}\n", place(span)));
            render_map(map, depth + 1, out);
        }
    }
}

fn place(span: Span) -> String {
    format!(
        "{}:{}({})-{}:{}({})",
        span.start.line,
        span.start.col,
        span.start.offset,
        span.end.line,
        span.end.col,
        span.end.offset
    )
}
