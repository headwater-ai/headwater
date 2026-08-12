// SPDX-License-Identifier: Apache-2.0
//! The loader's fixture corpus.
//!
//! Every case is a pair of files under `fixtures/`: the source, and what the
//! loader is expected to make of it. `accept/` records the tree with a span on
//! every node, and `reject/` records the rejection text an author would read.
//!
//! The corpus is data rather than assertions in Rust for one reason that Q2
//! names: M2 replaces the resolver, and it must do so without invalidating a
//! single M1 fixture. A rule that lives in a `.tree` file survives that
//! replacement, and a rule expressed as an `assert_eq!` inside the loader's own
//! unit tests does not.
//!
//! To re-record after a deliberate change:
//!
//!     HEADWATER_BLESS=1 cargo test -p headwater-yaml --test fixtures
//!
//! Read the diff before committing it. A blessed fixture is the change.

use headwater_yaml::error::render;
use headwater_yaml::span::Span;
use headwater_yaml::{load, Value};
use std::path::{Path, PathBuf};

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn blessing() -> bool {
    std::env::var_os("HEADWATER_BLESS").is_some()
}

/// Every `.yml` under `dir`, in name order, so a run reports the same way twice.
fn cases(dir: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .map(|entry| entry.expect("a directory entry").path())
        .filter(|path| path.extension().is_some_and(|e| e == "yml"))
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
fn accepted_sources_load_to_the_recorded_tree() {
    for path in cases(&fixtures_dir().join("accept")) {
        let source = std::fs::read_to_string(&path).expect("cannot read the fixture");
        match load(&source) {
            Ok(root) => {
                let mut out = String::new();
                render_value(&root.value, root.span, 0, &mut out);
                compare(&path, "tree", &out);
            }
            Err(errors) => panic!("{} was rejected:\n{}", path.display(), render(&errors)),
        }
    }
}

#[test]
fn rejected_sources_report_the_recorded_errors() {
    for path in cases(&fixtures_dir().join("reject")) {
        let source = std::fs::read_to_string(&path).expect("cannot read the fixture");
        match load(&source) {
            Ok(_) => panic!(
                "{} was accepted, and the fixture says it must not be",
                path.display()
            ),
            Err(errors) => compare(&path, "errors", &render(&errors)),
        }
    }
}

/// The two taxonomy sources this repository already has, loaded as they stand.
///
/// M1 exists to put the real corpus in front of the code while the design is
/// still cheap to change, and these two files are the whole of that corpus for
/// a loader. A constructed fixture cannot fail the way a file somebody wrote
/// for another purpose can.
#[test]
fn this_repositorys_own_sources_load() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root");
    for name in ["taxonomy.yml", "overlay.yml"] {
        let path = root.join(".headwater").join(name);
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        if let Err(errors) = load(&source) {
            panic!("{} was rejected:\n{}", path.display(), render(&errors));
        }
    }
}

/// A tree, one node per line, with the span that a finding would anchor to.
///
/// Positions read `line:col(byte)`. The byte offset is here because it is the
/// field with no visible symptom when it is wrong: a column error moves a
/// caret, and a byte error corrupts whatever slices the source.
fn render_value(value: &Value, span: Span, depth: usize, out: &mut String) {
    let pad = "  ".repeat(depth);
    match value {
        Value::Scalar(scalar) => {
            out.push_str(&format!(
                "{pad}scalar {} {:?} {}\n",
                scalar.style.name(),
                scalar.text,
                place(span)
            ));
        }
        Value::Seq(items) => {
            out.push_str(&format!("{pad}seq {}\n", place(span)));
            for item in items {
                render_value(&item.value, item.span, depth + 1, out);
            }
        }
        Value::Map(map) => {
            out.push_str(&format!("{pad}map {}\n", place(span)));
            for entry in map {
                out.push_str(&format!(
                    "{pad}  key {:?} {}\n",
                    entry.key.value,
                    place(entry.key.span)
                ));
                render_value(&entry.value.value, entry.value.span, depth + 2, out);
            }
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
