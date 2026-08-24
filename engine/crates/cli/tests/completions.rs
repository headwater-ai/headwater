// SPDX-License-Identifier: Apache-2.0
//! `headwater completions <shell>`: a script a shell loads, and four of them.
//!
//! # The clause
//!
//! [#321](https://github.com/headwater-ai/headwater/issues/321): "`headwater
//! completions bash|zsh|fish|powershell` writes a script to stdout. The name is
//! in `VERBS` and in spec 6's CLI grammar block, whose rule is that a name it
//! declares either runs or states its wait; this one runs. `headwater
//! completions bash > f && . f && complete -p headwater` succeeds."
//!
//! That last sentence is a **behavioral** bar and not a byte count, so
//! [`the_command_line_the_clause_states_loads_a_completion_into_a_bash`] runs
//! exactly it: the binary, a redirect, a `.` and a `complete -p`. A script that
//! parsed and registered nothing would pass every length assertion below and
//! fail that one.
//!
//! # Why a shell is an oracle, and what happens where it is absent
//!
//! Only a shell can say that a shell accepts a script. `bash` is therefore a
//! tool outside this repository in exactly the sense `python3 -m json.tool`,
//! `sha256sum` and the two schema validators already are, so it takes the shape
//! those four take: the reading is a note where the tool is absent, and
//! `HEADWATER_SHELL_ORACLE` turns "it did not run" into a failure. The `Test`
//! step of `.github/workflows/ci.yml` sets it, so a runner that lost `bash`
//! fails the job rather than going quiet.
//!
//! `zsh`, `fish` and PowerShell get no case of their own here, and the reason is
//! that this repository has no way to run one. What holds those three is
//! [`every_shell_writes_one_script_to_standard_output_and_nothing_beside_it`]
//! and [`a_script_carries_every_command_line_this_binary_dispatches`], which are
//! claims about the artifact rather than about a shell's reading of it. The
//! pull request that added this verb states which shells were executed and
//! where.
//!
//! # No corpus is read
//!
//! No invocation below passes `--root`, and every one exits 0. The script is a
//! function of the command tree alone, so this verb answers outside a
//! repository — which is where a caller installing a completion actually is.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The two streams held apart, because the script is on one and a refusal is on
/// the other.
#[derive(Debug)]
struct Ran {
    code: Option<i32>,
    out: Vec<u8>,
    err: Vec<u8>,
}

impl Ran {
    fn text(&self) -> String {
        String::from_utf8_lossy(&self.out).into_owned()
    }

    fn said(&self) -> String {
        String::from_utf8_lossy(&self.err).into_owned()
    }
}

fn scratch() -> PathBuf {
    let at = Path::new(env!("CARGO_TARGET_TMPDIR")).join("completions");
    std::fs::create_dir_all(&at).expect("the directory is there");
    at
}

/// The binary, with the environment a caller's shell would give it.
///
/// `COLUMNS` is cleared rather than left alone. A stray reading in the
/// environment of whoever runs `cargo test` would otherwise reach
/// [`the_script_does_not_move_with_the_terminal_of_whoever_asked_for_it`] as a
/// value neither run of that pair chose.
fn ran(arguments: &[&str]) -> Ran {
    let output = Command::new(env!("CARGO_BIN_EXE_headwater"))
        .args(arguments)
        .env_remove("COLUMNS")
        .output()
        .expect("the binary runs");
    Ran {
        code: output.status.code(),
        out: output.stdout,
        err: output.stderr,
    }
}

/// The four names the parser admits, which are the four the grammar declares.
const SHELLS: [&str; 4] = ["bash", "zsh", "fish", "powershell"];

/// The command line the clause states, run as the clause states it.
///
/// # This is the one case here that a shell decides
///
/// Everything else in this file reads the bytes the binary wrote. This runs
/// them. `.` is the source builtin, and `complete -p headwater` exits non-zero
/// with "no completion specification" for a name nothing registered — which is
/// what it does on a tree without this verb, and what it did on `9c9244e`
/// before this branch.
#[test]
fn the_command_line_the_clause_states_loads_a_completion_into_a_bash() {
    let script = scratch().join("f");
    let written = ran(&["completions", "bash"]);
    assert_eq!(written.code, Some(0), "{written:?}");
    std::fs::write(&script, &written.out).expect("the script writes");

    let required = std::env::var_os("HEADWATER_SHELL_ORACLE").is_some();
    let line = format!(
        ". {} && complete -p headwater",
        script.to_str().expect("a path")
    );
    let read = match Command::new("bash").args(["-c", &line]).output() {
        Ok(output) => output,
        Err(error) => {
            assert!(
                !required,
                "HEADWATER_SHELL_ORACLE is set and `bash` did not run, so no shell read this \
                 script: {error}"
            );
            eprintln!("note: `bash` did not run ({error}), so no shell read this script");
            return;
        }
    };
    assert!(
        read.status.success(),
        "`{line}` exited {}: {}",
        read.status,
        String::from_utf8_lossy(&read.stderr)
    );
    // `complete -p` prints the specification it found. A `-F` naming a function
    // the script did not define would still print, so the function's own name
    // is asserted beside it.
    let printed = String::from_utf8_lossy(&read.stdout).into_owned();
    assert!(
        printed.contains("-F _headwater") && printed.trim().ends_with("headwater"),
        "`complete -p headwater` printed {printed:?}"
    );
}

