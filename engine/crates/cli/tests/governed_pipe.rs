// SPDX-License-Identifier: Apache-2.0
//! `headwater check` and `headwater show` finish when a named pipe is in the
//! tree, at a document path or under a `governs` edge (#1269, #1333).
//!
//! A named pipe with no writer blocks a process that opens it to read, and it
//! never returns. The walk reports a pipe as an entry of its own kind, so the
//! census never reads one. An edge that names one, or a wildcard that matches
//! one, hands it to the reader that digests what an edge governs. That reader
//! takes regular files alone, so `check` never opens the pipe. Each case runs
//! the verb under a deadline and fails on it, because a verb that opened the
//! pipe does not end.

#![cfg(unix)]

mod common;
use common::Root;

/// One decision that governs `target` and records no `verified_revision`.
fn decision(target: &str) -> String {
    format!(
        "---\nid: HW-DR-0002\ntitle: The decision that governs a pipe\nstatus: \
         current\nstatus_since: 2026-08-01\nlast_verified: 2026-08-01\nsummary: One \
         decision that governs a named pipe under the tools directory.\nprovenance:\n  \
         warrant: asserted\n  agency: human\n  evidence_basis: unevidenced\nrelations:\n  \
         governs:\n    - {target}\n---\n\n# The decision that governs a pipe\n\n## \
         Context\n\nA fixture.\n\n## Decision\n\nIt governs a named pipe.\n\n## \
         Consequences\n\nThe check reads it.\n"
    )
}

/// The stub root, with a named pipe at `tools/pipe`, a regular file at
/// `tools/a.sh`, and one decision that governs `target`.
fn root(label: &str, target: &str) -> Root {
    let target = target.to_owned();
    Root::shaped(label, move |at| {
        std::fs::write(at.join("tools/a.sh"), "echo a\n").expect("the regular file writes");
        let fifo = std::process::Command::new("mkfifo")
            .arg(at.join("tools/pipe"))
            .status()
            .expect("mkfifo runs");
        assert!(fifo.success(), "the named pipe is made");
        std::fs::write(
            at.join("docs/decisions/0002-the-decision-that-governs-a-pipe.md"),
            decision(&target),
        )
        .expect("the decision writes");
    })
}

/// Run `headwater <args> --root <root>` and fail with `hung` if it has not
/// ended in 60 s, which is how a verb that opened a named pipe with no writer
/// ends. The exit status is not asserted: the stub corpus can carry findings
/// that have nothing to do with the pipe.
///
/// Standard output and standard error are drained on their own threads while
/// the verb runs. `check` prints more than a pipe buffer holds, so a caller
/// that reads only after the exit leaves the verb blocked on its own write,
/// and that hang is the harness's rather than the one this file pins.
fn under_deadline(root: &Root, args: &[&str], hung: &str) -> (String, String) {
    let (_, out, err) = ended(root, args, hung);
    (out, err)
}

/// [`under_deadline`], with the exit status the verb ended on, for a case
/// that pins a refusal (#1366).
fn ended(root: &Root, args: &[&str], hung: &str) -> (std::process::ExitStatus, String, String) {
    use std::io::Read;
    use std::time::{Duration, Instant};

    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_headwater"))
        .args(args)
        .arg("--root")
        .arg(&root.at)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("the binary runs");
    let drain = |mut from: Box<dyn Read + Send>| {
        std::thread::spawn(move || {
            let mut bytes = Vec::new();
            from.read_to_end(&mut bytes).expect("the stream reads");
            String::from_utf8_lossy(&bytes).into_owned()
        })
    };
    let out = drain(Box::new(child.stdout.take().expect("stdout is piped")));
    let err = drain(Box::new(child.stderr.take().expect("stderr is piped")));
    let deadline = Instant::now() + Duration::from_secs(60);
    let status = loop {
        if let Some(status) = child.try_wait().expect("the child is there") {
            break status;
        }
        if Instant::now() > deadline {
            child.kill().expect("the child stops");
            child.wait().expect("the child is reaped");
            panic!("{hung}");
        }
        std::thread::sleep(Duration::from_millis(50));
    };
    (
        status,
        out.join().expect("stdout drains"),
        err.join().expect("stderr drains"),
    )
}

/// The words the suspect rule reports a literal that names no regular file
/// with. A named pipe gets no digest, as a directory gets none, so the edge
/// never goes suspect, and a silence would not tell its author (#1333).
const NO_REGULAR_FILE: &str = "names no regular file";

