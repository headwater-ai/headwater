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
             tools/pipe`, and `tools/pipe` names no regular file, only a named pipe, a socket \
             or a device, which the source tree never opens, so this edge never goes suspect \
             when what it governs changes"
        ),
        "the edge that can never age is reported at info, in the full sentence (#1366): {out}"
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
    let (status, out, err) = ended(
        &root,
        &["show", "docs/x.md"],
        "show opened the named pipe at docs/x.md, and waited on it",
    );
    assert_eq!(status.code(), Some(1), "show refuses: {out}{err}");
    assert!(out.is_empty(), "show prints nothing of a named pipe: {out}");
    assert_eq!(
        flat(&err),
        "headwater: `docs/x.md` is a named pipe, a socket or a device, which the census never \
         opens, so `show` prints nothing",
    );
}

/// `show` refuses every row `explain` refuses, in `explain`'s sentence with
/// the verb changed (`docs/interfaces/headwater-show.md`, #1366). A symlink
/// row, whose link target the sentence names, and a directory the walk could
/// not read, which `show` once called a document that could not be read.
#[test]
fn show_and_explain_refuse_an_unwalkable_row_in_one_sentence() {
    use std::os::unix::fs::PermissionsExt;

    let root = Root::shaped("show-explain-unwalkable", |at| {
        std::fs::write(at.join("docs/real.md"), "# Real\n").expect("the file writes");
        std::os::unix::fs::symlink("real.md", at.join("docs/link.md")).expect("the link is made");
        std::fs::create_dir_all(at.join("docs/locked")).expect("the directory is made");
        std::fs::write(at.join("docs/locked/y.md"), "# Y\n").expect("the file writes");
    });
    let locked = root.at.join("docs/locked");
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000))
        .expect("the mode is set");
    let unreadable = std::fs::read_dir(&locked).is_err();

    let mut rows = vec!["docs/link.md"];
    // A process that a mode-000 directory does not stop, such as root, walks
    // it, so the directory is no row of its own there.
    if unreadable {
        rows.push("docs/locked");
    }
    for row in rows {
        let (explain_status, _, explain) = ended(&root, &["explain", row], "explain did not end");
        let (show_status, out, show) = ended(&root, &["show", row], "show did not end");
        assert_eq!(explain_status.code(), Some(1), "explain refuses {row}");
        assert_eq!(show_status.code(), Some(1), "show refuses {row}");
        assert!(out.is_empty(), "show prints nothing of {row}: {out}");
        let explain = flat(&explain);
        assert!(explain.contains("so `explain` prints nothing"), "{explain}");
        assert_eq!(
            flat(&show),
            explain.replace("so `explain` prints nothing", "so `show` prints nothing"),
            "show refuses {row} in explain's sentence"
        );
    }
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755))
        .expect("the mode is restored");
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
    assert!(
        flat(&out).contains("is a named pipe, a socket or a device, so it names no document"),
        "check says what the entry is: {out}"
    );
    assert!(
        !flat(&out).contains("write the path of the document whose"),
        "check never asks for a write into the pipe: {out}"
    );
    under_deadline(
        &root,
        &["new", "decision", "--title", "After the pipe"],
        "new opened the named pipe in the claim store, and waited on it",
    );
}

/// `sweep plan` reads the text of every document it briefs, and a named pipe
/// at a document path is a row of the census that it must not open. It ends
/// under the deadline and writes its briefing (#1366). `neighbors` over the
/// same root needs the pinned model, so its case below runs only where
/// `HEADWATER_MODEL_DIR` names the fetched files.
#[test]
fn sweep_plan_finishes_when_a_named_pipe_takes_a_document_path() {
    let root = document_pipe("neighbors-sweep-document-pipe");
    let (plan, _) = under_deadline(
        &root,
        &["sweep", "plan"],
        "sweep plan opened the named pipe at docs/x.md, and waited on it",
    );
    assert!(
        plan.contains("## The slice"),
        "sweep plan wrote its briefing: {plan}"
    );
}

/// The committed embedding pin, copied into a stub root so that `neighbors`
/// reads past `.headwater/embedding.yml` to the model files it names.
fn pin_into(at: &std::path::Path) {
    let pin =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../.headwater/embedding.yml");
    std::fs::create_dir_all(at.join(".headwater")).expect("the directory is made");
    std::fs::copy(pin, at.join(".headwater/embedding.yml")).expect("the pin copies");
}

/// Make a named pipe at `at`.
fn fifo(at: &std::path::Path) {
    let made = std::process::Command::new("mkfifo")
        .arg(at)
        .status()
        .expect("mkfifo runs");
    assert!(made.success(), "the named pipe is made");
}

/// `neighbors` reads the model file itself, and a named pipe there is a model
/// file that is missing, as `docs/interfaces/headwater-neighbors.md` says. The
/// verb refuses before it opens the pipe, so it needs no model bytes, and it
/// is the case `.claude/hooks/intent.sh` would hit on every prompt (#1366).
#[test]
fn neighbors_refuses_a_named_pipe_at_the_model_file() {
    let root = Root::shaped("neighbors-model-pipe", |at| {
        pin_into(at);
        std::fs::create_dir_all(at.join("model")).expect("the model directory is made");
        std::fs::write(at.join("model/vocab.txt"), "[PAD]\n").expect("the vocabulary writes");
        fifo(&at.join("model/model.onnx"));
    });
    let model = root.at.join("model");
    let model = model.to_str().expect("the path is UTF-8");
    let (status, out, err) = ended(
        &root,
        &["neighbors", "--model", model, "a", "task"],
        "neighbors opened the named pipe at model.onnx, and waited on it",
    );
    assert_eq!(status.code(), Some(1), "neighbors refuses: {err}");
    assert!(out.is_empty(), "neighbors prints nothing: {out}");
    assert!(flat(&err).contains("model.onnx"), "{err}");
    assert!(flat(&err).contains("is not a regular file"), "{err}");
}

/// A named pipe at `.headwater/embedding.yml` is a pin that could not be read,
/// and `neighbors` refuses in one sentence that names it (#1366).
#[test]
fn neighbors_refuses_a_named_pipe_at_the_pin() {
    let root = Root::shaped("neighbors-pin-pipe", |at| {
        std::fs::create_dir_all(at.join(".headwater")).expect("the directory is made");
        fifo(&at.join(".headwater/embedding.yml"));
    });
    let (status, out, err) = ended(
        &root,
        &["neighbors", "a", "task"],
        "neighbors opened the named pipe at .headwater/embedding.yml, and waited on it",
    );
    assert_eq!(status.code(), Some(1), "neighbors refuses: {err}");
    assert!(out.is_empty(), "neighbors prints nothing: {out}");
    assert!(flat(&err).contains(".headwater/embedding.yml"), "{err}");
    assert!(flat(&err).contains("is not a regular file"), "{err}");
}

/// The file-type test follows a link, so a pin that is a link to a regular
/// file is read, and the run goes on to refuse the missing model file rather
/// than the pin (#1366, verify round 1).
#[test]
fn neighbors_reads_a_pin_that_is_a_link_to_a_regular_file() {
    let root = Root::shaped("neighbors-linked-pin", |at| {
        let pin = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../.headwater/embedding.yml");
        std::fs::copy(pin, at.join("pin.yml")).expect("the pin copies");
        std::fs::create_dir_all(at.join(".headwater")).expect("the directory is made");
        std::os::unix::fs::symlink("../pin.yml", at.join(".headwater/embedding.yml"))
            .expect("the link is made");
    });
    let (status, out, err) = ended(
        &root,
        &["neighbors", "a", "task"],
        "neighbors did not end on a linked pin",
    );
    assert_eq!(status.code(), Some(1), "neighbors refuses: {err}");
    assert!(out.is_empty(), "neighbors prints nothing: {out}");
    assert!(flat(&err).contains("model.onnx"), "{err}");
    assert!(!flat(&err).contains("embedding.yml"), "{err}");
}

/// `neighbors` ranks the summaries the census parsed, and a named pipe at a
/// document path is a row that holds none, so the verb ends and never ranks
/// it (#1366). The model files are 90MB and fetched, so a run with no
/// `HEADWATER_MODEL_DIR` says it skipped rather than passing.
#[test]
fn neighbors_finishes_when_a_named_pipe_takes_a_document_path() {
    let Some(model) = std::env::var_os("HEADWATER_MODEL_DIR") else {
        eprintln!("skipped: set HEADWATER_MODEL_DIR to the fetched model files");
        return;
    };
    let root = Root::shaped("neighbors-document-pipe", |at| {
        pin_into(at);
        fifo(&at.join("docs/x.md"));
    });
    let model = model.to_str().expect("the path is UTF-8");
    let (status, out, err) = ended(
        &root,
        &["neighbors", "--model", model, "a", "task"],
        "neighbors opened the named pipe at docs/x.md, and waited on it",
    );
    assert_eq!(status.code(), Some(0), "neighbors ends: {err}");
    assert!(
        out.contains("summarized documents"),
        "neighbors ranked: {out}"
    );
    assert!(
        !out.contains("docs/x.md"),
        "the pipe is never ranked: {out}"
    );
}

/// The walk makes no row under a directory that is a symlink, so a path
/// through one holds no document, and `show` refuses it in the sentence
/// `explain` writes for such a path, as `docs/interfaces/headwater-show.md`
/// says (#1366, verify round 1).
#[test]
fn show_refuses_a_path_through_a_linked_directory_as_a_path_with_no_document() {
    let root = Root::shaped("show-linked-directory", |at| {
        std::os::unix::fs::symlink("decisions", at.join("docs/linkdir")).expect("the link is made");
    });
    let target = "docs/linkdir/0001-the-warrant-a-person-set.md";
    assert!(
        root.at.join(target).is_file(),
        "the link reaches a real document"
    );
    let (explain_status, _, explain) = ended(&root, &["explain", target], "explain did not end");
    let (show_status, out, show) = ended(&root, &["show", target], "show did not end");
    assert_eq!(explain_status.code(), Some(1), "explain refuses: {explain}");
    assert_eq!(show_status.code(), Some(1), "show refuses: {show}");
    assert!(out.is_empty(), "show prints nothing: {out}");
    assert!(
        flat(&show).contains("with no document written there yet"),
        "{show}"
    );
    assert_eq!(flat(&show), flat(&explain), "one sentence for both verbs");
}

/// A root whose file at `path` under `.headwater/` is replaced by a named pipe.
/// The pipe goes in after [`Root::shaped`] has resolved the root, because that
/// resolve reads the same files with no deadline over it.
fn declaration_pipe(label: &str, path: &str) -> Root {
    // The pin, so that `neighbors` reaches the lock rather than refusing a
    // missing pin first.
    let root = Root::shaped(label, pin_into);
    let at = root.at.join(".headwater").join(path);
    std::fs::remove_file(&at).expect("the file is there to replace");
    fifo(&at);
    root
}

/// Every verb that loads the taxonomy reads `.headwater/taxonomy.lock`, and
/// `taxonomy resolve` reads `.headwater/taxonomy.yml` first. A named pipe at
/// either path is refused in one sentence that names it, before anything
/// opens it, so no verb waits on the pipe for ever (#1366).
#[test]
fn check_show_and_explain_finish_when_a_named_pipe_takes_the_lock_or_the_consumer_declaration() {
    let document = "docs/decisions/0001-the-warrant-a-person-set.md";
    for (label, path) in [
        ("lock-pipe", "taxonomy.lock"),
        ("consumer-pipe", "taxonomy.yml"),
    ] {
        let root = declaration_pipe(label, path);
        let named = format!(".headwater/{path}");
        for args in [
            vec!["check", "--no-cache"],
            vec!["show", document],
            vec!["explain", document],
            vec!["neighbors", "a", "task"],
            vec!["taxonomy", "resolve", "--check"],
            vec!["taxonomy", "resolve"],
        ] {
            let hung = format!(
                "{} opened the named pipe at {named}, and waited on it",
                args.join(" ")
            );
            let (status, out, err) = ended(&root, &args, &hung);
            assert_eq!(status.code(), Some(1), "{args:?} refuses at {named}: {err}");
            assert!(out.is_empty(), "{args:?} prints nothing at {named}: {out}");
            assert!(flat(&err).contains(&named), "{args:?} names {named}: {err}");
            assert!(
                flat(&err).contains("not a regular file"),
                "{args:?} says why at {named}: {err}"
            );
        }
    }
}

/// `taxonomy resolve` reads the overlay and every package file as taxonomy
/// sources, so a named pipe at either is refused in the same sentence, in
/// both modes (#1366).
#[test]
fn taxonomy_resolve_finishes_when_a_named_pipe_takes_the_overlay_or_a_package_source() {
    for (label, path) in [
        ("overlay-pipe", "overlay.yml"),
        ("package-pipe", "packages/headwater-standard/taxonomy.yml"),
    ] {
        let root = declaration_pipe(label, path);
        let named = format!(".headwater/{path}");
        for args in [
            vec!["taxonomy", "resolve", "--check"],
            vec!["taxonomy", "resolve"],
        ] {
            let hung = format!(
                "{} opened the named pipe at {named}, and waited on it",
                args.join(" ")
            );
            let (status, out, err) = ended(&root, &args, &hung);
            assert_eq!(status.code(), Some(1), "{args:?} refuses at {named}: {err}");
            assert!(out.is_empty(), "{args:?} prints nothing at {named}: {out}");
            assert!(flat(&err).contains(&named), "{args:?} names {named}: {err}");
            assert!(
                flat(&err).contains("not a regular file"),
                "{args:?} says why at {named}: {err}"
            );
        }
    }
}

/// The file-type test follows a link, so a lock that is a link to a regular
/// file still reads, and `check` runs to its end (#1366).
#[test]
fn check_reads_a_lock_that_is_a_link_to_a_regular_file() {
    let root = Root::shaped("linked-lock", |_| {});
    let at = &root.at;
    std::fs::rename(at.join(".headwater/taxonomy.lock"), at.join("lock.target"))
        .expect("the lock moves");
    std::os::unix::fs::symlink("../lock.target", at.join(".headwater/taxonomy.lock"))
        .expect("the link is made");
    let (status, _, err) = ended(&root, &["check", "--no-cache"], "check did not end");
    assert_eq!(status.code(), Some(0), "check reads the linked lock: {err}");
}
