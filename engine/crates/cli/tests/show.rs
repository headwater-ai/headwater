// SPDX-License-Identifier: Apache-2.0
//! [#740](https://github.com/headwater-ai/headwater/issues/740): `headwater
//! show` prints the bytes of one document, found the way `explain` finds it.
//!
//! # What this target holds
//!
//! Two defects, one case each.
//!
//! A dump that re-renders the document rather than copying it. A reader who
//! pipes `show` into a file or a diff must get the file on disk back, so the
//! fixture document carries the three things a text round trip loses: CRLF
//! line endings, a character outside ASCII, and no newline at the end. The
//! comparison is on raw bytes and never through a lossy decode.
//!
//! A second resolver. `show` finds a document through the same function
//! `explain` does, so a target that `explain` refuses is refused by `show`
//! in the same sentence. The refusal names no verb, so the two standard
//! error streams are equal outright. A copy of the resolver drifts, which is
//! what #319 and #845 each had to repair in `explain` alone.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

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

/// The path of the one document this target writes, relative to the root.
const DOCUMENT: &str = "docs/decisions/0001-a-document-show-prints.md";

/// The identifier that document declares.
const IDENTIFIER: &str = "HW-DR-0001";

/// The front matter is LF, because the reader is not what this target tests.
/// The body carries CRLF, a character outside ASCII, and no final newline.
fn document() -> Vec<u8> {
    let front = "---\nid: HW-DR-0001\ntitle: A document show prints\nstatus: current\nstatus_since: 2026-09-27\nlast_verified: 2026-09-27\nsummary: A fixture for show, whose body a text round trip would change.\n---\n";
    let body = "\r\n# A document show prints\r\n\r\n## Context\r\n\r\nThe caf\u{e9} line ends in CRLF.\r\n\r\n## Decision\r\n\r\nThe last line has no newline.";
    [front.as_bytes(), body.as_bytes()].concat()
}

/// A scratch corpus over this repository's own taxonomy, with one decision
/// written in. The maintained source rather than the vendored artifact, the
/// same choice `classify.rs` makes, so this target does not go stale under a
/// version bump.
struct Root {
    at: PathBuf,
}

impl Root {
    /// `label` names the case, because cargo runs the cases of one target as
    /// threads of one process.
    fn new(label: &str) -> Root {
        let at =
            std::env::temp_dir().join(format!("headwater-cli-show-{}-{label}", std::process::id()));
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
            let to = at.join(".headwater").join(name);
            std::fs::create_dir_all(to.parent().expect("it has a parent"))
                .expect("the declaration directory is there");
            std::fs::copy(repository.join(".headwater").join(name), to)
                .expect("the declaration copies");
        }
        let path = at.join(DOCUMENT);
        std::fs::create_dir_all(path.parent().expect("it has a parent"))
            .expect("the shelf is made");
        std::fs::write(&path, document()).expect("the document writes");

        let root = Root { at };
        let resolved = root.run(&["taxonomy", "resolve"]);
        assert_eq!(
            resolved.status.code(),
            Some(0),
            "the fixture resolves\n{}",
            String::from_utf8_lossy(&resolved.stderr)
        );
        root
    }

    fn run(&self, arguments: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_headwater"))
            .args(arguments)
            .arg("--root")
            .arg(&self.at)
            .output()
            .expect("the binary runs")
    }

    /// The same run from inside the root, with `--root .`: the invocation a
    /// reader in a shell types. A relative root is what made an absolute
    /// target unclassifiable (#1227), so each path case runs both ways.
    fn run_inside(&self, arguments: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_headwater"))
            .args(arguments)
            .arg("--root")
            .arg(".")
            .current_dir(&self.at)
            .output()
            .expect("the binary runs")
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.at);
    }
}

