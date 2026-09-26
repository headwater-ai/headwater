// SPDX-License-Identifier: Apache-2.0
//! A stream that cannot be written ends every verb with status 1, and never
//! with the 101 of a panic.
//!
//! `docs/interfaces/headwater-check.md` stated this for `check` first, and
//! `tests/json.rs` holds it there. [#1157](https://github.com/headwater-ai/headwater/issues/1157)
//! found every other verb still writing through `println!`, which panics when
//! the write fails. A caller then read Rust's own `panicked` message and a
//! status it cannot tell from a defect in this engine.
//!
//! # Why the table is over every verb
//!
//! One verb was fixed and the rest were left, so a test over one or two verbs
//! passes while the others still panic. The table carries a row for every name
//! in [`headwater_verbs::VERBS`], and the first assertion holds that: a verb
//! added with no row fails here by name.
//!
//! `/dev/full` is the disk that is full, and it is on every Linux host.

#![cfg(target_os = "linux")]

mod common;

use common::{repository, Root};
use std::collections::BTreeSet;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Where a row runs.
#[derive(Clone, Copy)]
enum At {
    /// This repository, for a row that names a document only it carries.
    Repository,
    /// A scratch root shared by the other rows that read a corpus. It is
    /// smaller than this repository, and a row that writes writes there.
    Scratch,
    /// A fresh directory with one Markdown file in it, made for each run,
    /// for `init`, whose first run changes what the second one finds.
    Fresh,
    /// No corpus at all.
    Nowhere,
}

/// One command line of the table.
struct Row {
    /// The verb the row stands for, or `None` for a flag that runs no verb.
    verb: Option<&'static str>,
    at: At,
    args: &'static [&'static str],
    /// What the run reads on standard input, when it reads anything.
    input: Option<&'static str>,
}

const fn row(verb: &'static str, at: At, args: &'static [&'static str]) -> Row {
    Row {
        verb: Some(verb),
        at,
        args,
        input: None,
    }
}

/// Every verb, and at least one form of each of `help`, `explain`,
/// `completions`, `derived` and `route` that writes to standard output, so
/// the half of the table that fills standard output is not only refusals.
const ROWS: &[Row] = &[
    row("check", At::Scratch, &["check"]),
    row("change", At::Repository, &["change"]),
    row("gate", At::Repository, &["gate"]),
    row("conformance", At::Scratch, &["conformance"]),
    row("derived", At::Scratch, &["derived"]),
    row("merge-driver", At::Repository, &["merge-driver"]),
    row("route", At::Repository, &["route", "write", "a", "decision"]),
    row(
        "route",
        At::Repository,
        &["route", "--json", "write", "a", "decision"],
    ),
    row(
        "neighbors",
        At::Repository,
        &["neighbors", "write", "a", "decision"],
    ),
    row("explain", At::Repository, &["explain", "HW-DR-0049"]),
    row("query", At::Repository, &["query", "kind"]),
    row("capture", At::Scratch, &["capture"]),
    Row {
        verb: Some("mcp"),
        at: At::Repository,
        args: &["mcp"],
        input: Some("{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{}}\n"),
    },
    row("new", At::Scratch, &["new"]),
    row("infer", At::Scratch, &["infer"]),
    row("generate", At::Scratch, &["generate", "--check"]),
    row("import", At::Scratch, &["import"]),
    row("export", At::Scratch, &["export"]),
    row("sweep", At::Repository, &["sweep"]),
    row("sweep", At::Scratch, &["sweep", "plan"]),
    row("probe", At::Repository, &["probe"]),
    row("init", At::Fresh, &["init"]),
    row("taxonomy", At::Scratch, &["taxonomy", "validate"]),
    Row {
        verb: Some("json"),
        at: At::Nowhere,
        args: &["json", "field", "a"],
        input: Some("{\"a\":1}\n"),
    },
    row("help", At::Nowhere, &["help"]),
    row("help", At::Nowhere, &["help", "explain"]),
    row("completions", At::Nowhere, &["completions", "zsh"]),
    row("completions", At::Nowhere, &["completions", "bash"]),
    Row {
        verb: None,
        at: At::Nowhere,
        args: &["--help"],
        input: None,
    },
    Row {
        verb: None,
        at: At::Nowhere,
        args: &["--version"],
        input: None,
    },
];

/// Which of the two streams a run points at `/dev/full`.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Full {
    Neither,
    Stdout,
    Stderr,
}

struct Ran {
    code: Option<i32>,
    out: Vec<u8>,
    err: Vec<u8>,
}

fn full() -> std::fs::File {
    std::fs::File::options()
        .write(true)
        .open("/dev/full")
        .expect("/dev/full opens")
}

