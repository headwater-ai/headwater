// SPDX-License-Identifier: Apache-2.0
//! The report a person reads.
//!
//! One format and no flag selects another. The reader of this verb is somebody
//! closing a gap, and each gap is printed with the remediation the package
//! wrote for it.
//!
//! **The package identity is printed above every rung.** A level is a subset of
//! the rule set of one package, so a rung with no package beside it names
//! nothing. An adopter who forks the package to change the rule set moves the
//! name and the digest on these lines.
//!
//! # Every line of prose is filled, and the width is one constant
//!
//! [`WIDTH`] is the only width this file names, and no call site names a second
//! one: [`block`] derives the continuation indent from the label it is given.
//! The report carries text three different authors wrote — this engine, the
//! package, and the adopter who writes a waiver note — and the last of those has
//! no upper bound at all, so a line of this report is as long as whatever it
//! carries unless something fills it.
//!
//! **This is the only fill in the tree, and it belongs to this report.** The
//! other report that prints a `fix:` line is
//! [`headwater_check::finding`](../../check/src/finding.rs), at indent 2, from
//! text this engine authored as one short sentence per rule. That one is not
//! filled here: its recorded block is 793 lines with 161 already over this
//! width, and re-recording it inside a change about conformance would hide this
//! one. **When a later change fills the check report, `WIDTH` and [`block`] move
//! down into `headwater-check` and this crate calls them there** — this crate
//! already depends on that one and the reverse direction is a cycle the compiler
//! refuses, so the move is available and a second copy is not needed. Move them
//! rather than writing another.
//!
//! `engine/crates/audit/src/render.rs` reaches the same end by hand-wrapping 29
//! string literals, and `engine/crates/census/src/census.rs` two. A pre-wrapped
//! literal is not an implementation of a fill, so there is still one of those.
//! The line that matters is this: **a report may hand-wrap every line or fill
//! every line, and it may not do both.** That is why the disclaimer below is one
//! logical sentence now rather than a literal carrying its own break.

use crate::{Cover, LevelState, Reading, Report, Verdict};
use std::fmt::Write;

/// The widest line this report prints, in characters.
///
/// It is a constant rather than the width of whatever terminal ran the verb. A
/// recorded block compared against the running terminal's width compares
/// nothing, and the engine names no `libc`, no `terminal_size` and no
/// `unicode-width` in its lock.
pub const WIDTH: usize = 80;

/// One block of the report: `indent` spaces, then `label`, then `text` filled to
/// [`WIDTH`].
///
/// A continuation line is indented to `indent + label.chars().count()`, so the
/// label hangs the text under itself and **no caller names a width or a second
/// indent**. Where a line opens with an identifying token — `fix: `, a level
/// name, a rule name — that token is the label, and the text under it lines up.
///
/// Four things it does, each of which a reader of the output can see:
///
/// 1. `text` is split on `'\n'` first and each piece is filled on its own.
///    **A title is the caller that reaches this.** `Rule.title` and
///    `Level.title` are written by `lib.rs:259` and `:294` as
///    `text_of(entry, "title").to_string()` with no collapse, and a YAML literal
///    block scalar — `title: |-` — is ordinary YAML for a package author. The
///    other three fields that arrive here cannot carry a newline: `statement`,
///    `remediation` and the waiver `note` each go through `collapse` at
///    `lib.rs:260`, `:261` and `:470`, which is `split_whitespace().join(" ")`.
///    Held by
///    `tests/render.rs::a_title_the_package_wrote_over_two_lines_is_filled_line_by_line`,
///    which drives it from YAML rather than by handing this function a string.
/// 2. The fill is greedy on whitespace runs, and a whitespace run becomes one
///    space. That normalization is invisible in the YAML-folded strings a rule
///    set ships.
/// 3. **The count is in `char`s.** `str::len()` is bytes, and a level line
///    carries an em dash of three bytes, so a byte count would break a line two
///    characters early. A codepoint count is the approximation this engine
///    makes: it claims no display width and no grapheme boundary, which is the
///    right claim for a report of ASCII identifiers and English prose.
/// 4. A word longer than the column it lands in goes on its own line, whole, and
///    overflows. Nothing here breaks inside a word, hyphenates or truncates: a
///    digest, a path or a URL that arrived intact leaves intact.
fn block(out: &mut String, indent: usize, label: &str, text: &str) {
    let cont = " ".repeat(indent + label.chars().count());
    let mut opening = format!("{}{label}", " ".repeat(indent));
    for paragraph in text.split('\n') {
        let mut line = opening.clone();
        let mut filled = false;
        for word in paragraph.split_whitespace() {
            match filled {
                false => {
                    line.push_str(word);
                    filled = true;
                }
                true => match line.chars().count() + 1 + word.chars().count() <= WIDTH {
                    true => {
                        line.push(' ');
                        line.push_str(word);
                    }
                    false => {
                        out.push_str(&line);
                        out.push('\n');
                        line = cont.clone();
                        line.push_str(word);
                    }
                },
            }
        }
        out.push_str(&line);
        out.push('\n');
        opening = cont.clone();
    }
}

