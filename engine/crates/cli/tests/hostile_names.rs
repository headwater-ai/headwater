// SPDX-License-Identifier: Apache-2.0
//! No control character that a document supplies reaches the text that
//! `explain`, `route`, `show` and `generate` print (#2069).
//!
//! # The defect this target exists for
//!
//! [#1970](https://github.com/headwater-ai/headwater/issues/1970) made `check`
//! write a control character from a file name or a document value as visible
//! backslash text, and write the first colon of a value that opens with `::`
//! as `\x3a`, so a hostile name cannot forge a GitHub Actions workflow command
//! or drive a terminal. [#2069](https://github.com/headwater-ai/headwater/issues/2069)
//! found four more verbs that printed the same names raw: `explain` and
//! `route` on standard output, `show` in its refusals on standard error, and
//! `generate` in its report.
//!
//! # What each case holds
//!
//! Each case runs one verb over a corpus whose file name holds a line feed
//! that opens a workflow command and an escape sequence, and whose second
//! document carries an escape sequence in its summary and title. Neither
//! stream may hold an escape byte, and no line may open with `::`. The JSON
//! forms stay exact: the raw name comes back when the document is parsed.
//! `show` writes the document's bytes on standard output unchanged, because
//! `docs/interfaces/headwater-show.md` promises that a redirect gives back the
//! file. Unix only, because no Windows file name can hold a line feed.
#![cfg(unix)]

mod common;

/// The hostile file name, relative to the root.
const HOSTILE: &str = "docs/decisions/0003-x\n::error ::forged-name\u{1b}[31m.md";

/// A path with no document, whose name holds an escape sequence.
const MISSING: &str = "docs/decisions/nothing\u{1b}[31m.md";

fn body(id: &str, summary: &str, extra: &str) -> String {
    format!(
        "---\n{extra}id: {id}\ntitle: \"A hostile \\e[33mname\"\nstatus: current\n\
         status_since: 2026-08-01\nlast_verified: 2026-08-01\nsummary: \"{summary}\"\n\
         provenance:\n  warrant: accepted\n  agency: human\n  accepted_by: a.person\n  \
         evidence_basis: unevidenced\n---\n\n# A hostile name\n\n## Context\n\nA name.\n\n\
         ## Decision\n\nA name.\n\n## Consequences\n\nA name.\n"
    )
}

/// The corpus every case reads. `marker` puts the generated marker on the
/// hostile file, so `generate` lists it as written by no declaration. The
/// hostile file declares an edge to the second document, so `explain` of the
/// second prints a neighbour line whose far end is the hostile path, and
/// `explain` of the first prints the second's hostile summary as its cue.
fn corpus(label: &str, marker: bool) -> common::Root {
    common::Root::shaped(label, |at| {
        let marked = match marker {
            true => "\"headwater:generated\": \"stray_pages. Placed by hand.\"\n",
            false => "",
        };
        std::fs::write(
            at.join(HOSTILE),
            body(
                "HW-DR-0003",
                "A wombat document whose name holds a control character.",
                &format!("{marked}relations:\n  traces_to:\n    - HW-DR-0004\n"),
            ),
        )
        .expect("the hostile file name writes");
        std::fs::write(
            at.join("docs/decisions/0004-y.md"),
            body(
                "HW-DR-0004",
                "A summary that clears \\e[2J the terminal.",
                "",
            ),
        )
        .expect("the hostile summary writes");
    })
}

/// Neither stream of `run` holds an escape byte or a line that opens with
/// `::`, after leading white space.
fn clean(what: &str, run: &common::Ran) {
    for (stream, text) in [("stdout", &run.out), ("stderr", &run.err)] {
        assert!(
            !text.contains('\u{1b}'),
            "`{what}` writes an escape byte on {stream}: {run:?}"
        );
        let forged: Vec<&str> = text
            .lines()
            .filter(|line| line.trim_start().starts_with("::"))
            .collect();
        assert!(
            forged.is_empty(),
            "`{what}` forges a line on {stream}: {forged:?}"
        );
    }
}

