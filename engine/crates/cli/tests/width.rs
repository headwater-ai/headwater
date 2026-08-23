// SPDX-License-Identifier: Apache-2.0
//! How wide the help is, what decides it, and the escape byte that never comes.
//!
//! # Two clauses of [#321](https://github.com/headwater-ai/headwater/issues/321)
//!
//! Clause 12 asks that the default output be laid out at a fixed 80 columns
//! whether or not a terminal is attached, that `--wide` be the only reader of
//! `COLUMNS`, held to `[80, 120]`, and that it be refused alongside a machine
//! format. Clause 11 asks that no escape byte reach a caller, and it asks for
//! the file half to be asserted **independently of the terminal half** — a run
//! that writes a file consults no terminal, so "the terminal check passed" is
//! evidence about a different question.
//!
//! # Why the surface is walked rather than listed
//!
//! The command lines come off `headwater_cli::command`, which is the tree the
//! binary parses with. A list written here would go stale the moment a verb
//! arrived, and the verb it missed would be the one nobody laid out.
//!
//! # Why `COLUMNS` is cleared on every run
//!
//! `cargo test` inherits the environment of whoever started it, and a shell
//! that exports `COLUMNS` would otherwise decide what these cases measure. Each
//! case states the environment it wants, and the default cases state that
//! `COLUMNS` is absent.

use headwater_cli::paint::{WIDEST, WIDTH};
use std::path::{Path, PathBuf};
use std::process::Command as Process;

/// The repository this test tree sits in.
fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

struct Ran {
    code: Option<i32>,
    out: Vec<u8>,
    err: Vec<u8>,
}

/// One invocation, with `COLUMNS` set as the case asks and the streams apart.
fn ran(arguments: &[&str], columns: Option<&str>) -> Ran {
    let at = std::env::temp_dir().join(format!("headwater-width-{}", std::process::id()));
    std::fs::create_dir_all(&at).expect("the directory is there");
    let mut process = Process::new(env!("CARGO_BIN_EXE_headwater"));
    process.args(arguments).current_dir(&at).env_remove("COLUMNS");
    if let Some(value) = columns {
        process.env("COLUMNS", value);
    }
    let output = process.output().expect("the binary runs");
    Ran {
        code: output.status.code(),
        out: output.stdout,
        err: output.stderr,
    }
}

/// Every command line the tree answers to, root first.
fn command_lines() -> Vec<Vec<String>> {
    fn walk(command: &clap::Command, at: Vec<String>, into: &mut Vec<Vec<String>>) {
        into.push(at.clone());
        for inner in command.get_subcommands() {
            let mut next = at.clone();
            next.push(inner.get_name().to_string());
            walk(inner, next, into);
        }
    }
    let mut command = headwater_cli::command();
    command.build();
    let mut lines = Vec::new();
    walk(&command, Vec::new(), &mut lines);
    lines
}

/// Every `--help` the surface admits, as `(what was typed, what it printed)`.
fn surface(columns: Option<&str>, extra: &[&str]) -> Vec<(String, String)> {
    command_lines()
        .into_iter()
        .map(|line| {
            let mut arguments: Vec<&str> = line.iter().map(String::as_str).collect();
            arguments.push("--help");
            arguments.extend_from_slice(extra);
            let ran = ran(&arguments, columns);
            let typed = format!("headwater {}", arguments.join(" "));
            assert_eq!(ran.code, Some(0), "{typed} exits 0");
            assert!(ran.err.is_empty(), "{typed} writes nothing to standard error");
            (typed, String::from_utf8(ran.out).expect("the help is text"))
        })
        .collect()
}

/// The widest line of a surface, with the command line and the text.
fn widest(surface: &[(String, String)]) -> (usize, String, String) {
    surface
        .iter()
        .flat_map(|(typed, out)| {
            out.lines()
                .map(move |line| (line.chars().count(), typed.clone(), line.to_string()))
        })
        .max_by_key(|(width, _, _)| *width)
        .expect("the surface has a line")
}

/// **Clause 12, the bar.** No line of the help is wider than 80 columns.
///
/// It reports how many lines are over and the widest one, so a regression names
/// itself rather than saying that some line somewhere is long. The state this
/// replaced was 138 lines of 416 over 80, with a widest of 1,126.
#[test]
fn no_line_of_the_help_surface_is_wider_than_eighty_columns() {
    let surface = surface(None, &[]);
    let mut lines = 0;
    let mut over: Vec<String> = Vec::new();
    for (typed, out) in &surface {
        for (at, line) in out.lines().enumerate() {
            lines += 1;
            let width = line.chars().count();
            if width > WIDTH {
                over.push(format!("{typed}, line {}, {width} columns", at + 1));
            }
        }
    }
    let (widest, where_, text) = widest(&surface);
    assert!(
        over.is_empty(),
        "{} of {lines} lines over {WIDTH} columns across {} command lines. \
         The widest is {widest} columns, in `{where_}`:\n{text}\n\nEvery one:\n{}",
        over.len(),
        surface.len(),
        over.join("\n")
    );
    assert!(widest <= WIDTH, "the widest line is {widest} columns");
}

