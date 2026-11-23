// SPDX-License-Identifier: Apache-2.0
//! [#1968](https://github.com/headwater-ai/headwater/issues/1968): `headwater
//! explain` answers for a code file by naming the documents that govern it.
//!
//! Before this target, `explain` answered only for a census row or an
//! identifier, so `headwater explain engine/crates/cli/tests/linear_scale.rs`
//! said the file was "outside every corpus root" even though a document
//! governs it. The graph already knew the answer: the MCP
//! `governing_docs_for_path` tool reads it from the same edges. The cases
//! below hold the new answer and the four places where the old refusal has to
//! stay word for word, because `.claude/hooks/write.sh` reads the refusal for a
//! path with no file and `show` prints only a document's bytes.
//!
//! `docs/interfaces/headwater-explain.md` is the contract these cases hold.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

fn copy(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("the directory is there");
    for entry in std::fs::read_dir(from).expect("the fixture directory reads") {
        let entry = entry.expect("the entry reads");
        let target = to.join(entry.file_name());
        match entry.file_type().expect("the file type reads").is_dir() {
            true => copy(&entry.path(), &target),
            false => {
                std::fs::copy(entry.path(), &target).expect("the fixture copies");
            }
        }
    }
}

/// A scratch repository over this repository's own taxonomy source, the
/// shape `classify.rs` builds, with one interface contract whose `governs`
/// edge holds `governs`, and the source files this case names.
struct Root {
    at: PathBuf,
}

impl Root {
    /// `label` names the case, because cargo runs the cases of one target as
    /// threads of one process, and a directory keyed on the process alone is
    /// one that a second case removes while the first reads it.
    fn new(label: &str, governs: &str) -> Root {
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-explain-code-{}-{label}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).expect("the root is made");
        let repository = repository();
        copy(
            &repository.join("taxonomy-source/headwater-standard"),
            &at.join(".headwater/packages/headwater-standard"),
        );
        repoint_bundles(&at.join(".headwater/packages/headwater-standard"));
        copy(
            &repository.join("docs/taxonomies"),
            &at.join("docs/taxonomies"),
        );
        for name in ["taxonomy.yml", "overlay.yml"] {
            std::fs::copy(
                repository.join(".headwater").join(name),
                at.join(".headwater").join(name),
            )
            .expect("the declaration copies");
        }
        std::fs::create_dir_all(at.join("docs/interfaces")).expect("the shelf is made");
        std::fs::write(
            at.join("docs/interfaces/thing.md"),
            format!(
                "---\nid: HW-IFACE-thing\nstatus: current\nstatus_since: 2026-11-23\n\
                 summary: \"A fixture contract that governs one source file.\"\n\
                 last_verified: 2026-11-23\ntitle: \"thing\"\nrelations:\n  governs:\n    - {governs}\n---\n\n\
                 # thing\n\nA fixture contract for #1968.\n"
            ),
        )
        .expect("the contract writes");
        std::fs::create_dir_all(at.join("src")).expect("the source directory is made");
        std::fs::write(at.join("src/lib.rs"), "// governed\n").expect("the source writes");
        std::fs::write(at.join("src/other.rs"), "// not governed\n").expect("the source writes");