#[test]
fn check_no_cache_finishes_when_a_governs_edge_reaches_a_named_pipe() {
    let root = root("check-governed-pipe", "tools/pipe");
    let (out, err) = under_deadline(
        &root,
        &["check", "--no-cache"],
        "check opened the named pipe a governs edge names, and waited on it",
    );
    assert!(!out.contains("revision"), "no revision finding: {out}");
    assert!(!err.contains("revision"), "no revision error: {err}");
    // A pipe gets no digest, as a directory gets none, but it is not one:
    // the suspect rule must not call it a directory or tell the author to
    // write `tools/pipe/**`, which names nothing.
    assert!(!out.contains("tools/pipe/**"), "no directory remedy: {out}");
    assert!(
        !out.contains("names a directory"),
        "no directory finding: {out}"
    );
    // It reports the edge rather than passing it, because the edge can
    // never age (#1333).
    // At `Info`, the level a directory literal gets: the report says the
    // edge can never age, and nothing about the edge is wrong yet.
    assert!(
        flat(&out).contains(
            "· info relation.target.suspect (OB-REL-6): `HW-DR-0002` declares `governs: \
             tools/pipe`, and `tools/pipe` names no regular file"
        ),
        "the edge that can never age is reported at info: {out}"
    );
}

/// A wildcard that matches the named pipe and nothing else binds, as a
/// literal does, and gets the same report. The walk reports the pipe as an
/// entry of its own kind, and the resolver keeps that kind among what a
/// pattern matches, so the edge is not refused as matching nothing (#1333).
#[test]
fn a_wildcard_that_matches_only_a_named_pipe_is_reported_like_the_literal() {
    let root = root("check-governed-pipe-only-wildcard", "tools/pip*");
    let (out, _) = under_deadline(
        &root,
        &["check", "--no-cache"],
        "check opened the named pipe a governs wildcard matches, and waited on it",
    );
    let out = flat(&out);
    assert!(
        out.contains("`tools/pip*` names no regular file"),
        "the wildcard over only a pipe is reported: {out}"
    );
    assert!(
        !out.contains("no entry in the source tree matches"),
        "{out}"
    );
}

/// A wildcard over a regular file and a named pipe digests the file alone,
/// so the moved-digest finding counts the one entry the digest covers and
/// not the two the pattern matched (#1333).
#[test]
fn a_moved_wildcard_over_a_named_pipe_counts_only_what_its_digest_covers() {
    let root = root(
        "check-governed-pipe-count",
        "to: tools/**\n      verified_revision: sha256:0000",
    );
    let (out, _) = under_deadline(
        &root,
        &["check", "--no-cache"],
        "check opened the named pipe a governs wildcard matches, and waited on it",
    );
    let out = flat(&out);
    // The stub root holds other regular files under `tools/`, so the count
    // is read off the tree: every entry under it but the pipe.
    let regular = regular_files(&root.at.join("tools"));
    assert!(
        out.contains(&format!("the {regular} regular files it covers now read")),
        "the count is the {regular} entries the digest covers: {out}"
    );
    assert!(
        !out.contains(&format!("the {} regular files", regular + 1)),
        "the pipe is not counted: {out}"
    );
    assert!(!flat(&out).contains(NO_REGULAR_FILE), "{out}");
}

/// How many regular files are under `dir`, at any depth.
fn regular_files(dir: &std::path::Path) -> usize {
    std::fs::read_dir(dir)
        .expect("the directory reads")
        .map(|entry| entry.expect("the entry reads").path())
        .map(|path| match std::fs::symlink_metadata(&path) {
            Ok(meta) if meta.is_dir() => regular_files(&path),
            Ok(meta) if meta.is_file() => 1,
            _ => 0,
        })
        .sum()
}

/// A named pipe at a path the census reads as a document. The census opens
/// no named pipe, so `check` and `census` end, and the row says what the
/// entry is rather than calling it untyped or unreadable text (#1333).
fn document_pipe(label: &str) -> Root {
    Root::shaped(label, |at| {
        let fifo = std::process::Command::new("mkfifo")
            .arg(at.join("docs/x.md"))
            .status()
            .expect("mkfifo runs");
        assert!(fifo.success(), "the named pipe is made");
    })
}