/// The decisive case. Both spellings of the target print the file on disk,
/// byte for byte, with nothing on standard error; and each target `explain`
/// refuses is refused by `show` with no byte on standard output and the same
/// standard error.
#[test]
fn show_writes_the_bytes_on_disk_for_an_identifier_and_for_its_path() {
    let root = Root::new("bytes");
    let on_disk = std::fs::read(root.at.join(DOCUMENT)).expect("the document reads");
    assert_eq!(on_disk, document(), "the fixture wrote what it meant to");

    // The identifier must resolve through the typed reader, or the path case
    // alone could pass on an untyped row and the identifier case would be
    // testing nothing.
    let explained = root.run(&["explain", IDENTIFIER]);
    assert_eq!(
        explained.status.code(),
        Some(0),
        "the fixture document is typed and carries its identifier: {}",
        String::from_utf8_lossy(&explained.stderr)
    );

    for target in [IDENTIFIER, DOCUMENT] {
        let shown = root.run(&["show", target]);
        assert_eq!(
            shown.status.code(),
            Some(0),
            "`show {target}`: {}",
            String::from_utf8_lossy(&shown.stderr)
        );
        assert!(
            shown.stderr.is_empty(),
            "`show {target}` writes nothing on standard error: {}",
            String::from_utf8_lossy(&shown.stderr)
        );
        assert!(
            shown.stdout == on_disk,
            "`show {target}` writes the bytes on disk and nothing else:\n  got  {:?}\n  want {:?}",
            shown.stdout,
            on_disk
        );
    }

    for target in ["HW-DR-9999", "HW-DR-004", "engine/nowhere/at-all.rs"] {
        let shown = root.run(&["show", target]);
        let explained = root.run(&["explain", target]);
        assert_eq!(
            shown.status.code(),
            Some(1),
            "`show {target}` refuses: {}",
            String::from_utf8_lossy(&shown.stderr)
        );
        assert!(
            shown.stdout.is_empty(),
            "`show {target}` writes nothing on standard output"
        );
        assert_eq!(
            explained.status.code(),
            Some(1),
            "`explain {target}` refuses too"
        );
        assert_eq!(
            String::from_utf8_lossy(&shown.stderr),
            String::from_utf8_lossy(&explained.stderr),
            "`show` and `explain` refuse `{target}` in the same sentence, because one resolver answers both"
        );
    }
}

/// A file that is not UTF-8 at all is printed byte for byte. The case above
/// holds CRLF and a missing final newline, and its body is valid UTF-8, so a
/// lossy decode passed it: `from_utf8_lossy` writes U+FFFD, three bytes, for
/// each byte it cannot read. These bytes are what that decode changes.
#[test]
fn show_writes_bytes_that_are_not_utf_8_unchanged() {
    let root = Root::new("not-utf-8");
    let path = "docs/notes/not-utf-8.md";
    let bytes: &[u8] = b"\xff\xfe# Not UTF-8\r\n\r\nA byte \xe9 alone, and \xc3 cut short\r\n";
    let at = root.at.join(path);
    std::fs::create_dir_all(at.parent().expect("it has a parent")).expect("the shelf is made");
    std::fs::write(&at, bytes).expect("the document writes");

    let shown = root.run(&["show", path]);
    assert_eq!(
        shown.status.code(),
        Some(0),
        "the census carries the file as a row: {}",
        String::from_utf8_lossy(&shown.stderr)
    );
    assert!(
        shown.stdout == bytes,
        "`show` writes bytes that are not UTF-8 unchanged:\n  got  {:?}\n  want {:?}",
        shown.stdout,
        bytes
    );
}

/// A symlink under the corpus root is a census row that the walk does not
/// follow, and `show` does not follow it either. Before this case, `show`
/// read the link's target, which can be any file on the host, outside
/// `--root`.
#[cfg(unix)]
#[test]
fn show_refuses_a_symlink_and_prints_nothing_of_its_target() {
    let root = Root::new("symlink");
    let outside = root.at.with_extension("outside");
    std::fs::write(&outside, b"a secret outside the root\n").expect("the target writes");
    let path = "docs/decisions/9998-link.md";
    std::os::unix::fs::symlink(&outside, root.at.join(path)).expect("the link is made");

    let shown = root.run(&["show", path]);
    let _ = std::fs::remove_file(&outside);
    assert_eq!(
        shown.status.code(),
        Some(1),
        "`show` refuses a symlink: {}",
        String::from_utf8_lossy(&shown.stderr)
    );
    assert!(
        shown.stdout.is_empty(),
        "`show` prints no byte of the link's target: {:?}",
        String::from_utf8_lossy(&shown.stdout)
    );
    assert!(
        String::from_utf8_lossy(&shown.stderr).contains("is a symlink"),
        "the refusal names the link: {}",
        String::from_utf8_lossy(&shown.stderr)
    );
}