        let root = Root { at };
        let resolved = root.run(&["taxonomy", "resolve"]);
        assert_eq!(
            resolved.code,
            Some(0),
            "the fixture resolves\n{}{}",
            resolved.out,
            resolved.err
        );
        root
    }

    fn run(&self, arguments: &[&str]) -> Ran {
        let output = Command::new(env!("CARGO_BIN_EXE_headwater"))
            .args(arguments)
            .arg("--root")
            .arg(&self.at)
            .output()
            .expect("the binary runs");
        Ran {
            code: output.status.code(),
            out: String::from_utf8_lossy(&output.stdout).into_owned(),
            err: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }

    /// The `structuredContent` of the MCP `governing_docs_for_path` tool for
    /// `path`, as the one line of JSON the server writes.
    fn served(&self, path: &str) -> String {
        let input = format!(
            "{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{{}}}}\n\
             {{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":\
             {{\"name\":\"governing_docs_for_path\",\"arguments\":{{\"path\":\"{path}\"}}}}}}\n"
        );
        let mut child = Command::new(env!("CARGO_BIN_EXE_headwater"))
            .args(["mcp", "--now", "2026-11-23", "--root"])
            .arg(&self.at)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("the binary runs");
        child
            .stdin
            .take()
            .expect("the standard input is piped")
            .write_all(input.as_bytes())
            .expect("the requests write");
        let served = child.wait_with_output().expect("the server ends");
        let stdout = String::from_utf8_lossy(&served.stdout).into_owned();
        let line = stdout
            .lines()
            .find(|line| line.contains("\"id\":2,"))
            .unwrap_or_else(|| panic!("the server answers the call\n{stdout}"));
        let structured = line
            .split("\"structuredContent\":")
            .nth(1)
            .unwrap_or_else(|| panic!("the answer carries structuredContent\n{line}"));
        // The structured member is an object, and the response closes after
        // it with the one brace of the result and the one of the message.
        structured
            .strip_suffix("}}")
            .unwrap_or_else(|| panic!("structuredContent is the last member\n{line}"))
            .to_string()
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.at);
    }
}

#[derive(Debug)]
struct Ran {
    code: Option<i32>,
    out: String,
    err: String,
}

/// The JSON the verb writes, indented, as the one line the server writes:
/// every space and newline outside a string goes.
fn compact(pretty: &str) -> String {
    let mut out = String::new();
    let mut quoted = false;
    let mut escaped = false;
    for character in pretty.chars() {
        match (quoted, escaped, character) {
            (true, true, _) => escaped = false,
            (true, false, '\\') => escaped = true,
            (_, false, '"') => quoted = !quoted,
            (false, _, ' ' | '\n') => continue,
            _ => {}
        }
        out.push(character);
    }
    out
}

const REFUSED: &str = "is outside every corpus root this repository declares";

/// The decisive case. One document governs `src/lib.rs`, the file is on the
/// tree, and `explain` names the document by path and identifier. Before
/// #1968 the verb refused it with exit 1 as outside every corpus root.
#[test]
fn a_code_file_that_a_document_governs_is_explained_by_naming_that_document() {
    let root = Root::new("decisive", "src/lib.rs");
    let explained = root.run(&["explain", "src/lib.rs"]);
    assert_eq!(
        explained.code,
        Some(0),
        "a governed code file: {explained:?}"
    );
    assert!(
        explained.out.starts_with("src/lib.rs\n"),
        "the answer opens on the path: {explained:?}"
    );
    assert!(
        explained
            .out
            .contains("governed by docs/interfaces/thing.md (thing) — A fixture contract"),
        "the answer names the governing document as `route` points at it: {explained:?}"
    );
    assert!(
        explained.out.contains("    HW-IFACE-thing\n"),
        "the answer names the identifier: {explained:?}"
    );
    assert_eq!(
        explained.err, "",
        "nothing on standard error: {explained:?}"
    );
}

/// `--json` writes the object the MCP `governing_docs_for_path` tool carries
/// as its `structuredContent`, with the one pointer, and `--paths-at-most`
/// changes nothing in it.
#[test]
fn the_json_answer_for_a_code_file_is_the_structured_content_of_the_mcp_tool() {
    let root = Root::new("json", "src/lib.rs");
    let explained = root.run(&["explain", "src/lib.rs", "--json"]);
    assert_eq!(
        explained.code,
        Some(0),
        "a governed code file: {explained:?}"
    );
    assert!(
        explained.out.starts_with("{\n  \"pointers\": [\n"),
        "the answer is the pointers object: {explained:?}"
    );
    assert_eq!(
        explained.out.matches("\"path\":").count(),
        1,
        "one pointer: {explained:?}"
    );
    assert!(
        !explained.out.contains("\"related\""),
        "no `related` member, so the edit hook reads no governing document from it: {explained:?}"
    );
    assert_eq!(
        compact(&explained.out),
        root.served("src/lib.rs"),
        "the verb and the MCP tool write one value"
    );
    let bounded = root.run(&["explain", "src/lib.rs", "--json", "--paths-at-most", "1"]);
    assert_eq!(bounded.code, Some(0), "the bound is accepted: {bounded:?}");
    assert_eq!(bounded.out, explained.out, "the bound changes nothing here");
}

/// A file on the tree that no governing edge reaches keeps its refusal word
/// for word.
#[test]
fn a_code_file_that_nothing_governs_keeps_its_refusal() {
    let root = Root::new("ungoverned", "src/lib.rs");
    let explained = root.run(&["explain", "src/other.rs"]);
    assert_eq!(explained.code, Some(1), "an ungoverned file: {explained:?}");
    assert_eq!(
        explained.out, "",
        "nothing on standard output: {explained:?}"
    );
    assert_eq!(
        explained.err.trim_end(),
        format!("headwater: `src/other.rs` {REFUSED}"),
        "the refusal is unchanged: {explained:?}"
    );
}

/// A pattern reaches a path with no file behind it, and the path keeps its
/// refusal: `.claude/hooks/write.sh` reads the refusal for a path that does
/// not exist yet, and a pattern over a shelf must not change what it reads.
#[test]
fn a_path_with_no_file_keeps_its_refusal_even_where_a_pattern_reaches_it() {
    let root = Root::new("missing", "src/**");
    let governed = root.run(&["explain", "src/lib.rs"]);
    assert_eq!(governed.code, Some(0), "the pattern binds: {governed:?}");
    let explained = root.run(&["explain", "src/missing.rs"]);
    assert_eq!(
        explained.code,
        Some(1),
        "a path with no file: {explained:?}"
    );
    assert_eq!(
        explained.out, "",
        "nothing on standard output: {explained:?}"
    );
    assert!(
        explained
            .err
            .contains(&format!("`src/missing.rs` {REFUSED}")),
        "the refusal is unchanged: {explained:?}"
    );
}

/// The path forms a shell or an editor writes find the code file that
/// `src/lib.rs` finds (#1227), and a link inside the root answers for the file
/// at its end as the MCP tool does (#1433). A directory that a pattern reaches
/// is not a file, and keeps its refusal.
#[cfg(unix)]
#[test]
fn a_code_file_is_found_by_each_spelling_of_its_path_and_through_a_link() {
    let root = Root::new("spellings", "src/lib.rs");
    std::os::unix::fs::symlink("lib.rs", root.at.join("src/link.rs")).expect("the link is made");
    let absolute = root.at.join("src/lib.rs").display().to_string();
    for target in [
        "./src/lib.rs",
        "src/../src/lib.rs",
        absolute.as_str(),
        "src/link.rs",
    ] {
        let explained = root.run(&["explain", target]);
        assert_eq!(explained.code, Some(0), "{target}: {explained:?}");
        assert!(
            explained
                .out
                .contains("governed by docs/interfaces/thing.md"),
            "{target} names the governing document: {explained:?}"
        );
    }
    let pattern = Root::new("directory", "src/**");
    let explained = pattern.run(&["explain", "src"]);
    assert_eq!(explained.code, Some(1), "a directory: {explained:?}");
    assert_eq!(
        explained.out, "",
        "nothing on standard output: {explained:?}"
    );
}

/// A link that a document governs by its own path is found by each spelling
/// of that path, and not only by the file at its end: the normalized path is
/// what the edges are matched against.
#[cfg(unix)]
#[test]
fn a_governed_link_is_matched_on_its_normalized_path() {
    let root = Root::new("governed-link", "src/link.rs");
    std::os::unix::fs::symlink("other.rs", root.at.join("src/link.rs")).expect("the link is made");
    for target in ["src/link.rs", "./src/link.rs"] {
        let explained = root.run(&["explain", target]);
        assert_eq!(explained.code, Some(0), "{target}: {explained:?}");
        assert!(
            explained
                .out
                .contains("governed by docs/interfaces/thing.md"),
            "{target} names the governing document: {explained:?}"
        );
    }
}

/// A pattern can reach both a link and the file at its end. The document that
/// governs both is named once, because the union of #1433 drops a document it
/// already holds.
#[cfg(unix)]
#[test]
fn a_document_that_governs_a_link_and_its_target_is_named_once() {
    let root = Root::new("link-and-target", "src/*.rs");
    std::os::unix::fs::symlink("other.rs", root.at.join("src/link.rs")).expect("the link is made");
    let explained = root.run(&["explain", "src/link.rs"]);
    assert_eq!(explained.code, Some(0), "a governed link: {explained:?}");
    assert_eq!(
        explained
            .out
            .matches("governed by docs/interfaces/thing.md")
            .count(),
        1,
        "one line for the one document: {explained:?}"
    );
}

/// A document is a census row, and a pattern that governs it does not turn it
/// into a code file: `explain` still prints its derivation.
#[test]
fn a_document_that_a_pattern_governs_is_still_explained_as_a_document() {
    let root = Root::new("governed-document", "docs/**");
    let explained = root.run(&["explain", "docs/interfaces/thing.md"]);
    assert_eq!(explained.code, Some(0), "a document: {explained:?}");
    assert!(
        explained.out.contains("  kind interface_contract\n"),
        "the derivation, not the governing list: {explained:?}"
    );
    assert!(
        !explained.out.contains("governed by"),
        "no governing list: {explained:?}"
    );
}

/// `show` prints the bytes of a document, and a governed code file is not
/// one, so `show` refuses it in the sentence it always wrote.
#[test]
fn show_still_refuses_a_governed_code_file() {
    let root = Root::new("show", "src/lib.rs");
    let shown = root.run(&["show", "src/lib.rs"]);
    assert_eq!(shown.code, Some(1), "show on a code file: {shown:?}");
    assert_eq!(shown.out, "", "nothing on standard output: {shown:?}");
    assert!(
        shown.err.contains(&format!("`src/lib.rs` {REFUSED}")),
        "the refusal is unchanged: {shown:?}"
    );
}

/// Repoint `contents.bundles` in a scratch copy of the authored manifest, as
/// `classify.rs` does and for its reason: the scalar is relative to the
/// package directory, which [`headwater_resolve::package::PACKAGES`] places.
fn repoint_bundles(package: &Path) {
    let up = "../".repeat(headwater_resolve::package::PACKAGES.split('/').count() + 1);
    let manifest = package.join(headwater_resolve::package::MANIFEST);
    let text = std::fs::read_to_string(&manifest).expect("the scratch manifest reads");
    let from = "  bundles: ../../docs/taxonomies";
    assert!(text.contains(from), "the authored manifest states `{from}`");
    let to = format!("  bundles: {up}docs/taxonomies");
    std::fs::write(&manifest, text.replace(from, &to)).expect("the scratch manifest writes");
}
