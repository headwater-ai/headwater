// SPDX-License-Identifier: Apache-2.0
//! What the help says, held against the table it is written out of.
//!
//! [`super`] holds the *shape* of the command tree — every command line the
//! parser answers to, against `headwater_verbs::VERBS`, in both directions.
//! Nothing held the *words*. A verb could ship with nothing against its name
//! and a flag could ship undescribed, and every case in this crate stayed
//! green: [#321](https://github.com/headwater-ai/headwater/issues/321) clause 6
//! names five flags that did exactly that — `--facet`, `--tier`, `--arm`,
//! `--category` and `--seed` — and the verification of the parser migration
//! recorded that the walk of the command tree can never catch a flag, because
//! it compares command lines and a flag is not one.
//!
//! # The three routes are one command, so they cannot drift apart
//!
//! `headwater help check`, `headwater check --help` and `headwater check -h`
//! all render the same node of the tree `headwater_cli::command` builds. No
//! `long_about` and no `long_help` exists anywhere in that tree, which is what
//! makes the two spellings of the flag render the same text, and
//! `headwater help` prints through `Command::print_help` for the same reason.
//! The case below asserts the three are byte-identical rather than merely all
//! non-empty, because "each carries something" is satisfied by three different
//! answers to one question.
//!
//! # Why the summaries are asserted against `VERBS` rather than against a string
//!
//! A case that pinned the text would be a second copy of it, and a second copy
//! of the verb list is what #257 was filed about. Every assertion here reads
//! the expected text out of `headwater_verbs::VERBS` at run time, so changing a
//! summary there moves the help and this file follows without an edit — and
//! moving a summary *into the parser* breaks it, because the table would then
//! carry a string the rendered help does not.

use headwater_cli::command;
use std::process::Command as Process;

/// One invocation from a directory that is not a corpus, with the streams apart.
struct Ran {
    code: Option<i32>,
    out: String,
    err: String,
}