/// The same row, asked of `explain`. The census never read the link, so it
/// knows no kind and nothing required of it, and `explain` refuses the row
/// rather than answering as for a document (#1366). `--json` writes nothing.
#[cfg(unix)]
#[test]
fn explain_refuses_a_symlink_row_and_prints_nothing() {
    let root = Root::new("explain-symlink");
    let outside = root.at.with_extension("outside");
    std::fs::write(&outside, b"a secret outside the root\n").expect("the target writes");
    let path = "docs/decisions/9998-link.md";
    std::os::unix::fs::symlink(&outside, root.at.join(path)).expect("the link is made");

    for args in [&["explain", path][..], &["explain", path, "--json"]] {
        let explained = root.run(args);
        let stderr = String::from_utf8_lossy(&explained.stderr)
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        assert_eq!(
            explained.status.code(),
            Some(1),
            "{args:?} refuses a symlink row: {stderr}"
        );
        assert!(
            explained.stdout.is_empty(),
            "{args:?} prints nothing of a symlink row: {:?}",
            String::from_utf8_lossy(&explained.stdout)
        );
        assert!(
            stderr.contains("is a symlink to") && stderr.contains("so `explain` prints nothing"),
            "{args:?} names the row and why: {stderr}"
        );
    }
    let _ = std::fs::remove_file(&outside);

    // The control: an untyped document the walk did read still answers, as
    // `docs/interfaces/headwater-explain.md` promises it an explanation.
    let untyped = "docs/decisions/9997-untyped.md";
    std::fs::write(root.at.join(untyped), "# No front matter\n").expect("the file writes");
    for args in [&["explain", untyped][..], &["explain", untyped, "--json"]] {
        let explained = root.run(args);
        assert_eq!(
            explained.status.code(),
            Some(0),
            "{args:?} explains an untyped document: {}",
            String::from_utf8_lossy(&explained.stderr)
        );
        assert!(
            String::from_utf8_lossy(&explained.stdout).contains(untyped),
            "{args:?} names the untyped document: {:?}",
            String::from_utf8_lossy(&explained.stdout)
        );
    }
}

/// A bare `show` names what it takes, in the shape a bare `explain` does.
#[test]
fn a_bare_show_names_what_it_takes() {
    let root = Root::new("bare");
    let shown = root.run(&["show"]);
    assert_eq!(shown.status.code(), Some(1));
    assert!(shown.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&shown.stderr).contains("`show` takes a path or an identifier"),
        "{}",
        String::from_utf8_lossy(&shown.stderr)
    );
}

/// Repoint `contents.bundles` in a scratch copy of the authored manifest, as
/// `classify.rs` does and for the reason it states.
fn repoint_bundles(package: &Path) {
    let up = "../".repeat(headwater_resolve::package::PACKAGES.split('/').count() + 1);
    let manifest = package.join(headwater_resolve::package::MANIFEST);
    let text = std::fs::read_to_string(&manifest).expect("the scratch manifest reads");
    let from = "  bundles: ../../docs/taxonomies";
    assert!(text.contains(from), "the authored manifest states `{from}`");
    let to = format!("  bundles: {up}docs/taxonomies");
    std::fs::write(&manifest, text.replace(from, &to)).expect("the scratch manifest writes");
}

/// One way to run the binary over a [`Root`].
type Run = fn(&Root, &[&str]) -> Output;

/// The two ways each path case runs: with an absolute `--root`, and from
/// inside the root with `--root .`.
const RUNS: [(&str, Run); 2] = [
    ("--root <absolute>", Root::run),
    ("--root . from inside", Root::run_inside),
];