/// **Clause 12, the invariant.** The layout is not a function of the terminal.
///
/// A run whose output is a pipe, a run whose output is a file and a run under a
/// terminal write the same bytes, because nothing in the path asks. `clap`'s
/// `dimensions()` is `(None, None)` without the `wrap_help` feature and this
/// workspace does not take it, so `terminal_size` is not in the lock and no
/// crate here can measure a terminal at all. What is left for a case to hold is
/// the one input that could still reach the layout, which is `COLUMNS`.
#[test]
fn a_run_that_states_columns_and_one_that_does_not_write_the_same_bytes() {
    for line in [vec!["--help"], vec!["check", "--help"], vec!["help", "check"]] {
        let bare = ran(&line, None);
        for absurd in ["1", "40", "500", "100000", "not a number"] {
            let stated = ran(&line, Some(absurd));
            assert_eq!(
                bare.out,
                stated.out,
                "`headwater {}` under COLUMNS={absurd} writes what it writes with none",
                line.join(" ")
            );
            assert_eq!(bare.code, stated.code);
            assert!(stated.err.is_empty());
        }
    }
}

/// **Clause 12, `--wide`.** It reads `COLUMNS` and holds it to `[80, 120]`.
///
/// The clamp is asserted by identity rather than by a width, because a width is
/// a property of the longest paragraph and an identity is a property of the
/// clamp: 40 is the layout every other run gets, 500 is the layout 120 gets,
/// and 100 is neither of them.
#[test]
fn the_width_a_caller_asks_for_is_read_and_held_to_the_band() {
    let line = ["check", "--help"];
    let ordinary = ran(&line, None).out;
    let wide = |columns: &str| {
        let mut arguments = line.to_vec();
        arguments.push("--wide");
        ran(&arguments, Some(columns)).out
    };

    let narrow = wide("40");
    assert_eq!(narrow, ordinary, "a width under {WIDTH} is {WIDTH}");
    assert_eq!(wide("80"), ordinary);

    let widest_asked = wide("500");
    assert_eq!(widest_asked, wide("120"), "a width over {WIDEST} is {WIDEST}");
    assert_ne!(widest_asked, ordinary, "{WIDEST} is not {WIDTH}");

    let between = wide("100");
    assert_ne!(between, ordinary);
    assert_ne!(between, widest_asked);

    for (asked, out) in [
        (WIDTH, &narrow),
        (100, &between),
        (WIDEST, &widest_asked),
    ] {
        let text = String::from_utf8(out.clone()).expect("the help is text");
        for line in text.lines() {
            assert!(
                line.chars().count() <= asked,
                "at {asked} columns this line is {}: {line}",
                line.chars().count()
            );
        }
    }
}

/// **Clause 12, the whole surface at the widest a caller may ask for.**
#[test]
fn the_widest_a_caller_may_ask_for_lays_the_whole_surface_out_inside_it() {
    let surface = surface(Some("500"), &["--wide"]);
    let (widest, where_, text) = widest(&surface);
    assert!(
        widest <= WIDEST,
        "`{where_}` has a line of {widest} columns at the widest band:\n{text}"
    );
    assert!(widest > WIDTH, "the widest band widened nothing: {widest}");
}

/// **Clause 12, the refusal.** `--wide` against a machine format is refused.
///
/// Not accepted and ignored. The message names the flag and the format, and the
/// status is exactly 1, because a 2 would make the exit table of
/// `docs/interfaces/headwater-check.md` false.
#[test]
fn a_width_asked_for_a_format_nothing_lays_out_is_refused() {
    for (verb, format) in [
        ("check", "json"),
        ("check", "sarif"),
        ("check", "markdown"),
        ("capture", "json"),
        ("export", "json"),
    ] {
        let ran = ran(&[verb, "--wide", "--format", format], None);
        assert_eq!(
            ran.code,
            Some(1),
            "`headwater {verb} --wide --format {format}` is refused with 1"
        );
        let said = String::from_utf8_lossy(&ran.err).into_owned();
        assert!(said.contains("--wide"), "the refusal names the flag: {said}");
        assert!(
            said.contains(format),
            "the refusal names the format: {said}"
        );
        assert!(ran.out.is_empty(), "a refusal writes no artifact");
    }
}