fn ran(label: &str, arguments: &[&str]) -> Ran {
    let at =
        std::env::temp_dir().join(format!("headwater-cli-help-{}-{label}", std::process::id()));
    let _ = std::fs::remove_dir_all(&at);
    std::fs::create_dir_all(&at).expect("the directory is there");
    let output = Process::new(env!("CARGO_BIN_EXE_headwater"))
        .args(arguments)
        .current_dir(&at)
        .output()
        .expect("the binary runs");
    Ran {
        code: output.status.code(),
        out: String::from_utf8_lossy(&output.stdout).into_owned(),
        err: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

/// Every run of whitespace as one space, so an assertion survives a rewrap.
///
/// `clap` does no wrapping in this build — `StyledStr::wrap` is compiled out
/// without the `wrap_help` feature, which this workspace does not take — so
/// help text is one line however long it is today. #321 clause 13 changes that,
/// and a case that compared raw bytes would go red on the piece that does it
/// for no reason a reader would recognize.
fn flat(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The help of one node of the command tree, rendered in process.
fn help_of(words: &[&str]) -> String {
    let mut root = command();
    root.build();
    let mut cursor = &mut root;
    for word in words {
        cursor = cursor
            .find_subcommand_mut(word)
            .unwrap_or_else(|| panic!("the parser answers to `{}`", words.join(" ")));
    }
    cursor.render_help().to_string()
}

/// Every argument the parser admits says what it is.
///
/// # This is the guard clause 7 cannot be
///
/// `tests/verbs.rs` walks the same tree and compares **command lines**. A flag
/// is not a command line, so a flag added to any verb — global or not,
/// described or not — is invisible there. That is how `--facet`, `--tier`,
/// `--arm`, `--category` and `--seed` reached a shipped binary with no
/// description anywhere: the synopsis named them and the flag block did not,
/// and nothing read the two against each other.
///
/// It is asserted over **every** argument rather than every flag, because a
/// positional a caller has to guess at is the same defect one level along:
/// `headwater taxonomy vendor <dir>` says nothing about what the directory is
/// unless somebody writes it down.
///
/// The failure names every offender at once. A case that returned at the first
/// one would make a reader run it as many times as there are undescribed flags.
#[test]
fn every_argument_the_parser_admits_says_what_it_is() {
    fn walk(prefix: &str, command: &clap::Command, silent: &mut Vec<String>) {
        for argument in command.get_arguments() {
            let says = argument
                .get_help()
                .map(ToString::to_string)
                .unwrap_or_default();
            if says.trim().is_empty() {
                silent.push(format!("{prefix}: {}", argument.get_id()));
            }
        }
        for word in command.get_subcommands() {
            walk(&format!("{prefix} {}", word.get_name()), word, silent);
        }
    }

    let mut root = command();
    root.build();
    let mut silent = Vec::new();
    walk(headwater_verbs::BINARY, &root, &mut silent);
    assert!(
        silent.is_empty(),
        "{} arguments of this parser carry no help text, and a caller who meets one has nothing \
         to read: {silent:#?}",
        silent.len()
    );
}

/// The three routes clause 4 names reach one text, and it is the table's.
///
/// Byte-identity is the assertion rather than three separate containments,
/// because three routes that each carry *something* is the state a reader
/// cannot rely on: the point of the clause is that a caller who reaches for the
/// habit they already have arrives at the same page.
#[test]
fn the_long_description_of_a_verb_is_reachable_three_ways() {
    let verb = headwater_verbs::parse("check").expect("`check` is a verb of this binary");
    let routes = [
        ("flag-long", vec!["check", "--help"]),
        ("flag-short", vec!["check", "-h"]),
        ("verb", vec!["help", "check"]),
    ];
    let mut rendered: Vec<(String, String)> = Vec::new();
    for (label, arguments) in routes {
        let ran = ran(label, &arguments);
        assert_eq!(
            ran.code,
            Some(0),
            "`headwater {}` is a question rather than a mistake:\n{}{}",
            arguments.join(" "),
            ran.out,
            ran.err
        );
        assert_eq!(
            ran.err,
            "",
            "`headwater {}` writes nothing to standard error",
            arguments.join(" ")
        );
        assert!(
            flat(&ran.out).contains(&flat(verb.description)),
            "`headwater {}` carries the description the dispatch table holds for `check`:\n{}",
            arguments.join(" "),
            ran.out
        );
        rendered.push((arguments.join(" "), ran.out));
    }
    let (first, text) = &rendered[0];
    for (other, also) in &rendered[1..] {
        assert_eq!(
            text, also,
            "`headwater {first}` and `headwater {other}` print the same bytes"
        );
    }
}

/// The first screen is grouped, and every group and every summary comes off the
/// dispatch table.
///
/// A group declared in the parser would be the fifth hand-kept copy of the verb
/// list that [#257](https://github.com/headwater-ai/headwater/issues/257) was
/// filed about, so what this asserts is the direction of the read: change
/// `headwater_verbs::VERBS` and the screen moves with no edit to
/// `engine/crates/cli/src/lib.rs`. Move a summary into the parser and this case
/// fails, because the table then carries a line the screen does not.
#[test]
fn the_first_screen_prints_the_group_and_the_summary_the_table_carries() {
    let screen = flat(&help_of(&[]));
    for group in headwater_verbs::groups() {
        assert!(
            screen.contains(&format!("{group}:")),
            "the first screen carries the heading `{group}:`"
        );
    }
    for verb in headwater_verbs::VERBS {
        assert!(
            screen.contains(&flat(verb.summary)),
            "the first screen carries `{}`'s summary from the dispatch table",
            verb.name
        );
        assert!(
            screen.contains(&format!(" {} ", verb.name)),
            "the first screen names `{}`",
            verb.name
        );
    }
}

/// Every verb and every second word prints the description the table carries.
///
/// The case above holds the first screen and this one holds what is behind it,
/// so a verb whose summary is on the screen and whose long form is empty is
/// caught here rather than by a reader.
#[test]
fn the_long_description_of_every_command_line_is_what_the_table_carries() {
    for verb in headwater_verbs::VERBS {
        let rendered = flat(&help_of(&[verb.name]));
        assert!(
            rendered.contains(&flat(verb.description)),
            "`headwater {} --help` carries the description the table holds:\n{rendered}",
            verb.name
        );
        for word in verb.words {
            let rendered = flat(&help_of(&[verb.name, word.name]));
            assert!(
                rendered.contains(&flat(word.description)),
                "`headwater {} {} --help` carries the description the table holds:\n{rendered}",
                verb.name,
                word.name
            );
        }
    }
}

/// The first screen fits a screen, which is the bar clause 4 sets.
///
/// The number is a ceiling rather than a pin. A pin is a fixture that goes red
/// on every verb added and teaches a reader to re-bless it; a ceiling goes red
/// only when the screen stops being a screen. The help this replaced was **357
/// lines**, which is what the ceiling is for.
#[test]
fn the_first_screen_is_one_screen() {
    let screen = help_of(&[]);
    let lines = screen.lines().count();
    assert!(
        lines <= 60,
        "`headwater --help` is {lines} lines, and the first screen is meant to fit one:\n{screen}"
    );
    assert!(
        lines > 20,
        "`headwater --help` is {lines} lines, which is too few to carry {} verbs with a line each",
        headwater_verbs::VERBS.len()
    );
}

/// Neither spelling of either global answer writes to standard error, at either
/// level of the tree.
///
/// The verification of the parser migration found the existing case loops over
/// the root alone, so a regression that moved `headwater check --help` to a
/// non-zero status would pass the suite. This closes it at both levels and for
/// both spellings.
#[test]
fn every_global_answer_exits_zero_on_standard_output_alone() {
    for (label, arguments) in [
        ("root-long", vec!["--help"]),
        ("root-short", vec!["-h"]),
        ("verb-long", vec!["check", "--help"]),
        ("verb-short", vec!["check", "-h"]),
        ("word-long", vec!["sweep", "plan", "--help"]),
        ("version-long", vec!["--version"]),
        ("version-short", vec!["-V"]),
        ("version-after-verb", vec!["check", "--version"]),
        ("help-verb", vec!["help"]),
        ("help-verb-word", vec!["help", "taxonomy", "diff"]),
    ] {
        let ran = ran(label, &arguments);
        let named = arguments.join(" ");
        assert_eq!(
            ran.code,
            Some(0),
            "`headwater {named}`:\n{}{}",
            ran.out,
            ran.err
        );
        assert_eq!(
            ran.err, "",
            "`headwater {named}` writes nothing to standard error"
        );
        assert!(
            !ran.out.is_empty(),
            "`headwater {named}` writes an answer to standard output"
        );
    }
}