/// [#1227](https://github.com/headwater-ai/headwater/issues/1227): a path is
/// typed the way a shell or an editor spells it. `./`, a `..` that stays in
/// the repository and an absolute path under the root name the same document
/// as the plain relative path, so `explain` and `show` answer each one as they
/// answer that path. Each spelling runs with an absolute `--root` and again
/// from inside the root with `--root .`, because the relative root is the run
/// that read an absolute target as unclassifiable.
#[test]
fn explain_and_show_resolve_every_spelling_of_a_path_inside_the_repository() {
    let root = Root::new("spellings");
    let on_disk = std::fs::read(root.at.join(DOCUMENT)).expect("the document reads");
    let plain = root.run(&["explain", DOCUMENT]);
    assert_eq!(plain.status.code(), Some(0), "the plain path resolves");

    let spellings = [
        format!("./{DOCUMENT}"),
        format!("docs/../{DOCUMENT}"),
        root.at.join(DOCUMENT).display().to_string(),
    ];
    for target in &spellings {
        for (how, run) in RUNS {
            let explained = run(&root, &["explain", target]);
            assert_eq!(
                explained.status.code(),
                Some(0),
                "`explain {target}` with {how}: {}",
                String::from_utf8_lossy(&explained.stderr)
            );
            assert_eq!(
                String::from_utf8_lossy(&explained.stdout),
                String::from_utf8_lossy(&plain.stdout),
                "`explain {target}` with {how} explains the document the plain path names"
            );
            let shown = run(&root, &["show", target]);
            assert_eq!(
                shown.status.code(),
                Some(0),
                "`show {target}` with {how}: {}",
                String::from_utf8_lossy(&shown.stderr)
            );
            assert!(
                shown.stdout == on_disk,
                "`show {target}` with {how} writes the bytes on disk"
            );
        }
    }
}

/// The other half of #1227: a path that leaves the repository, absolute or by
/// `..`, is refused as outside it. It is never read as a corpus path with no
/// document written there, which is the sentence a hook takes as leave to
/// write. `explain --json` is refused the same way, with nothing on standard
/// output (HW-DR-0043).
#[test]
fn explain_and_show_refuse_a_path_outside_the_repository_as_outside_it() {
    let root = Root::new("outside");
    let beside = root
        .at
        .parent()
        .expect("the root has a parent")
        .join("outside.md")
        .display()
        .to_string();
    for target in ["/etc/passwd", "../../outside.md", beside.as_str()] {
        for (how, run) in RUNS {
            for arguments in [
                vec!["explain", target],
                vec!["explain", target, "--json"],
                vec!["show", target],
            ] {
                let refused = run(&root, &arguments);
                let asked = arguments.join(" ");
                let stderr = String::from_utf8_lossy(&refused.stderr);
                assert_eq!(
                    refused.status.code(),
                    Some(1),
                    "`{asked}` with {how} refuses: {stderr}"
                );
                assert!(
                    refused.stdout.is_empty(),
                    "`{asked}` with {how} writes nothing on standard output"
                );
                assert!(
                    stderr.contains("outside this repository"),
                    "`{asked}` with {how} says it is outside this repository: {stderr}"
                );
                assert!(
                    !stderr.contains("no document written there"),
                    "`{asked}` with {how} is not read as a corpus path: {stderr}"
                );
            }
        }
    }
}