impl Report {
    pub fn render(&self) -> String {
        let mut out = String::new();

        // The four header lines are identity rather than prose: a package name,
        // a version, a digest, a date. Each is one or two tokens that a fill
        // could only leave alone — a digest is 73 characters with its indent —
        // so they are written as they are and the boundary is stated here.
        out.push_str("conformance\n");
        let _ = writeln!(out, "  {} {}", self.package, self.version);
        match &self.digest {
            Some(digest) => {
                let _ = writeln!(out, "  {digest}");
            }
            None => out.push_str("  no digest pinned\n"),
        }
        let _ = writeln!(out, "  at {}", self.now);

        out.push_str("\nrules\n");
        for reading in &self.readings {
            out.push_str(&rule(reading));
        }

        out.push_str("\nlevels\n");
        match self.levels.is_empty() {
            true => block(&mut out, 2, "", "this package declares no level"),
            false => {
                for state in &self.levels {
                    out.push_str(&level(state));
                }
            }
        }

        out.push('\n');
        match &self.reached {
            Some(name) => block(
                &mut out,
                0,
                "",
                &format!("{name} reached, against {} {}", self.package, self.version),
            ),
            None => block(
                &mut out,
                0,
                "",
                &format!(
                    "no level reached, against {} {}",
                    self.package, self.version
                ),
            ),
        }

        // The sentence that keeps the number from reading as a grade. It is
        // printed on every run, including a run that reaches the top rung. It is
        // written here as one logical sentence and filled below, because a
        // literal carrying its own break beside a fill is two answers to one
        // question.
        block(
            &mut out,
            2,
            "",
            "a level states what this repository wired up. It measures nothing about the corpus, \
             no key declares one, and a waiver moves the exit status and never the level.",
        );

        let waived: Vec<&Reading> = self
            .readings
            .iter()
            .filter(|reading| !matches!(reading.cover, Cover::None))
            .collect();
        if !waived.is_empty() {
            out.push_str("\nwaivers\n");
            for reading in waived {
                out.push_str(&waiver(reading));
            }
        }

        out
    }
}

fn rule(reading: &Reading) -> String {
    let mut out = String::new();
    let mark = match &reading.verdict {
        Verdict::Met => "met",
        Verdict::Gap(_) => "gap",
        Verdict::NotDecided(_) => "not decided",
    };
    let _ = writeln!(out, "  {} {}", reading.rule.name, mark);
    block(&mut out, 4, "", &reading.rule.title);
    match &reading.verdict {
        Verdict::Met => {}
        Verdict::Gap(detail) => {
            block(&mut out, 4, "", detail);
            block(&mut out, 4, "fix: ", &reading.rule.remediation);
        }
        Verdict::NotDecided(_) => {
            block(
                &mut out,
                4,
                "",
                "no reading of a tree decides this, and no attestation record exists yet. It is \
                 neither met nor missing",
            );
            block(&mut out, 4, "fix: ", &reading.rule.remediation);
        }
    }
    out
}

fn level(state: &LevelState) -> String {
    let mut out = String::new();
    let verdict = match state.reached {
        true => "reached",
        false => "not reached",
    };
    block(
        &mut out,
        2,
        &format!("{} ", state.name),
        &format!(
            "{} — {verdict}, {} of {} rules met",
            state.title,
            state.met,
            state.rules.len()
        ),
    );
    if state.gaps > 0 {
        block(
            &mut out,
            4,
            "",
            &format!(
                "{} gap{}, {} of them waived",
                state.gaps,
                match state.gaps {
                    1 => "",
                    _ => "s",
                },
                state.waived
            ),
        );
    }
    if state.undecided > 0 {
        block(
            &mut out,
            4,
            "",
            &format!(
                "{} rule{} no tree decides, so this rung waits on an attestation record",
                state.undecided,
                match state.undecided {
                    1 => "",
                    _ => "s",
                }
            ),
        );
    }
    out
}

fn waiver(reading: &Reading) -> String {
    let mut out = String::new();
    let (waiver, state) = match &reading.cover {
        Cover::None => return out,
        Cover::Live(waiver) => (waiver, "stands until"),
        Cover::Expired(waiver) => (waiver, "EXPIRED on"),
    };
    block(
        &mut out,
        2,
        &format!("{} ", waiver.rule),
        &format!(
            "{state} {}, {}, owner {}",
            waiver.until,
            waiver.reason.name(),
            waiver.owner
        ),
    );
    if let Some(note) = &waiver.note {
        block(&mut out, 4, "", note);
    }
    if matches!(reading.cover, Cover::Expired(_)) {
        block(
            &mut out,
            4,
            "",
            "it covers nothing. The rule under an expired waiver is evaluated as though no waiver \
             stood there",
        );
    }
    out
}
