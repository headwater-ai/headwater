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
    process
        .args(arguments)
        .current_dir(&at)
        .env_remove("COLUMNS");
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
            assert!(
                ran.err.is_empty(),
                "{typed} writes nothing to standard error"
            );
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
    for line in [
        vec!["--help"],
        vec!["check", "--help"],
        vec!["help", "check"],
    ] {
        let bare = ran(&line, None);
        // The comparison is over the text rather than over the bytes, so that a
        // failure prints the two layouts and not two thousand integers.
        let written = String::from_utf8(bare.out.clone()).expect("the help is text");
        for absurd in ["1", "40", "500", "100000", "not a number"] {
            let stated = ran(&line, Some(absurd));
            assert_eq!(
                written,
                String::from_utf8(stated.out).expect("the help is text"),
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
    // Text rather than bytes, so a failure prints the two layouts.
    let text = |ran: Ran| String::from_utf8(ran.out).expect("the help is text");
    let ordinary = text(ran(&line, None));
    let wide = |columns: &str| {
        let mut arguments = line.to_vec();
        arguments.push("--wide");
        text(ran(&arguments, Some(columns)))
    };

    let narrow = wide("40");
    assert_eq!(narrow, ordinary, "a width under {WIDTH} is {WIDTH}");
    assert_eq!(wide("80"), ordinary);

    let widest_asked = wide("500");
    assert_eq!(
        widest_asked,
        wide("120"),
        "a width over {WIDEST} is {WIDEST}"
    );
    assert_ne!(widest_asked, ordinary, "{WIDEST} is not {WIDTH}");

    let between = wide("100");
    assert_ne!(between, ordinary);
    assert_ne!(between, widest_asked);

    for (asked, out) in [(WIDTH, &narrow), (100, &between), (WIDEST, &widest_asked)] {
        for line in out.lines() {
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

/// **Clause 12, the refusal.** `--wide` on a run that lays nothing out.
///
/// Not accepted and ignored. The status is exactly 1, because a 2 would make
/// the exit table of `docs/interfaces/headwater-check.md` false, and the message
/// names the flag and the reason. Where a machine format was asked for, which is
/// the case clause 12 names, that format is the reason the message gives.
///
/// **Every row states the fragment it expects**, rather than deriving it from a
/// format name. A machine format reaches this binary under two spellings —
/// `--format json` and `--json` — and a table that derived the message from the
/// `--format` value could not carry a row for the second one. That gap is how
/// `check --wide --json` was answered rather than refused for the length of one
/// review.
#[test]
fn a_width_asked_for_a_run_that_lays_nothing_out_is_refused() {
    for (line, says) in [
        (
            vec!["check", "--wide", "--format", "json"],
            "`--format json` writes an artifact",
        ),
        (
            vec!["check", "--wide", "--format", "sarif"],
            "`--format sarif` writes an artifact",
        ),
        (
            vec!["check", "--wide", "--format", "markdown"],
            "`--format markdown` writes an artifact",
        ),
        (
            vec!["capture", "--wide", "--format", "json"],
            "`--format json` writes an artifact",
        ),
        (
            vec!["export", "--wide", "--format", "json"],
            "`--format json` writes an artifact",
        ),
        // `capture --format text` lays nothing out either, so the boundary is
        // not "the format is text" — it is the one report that is laid out,
        // which is `check`'s. Everything else still refuses.
        (
            vec!["capture", "--wide", "--format", "text"],
            "lays nothing out",
        ),
        (vec!["taxonomy", "audit", "--wide"], "lays nothing out"),
        // **The second spelling of one machine format.** `--json` is a boolean
        // and `--format json` is a value, and HW-DR-0033 rules that they reach
        // one target. A predicate that read only `--format` answered
        // `check --wide --json` with the flag doing nothing, which is the defect
        // this whole refusal exists to prevent.
        (
            vec!["check", "--wide", "--json"],
            "`--json` writes an artifact",
        ),
        // Reading `--json` is not a `check`-only rule: every verb that declares
        // the flag names it as the reason, the way `--format json` is named.
        (
            vec!["capture", "--wide", "--json"],
            "`--json` writes an artifact",
        ),
    ] {
        let typed = line.join(" ");
        let ran = ran(&line, None);
        assert_eq!(ran.code, Some(1), "`headwater {typed}` is refused with 1");
        let said = String::from_utf8_lossy(&ran.err).into_owned();
        assert!(
            said.contains("--wide"),
            "the refusal names the flag: {said}"
        );
        assert!(
            said.contains("how wide the help"),
            "the refusal says what the flag does: {said}"
        );
        assert!(
            said.contains(says),
            "`headwater {typed}` should say `{says}`, and said: {said}"
        );
        assert!(ran.out.is_empty(), "a refusal writes no artifact");
    }
}

/// **The two spellings of one machine format reach the same answer.**
///
/// `--json` is documented as "the same artifact `--format json` writes, byte for
/// byte", and HW-DR-0033 rules that two names for one target is a question
/// answered twice. So `--wide` must treat them the same way, and this asserts it
/// by identity rather than by reading two messages.
#[test]
fn both_spellings_of_a_machine_format_answer_a_width_the_same_way() {
    let root = repository();
    let root = root.to_str().expect("the path is text").to_string();
    let one = ran(&["check", "--wide", "--json", "--root", &root], None);
    let two = ran(
        &["check", "--wide", "--format", "json", "--root", &root],
        None,
    );
    assert_eq!(one.code, Some(1), "`--json` beside `--wide` is refused");
    assert_eq!(one.code, two.code, "the two spellings exit the same way");
    assert!(
        one.out.is_empty() && two.out.is_empty(),
        "neither wrote one"
    );
    // Without `--wide`, both still write the same artifact, so the refusal above
    // is about the width flag and not about the format.
    let plain_one = ran(&["check", "--json", "--root", &root], None);
    let plain_two = ran(&["check", "--format", "json", "--root", &root], None);
    assert_eq!(plain_one.code, Some(0));
    assert_eq!(
        plain_one.out, plain_two.out,
        "the two spellings write the same artifact"
    );
}

/// **The other half of the boundary.** The report that is laid out answers it.
///
/// This is the inversion of the two rows the case above used to carry. Before
/// [#340](https://github.com/headwater-ai/headwater/issues/340) the text report
/// was composed at no width and `check --wide` was refused; it is laid out now,
/// so the flag does something and the run is answered. It cannot join
/// `a_width_asked_for_a_run_that_prints_help_is_answered`, which runs with no
/// corpus and would find nothing to lay out.
#[test]
fn a_width_asked_for_the_report_that_is_laid_out_is_answered() {
    let root = repository();
    let root = root.to_str().expect("the path is text").to_string();
    for line in [
        vec!["check", "--wide", "--root", &root],
        vec!["check", "--wide", "--format", "text", "--root", &root],
    ] {
        let typed = line.join(" ");
        let ran = ran(&line, Some("120"));
        assert_eq!(ran.code, Some(0), "`headwater {typed}` is answered");
        assert!(!ran.out.is_empty(), "`headwater {typed}` wrote a report");
        let said = String::from_utf8_lossy(&ran.err).into_owned();
        assert!(
            !said.contains("--wide"),
            "`headwater {typed}` says nothing about the flag: {said}"
        );
    }
}

/// The runs `--wide` does lay out are the runs that print help.
///
/// `clap` answers `--help` before the refusal above can run, and
/// `headwater help <verb>` is a verb of this binary that reaches it and is let
/// through. Both routes are asserted, because they are admitted by different
/// code and only one of them is a check anybody wrote.
#[test]
fn a_width_asked_for_a_run_that_prints_help_is_answered() {
    for line in [
        vec!["--wide", "--help"],
        vec!["check", "--wide", "--help"],
        vec!["check", "--wide", "--format", "text", "--help"],
        vec!["help", "check", "--wide"],
        vec!["--wide", "help", "check"],
        vec!["help", "--wide"],
    ] {
        let typed = line.join(" ");
        let ran = ran(&line, Some("120"));
        assert_eq!(ran.code, Some(0), "`headwater {typed}` is answered");
        assert!(!ran.out.is_empty(), "`headwater {typed}` printed help");
        assert!(
            ran.err.is_empty(),
            "`headwater {typed}` says nothing on standard error"
        );
    }
}

/// **Clause 11, the terminal half.** No escape byte reaches a caller.
///
/// The four conditions the clause names are asserted here over the help, which
/// is the text a terminal would be the reason to color. `NO_COLOR` and
/// `TERM=dumb` are stated rather than assumed, because a case that only ran
/// under the harness's own environment would be measuring the harness.
#[test]
fn no_escape_byte_reaches_a_caller_under_any_of_the_four_conditions() {
    /// A label, the environment the case states, and the command line.
    struct Case(
        &'static str,
        &'static [(&'static str, &'static str)],
        &'static [&'static str],
    );

    let cases = [
        Case("plain", &[], &["--help"]),
        Case("NO_COLOR=1", &[("NO_COLOR", "1")], &["--help"]),
        Case("NO_COLOR=", &[("NO_COLOR", "")], &["--help"]),
        Case("NO_COLOR=0", &[("NO_COLOR", "0")], &["--help"]),
        Case("TERM=dumb", &[("TERM", "dumb")], &["--help"]),
        Case("--no-color", &[], &["--no-color", "--help"]),
        Case("--no-color deep", &[], &["check", "--no-color", "--help"]),
        Case("CLICOLOR_FORCE", &[("CLICOLOR_FORCE", "1")], &["--help"]),
        // Every page of the help family, and `CLICOLOR_FORCE` over each shape
        // of it. The root screen is one template, a verb page is `clap`'s own
        // `{options}` renderer, a verb with second words is a third template,
        // and `headwater help <verb>` reaches the second of those through
        // `print_help_for` rather than through the parse. A palette wired at
        // the tree keeps all four plain or breaks all four, and only running
        // each says which.
        Case("verb page", &[], &["check", "--help"]),
        Case("second-word page", &[], &["taxonomy", "--help"]),
        Case("help verb", &[], &["help", "check"]),
        Case("help second word", &[], &["help", "taxonomy", "diff"]),
        Case(
            "CLICOLOR_FORCE on a verb page",
            &[("CLICOLOR_FORCE", "1")],
            &["check", "--help"],
        ),
        Case(
            "CLICOLOR_FORCE on help verb",
            &[("CLICOLOR_FORCE", "1")],
            &["help", "check"],
        ),
        // A completion script is written out of the same tree, so a palette on
        // that tree is one line away from an escape byte inside a shell script.
        Case(
            "CLICOLOR_FORCE on a completion script",
            &[("CLICOLOR_FORCE", "1")],
            &["completions", "bash"],
        ),
        Case(
            "NO_COLOR on a verb page",
            &[("NO_COLOR", "1")],
            &["check", "--help"],
        ),
        Case(
            "--no-color on help verb",
            &[],
            &["--no-color", "help", "check"],
        ),
    ];
    for Case(label, environment, arguments) in cases {
        let at = std::env::temp_dir().join(format!("headwater-color-{}", std::process::id()));
        std::fs::create_dir_all(&at).expect("the directory is there");
        let mut process = Process::new(env!("CARGO_BIN_EXE_headwater"));
        process
            .args(arguments)
            .current_dir(&at)
            .env_remove("COLUMNS");
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

/// **Every refusal this binary prints, not only the four conditions above.**
///
/// `fail`, `refuse`, `defect` and the ~40 refusal lines the verb handlers
/// print inline all go through `err`, the one wrap `fail` already applied to
/// its own message. This runs a refusal that is neither `fail` (a grammar
/// mistake) nor a bare-invocation refusal, so it is evidence about `refuse`
/// and about the inline sites rather than about the one path the four
/// conditions above already cover. A directory with no `.headwater/` is the
/// simplest one every verb reaches through `load`.
#[test]
fn no_escape_byte_reaches_a_refusal_that_is_not_fail_or_the_bare_invocation() {
    let at = std::env::temp_dir().join(format!("headwater-refusal-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&at);
    std::fs::create_dir_all(&at).expect("the directory is there");
    let output = Process::new(env!("CARGO_BIN_EXE_headwater"))
        .arg("check")
        .current_dir(&at)
        .env_remove("COLUMNS")
        .output()
        .expect("the binary runs");
    assert_eq!(output.status.code(), Some(1), "no lock, so the run refuses");
    assert!(
        !output.stdout.is_empty() || !output.stderr.is_empty(),
        "the refusal wrote something"
    );
    assert!(
        !contains_escape(&output.stdout),
        "the refusal writes an escape byte to standard output"
    );
    assert!(
        !contains_escape(&output.stderr),
        "the refusal writes an escape byte to standard error"
    );
    let text = String::from_utf8(output.stderr).expect("the refusal is text");
    assert!(
        text.contains("taxonomy.lock"),
        "the refusal names the lock this run looked for:\n{text}"
    );
}

/// **The palette moves no break, on every page of the help.**
///
/// Every case above runs headless, so the painted help is a page none of them
/// can reach: a process on a pipe renders `ColorMode::Plain` whatever else is
/// true of it. This one asks the library for both trees at one width and holds
/// the two properties that are worth nothing apart.
///
/// The first is the fold: strip the SGR sequences from the painted page and it
/// is the plain page byte for byte, so no break moved and no column shifted
/// when the color arrived. `paint::painted_row` folds before it paints for
/// exactly this reason, and `headwater_check::fill` measures a label in
/// characters, so a paint that ran first would be counted as text.
///
/// The second is that the painted page is painted at all. The strip property
/// alone is satisfied by a palette that emits nothing anywhere — which is the
/// defect [#479](https://github.com/headwater-ai/headwater/issues/479) was
/// filed about and the reason `tools/color-fixtures.sh` exists — so the count
/// of painted pages is compared against the count of pages.
#[test]
fn stripping_the_painted_help_gives_the_plain_help_byte_for_byte() {
    use headwater_cli::paint::ColorMode;

    let mut painted = headwater_cli::command_in(WIDTH, ColorMode::Ansi);
    let mut plain = headwater_cli::command_in(WIDTH, ColorMode::Plain);
    painted.build();
    plain.build();

    let lines = command_lines();
    assert!(!lines.is_empty(), "the tree has pages to render");
    let mut colored = 0usize;
    for words in &lines {
        let typed = format!("headwater {}", words.join(" "));
        let with_color = help_at(&mut painted, words);
        let without = help_at(&mut plain, words);
        assert!(
            !contains_escape(without.as_bytes()),
            "`{typed} --help` renders an escape byte in ColorMode::Plain"
        );
        assert_eq!(
            strip_sgr(&with_color),
            without,
            "`{typed} --help` moves a byte when it is painted"
        );
        if contains_escape(with_color.as_bytes()) {
            colored += 1;
        }
    }
    assert_eq!(
        colored,
        lines.len(),
        "every page of the help is painted in ColorMode::Ansi, and {} of {} were",
        colored,
        lines.len()
    );
}

/// The help of one node of a built tree, named by the words that reach it.
///
/// `StyledStr::ansi()` and not `to_string()`. `Display for StyledStr` walks
/// `iter_text`, which is the text with every sequence already gone, so a test
/// that read it would compare a stripped page against a stripped page and pass
/// on a tree with no palette on it at all. `ansi()` is the bytes `print_help`
/// hands its writer, which is the thing a caller sees.
fn help_at(command: &mut clap::Command, words: &[String]) -> String {
    let mut cursor = command;
    for word in words {
        cursor = cursor
            .find_subcommand_mut(word.as_str())
            .unwrap_or_else(|| panic!("the tree carries `{word}`"));
    }
    cursor.render_help().ansi().to_string()
}

/// The same text with every SGR sequence removed and nothing else touched.
///
/// It reads `ESC [ … m` and drops it. That is the whole of what this binary and
/// `clap` emit — `paint::paint` writes `\x1b[1;32m` and `\x1b[0m`, and `anstyle`
/// writes the same shape — so a sequence of another kind arriving here would be
/// left in place and reported as a difference rather than passed over.
fn strip_sgr(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find("\x1b[") {
        out.push_str(&rest[..at]);
        let tail = &rest[at + 2..];
        match tail.find('m') {
            Some(end) => rest = &tail[end + 1..],
            None => {
                out.push_str(&rest[at..]);
                return out;
            }
        }
    }
    out.push_str(rest);
    out
}

/// The two bytes that open every ANSI colour sequence.
fn contains_escape(bytes: &[u8]) -> bool {
    bytes.windows(2).any(|pair| pair == [0x1b, b'['])
}

// # The masthead and the new `--no-color`, which HW-DR-0045 rules on
//
// Every case above runs headless, through a pipe rather than a terminal, so
// HW-DR-0045's per-stream sensing is expected to render no color here at all
// and every case above is expected to keep passing unchanged — that is itself
// evidence for the claim the decision makes about its own fixtures. What is
// new to assert here is content rather than color: the masthead itself, its
// scope, and the two flags that turn it off. A phrase asserted against folded
// help text is checked with whitespace collapsed first, because a fold can
// land inside a phrase without changing what it says.

/// Whitespace collapsed to single spaces, so a fold point inside an asserted
/// phrase cannot turn a true claim into a failing one.
fn flat(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// **HW-DR-0045.** The root screen alone carries the masthead, above `Usage:`.
///
/// A verb's own page, `--version` and the bare-invocation refusal each already
/// promise something a masthead would break, and the decision keeps every one
/// of those promises rather than growing a line onto it.
#[test]
fn the_root_screen_alone_carries_the_masthead() {
    let version = String::from_utf8(ran(&["--version"], None).out).expect("the version is text");
    let version = version.trim();

    let root = String::from_utf8(ran(&["--help"], None).out).expect("the help is text");
    let lines: Vec<&str> = root.lines().collect();
    assert_eq!(
        lines[0],
        format!("headwater {version} — a documentation corpus, governed and checked like code"),
        "the first line names the binary and the version, not {:?}",
        lines[0]
    );
    assert!(
        !lines[1].is_empty() && lines[1].chars().all(|c| c == '─'),
        "the second line is a rule of box-drawing dashes, not {:?}",
        lines[1]
    );
    let usage_at = lines
        .iter()
        .position(|line| line.starts_with("Usage:"))
        .expect("the root screen has a usage line");
    assert!(usage_at > 1, "the rule sits above Usage:");

    let help = String::from_utf8(ran(&["help"], None).out).expect("the help is text");
    assert_eq!(
        root, help,
        "`headwater help` and `headwater --help` still agree"
    );

    let verb = String::from_utf8(ran(&["check", "--help"], None).out).expect("the help is text");
    assert!(
        !verb
            .lines()
            .any(|line| !line.is_empty() && line.chars().all(|c| c == '─')),
        "a verb's own page carries no rule, and so no masthead"
    );

    let version_run = ran(&["--version"], None);
    let version_out = String::from_utf8(version_run.out).expect("the version is text");
    assert_eq!(
        version_out.lines().count(),
        1,
        "--version still writes exactly one line"
    );

    let bare = ran(&[], None);
    assert_eq!(bare.code, Some(1), "bare invocation still refuses");
    let bare_err = String::from_utf8(bare.err).expect("the refusal is text");
    assert!(
        bare_err.contains("no verb"),
        "the refusal keeps its wording, not {bare_err:?}"
    );
    assert!(
        !bare_err
            .lines()
            .any(|line| !line.is_empty() && line.chars().all(|c| c == '─')),
        "the refusal carries no masthead"
    );
}

/// **HW-DR-0045.** `--no-banner` and `HEADWATER_NO_BANNER` suppress the
/// masthead back to today's plain name line, and both are accepted (and
/// inert) on a verb's own page, the posture `--no-color` already set.
#[test]
fn no_banner_and_its_environment_variable_suppress_the_masthead_and_both_are_accepted_everywhere() {
    struct Case(
        &'static str,
        &'static [(&'static str, &'static str)],
        &'static [&'static str],
    );
    let cases = [
        Case("--no-banner", &[], &["--no-banner", "--help"]),
        Case(
            "HEADWATER_NO_BANNER=1",
            &[("HEADWATER_NO_BANNER", "1")],
            &["--help"],
        ),
    ];
    for Case(label, environment, arguments) in cases {
        let at = std::env::temp_dir().join(format!("headwater-banner-{}", std::process::id()));
        std::fs::create_dir_all(&at).expect("the directory is there");
        let mut process = Process::new(env!("CARGO_BIN_EXE_headwater"));
        process
            .args(arguments)
            .current_dir(&at)
            .env_remove("COLUMNS");
        for (name, value) in environment {
            process.env(name, value);
        }
        let output = process.output().expect("the binary runs");
        assert_eq!(output.status.code(), Some(0), "{label} exits 0");
        let out = String::from_utf8(output.stdout).expect("the help is text");
        assert!(
            !out.lines()
                .any(|line| !line.is_empty() && line.chars().all(|c| c == '─')),
            "{label} suppresses the masthead's rule"
        );
        assert_eq!(
            out.lines().next(),
            Some("headwater — a documentation corpus, governed and checked like code"),
            "{label} reverts the first line to today's plain name line"
        );
    }

    let deep = ran(&["check", "--no-banner", "--help"], None);
    assert_eq!(deep.code, Some(0), "--no-banner is accepted on a verb page");
    assert!(
        deep.err.is_empty(),
        "and it is silent there, same as --no-color"
    );
}

/// **HW-DR-0045.** `--no-color`'s own text states the new behavior on both
/// screens it appears on, not the old claim that this binary has no color to
/// turn off, and `--no-banner` has an entry of its own beside it.
#[test]
fn no_color_s_own_text_states_the_new_behavior_and_no_banner_has_an_entry() {
    let root = flat(&String::from_utf8(ran(&["--help"], None).out).expect("the help is text"));
    assert!(
        !root.contains("no run of this binary emits color"),
        "the root screen's --no-color summary no longer claims this binary has no color"
    );
    assert!(
        root.contains("suppress the masthead"),
        "--no-banner has a summary line on the root screen, not just in:\n{root}"
    );

    let verb =
        flat(&String::from_utf8(ran(&["check", "--help"], None).out).expect("the help is text"));
    assert!(
        verb.contains("senses whether each stream is a terminal"),
        "--no-color's full description states the new default, not:\n{verb}"
    );
    assert!(
        !verb.contains("already does"),
        "--no-color's full description no longer claims this binary already writes no color"
    );
}

// # The report of `headwater check`, which #340 laid out
//
// The help surface above is walked out of the command tree. The report cannot
// be: it is a function of a corpus, so every case below runs the verb over this
// repository through `--root` and reads what it printed.
//
// **The read-set block is exempt and every case says so.** It is the artifact
// `check --read-set` writes and `headwater gate` parses, so the fill does not
// touch it, and a case that measured it would be asking the wrong question of
// the wrong bytes. `the_read_set_block_is_the_artifact_the_flag_writes` is
// where the exemption is held to its side of the bargain.

/// The report over this repository, as bytes, with the streams apart.
fn report(extra: &[&str], columns: Option<&str>) -> Ran {
    let root = repository();
    let root = root.to_str().expect("the path is text").to_string();
    let mut line: Vec<&str> = vec!["check", "--root", &root];
    line.extend_from_slice(extra);
    ran(&line, columns)
}

/// A report split into the part that is laid out and the read set that is not.
fn laid_out(report: &str) -> &str {
    report
        .split_once("\nread set\n")
        .expect("the report carries a read-set block")
        .0
}

/// **The bar, re-scoped.** No line of the report is wider than 80 columns
/// except one whose own word already is.
///
/// The literal bar #340 asked for — no line over 80 at all — cannot be met and
/// should not be. `headwater_check::fill` never breaks inside a word, because a
/// path broken across a fold point is a path a caller cannot retype and a path
/// `headwater_adapter::census` no longer reads off the line a finding block
/// opens on, which is where that audit takes the location of a text finding. So
/// the wide lines are partitioned: **avoidable**, where the
/// longest word plus the line's own indent would have fitted and the fill should
/// have narrowed it, and **unfoldable**, where one word is already past the
/// room. The avoidable count is asserted to be zero and the unfoldable one names
/// itself.
///
/// The state this replaced was 320 of 684 lines over 80, with a widest of 472 —
/// six screen widths, and a census exclusion reason rather than a finding.
#[test]
fn no_line_of_the_check_report_is_wider_than_eighty_columns() {
    let ran = report(&[], None);
    assert_eq!(ran.code, Some(0), "the report ran");
    let out = String::from_utf8(ran.out).expect("the report is text");
    let body = laid_out(&out);

    let mut avoidable: Vec<String> = Vec::new();
    let mut unfoldable: Vec<String> = Vec::new();
    let mut lines = 0;
    let mut widest = (0usize, String::new());
    for (at, line) in body.lines().enumerate() {
        lines += 1;
        let width = line.chars().count();
        if width <= WIDTH {
            continue;
        }
        let entry = format!("line {}, {width} columns: {line}", at + 1);
        match headwater_check::fill::unfoldable(line, WIDTH) {
            true => unfoldable.push(line.to_string()),
            false => {
                if width > widest.0 {
                    widest = (width, line.to_string());
                }
                avoidable.push(entry);
            }
        }
    }
    assert!(
        avoidable.is_empty(),
        "{} of {lines} laid-out lines are wider than {WIDTH} columns and the fill could have \
         narrowed every one. The widest is {} columns:\n{}\n\nEvery one:\n{}",
        avoidable.len(),
        widest.0,
        widest.1,
        avoidable.join("\n")
    );
    // The residual is real and is named rather than hidden. Every member is a
    // line one of whose words is a path this corpus chose to be that long.
    assert!(
        !unfoldable.is_empty(),
        "no line of the report carries an unbreakable word, so the exemption \
         above is no longer measuring anything and should be deleted"
    );
    println!(
        "{} of {lines} laid-out lines are over {WIDTH} columns, every one because a single word \
         of it already is",
        unfoldable.len()
    );
    // The exemption is not a licence for a badly filled line that happens to
    // carry one long word. Take the long word out and what is left fits, so the
    // overflow is attributable to the word and to nothing the fill decided.
    for line in unfoldable.iter().map(String::as_str) {
        let longest = line
            .split_whitespace()
            .map(|word| word.chars().count())
            .max()
            .expect("the line has a word");
        assert!(
            line.chars().count() - longest <= WIDTH,
            "an unfoldable line that is also too long without its long word: {line}"
        );
    }
}

/// **A shape invariant, which a width assertion cannot express.**
///
/// A finding's location line is `<path>:<line>:<column> <severity>`. The fill
/// must never wrap the severity onto a line of its own: that hands a reader half
/// an identity, and it hands `.githooks/pre-commit` a line whose whole content
/// is `error`, which its selector reads as the header of a new finding — so the
/// rule line and the `fix:` line under the real header are dropped and a refused
/// commit explains nothing.
///
/// **`no_line_of_the_check_report_is_wider_than_eighty_columns` is structurally
/// blind to this**, and that is why this case exists beside it. The broken
/// output is two lines of 80 and 9 columns; counting columns cannot see it. A
/// layout change needs at least one assertion about the shapes a fold can emit,
/// not only about their widths.
///
/// The band that produced it was narrow — a location line whose opening word
/// reaches the width without passing it — and it held 38 of this corpus's 334
/// documents. This runs at three widths, so the band moves under it.
///
/// **This case reads the corpus as it stands, so on its own it is not enough.**
/// The three findings this repository reports today all sit on short paths, and
/// a case that only ever sees those would have passed while the defect stood.
/// The two cases that provoke the boundary rather than waiting for it are
/// `headwater_check::fill::tests::a_two_word_line_is_never_split_one_word_to_a_line`,
/// which walks the opening width one column at a time, and the
/// `a finding whose location line lands on the width boundary` case of
/// `.githooks/fixtures.sh`, which drives the real commit hook over a document
/// whose path was chosen to land in the band.
#[test]
fn no_finding_states_its_severity_on_a_line_of_its_own() {
    for columns in [None, Some("100"), Some("120")] {
        let extra: &[&str] = match columns {
            None => &[],
            Some(_) => &["--wide"],
        };
        let ran = report(extra, columns);
        assert_eq!(ran.code, Some(0), "the report ran");
        let out = String::from_utf8(ran.out).expect("the report is text");
        let asked = columns.unwrap_or("80");
        for (at, line) in out.lines().enumerate() {
            assert!(
                !matches!(
                    line.trim(),
                    "error" | "warn" | "info" | "✗ error" | "▲ warn" | "· info"
                ),
                "at COLUMNS={asked}, line {} of the report is a bare severity word, so a \
                 finding's location and its severity are on two lines:\n{}",
                at + 1,
                out.lines()
                    .skip(at.saturating_sub(2))
                    .take(5)
                    .collect::<Vec<&str>>()
                    .join("\n")
            );
        }
        // At least one location line, so a corpus that reported nothing does not
        // pass this case by having no findings in it. Plain mode prints a
        // severity glyph beside the word — `✗`, `▲` or `·`, see
        // `headwater_check::paint` — so the line is a path, a glyph and a word
        // rather than a path and a word alone.
        let located = out
            .lines()
            .filter(|line| {
                let trimmed = line.trim_end();
                trimmed.split_whitespace().count() == 3
                    && (trimmed.ends_with(" ✗ error")
                        || trimmed.ends_with(" ▲ warn")
                        || trimmed.ends_with(" · info"))
            })
            .count();
        assert!(
            located > 0,
            "at COLUMNS={asked} no finding location line was found, so this case is \
             measuring nothing"
        );
    }
}

/// **The exemption, held to its side of the bargain.**
///
/// The read-set block of the report is the file `--read-set` writes, indented
/// two spaces and in order. That sentence is in `crates/adapter/src/text.rs` and
/// `headwater gate` depends on it: `Recorded::parse` reads the file as a
/// grammar, and a fold inside it would break the verb rather than the prose.
/// This case goes red the day somebody folds that block.
#[test]
fn the_read_set_block_of_the_report_is_the_artifact_the_flag_writes() {
    let at = std::env::temp_dir().join(format!("headwater-readset-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&at);
    std::fs::create_dir_all(&at).expect("the directory is there");
    let path = at.join("read-set");
    let written = path.to_str().expect("the path is text").to_string();

    let ran = report(&["--read-set", &written], None);
    assert_eq!(ran.code, Some(0), "the report ran");
    let out = String::from_utf8(ran.out).expect("the report is text");
    let file = std::fs::read_to_string(&path).expect("the flag wrote the file");
    assert!(!file.is_empty(), "the file has a read set in it");

    let block = out
        .split_once("\nread set\n")
        .expect("the report carries a read-set block")
        .1;
    let indented: String = file
        .lines()
        .map(|line| match line.is_empty() {
            true => String::from("\n"),
            false => format!("  {line}\n"),
        })
        .collect();
    assert_eq!(
        block, indented,
        "the read-set block of the report is not the file the flag wrote"
    );
}

/// **The invariant, over the report rather than over the help.**
///
/// A run that states `COLUMNS` and one that does not write the same bytes,
/// because nothing reads the variable without `--wide`. Bytes rather than text:
/// the question is whether one byte differs, and a text comparison of a
/// 700-line report answers it less directly.
#[test]
fn a_report_that_states_columns_and_one_that_does_not_write_the_same_bytes() {
    let bare = report(&[], None);
    assert_eq!(bare.code, Some(0));
    for absurd in ["1", "40", "500", "100000", "not a number"] {
        let stated = report(&[], Some(absurd));
        assert_eq!(
            bare.out, stated.out,
            "the report under COLUMNS={absurd} is not the report with none"
        );
        assert_eq!(bare.code, stated.code);
    }
}

/// **The width a caller does ask for, held to the band.**
///
/// Asserted by identity, the way the help case is: 40 is the layout every other
/// run gets, 500 is the layout 120 gets, and 100 is neither. Then every laid-out
/// line at each band is inside the width it asked for, or is a line one of whose
/// words already was.
#[test]
fn the_width_a_caller_asks_for_lays_the_report_out_and_is_held_to_the_band() {
    let text = |ran: Ran| String::from_utf8(ran.out).expect("the report is text");
    let ordinary = text(report(&[], None));
    let wide = |columns: &str| text(report(&["--wide"], Some(columns)));

    let narrow = wide("40");
    assert_eq!(narrow, ordinary, "a width under {WIDTH} is {WIDTH}");
    assert_eq!(wide("80"), ordinary);

    let widest_asked = wide("500");
    assert_eq!(
        widest_asked,
        wide("120"),
        "a width over {WIDEST} is {WIDEST}"
    );
    assert_ne!(widest_asked, ordinary, "{WIDEST} is not {WIDTH}");

    let between = wide("100");
    assert_ne!(between, ordinary);
    assert_ne!(between, widest_asked);

    for (asked, out) in [(WIDTH, &narrow), (100, &between), (WIDEST, &widest_asked)] {
        for line in laid_out(out).lines() {
            assert!(
                line.chars().count() <= asked || headwater_check::fill::unfoldable(line, asked),
                "at {asked} columns this line is {} and the fill could have narrowed it: {line}",
                line.chars().count()
            );
        }
    }
}

/// **The layout is not a function of what the report is attached to.**
///
/// The pipe leg is the captured standard output above. The file leg runs the
/// binary with its output redirected by a shell and reads the file back, which
/// consults no terminal at all. The terminal leg runs it under `script(1)`,
/// which gives the process a pseudo-terminal, and is **skipped with a printed
/// line where `script` is not installed** — the posture
/// `.claude/hooks/fixtures-live.sh` takes toward an uninstalled harness. The
/// workspace declares no `libc`, so an in-process `openpty` would cost a
/// dependency `HW-DR-0033` prices, and `script` is the answer that costs none.
#[test]
fn a_report_written_to_a_pipe_a_file_and_a_terminal_is_the_same_bytes() {
    let root = repository();
    let root = root.to_str().expect("the path is text").to_string();
    let binary = env!("CARGO_BIN_EXE_headwater");
    let at = std::env::temp_dir().join(format!("headwater-streams-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&at);
    std::fs::create_dir_all(&at).expect("the directory is there");

    let piped = report(&[], None);
    assert_eq!(piped.code, Some(0), "the piped run exits 0");

    let file = at.join("report");
    let shell = Process::new("sh")
        .arg("-c")
        .arg(format!(
            "'{binary}' check --root '{root}' > '{}'",
            file.display()
        ))
        .current_dir(&at)
        .env_remove("COLUMNS")
        .output()
        .expect("the shell runs");
    assert_eq!(shell.status.code(), Some(0), "the redirected run exits 0");
    let written = std::fs::read(&file).expect("the shell wrote the file");
    assert_eq!(
        piped.out, written,
        "a pipe and a file are not the same bytes"
    );

    let found = Process::new("sh")
        .arg("-c")
        .arg("command -v script")
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false);
    if !found {
        println!("skipped: `script` is not on PATH, so no terminal leg ran");
        return;
    }
    let under = at.join("terminal");
    let pty = Process::new("script")
        .args([
            "-qec",
            &format!("'{binary}' check --root '{root}' > '{}'", under.display()),
            "/dev/null",
        ])
        .current_dir(&at)
        .env_remove("COLUMNS")
        .output()
        .expect("`script` runs");
    assert!(pty.status.success(), "the run under a terminal exits 0");
    // The redirection is inside the pseudo-terminal, so the file carries the
    // report and not the session transcript. The carriage returns a line
    // discipline inserts belong to the terminal rather than to this binary, so
    // they are stripped before the comparison.
    let seen: Vec<u8> = std::fs::read(&under)
        .expect("the run under a terminal wrote the file")
        .into_iter()
        .filter(|byte| *byte != b'\r')
        .collect();
    assert_eq!(
        piped.out, seen,
        "a pipe and a terminal are not the same bytes"
    );
}