/// `text` with every run of white space made one space, because the report
/// wraps a finding across lines wherever its width falls.
fn flat(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn check_no_cache_finishes_when_a_named_pipe_takes_a_document_path() {
    let root = document_pipe("check-document-pipe");
    let (out, _) = under_deadline(
        &root,
        &["check", "--no-cache"],
        "check opened the named pipe at docs/x.md, and waited on it",
    );
    let out = flat(&out);
    assert!(
        out.contains(
            "docs/x.md unwalkable: a named pipe, a socket or a device, which the census never opens"
        ),
        "the census row says what the entry is: {out}"
    );
}

/// `derived` walks the tree itself, not through the census, and reads each
/// file it finds. It lists no named pipe, so it ends (#1333, verify round 1).
#[test]
fn derived_finishes_when_a_named_pipe_takes_a_document_path() {
    let root = document_pipe("derived-document-pipe");
    under_deadline(
        &root,
        &["derived"],
        "derived opened the named pipe at docs/x.md, and waited on it",
    );
}

/// The same, with the named pipe reached through a symlink at the document
/// path. `derived` follows a link, so it must ask what the link names, not
/// what the link is (#1333, verify round 2).
#[test]
fn derived_finishes_when_a_symlink_at_a_document_path_names_a_named_pipe() {
    let root = Root::shaped("derived-document-link-to-pipe", |at| {
        let fifo = std::process::Command::new("mkfifo")
            .arg(at.join("tools/pipe"))
            .status()
            .expect("mkfifo runs");
        assert!(fifo.success(), "the named pipe is made");
        std::os::unix::fs::symlink("../tools/pipe", at.join("docs/x.md"))
            .expect("the link is made");
    });
    under_deadline(
        &root,
        &["derived"],
        "derived followed the link at docs/x.md to the named pipe, and waited on it",
    );
}

/// `show` reads the bytes of a document, and the census it finds the
/// document in is the walk that met the pipe. The row is not a document, so
/// `show` refuses it rather than opening it.
#[test]
fn show_finishes_and_refuses_a_named_pipe_at_a_document_path() {
    let root = document_pipe("show-document-pipe");
    let (out, err) = under_deadline(
        &root,
        &["show", "docs/x.md"],
        "show opened the named pipe at docs/x.md, and waited on it",
    );
    assert!(out.is_empty(), "show prints nothing of a named pipe: {out}");
    assert!(flat(&err).contains("the census never opens"), "{err}");
}

/// `explain` finds the row `show` refuses, and the row is not a document: the
/// census never opened the entry, so it knows no kind and no requirement to
/// state. `explain` refuses it in `show`'s shape, and `--json` writes nothing,
/// as a refusal under `--json` does (HW-DR-0043, #1366).
#[test]
fn explain_refuses_a_named_pipe_at_a_document_path() {
    let root = document_pipe("explain-document-pipe");
    for args in [
        &["explain", "docs/x.md"][..],
        &["explain", "docs/x.md", "--json"],
    ] {
        let (status, out, err) = ended(
            &root,
            args,
            "explain opened the named pipe at docs/x.md, and waited on it",
        );
        assert_eq!(status.code(), Some(1), "{args:?} refuses: {out}{err}");
        assert!(
            out.is_empty(),
            "{args:?} prints nothing of a named pipe: {out}"
        );
        let err = flat(&err);
        assert!(err.contains("the census never opens"), "{args:?}: {err}");
        assert!(
            err.contains("so `explain` prints nothing"),
            "{args:?}: {err}"
        );
    }
}

/// A cached verdict over a directory literal does not survive the directory
/// being replaced by a named pipe of the same name. Both get no digest, so a
/// cache key that states only the digest names the two states alike, and a
/// warm check would repeat the directory finding and its `/**` remedy about
/// a path that is now a pipe. The warm run must say what a cold run says.
#[test]
fn a_cached_directory_verdict_does_not_outlive_the_directory_becoming_a_pipe() {
    let root = Root::shaped("check-governed-dir-to-pipe", |at| {
        std::fs::create_dir_all(at.join("tools/thing")).expect("the directory is made");
        std::fs::write(
            at.join("docs/decisions/0002-the-decision-that-governs-a-pipe.md"),
            decision("tools/thing"),
        )
        .expect("the decision writes");
    });
    let (before, _) = under_deadline(
        &root,
        &["check"],
        "check waited on the directory, which it cannot open as a pipe",
    );
    assert!(
        before.contains("`tools/thing` names a directory"),
        "the directory literal is reported first: {before}"
    );

    std::fs::remove_dir(root.at.join("tools/thing")).expect("the directory goes");
    let fifo = std::process::Command::new("mkfifo")
        .arg(root.at.join("tools/thing"))
        .status()
        .expect("mkfifo runs");
    assert!(fifo.success(), "the named pipe is made");

    let (warm, _) = under_deadline(
        &root,
        &["check"],
        "a warm check opened the named pipe that replaced the directory",
    );
    let (cold, _) = under_deadline(
        &root,
        &["check", "--no-cache"],
        "a cold check opened the named pipe that replaced the directory",
    );
    assert!(!cold.contains("tools/thing/**"), "{cold}");
    assert!(
        !warm.contains("tools/thing/**"),
        "the cached directory verdict outlived the directory: {warm}"
    );
    assert!(flat(&cold).contains(NO_REGULAR_FILE), "{cold}");
    assert!(flat(&warm).contains(NO_REGULAR_FILE), "{warm}");
}

/// A cached verdict over a named pipe does not survive the pipe being
/// replaced by a regular file the process cannot read. Neither gets a
/// digest, and neither is a directory, but the rule reports the pipe and
/// passes the unreadable file, so the cache key has to tell them apart
/// (#1333). The case needs a process that a mode-000 file stops, so it
/// returns early when run as root.
#[test]
fn a_cached_pipe_verdict_does_not_outlive_the_pipe_becoming_an_unreadable_file() {
    use std::os::unix::fs::PermissionsExt;

    let root = root("check-governed-pipe-to-locked", "tools/pipe");
    let (before, _) = under_deadline(&root, &["check"], "check opened the named pipe");
    assert!(flat(&before).contains(NO_REGULAR_FILE), "{before}");

    let locked = root.at.join("tools/pipe");
    std::fs::remove_file(&locked).expect("the pipe goes");
    std::fs::write(&locked, "echo locked\n").expect("the file writes");
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000))
        .expect("the mode is set");
    if std::fs::read(&locked).is_ok() {
        return;
    }

    let (warm, _) = under_deadline(&root, &["check"], "a warm check did not end");
    let (cold, _) = under_deadline(&root, &["check", "--no-cache"], "a cold check did not end");
    assert!(!flat(&cold).contains(NO_REGULAR_FILE), "{cold}");
    assert!(
        !flat(&warm).contains(NO_REGULAR_FILE),
        "the cached pipe verdict outlived the pipe: {warm}"
    );
}