/// The one format that is laid out is the one `--wide` is not refused with.
#[test]
fn a_width_asked_for_the_format_a_reader_reads_is_not_refused() {
    let ran = ran(&["check", "--wide", "--format", "text", "--help"], None);
    assert_eq!(ran.code, Some(0));
    let refused = ran(&["check", "--wide", "--format", "text", "--root", "/nonexistent"], None);
    let said = String::from_utf8_lossy(&refused.err).into_owned();
    assert!(
        !said.contains("`--wide`"),
        "`--format text` is laid out, so `--wide` is not the objection: {said}"
    );
}

/// **Clause 11, the terminal half.** No escape byte reaches a caller.
///
/// The four conditions the clause names are asserted here over the help, which
/// is the text a terminal would be the reason to color. `NO_COLOR` and
/// `TERM=dumb` are stated rather than assumed, because a case that only ran
/// under the harness's own environment would be measuring the harness.
#[test]
fn no_escape_byte_reaches_a_caller_under_any_of_the_four_conditions() {
    let cases: Vec<(&str, Vec<(&str, &str)>, Vec<&str>)> = vec![
        ("plain", vec![], vec!["--help"]),
        ("NO_COLOR=1", vec![("NO_COLOR", "1")], vec!["--help"]),
        ("NO_COLOR=", vec![("NO_COLOR", "")], vec!["--help"]),
        ("NO_COLOR=0", vec![("NO_COLOR", "0")], vec!["--help"]),
        ("TERM=dumb", vec![("TERM", "dumb")], vec!["--help"]),
        ("--no-color", vec![], vec!["--no-color", "--help"]),
        ("--no-color deep", vec![], vec!["check", "--no-color", "--help"]),
        ("CLICOLOR_FORCE", vec![("CLICOLOR_FORCE", "1")], vec!["--help"]),
    ];
    for (label, environment, arguments) in cases {
        let at = std::env::temp_dir().join(format!("headwater-color-{}", std::process::id()));
        std::fs::create_dir_all(&at).expect("the directory is there");
        let mut process = Process::new(env!("CARGO_BIN_EXE_headwater"));
        process.args(&arguments).current_dir(&at).env_remove("COLUMNS");
        for (name, value) in environment {
            process.env(name, value);
        }
        let output = process.output().expect("the binary runs");
        assert_eq!(output.status.code(), Some(0), "{label} exits 0");
        assert!(
            !contains_escape(&output.stdout),
            "{label} writes an escape byte to standard output"
        );
        assert!(
            !contains_escape(&output.stderr),
            "{label} writes an escape byte to standard error"
        );
    }
}

/// **Clause 11, the machine-format half, held on its own.**
///
/// This asserts nothing about a terminal and reads no result of the case above.
/// A writer of an artifact never asked what it was attached to, so a run of
/// that writer is the only evidence about it.
#[test]
fn no_escape_byte_reaches_a_machine_format() {
    // The case runs from a temporary directory, so `--root` names the
    // repository this test tree sits in rather than the corpus of the moment.
    let root = repository();
    let root = root.to_str().expect("the path is text").to_string();
    for format in ["json", "sarif", "markdown"] {
        let ran = ran(&["check", "--format", format, "--root", &root], None);
        assert_eq!(ran.code, Some(0), "`--format {format}` ran");
        assert!(!ran.out.is_empty(), "`--format {format}` wrote a report");
        assert!(
            !contains_escape(&ran.out),
            "`--format {format}` writes an escape byte to standard output"
        );
        assert!(
            !contains_escape(&ran.err),
            "`--format {format}` writes an escape byte to standard error"
        );
    }
}

/// **Clause 11, the written-file half, held on its own.**
///
/// A file written by `--read-set` or `--register` is bytes nobody was looking
/// at when they were produced, so no terminal reading could have reached them
/// and no terminal case is evidence about them. This runs the writer and reads
/// the file back.
#[test]
fn no_escape_byte_reaches_a_file_this_binary_writes() {
    let root = repository();
    let root = root.to_str().expect("the path is text").to_string();
    let at = std::env::temp_dir().join(format!("headwater-artifacts-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&at);
    std::fs::create_dir_all(&at).expect("the directory is there");

    for name in ["read-set", "register"] {
        let path = at.join(format!("{name}.json"));
        let written = path.to_str().expect("the path is text").to_string();
        let ran = ran(
            &["check", "--root", &root, &format!("--{name}"), &written],
            None,
        );
        assert_eq!(ran.code, Some(0), "`--{name}` ran");
        let bytes = std::fs::read(&path).unwrap_or_else(|_| panic!("`--{name}` wrote {written}"));
        assert!(!bytes.is_empty(), "`--{name}` wrote something");
        assert!(
            !contains_escape(&bytes),
            "the file `--{name}` wrote carries an escape byte"
        );
    }
}

/// The two bytes that open every ANSI colour sequence.
fn contains_escape(bytes: &[u8]) -> bool {
    bytes.windows(2).any(|pair| pair == [0x1b, b'['])
}