/// [#1249](https://github.com/headwater-ai/headwater/issues/1249): five routes
/// find a document by a typed path, and a path outside the repository gets one
/// sentence from all five, byte for byte. The routes are `explain` and `show`,
/// and the MCP `explain`, `related` and `governing_docs_for_path` tools. A
/// symlink in the root that leads out of it is outside too: `escape/x.md` is
/// a file on another part of the host, and no route reads it as a path of
/// this corpus.
#[cfg(unix)]
#[test]
fn every_path_route_refuses_a_path_outside_the_repository_in_one_sentence() {
    use std::io::Write;

    let root = Root::new("one-sentence");
    let elsewhere = root.at.with_extension("elsewhere");
    let _ = std::fs::remove_dir_all(&elsewhere);
    std::fs::create_dir_all(&elsewhere).expect("the outside directory is made");
    std::fs::write(elsewhere.join("x.md"), b"a file outside the root\n")
        .expect("the outside file writes");
    std::os::unix::fs::symlink(&elsewhere, root.at.join("escape")).expect("the link is made");

    let targets = [
        "escape/x.md",
        "./escape/x.md",
        "/etc/passwd",
        "../outside.md",
    ];
    let mut failures: Vec<String> = Vec::new();
    for target in targets {
        let sentence = headwater_query::outside_text(target);
        assert_eq!(
            sentence,
            format!("`{target}` is outside this repository, or is not a path it can read"),
            "the shared sentence is the documented one"
        );
        for (how, run) in RUNS {
            for verb in ["explain", "show"] {
                let refused = run(&root, &[verb, target]);
                let stderr = String::from_utf8_lossy(&refused.stderr).into_owned();
                if refused.status.code() != Some(1)
                    || !refused.stdout.is_empty()
                    || stderr != format!("headwater: {sentence}\n")
                {
                    failures.push(format!("`{verb} {target}` with {how}: {stderr:?}"));
                }
            }
        }

        let mut input = String::from(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{}}\n",
        );
        let tools = [
            ("explain", "target"),
            ("related", "target"),
            ("governing_docs_for_path", "path"),
        ];
        for (at, (tool, key)) in tools.iter().enumerate() {
            input.push_str(&format!(
                "{{\"jsonrpc\":\"2.0\",\"id\":{},\"method\":\"tools/call\",\"params\":\
                 {{\"name\":\"{tool}\",\"arguments\":{{\"{key}\":\"{target}\"}}}}}}\n",
                at + 2
            ));
        }
        let mut child = Command::new(env!("CARGO_BIN_EXE_headwater"))
            .args(["mcp", "--now", "2026-09-28", "--root"])
            .arg(&root.at)
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
        for (at, (tool, _)) in tools.iter().enumerate() {
            let id = format!("\"id\":{},", at + 2);
            let line = stdout.lines().find(|line| line.contains(&id)).unwrap_or("");
            // The sentence holds no quote, so the text member ends at the
            // first one, and its one escape is the closing newline.
            let text = line
                .split("\"text\":\"")
                .nth(1)
                .and_then(|rest| rest.split('"').next())
                .unwrap_or("")
                .replace("\\n", "\n");
            if text != format!("{sentence}\n") {
                failures.push(format!("MCP `{tool} {target}`: {text:?}"));
            }
        }
    }
    let _ = std::fs::remove_dir_all(&elsewhere);
    assert!(
        failures.is_empty(),
        "each route refuses in the one sentence:\n{}",
        failures.join("\n")
    );
}

/// A `./` spelling is a path, and never an identifier. `./HW-DR-0001` names
/// the file `HW-DR-0001` at the root, and no document is written there, so it
/// is refused as a path outside every corpus root, which is what `main` said
/// before #1227. Only the path of a document answers a retried spelling.
#[test]
fn a_dot_slash_spelling_of_an_identifier_is_a_path_and_is_refused_as_one() {
    let root = Root::new("dot-identifier");
    let target = format!("./{IDENTIFIER}");
    for (how, run) in RUNS {
        for verb in ["explain", "show"] {
            let refused = run(&root, &[verb, &target]);
            let stderr = String::from_utf8_lossy(&refused.stderr);
            assert_eq!(
                refused.status.code(),
                Some(1),
                "`{verb} {target}` with {how} is not the identifier: {stderr}"
            );
            assert!(refused.stdout.is_empty(), "`{verb} {target}` with {how}");
            assert!(
                stderr.contains("is outside every corpus root this repository declares"),
                "`{verb} {target}` with {how} reads as a path: {stderr}"
            );
        }
    }
}

/// An absolute path with no file behind it, under `--root .`, is a path of
/// this corpus. No canonical read is possible without a file, so only the
/// root made absolute can strip it. Without that, the path reads as outside
/// the repository, and a hook that asks where a new document may go is told
/// the wrong thing.
#[test]
fn an_absolute_path_with_no_file_under_a_relative_root_is_a_path_of_this_corpus() {
    let root = Root::new("absolute-missing");
    let target = root
        .at
        .join("docs/decisions/0002-not-written.md")
        .display()
        .to_string();
    let refused = root.run_inside(&["explain", &target]);
    let stderr = String::from_utf8_lossy(&refused.stderr);
    assert_eq!(refused.status.code(), Some(1), "{stderr}");
    assert!(
        stderr.contains("is a path of this corpus, with no document written there yet"),
        "`explain {target}` with `--root .` reads as a corpus path: {stderr}"
    );
}