/// The same swap, with the directory replaced by a regular file that the
/// process cannot read. That file has no digest either, and it is not a
/// directory, so a warm check must not repeat the cached directory finding
/// (#1269, verify round 1). The case needs a process that a mode-000 file
/// stops, so it returns early when run as root.
#[test]
fn a_cached_directory_verdict_does_not_outlive_the_directory_becoming_an_unreadable_file() {
    use std::os::unix::fs::PermissionsExt;

    let root = Root::shaped("check-governed-dir-to-locked", |at| {
        std::fs::create_dir_all(at.join("tools/thing")).expect("the directory is made");
        std::fs::write(
            at.join("docs/decisions/0002-the-decision-that-governs-a-pipe.md"),
            decision("tools/thing"),
        )
        .expect("the decision writes");
    });
    let (before, _) = under_deadline(&root, &["check"], "check did not end over a directory");
    assert!(
        before.contains("`tools/thing` names a directory"),
        "the directory literal is reported first: {before}"
    );

    let locked = root.at.join("tools/thing");
    std::fs::remove_dir(&locked).expect("the directory goes");
    std::fs::write(&locked, "echo locked\n").expect("the file writes");
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000))
        .expect("the mode is set");
    if std::fs::read(&locked).is_ok() {
        return;
    }

    let (warm, _) = under_deadline(&root, &["check"], "a warm check did not end");
    let (cold, _) = under_deadline(&root, &["check", "--no-cache"], "a cold check did not end");
    assert!(!cold.contains("tools/thing/**"), "{cold}");
    assert!(
        !warm.contains("tools/thing/**"),
        "the cached directory verdict outlived the directory: {warm}"
    );
}

/// A named pipe in the identifier claim store, where a claim file belongs.
/// `check` and `new` both read the store, and a reader that opened the pipe
/// would wait on it for ever. The store reads the entry as a claim that names
/// nobody, and `check` reports it at the claim's path, so the entry is not
/// dropped in silence (#1366).
#[test]
fn check_and_new_finish_when_a_claim_file_is_a_named_pipe() {
    let root = Root::shaped("claim-pipe", |at| {
        let scheme = at.join(".headwater/ids/decision_id");
        std::fs::create_dir_all(&scheme).expect("the scheme directory is made");
        let fifo = std::process::Command::new("mkfifo")
            .arg(scheme.join("HW-DR-9990"))
            .status()
            .expect("mkfifo runs");
        assert!(fifo.success(), "the named pipe is made");
    });
    let (out, _) = under_deadline(
        &root,
        &["check", "--no-cache"],
        "check opened the named pipe in the claim store, and waited on it",
    );
    assert!(
        flat(&out).contains(".headwater/ids/decision_id/HW-DR-9990"),
        "check names the pipe in the claim store: {out}"
    );
    under_deadline(
        &root,
        &["new", "decision", "--title", "After the pipe"],
        "new opened the named pipe in the claim store, and waited on it",
    );
}