/// Every shell writes one script, on one stream, and says nothing on the other.
///
/// The empty standard error is the half a caller redirecting into a file
/// depends on: `headwater completions bash > f` puts the script in `f` and
/// leaves the terminal clean, and a note written beside it would be invisible
/// in the file and confusing in the terminal.
#[test]
fn every_shell_writes_one_script_to_standard_output_and_nothing_beside_it() {
    for shell in SHELLS {
        let written = ran(&["completions", shell]);
        assert_eq!(written.code, Some(0), "`{shell}`: {written:?}");
        assert!(
            written.out.len() > 1_000,
            "`{shell}` wrote {} bytes on standard output",
            written.out.len()
        );
        assert!(
            written.err.is_empty(),
            "`{shell}` wrote {:?} on standard error",
            written.said()
        );
    }
}

/// No escape byte reaches a completion script.
///
/// # A fifth surface for clause 11
///
/// `engine/crates/cli/tests/width.rs` holds the absence of an escape byte in
/// three places that do not read one another: the help under four conditions,
/// the three machine formats, and the two files `check` writes. A completion
/// script is a fourth artifact of that kind and the first one a shell executes,
/// so it is held here rather than assumed from `ColorChoice::Never`.
#[test]
fn no_escape_byte_reaches_a_completion_script() {
    for shell in SHELLS {
        let written = ran(&["completions", shell]);
        for (stream, bytes) in [("standard output", &written.out), ("standard error", &written.err)]
        {
            assert!(
                !bytes.windows(2).any(|pair| pair == b"\x1b["),
                "`{shell}` wrote an escape byte on {stream}"
            );
        }
    }
}

/// A script offers the command lines this binary answers to.
///
/// # Why this is a content assertion and not a length one
///
/// `clap_complete` walks the command tree `headwater_cli::command` builds, and
/// `engine/crates/cli/tests/verbs.rs` holds that tree against
/// `headwater_verbs::VERBS` in both directions. So a verb missing from a script
/// is either a verb missing from the tree, which that file reports, or a
/// generator that dropped it, which nothing else here would see. This is the
/// second half.
#[test]
fn a_script_carries_every_command_line_this_binary_dispatches() {
    for shell in SHELLS {
        let script = ran(&["completions", shell]).text();
        for verb in headwater_verbs::VERBS {
            assert!(
                script.contains(verb.name),
                "the `{shell}` script names no `{}`",
                verb.name
            );
            for word in verb.words {
                assert!(
                    script.contains(word.name),
                    "the `{shell}` script names no `{} {}`",
                    verb.name,
                    word.name
                );
            }
        }
    }
}

/// A fifth shell is refused with the four printed, and no script is written.
///
/// `clap_complete::Shell` carries `Elvish`, so this is the name a caller would
/// most plausibly reach for and find absent. The refusal is the value parser's
/// rather than a match arm underneath one, which is why it can print the set.
#[test]
fn a_shell_this_binary_does_not_carry_is_refused_with_the_four_printed() {
    let refused = ran(&["completions", "elvish"]);
    assert_eq!(refused.code, Some(1), "{refused:?}");
    assert!(
        refused.out.is_empty(),
        "a refused shell wrote {} bytes of script",
        refused.out.len()
    );
    let said = refused.said();
    for shell in SHELLS {
        assert!(said.contains(shell), "the refusal names no `{shell}`: {said}");
    }
}

/// A run with no shell names the four and says where a script goes.
///
/// The operand is optional to the parser for the reason every other required
/// operand of this binary is: `clap`'s missing-argument message names the value
/// and nothing else, and a caller who does not know that a completion script is
/// redirected somewhere by hand is exactly the caller who typed this.
#[test]
fn a_run_with_no_shell_names_the_four_and_writes_no_script() {
    let refused = ran(&["completions"]);
    assert_eq!(refused.code, Some(1), "{refused:?}");
    assert!(refused.out.is_empty(), "{refused:?}");
    let said = refused.said();
    for shell in SHELLS {
        assert!(said.contains(shell), "the refusal names no `{shell}`: {said}");
    }
    assert!(
        said.contains("> f"),
        "the refusal shows no form that redirects the script: {said}"
    );
}

/// The script does not move with the terminal of whoever asked for it.
///
/// # The property, and why it needs an assertion rather than an argument
///
/// Clause 12 of #321 fixes the help at 80 columns "whether or not a terminal is
/// attached, so a piped run and a fixture run are byte-identical", and `--wide`
/// is the one reader of `COLUMNS`. A completion script carries the same help
/// strings, so it inherits the same question. Two things answer it and the
/// second is the one that could rot: `main` builds the tree at
/// `headwater_cli::paint::WIDTH` rather than at `paint::width()`, and `--wide`
/// is refused on a run that prints no help. If either changed, a caller's
/// terminal would reach the bytes a shell executes.
#[test]
fn the_script_does_not_move_with_the_terminal_of_whoever_asked_for_it() {
    for shell in SHELLS {
        let narrow = Command::new(env!("CARGO_BIN_EXE_headwater"))
            .args(["completions", shell])
            .env("COLUMNS", "40")
            .output()
            .expect("the binary runs");
        let wide = Command::new(env!("CARGO_BIN_EXE_headwater"))
            .args(["completions", shell])
            .env("COLUMNS", "200")
            .output()
            .expect("the binary runs");
        assert_eq!(
            narrow.stdout, wide.stdout,
            "the `{shell}` script moves with `COLUMNS`"
        );
        assert_eq!(narrow.stdout, ran(&["completions", shell]).out);
    }
    let widened = ran(&["completions", "bash", "--wide"]);
    assert_eq!(widened.code, Some(1), "{widened:?}");
    assert!(widened.out.is_empty(), "{widened:?}");
}
