// SPDX-License-Identifier: Apache-2.0
//! `headwater check` finishes when a `governs` edge reaches a named pipe
//! (#1269).
//!
//! A named pipe with no writer blocks a process that opens it to read, and it
//! never returns. The walk reports a pipe as a file, so an edge that names one,
//! or a wildcard that matches one, hands it to the reader that digests what an
//! edge governs. That reader takes regular files alone, so `check` never opens
//! the pipe. Each case runs the verb under a deadline and fails on it, because
//! a check that opened the pipe does not end.

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
    loop {
        if child.try_wait().expect("the child is there").is_some() {
            break;
        }
        if Instant::now() > deadline {
            child.kill().expect("the child stops");
            child.wait().expect("the child is reaped");
            panic!("{hung}");
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    (
        out.join().expect("stdout drains"),
        err.join().expect("stderr drains"),
    )
}

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
}

#[test]
fn check_no_cache_finishes_when_a_governs_wildcard_matches_a_named_pipe() {
    let root = root("check-governed-pipe-wildcard", "tools/**");
    let (out, err) = under_deadline(
        &root,
        &["check", "--no-cache"],
        "check opened the named pipe a governs wildcard matches, and waited on it",
    );
    assert!(!out.contains("revision"), "no revision finding: {out}");
    assert!(!err.contains("revision"), "no revision error: {err}");
}