#[test]
fn explain_writes_a_hostile_path_and_value_as_visible_text() {
    let root = corpus("hostile-explain", false);
    let hostile = root.run(&["explain", "--no-color", "HW-DR-0003"]);
    assert_eq!(hostile.code, Some(0), "explain answers: {hostile:?}");
    assert!(
        hostile.out.contains("forged-name"),
        "explain names the hostile file: {hostile:?}"
    );
    clean("explain HW-DR-0003", &hostile);
    let summary = root.run(&["explain", "--no-color", "HW-DR-0004"]);
    assert_eq!(summary.code, Some(0), "explain answers: {summary:?}");
    assert!(
        summary.out.contains("clears"),
        "explain prints the hostile summary: {summary:?}"
    );
    assert!(
        summary.out.contains("from docs/decisions/0003-x\\n::error"),
        "explain names the hostile path as the far end of the edge: {summary:?}"
    );
    assert!(
        hostile.out.contains("\\x1b[2J"),
        "explain prints the hostile summary as the cue of the edge: {hostile:?}"
    );
    clean("explain HW-DR-0004", &summary);
    let missing = root.run(&["explain", "--no-color", MISSING]);
    assert_ne!(missing.code, Some(0), "explain refuses: {missing:?}");
    assert!(
        missing.err.contains("nothing"),
        "explain names the missing path: {missing:?}"
    );
    clean("explain <missing>", &missing);
}

/// Whether the JSON document `text` parses and holds [`HOSTILE`] exactly, as
/// one string value anywhere in it.
fn holds_raw(text: &str) -> bool {
    fn walk(value: &headwater_yaml::Value) -> bool {
        match value {
            headwater_yaml::Value::Scalar(scalar) => scalar.text == HOSTILE,
            headwater_yaml::Value::Seq(items) => items.iter().any(|item| walk(&item.value)),
            headwater_yaml::Value::Map(map) => map.iter().any(|entry| walk(&entry.value.value)),
        }
    }
    let loaded = headwater_yaml::load(text).expect("the JSON parses");
    walk(&loaded.value)
}

#[test]
fn explain_json_keeps_the_raw_name() {
    let root = corpus("hostile-explain-json", false);
    let run = root.run(&["explain", "--json", "HW-DR-0003"]);
    assert_eq!(run.code, Some(0), "explain --json answers: {run:?}");
    assert!(
        holds_raw(&run.out),
        "explain --json holds the raw name: {}",
        run.out
    );
}

#[test]
fn route_writes_a_hostile_pointer_as_visible_text() {
    let root = corpus("hostile-route", false);
    let run = root.run(&["route", "--no-color", "what does HW-DR-0003 decide"]);
    assert_eq!(run.code, Some(0), "route answers: {run:?}");
    assert!(
        run.out.contains("forged-name"),
        "route offers the hostile document: {run:?}"
    );
    clean("route", &run);
    let json = root.run(&["route", "--json", "what does HW-DR-0003 decide"]);
    assert_eq!(json.code, Some(0), "route --json answers: {json:?}");
    assert!(
        holds_raw(&json.out),
        "route --json holds the raw name: {}",
        json.out
    );
}

#[test]
fn show_refuses_in_visible_text_and_writes_the_bytes_unchanged() {
    let root = corpus("hostile-show", false);
    let missing = root.run(&["show", "--no-color", MISSING]);
    assert_ne!(missing.code, Some(0), "show refuses: {missing:?}");
    assert!(
        missing.err.contains("nothing"),
        "show names the missing path: {missing:?}"
    );
    clean("show <missing>", &missing);
    // A row the walk could not read, at a hostile path: `show` and `explain`
    // refuse it in the one sentence `Explanation::refusal_for` writes.
    let pipe = "docs/decisions/0009-p\u{1b}[31m.md";
    let made = std::process::Command::new("mkfifo")
        .arg(root.at.join(pipe))
        .status()
        .expect("mkfifo runs");
    assert!(made.success(), "the named pipe is made");
    for verb in ["show", "explain"] {
        let refused = root.run(&[verb, "--no-color", pipe]);
        assert_ne!(
            refused.code,
            Some(0),
            "{verb} refuses the pipe: {refused:?}"
        );
        assert!(
            refused.err.contains("0009-p"),
            "{verb} names the pipe: {refused:?}"
        );
        clean(&format!("{verb} <pipe>"), &refused);
    }
    let output = root
        .command()
        .args(["show", "HW-DR-0004", "--root"])
        .arg(&root.at)
        .output()
        .expect("the binary runs");
    assert!(output.status.success(), "show answers: {output:?}");
    let file = std::fs::read(root.at.join("docs/decisions/0004-y.md")).expect("the file reads");
    assert_eq!(output.stdout, file, "show writes the file's bytes");
}

#[test]
fn generate_writes_a_hostile_path_as_visible_text() {
    let root = corpus("hostile-generate", true);
    let run = root.run(&["generate", "--check", "--no-color"]);
    assert!(
        run.out.contains("forged-name") || run.err.contains("forged-name"),
        "generate names the hostile file: {run:?}"
    );
    clean("generate --check", &run);
}
