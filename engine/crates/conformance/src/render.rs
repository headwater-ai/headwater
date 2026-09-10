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
//! # Every line of prose is filled, and the fill is not in this file
//!
//! [`WIDTH`] is the only width this file names, and no call site names a second
//! one: [`block`] derives the continuation indent from the label it is given.
//! The report carries text three different authors wrote — this engine, the
//! package, and the adopter who writes a waiver note — and the last of those has
//! no upper bound at all, so a line of this report is as long as whatever it
//! carries unless something fills it.
//!
//! **Both are `headwater_check::fill` now, and this file names them rather than
//! implementing them.** This module carried the only fill in the tree and the
//! instruction that came with it: when a later change filled the check report,
//! `WIDTH` and `block` were to move down into `headwater-check` and this crate
//! was to call them there. [#340](https://github.com/headwater-ai/headwater/issues/340)
//! is that change, and this is the record that the move happened. The check
//! report is laid out by `headwater_check::fill::filled`, from the same width
//! and the same rule about a word too long to break, and
//! `crates/conformance/fixtures/wrapped.report` did not move.
//!
//! `engine/crates/audit/src/render.rs` reaches the same end by hand-wrapping 29
//! string literals, and `engine/crates/census/src/census.rs` two. A pre-wrapped
//! literal is not an implementation of a fill, so there is still one of those.
//! The line that matters is this: **a report may hand-wrap every line or fill
//! every line, and it may not do both.** That is why the disclaimer below is one
//! logical sentence now rather than a literal carrying its own break.
//!
//! # Color is applied after the fill, never before it
//!
//! [`headwater_check::fill::block`] measures its label in **characters** to
//! derive the continuation indent, and it measures each line the same way to
//! decide where to break. An escape sequence introduced before that runs is
//! counted as text, so every break moves and the label hangs at the wrong
//! column. So [`labeled`] fills with the plain label and swaps the painted form
//! in afterwards, and every other painted token here is written outside the
//! fill. `headwater_query::route`'s renderer folds first and paints after for
//! the same reason.

use crate::{Cover, Installed, LevelState, PinCheck, Reading, Report, Verdict};
use headwater_check::paint::{dim, paint, ColorMode, Role};
use std::fmt::Write;

/// The widest line this report prints, in characters.
///
/// One constant, owned by [`headwater_check::fill`] and named here. It is a
/// constant rather than the width of whatever terminal ran the verb, for the
/// reason that module states.
pub use headwater_check::fill::WIDTH;

/// One block of the report: `indent` spaces, then `label`, then `text` filled to
/// [`WIDTH`].
///
/// The implementation is [`headwater_check::fill::block`] and the whole of what
/// this adds is the width. A continuation line is indented to
/// `indent + label.chars().count()`, so the label hangs the text under itself
/// and **no caller here names a width or a second indent**. Where a line opens
/// with an identifying token — `fix: `, a level name, a rule name — that token
/// is the label, and the text under it lines up.
///
/// **A title is the caller that reaches the newline handling.** `Rule.title` and
/// `Level.title` are written by `lib.rs:259` and `:294` as
/// `text_of(entry, "title").to_string()` with no collapse, and a YAML literal
/// block scalar — `title: |-` — is ordinary YAML for a package author. The other
/// three fields that arrive here cannot carry a newline: `statement`,
/// `remediation` and the waiver `note` each go through `collapse` at
/// `lib.rs:260`, `:261` and `:470`, which is `split_whitespace().join(" ")`.
/// Held by
/// `tests/render.rs::a_title_the_package_wrote_over_two_lines_is_filled_line_by_line`,
/// which drives it from YAML rather than by handing this function a string.
fn block(out: &mut String, indent: usize, label: &str, text: &str) {
    headwater_check::fill::block(out, indent, label, text, WIDTH);
}

/// [`block`] whose label carries a name this report paints.
///
/// `label` is what the fill measures and `name` is the substring of it that
/// takes the color, so `fix: ` is filled at five characters and printed with
/// three of them green. The swap is over the filled string rather than over the
/// argument, for the reason the module comment gives: a painted label handed to
/// the fill moves every line break under it.
///
/// The first occurrence of `name` in the filled block is the label, because the
/// fill writes `indent` spaces and then the label before any of the text. A
/// `name` that is empty paints nothing and is written through unchanged, so a
/// caller that has no token to color reaches [`block`] instead.
fn labeled(
    out: &mut String,
    indent: usize,
    label: &str,
    name: &str,
    text: &str,
    role: Role,
    mode: ColorMode,
) {
    let mut one = String::new();
    block(&mut one, indent, label, text);
    match name.is_empty() {
        true => out.push_str(&one),
        false => out.push_str(&one.replacen(name, &paint(role, name, mode), 1)),
    }
}