/// A `--root` reached through a symlink, and a target typed through the real
/// directory. The two absolute paths share no prefix as written, so only the
/// canonical read of both finds the document.
#[cfg(unix)]
#[test]
fn an_absolute_target_finds_its_document_under_a_root_reached_through_a_symlink() {
    let root = Root::new("linked-root");
    let link = root.at.with_file_name(format!(
        "{}-link",
        root.at
            .file_name()
            .expect("the root has a name")
            .to_string_lossy()
    ));
    let _ = std::fs::remove_file(&link);
    std::os::unix::fs::symlink(&root.at, &link).expect("the link is made");
    let target = root.at.join(DOCUMENT).display().to_string();
    let shown = Command::new(env!("CARGO_BIN_EXE_headwater"))
        .args(["show", &target, "--root"])
        .arg(&link)
        .output()
        .expect("the binary runs");
    let _ = std::fs::remove_file(&link);
    assert_eq!(
        shown.status.code(),
        Some(0),
        "`show {target}` under a linked root: {}",
        String::from_utf8_lossy(&shown.stderr)
    );
    assert!(shown.stdout == document(), "the bytes on disk");
}

/// An absolute path through a link that sits outside the root and leads into
/// a directory under it names the document there, with the file on disk and
/// without one. No leading part of the path is the root, so the part that
/// exists is made canonical and the rest is joined back on.
#[cfg(unix)]
#[test]
fn an_absolute_path_through_a_link_into_the_root_finds_its_document() {
    let root = Root::new("link-into");
    let link = root.at.with_file_name(format!(
        "{}-decisions",
        root.at
            .file_name()
            .expect("the root has a name")
            .to_string_lossy()
    ));
    let _ = std::fs::remove_file(&link);
    std::os::unix::fs::symlink(root.at.join("docs/decisions"), &link).expect("the link is made");
    let present = Path::new(DOCUMENT)
        .file_name()
        .expect("the document has a name");
    let target = link.join(present).display().to_string();
    let shown = root.run(&["show", &target]);
    let missing = link.join("0002-not-written.md").display().to_string();
    let refused = root.run(&["explain", &missing]);
    let _ = std::fs::remove_file(&link);
    assert_eq!(
        shown.status.code(),
        Some(0),
        "`show {target}`: {}",
        String::from_utf8_lossy(&shown.stderr)
    );
    assert!(shown.stdout == document(), "the bytes on disk");
    let stderr = String::from_utf8_lossy(&refused.stderr);
    assert_eq!(refused.status.code(), Some(1), "{stderr}");
    assert!(
        stderr.contains("is a path of this corpus, with no document written there yet"),
        "`explain {missing}` through a link into the root: {stderr}"
    );
}

