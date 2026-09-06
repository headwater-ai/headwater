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
//! [`every_shell_writes_one_script_to_standard_output_and_nothing_beside_it`],
//! [`a_script_carries_every_command_line_this_binary_dispatches`] and
//! [`no_quoted_string_of_a_completion_script_spans_a_newline`], which are
//! claims about the artifact rather than about a shell's reading of it. The
//! pull request that added this verb states which shells were executed and
//! where.
//!
//! # Where a shell stops being an oracle, which is a measurement
//!
//! The paragraph above says a shell is the only thing that can say a shell
//! accepts a script, and that stays true. It does not follow that a shell can
//! say a script is *right*, and for the defect
//! [`no_quoted_string_of_a_completion_script_spans_a_newline`] reports it
//! cannot. That was measured rather than assumed, on 2026-09-06 against a zsh
//! script carrying thirteen broken specifications, with zsh 5.9 in a container
//! because this host and this repository's self-hosted runner carry no `zsh`:
//! `zsh -n` exited 0, sourcing the script under `compinit` exited 0, and
//! `_headwater` was defined afterwards. All three pass on the broken artifact,
//! because a single-quoted string in zsh may legally span a newline and
//! `clap_complete` escapes the `:` that `_arguments` splits a specification on.
//! Nothing is malformed. The newline lands in the message `_arguments`
//! displays and nowhere else.
//!
//! A `zsh/zpty` harness driving a real completion is the only remaining route
//! through a shell, and it is not an oracle either: a pseudoterminal wraps a
//! long description at its own width whatever the script said, so the capture
//! cannot separate an emitted newline from a terminal wrap. The instrument that
//! can see this defect reads the bytes in the shell's own quoting grammar,
//! which is what that test is.
//!
//! It therefore spawns nothing and cannot skip. `HEADWATER_SHELL_ORACLE` is
//! deliberately not consulted there: this repository's only runner has no
//! `zsh`, no `fish` and no `pwsh`, so gating an arm on that variable would fail
//! the runner permanently and leaving it ungated would skip permanently. A
//! byte assertion that always runs is better than both.
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
        for (stream, bytes) in [
            ("standard output", &written.out),
            ("standard error", &written.err),
        ] {
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
        assert!(
            said.contains(shell),
            "the refusal names no `{shell}`: {said}"
        );
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
        assert!(
            said.contains(shell),
            "the refusal names no `{shell}`: {said}"
        );
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

/// One shell's rule for where a quoted string starts and stops.
///
/// Each field below is a difference between the four that the scan needs, and
/// each one was measured against the script it describes rather than taken from
/// a manual. A field set wrong shows up as `unterminated`, because a scan that
/// misreads a close runs to the end of the file inside a string.
struct Quoting {
    /// The byte that opens a string and the byte that closes it.
    quote: u8,
    /// A backslash inside a string takes the next byte with it. `fish` writes
    /// `package\'s` inside a description and needs this; `zsh` and PowerShell
    /// write no backslash escape inside a string and must not have it, or a
    /// `\:` before a quote would swallow the close.
    backslash_escapes_inside: bool,
    /// A doubled quote inside a string is one literal quote. PowerShell writes
    /// `package''s`; `zsh` writes the same character as `'\''`, which this scan
    /// reads as a close, an escaped quote outside, and a fresh open.
    doubled_quote_inside: bool,
    /// A `#` outside a string runs to the end of its line. `fish` needs it: the
    /// generator's own first line is a comment reading "cmd's options", and
    /// without this the apostrophe there opens a string that never closes and
    /// every description after it is reported. That false positive was observed
    /// before this field existed.
    hash_comments: bool,
}

/// A string that opened on one line and closed on another.
struct Spanned {
    line: usize,
    newlines: usize,
    head: String,
}

/// What one pass of a script under one quoting model saw.
struct Scan {
    opens: usize,
    spanning: Vec<Spanned>,
    unterminated: bool,
}

/// Every quoted string of a script, and which of them span a newline.
///
/// The scan is over bytes, and every byte it branches on is ASCII, so the
/// slices it takes are on character boundaries whatever the description says.
fn scan(script: &str, model: &Quoting) -> Scan {
    let bytes = script.as_bytes();
    let mut opens = 0;
    let mut spanning = Vec::new();
    let mut line = 1;
    let mut i = 0;
    // The byte after the opening quote, and the line the quote was on.
    let mut inside: Option<(usize, usize)> = None;
    while i < bytes.len() {
        let byte = bytes[i];
        if let Some((start, at)) = inside {
            if model.backslash_escapes_inside && byte == b'\\' && i + 1 < bytes.len() {
                if bytes[i + 1] == b'\n' {
                    line += 1;
                }
                i += 2;
                continue;
            }
            if byte == model.quote {
                if model.doubled_quote_inside && bytes.get(i + 1) == Some(&model.quote) {
                    i += 2;
                    continue;
                }
                let body = &script[start..i];
                let newlines = body.matches('\n').count();
                if newlines > 0 {
                    spanning.push(Spanned {
                        line: at,
                        newlines,
                        head: body.lines().next().unwrap_or_default().to_owned(),
                    });
                }
                inside = None;
                i += 1;
                continue;
            }
            if byte == b'\n' {
                line += 1;
            }
            i += 1;
            continue;
        }
        // A backslash outside a string takes the next byte with it, and the
        // byte it most often takes here is the newline continuing a `zsh`
        // specification. Counting the line without this reads every one of
        // those as no line at all.
        if byte == b'\\' && i + 1 < bytes.len() {
            if bytes[i + 1] == b'\n' {
                line += 1;
            }
            i += 2;
            continue;
        }
        if model.hash_comments && byte == b'#' {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if byte == b'\n' {
            line += 1;
            i += 1;
            continue;
        }
        if byte == model.quote {
            opens += 1;
            inside = Some((i + 1, line));
            i += 1;
            continue;
        }
        i += 1;
    }
    Scan {
        opens,
        spanning,
        unterminated: inside.is_some(),
    }
}

/// No quoted string a completion script writes runs past the end of its line.
///
/// # What goes wrong when one does
///
/// `clap_complete` writes a `zsh` positional as
/// `'::name -- <help>:<action>'`, and it puts the help in there exactly as the
/// command tree carries it. This binary folds every help string to
/// [`headwater_cli::paint::WIDTH`] before `clap` sees it, which is clause 12 of
/// [#321](https://github.com/headwater-ai/headwater/issues/321) and is right
/// for a help screen. A completion script is not a help screen. The fold's
/// newlines land inside the quotes and `_arguments` prints them, so a caller
/// who presses tab gets a description broken across lines at a width nobody
/// chose, with the reservation this binary makes for `clap`'s
/// `[possible values: …]` folded in beside it.
///
/// # The defect is `clap_complete`'s split, not `zsh`'s
///
/// Only the `zsh` script carries it today, and that is not a property of `zsh`.
/// All four scripts come from one tree whose help strings are all folded.
/// `clap_complete` flattens **flag** and **subcommand** help and does not
/// flatten **positional** help, and its `bash`, `fish` and PowerShell
/// generators emit no positional description at all. So those three are clean
/// for a reason this repository does not control, and a dependency bump that
/// starts emitting positional help for `fish` or PowerShell brings the
/// identical defect there. That is why this reads all four rather than `zsh`,
/// and why the fix flattens the tree rather than patching the `zsh` path.
///
/// # Why the arms cannot go vacuous
///
/// An arm that finds no string to read would pass while reporting nothing, so
/// each one states what it expects to find. `bash` is the interesting one: its
/// generator writes word lists and no descriptions, so it writes **no**
/// single-quoted string at all, and the day it writes one is the day this arm
/// has to be given a quoting model of its own. Its double-quoted strings are
/// read on the same terms as the other three.
#[test]
fn no_quoted_string_of_a_completion_script_spans_a_newline() {
    // A floor rather than a count, so an ordinary edit to a help string does
    // not move it, and a generator that stopped writing descriptions does.
    const FLOOR: usize = 100;
    let single = |backslash_escapes_inside, doubled_quote_inside, hash_comments| Quoting {
        quote: b'\'',
        backslash_escapes_inside,
        doubled_quote_inside,
        hash_comments,
    };
    let rows = [
        // `bash`: no description, so no single-quoted string, and the word
        // lists it does write are double-quoted.
        ("bash", single(false, false, false), Some(0)),
        (
            "bash",
            Quoting {
                quote: b'"',
                backslash_escapes_inside: true,
                doubled_quote_inside: false,
                hash_comments: false,
            },
            None,
        ),
        ("zsh", single(false, false, false), None),
        ("fish", single(true, false, true), None),
        ("powershell", single(false, true, false), None),
    ];

    let mut wrong = String::new();
    for (shell, model, exactly) in rows {
        let script = ran(&["completions", shell]).text();
        let read = scan(&script, &model);
        let quote = model.quote as char;
        assert!(
            !read.unterminated,
            "the `{shell}` script ends inside a {quote}-quoted string, so this model does not \
             read it"
        );
        match exactly {
            Some(count) => assert_eq!(
                read.opens, count,
                "the `{shell}` script writes {} {quote}-quoted strings and this arm expects \
                 {count}; a generator that started writing descriptions here needs a quoting \
                 model of its own rather than this assertion relaxed",
                read.opens
            ),
            None => assert!(
                read.opens >= FLOOR,
                "the `{shell}` script writes only {} {quote}-quoted strings, so this arm read \
                 almost nothing",
                read.opens
            ),
        }
        if read.spanning.is_empty() {
            continue;
        }
        let newlines: usize = read.spanning.iter().map(|one| one.newlines).sum();
        wrong.push_str(&format!(
            "\n  `{shell}`: {} of {} {quote}-quoted strings span a newline, {newlines} newlines \
             in all\n",
            read.spanning.len(),
            read.opens
        ));
        for one in &read.spanning {
            wrong.push_str(&format!(
                "    line {}, {} newline(s): {}\n",
                one.line, one.newlines, one.head
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "a completion script carries a folded help string inside a quoted description, so a \
         shell prints it broken across lines:\n{wrong}"
    );
}
