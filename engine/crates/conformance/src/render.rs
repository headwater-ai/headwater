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

use crate::{Cover, LevelState, Reading, Report, Verdict};
use std::fmt::Write;

impl Report {
    pub fn render(&self) -> String {
        let mut out = String::new();

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
            true => out.push_str("  this package declares no level\n"),
            false => {
                for state in &self.levels {
                    out.push_str(&level(state));
                }
            }
        }

        match &self.reached {
            Some(name) => {
                let _ = writeln!(
                    out,
                    "\n{name} reached, against {} {}",
                    self.package, self.version
                );
            }
            None => {
                let _ = writeln!(
                    out,
                    "\nno level reached, against {} {}",
                    self.package, self.version
                );
            }
        }

        // The sentence that keeps the number from reading as a grade. It is
        // printed on every run, including a run that reaches the top rung.
        out.push_str(
            "  a level states what this repository wired up. It measures nothing about the \
             corpus,\n  no key declares one, and a waiver moves the exit status and never the \
             level.\n",
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
    let _ = writeln!(out, "    {}", reading.rule.title);
    match &reading.verdict {
        Verdict::Met => {}
        Verdict::Gap(detail) => {
            let _ = writeln!(out, "    {detail}");
            let _ = writeln!(out, "    fix: {}", reading.rule.remediation);
        }
        Verdict::NotDecided(_) => {
            let _ = writeln!(
                out,
                "    no reading of a tree decides this, and no attestation record exists yet. \
                 It is neither met nor missing"
            );
            let _ = writeln!(out, "    fix: {}", reading.rule.remediation);
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
    let _ = writeln!(
        out,
        "  {} {} — {verdict}, {} of {} rules met",
        state.name,
        state.title,
        state.met,
        state.rules.len()
    );
    if state.gaps > 0 {
        let _ = writeln!(
            out,
            "    {} gap{}, {} of them waived",
            state.gaps,
            match state.gaps {
                1 => "",
                _ => "s",
            },
            state.waived
        );
    }
    if state.undecided > 0 {
        let _ = writeln!(
            out,
            "    {} rule{} no tree decides, so this rung waits on an attestation record",
            state.undecided,
            match state.undecided {
                1 => "",
                _ => "s",
            }
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
    let _ = writeln!(
        out,
        "  {} {} {}, {}, owner {}",
        waiver.rule,
        state,
        waiver.until,
        waiver.reason.name(),
        waiver.owner
    );
    if let Some(note) = &waiver.note {
        let _ = writeln!(out, "    {note}");
    }
    if matches!(reading.cover, Cover::Expired(_)) {
        let _ = writeln!(
            out,
            "    it covers nothing. The rule under an expired waiver is evaluated as though no \
             waiver stood there"
        );
    }
    out
}