/// The same link into the root, under a `--root` given through a symlink and
/// under `--root ..` from a directory below the root. Neither spelling of the
/// root is its canonical path, and only the canonical path is a prefix of the
/// target once the link in it is followed.
#[cfg(unix)]
#[test]
fn a_link_into_the_root_finds_its_document_under_a_linked_or_relative_root() {
    let root = Root::new("link-into-roots");
    let name = root
        .at
        .file_name()
        .expect("the root has a name")
        .to_string_lossy()
        .to_string();
    let into = root.at.with_file_name(format!("{name}-decisions"));
    let via = root.at.with_file_name(format!("{name}-via"));
    let _ = std::fs::remove_file(&into);
    let _ = std::fs::remove_file(&via);
    std::os::unix::fs::symlink(root.at.join("docs/decisions"), &into).expect("the link in");
    std::os::unix::fs::symlink(&root.at, &via).expect("the linked root");
    let present = Path::new(DOCUMENT)
        .file_name()
        .expect("the document has a name");
    let target = into.join(present).display().to_string();
    let missing = into.join("0002-not-written.md").display().to_string();
    let under_link = |verb: &str, path: &str| {
        Command::new(env!("CARGO_BIN_EXE_headwater"))
            .args([verb, path, "--root"])
            .arg(&via)
            .output()
            .expect("the binary runs")
    };
    let under_parent = |verb: &str, path: &str| {
        Command::new(env!("CARGO_BIN_EXE_headwater"))
            .args([verb, path, "--root", ".."])
            .current_dir(root.at.join("docs"))
            .output()
            .expect("the binary runs")
    };
    let shown = [
        ("--root <link>", under_link("show", &target)),
        ("--root ..", under_parent("show", &target)),
    ];
    let refused = [
        ("--root <link>", under_link("explain", &missing)),
        ("--root ..", under_parent("explain", &missing)),
    ];
    let _ = std::fs::remove_file(&into);
    let _ = std::fs::remove_file(&via);
    for (how, shown) in &shown {
        assert_eq!(
            shown.status.code(),
            Some(0),
            "`show {target}` with {how}: {}",
            String::from_utf8_lossy(&shown.stderr)
        );
        assert!(shown.stdout == document(), "the bytes on disk, with {how}");
    }
    for (how, refused) in &refused {
        let stderr = String::from_utf8_lossy(&refused.stderr);
        assert_eq!(refused.status.code(), Some(1), "with {how}: {stderr}");
        assert!(
            stderr.contains("is a path of this corpus, with no document written there yet"),
            "`explain {missing}` with {how}: {stderr}"
        );
    }
}

/// An absolute path that leaves the root with `..` and comes back in names
/// the document under it, as `a/../x` names `x`. Its leading parts name the
/// root twice, and only the longer of the two leaves the rest of the path
/// inside the root.
#[test]
fn an_absolute_path_that_leaves_the_root_and_comes_back_finds_its_document() {
    let root = Root::new("back-in");
    let name = root.at.file_name().expect("the root has a name");
    let target = root
        .at
        .join("..")
        .join(name)
        .join(DOCUMENT)
        .display()
        .to_string();
    let shown = root.run(&["show", &target]);
    assert_eq!(
        shown.status.code(),
        Some(0),
        "`show {target}`: {}",
        String::from_utf8_lossy(&shown.stderr)
    );
    assert!(shown.stdout == document(), "the bytes on disk");
}

/// An absolute path with no file behind it, typed through a symlink to the
/// root, from a shell whose working directory is that symlink, under
/// `--root .` ([#1334](https://github.com/headwater-ai/headwater/issues/1334)).
/// The process reads its working directory as the physical path, so the
/// target as typed shares no prefix with the root, and no file exists to make
/// canonical. Every macOS temporary directory is reached this way, through
/// `/var -> /private/var`, so this test makes its own link and does not
/// depend on the host's `TMPDIR`.
#[cfg(unix)]
#[test]
fn an_absolute_path_with_no_file_typed_through_a_linked_root_is_a_path_of_this_corpus() {
    let root = Root::new("linked-missing");
    let link = root.at.with_file_name(format!(
        "{}-link",
        root.at
            .file_name()
            .expect("the root has a name")
            .to_string_lossy()
    ));
    let _ = std::fs::remove_file(&link);
    std::os::unix::fs::symlink(&root.at, &link).expect("the link is made");
    let target = link
        .join("docs/decisions/0002-not-written.md")
        .display()
        .to_string();
    let refused = Command::new(env!("CARGO_BIN_EXE_headwater"))
        .args(["explain", &target, "--root", "."])
        .current_dir(&link)
        .output()
        .expect("the binary runs");
    let _ = std::fs::remove_file(&link);
    let stderr = String::from_utf8_lossy(&refused.stderr);
    assert_eq!(refused.status.code(), Some(1), "{stderr}");
    assert!(refused.stdout.is_empty(), "nothing on stdout: {stderr}");
    assert!(
        stderr.contains("is a path of this corpus, with no document written there yet"),
        "`explain {target}` through a linked root reads as a corpus path: {stderr}"
    );
}