impl Report {
    /// The report a person reads, in `mode`.
    ///
    /// `mode` is the color decision the caller already made — `paint::
    /// stdout_color()` under `headwater-cli`, or [`ColorMode::Plain`] for the
    /// JSON format and for every recorded fixture. Nothing here reads a stream.
    pub fn render(&self, mode: ColorMode) -> String {
        let mut out = String::new();

        // Four of the header lines are identity rather than prose: a package
        // name, a version, a digest, a date. Each is one or two tokens that a
        // fill could only leave alone — a digest is 73 characters with its
        // indent — so they are written as they are and the boundary is stated
        // here. The fifth is [`pin`], which is a sentence, so it is filled.
        let _ = writeln!(out, "{}", paint(Role::Heading, "conformance", mode));
        let _ = writeln!(
            out,
            "  {} {}",
            paint(Role::Path, &self.package, mode),
            dim(&self.version, mode)
        );
        match &self.digest {
            Some(digest) => {
                let _ = writeln!(out, "  {}", dim(digest, mode));
                block(&mut out, 2, "", &pin(&self.pin, digest));
            }
            None => out.push_str("  no digest pinned\n"),
        }
        let _ = writeln!(out, "  at {}", dim(&self.now.to_string(), mode));

        let _ = writeln!(out, "\n{}", paint(Role::Heading, "rules", mode));
        for reading in &self.readings {
            out.push_str(&rule(reading, mode));
        }

        let _ = writeln!(out, "\n{}", paint(Role::Heading, "levels", mode));
        match self.levels.is_empty() {
            true => block(&mut out, 2, "", "this package declares no level"),
            false => {
                for state in &self.levels {
                    out.push_str(&level(state, mode));
                }
            }
        }

        // The verdict line, and the one place a level name is the subject of a
        // sentence rather than a label. The name takes the color and the rest of
        // the sentence does not, so a reader scanning for the rung finds it.
        out.push('\n');
        match &self.reached {
            Some(name) => labeled(
                &mut out,
                0,
                "",
                name,
                &format!("{name} reached, against {} {}", self.package, self.version),
                Role::Verb,
                mode,
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
            let _ = writeln!(out, "\n{}", paint(Role::Heading, "waivers", mode));
            for reading in waived {
                out.push_str(&waiver(reading, mode));
            }
        }

        out
    }
}

/// The line under the digest that says whether this run checked it.
///
/// **It is printed only where a digest is printed.** A header that already says
/// `no digest pinned` states no number, so there is nothing there to qualify,
/// and a second sentence about a digest nobody wrote is noise.
///
/// Where a rule of the set reads the pin, this names it and stops: that rule's
/// own line below carries `met` or `gap`, and a second statement of the same
/// verdict here would be a second answer to one question. Where no rule reads
/// it, this reports what the installed package declares and compares nothing —
/// the two numbers are put in front of the reader and no rung and no exit
/// status moves, which is the whole of what this change does.
fn pin(check: &PinCheck, pinned: &str) -> String {
    match check {
        PinCheck::By(rule) => {
            format!("checked against the installed release record by `{rule}`")
        }
        PinCheck::Unchecked(installed) => format!(
            "not checked by any rule of this set: {}",
            match installed {
                Installed::Declares(digest) if digest == pinned =>
                    "the installed release record declares the same digest".to_string(),
                Installed::Declares(digest) =>
                    format!("the installed release record declares {digest}"),
                Installed::NoRecord =>
                    "the installed package carries no release record, so no published artifact \
                     stands behind it"
                        .to_string(),
                Installed::Absent => "no package of that name is installed".to_string(),
                Installed::Unreadable(says) =>
                    format!("the installed release record does not read: {says}"),
            }
        ),
    }
}

/// One rule of the set, with its verdict.
///
/// The verdict takes the color and the rule name does not, which is the
/// convention `Finding::render` already sets: the name is what a reader looks
/// up and the verdict is what a reader scans for. `fix` is green there and it is
/// green here.
fn rule(reading: &Reading, mode: ColorMode) -> String {
    let mut out = String::new();
    let (mark, role) = match &reading.verdict {
        Verdict::Met => ("met", Role::Verb),
        Verdict::Gap(_) => ("gap", Role::Warn),
        Verdict::NotDecided(_) => ("not decided", Role::Info),
    };
    let _ = writeln!(out, "  {} {}", reading.rule.name, paint(role, mark, mode));
    block(&mut out, 4, "", &reading.rule.title);
    match &reading.verdict {
        Verdict::Met => {}
        Verdict::Gap(detail) => {
            block(&mut out, 4, "", detail);
            labeled(
                &mut out,
                4,
                "fix: ",
                "fix",
                &reading.rule.remediation,
                Role::Verb,
                mode,
            );
        }
        Verdict::NotDecided(_) => {
            block(
                &mut out,
                4,
                "",
                "no reading of a tree decides this, and no attestation record exists yet. It is \
                 neither met nor missing",
            );
            labeled(
                &mut out,
                4,
                "fix: ",
                "fix",
                &reading.rule.remediation,
                Role::Verb,
                mode,
            );
        }
    }
    out
}

fn level(state: &LevelState, mode: ColorMode) -> String {
    let mut out = String::new();
    let verdict = match state.reached {
        true => "reached",
        false => "not reached",
    };
    labeled(
        &mut out,
        2,
        &format!("{} ", state.name),
        &state.name,
        &format!(
            "{} — {verdict}, {} of {} rules met",
            state.title,
            state.met,
            state.rules.len()
        ),
        match state.reached {
            true => Role::Verb,
            false => Role::Warn,
        },
        mode,
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

fn waiver(reading: &Reading, mode: ColorMode) -> String {
    let mut out = String::new();
    let (waiver, state, role) = match &reading.cover {
        Cover::None => return out,
        Cover::Live(waiver) => (waiver, "stands until", Role::Obligation),
        // An expired waiver covers nothing, so it is the one line of this
        // report that reads as an error rather than as a state.
        Cover::Expired(waiver) => (waiver, "EXPIRED on", Role::Error),
    };
    labeled(
        &mut out,
        2,
        &format!("{} ", waiver.rule),
        &waiver.rule,
        &format!(
            "{state} {}, {}, owner {}",
            waiver.until,
            waiver.reason.name(),
            waiver.owner
        ),
        role,
        mode,
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