/// A fresh directory for `init`, keyed on a counter as well as the process,
/// because cargo runs the cases of one target as threads of one process.
fn fresh() -> PathBuf {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let at = std::env::temp_dir().join(format!(
        "headwater-unwritable-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = std::fs::remove_dir_all(&at);
    std::fs::create_dir_all(at.join("docs")).expect("the fresh root is made");
    std::fs::write(at.join("docs/a.md"), "# A\n\nOne paragraph.\n").expect("the document writes");
    at
}

fn run(row: &Row, root: Option<&Path>, which: Full) -> Ran {
    let mut command = Command::new(env!("CARGO_BIN_EXE_headwater"));
    command.args(row.args);
    if let Some(root) = root {
        command.arg("--root").arg(root);
    }
    command.stdin(match row.input {
        Some(_) => Stdio::piped(),
        None => Stdio::null(),
    });
    match which {
        Full::Neither => command.stdout(Stdio::piped()).stderr(Stdio::piped()),
        Full::Stdout => command.stdout(full()).stderr(Stdio::piped()),
        Full::Stderr => command.stdout(Stdio::piped()).stderr(full()),
    };
    let mut child = command.spawn().expect("the binary runs");
    if let Some(input) = row.input {
        let mut stdin = child.stdin.take().expect("standard input is piped");
        // A verb that failed before it read its input closes the pipe, and
        // what it did then is what the row asserts, so this write may fail.
        let _ = stdin.write_all(input.as_bytes());
    }
    let output = child.wait_with_output().expect("the binary finishes");
    Ran {
        code: output.status.code(),
        out: output.stdout,
        err: output.stderr,
    }
}

#[test]
fn every_verb_exits_1_and_never_101_when_a_stream_it_writes_is_full() {
    let named: BTreeSet<&str> = ROWS.iter().filter_map(|row| row.verb).collect();
    let carried: BTreeSet<&str> = headwater_verbs::VERBS
        .iter()
        .map(|verb| verb.name)
        .collect();
    assert_eq!(
        named, carried,
        "every verb this binary carries has a row in this table, and no row names a verb it does not carry"
    );

    let scratch = Root::new("unwritable");
    let repository = repository();
    let mut wrong = Vec::new();
    let mut filled_stdout = 0;
    let mut made = Vec::new();

    for row in ROWS {
        let label = row.args.join(" ");
        let mut root = || -> Option<PathBuf> {
            match row.at {
                At::Repository => Some(repository.clone()),
                At::Scratch => Some(scratch.at.clone()),
                At::Fresh => {
                    let at = fresh();
                    made.push(at.clone());
                    Some(at)
                }
                At::Nowhere => None,
            }
        };
        let control = run(row, root().as_deref(), Full::Neither);
        assert_ne!(
            control.code,
            Some(101),
            "`{label}` panics with both streams writable: {}",
            String::from_utf8_lossy(&control.err)
        );

        for which in [Full::Stdout, Full::Stderr] {
            let ran = run(row, root().as_deref(), which);
            let said = String::from_utf8_lossy(match which {
                Full::Stdout => &ran.err,
                _ => &ran.out,
            })
            .into_owned();
            let wrote = match which {
                Full::Stdout => !control.out.is_empty(),
                _ => !control.err.is_empty(),
            };
            let mut fault = Vec::new();
            if ran.code == Some(101) {
                fault.push("exited 101".to_string());
            }
            if wrote && ran.code != Some(1) {
                fault.push(format!("exited {:?} and not 1", ran.code));
            }
            if said.contains("panicked") {
                fault.push("printed `panicked`".to_string());
            }
            if which == Full::Stdout && wrote {
                filled_stdout += 1;
                if !said.contains("standard output") {
                    fault.push("did not name standard output on standard error".to_string());
                }
            }
            if !fault.is_empty() {
                wrong.push(format!(
                    "`{label}` with {which:?} full: {}. It said: {}",
                    fault.join(", "),
                    said.trim_end()
                ));
            }
        }
    }
    for at in made {
        let _ = std::fs::remove_dir_all(at);
    }

    assert!(
        wrong.is_empty(),
        "{} of {} runs met a full stream wrongly:\n{}",
        wrong.len(),
        ROWS.len() * 2,
        wrong.join("\n")
    );
    assert!(
        filled_stdout >= 5,
        "the table filled standard output on a run that writes there {filled_stdout} times, \
         and help, explain, completions, derived and route each owe one"
    );
}

/// A reader that stops early closes the pipe, and the next write fails with
/// `EPIPE`, because Rust ignores `SIGPIPE`. That is the same failure as a full
/// disk, and it ends the same way.
///
/// Each command writes more than the 64 KiB a Linux pipe buffers, measured on
/// 2026-09-26: `completions zsh` wrote 131,728 bytes and `sweep plan` over
/// this repository wrote 222,208.
#[test]
fn a_reader_that_closes_early_does_not_make_a_verb_panic() {
    let repository = repository();
    let cases: [(&str, Vec<&std::ffi::OsStr>); 2] = [
        (
            "completions zsh",
            vec!["completions".as_ref(), "zsh".as_ref()],
        ),
        (
            "sweep plan",
            vec![
                "sweep".as_ref(),
                "plan".as_ref(),
                "--root".as_ref(),
                repository.as_os_str(),
            ],
        ),
    ];
    for (label, args) in cases {
        let mut child = Command::new(env!("CARGO_BIN_EXE_headwater"))
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("the binary runs");
        let mut stdout = child.stdout.take().expect("standard output is piped");
        let mut first = [0u8; 16];
        stdout
            .read_exact(&mut first)
            .expect("the verb writes its first bytes");
        drop(stdout);
        let output = child.wait_with_output().expect("the binary finishes");
        let said = String::from_utf8_lossy(&output.stderr).into_owned();
        assert_eq!(
            output.status.code(),
            Some(1),
            "`{label}` read by a reader that closed early exits 1: {said}"
        );
        assert!(
            !said.contains("panicked"),
            "`{label}` did not panic: {said}"
        );
        assert!(
            said.contains("standard output"),
            "`{label}` names standard output on standard error: {said}"
        );
    }
}
