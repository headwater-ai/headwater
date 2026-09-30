// SPDX-License-Identifier: Apache-2.0
//! The CPU time of each stage of one `headwater check`, for a measurement
//! build alone.
//!
//! [#1450](https://github.com/headwater-ai/headwater/issues/1450) found that a
//! warm `check --strict` cost about 150 ms past Phase A and that nothing said
//! where. `tools/measure/check-phases.sh` is the command that says it, and this
//! module writes what that script reads.
//!
//! It sits in this crate so that the stages inside [`crate::run`] and the
//! stages of the verb around it are counted by one clock. Every function here
//! does nothing unless the crate is built with the non-default `phase-times`
//! feature, which the `phase-times` feature of `headwater-cli` turns on.
//! `release.yml` builds the default features, so no released binary carries
//! it, and a shipped binary writes the same bytes to both streams that it
//! wrote before. No environment variable and no flag turns it on, because
//! `docs/interfaces/headwater-check.md` states that no environment variable
//! reaches `check`, and a flag would be surface that an adopter reads.
//!
//! With the feature, [`mark`] writes one line to standard error, in the shape
//! `phase <name> <microseconds>`. The figure is the CPU time of this thread
//! since the previous mark, or since the thread started for the first mark.
//! It is read from `/proc/thread-self/schedstat`, which counts user and system
//! time together. `getrusage` needs an `unsafe` call, and the workspace
//! forbids `unsafe`. `/proc/self/stat` counts in hundredths of a second, which
//! is coarser than most of the stages. On a host with no such file, the
//! feature writes nothing, and the script refuses for want of lines.
//!
//! The check path runs on one thread, so the time of the thread is the time of
//! the process. The script holds that claim: it refuses when the stages do not
//! sum to within 10% of what `time -p` measures for the process.

#[cfg(feature = "phase-times")]
mod on {
    use std::cell::Cell;
    use std::io::Write as _;

    thread_local! {
        static LAST: Cell<u64> = const { Cell::new(0) };
    }

    /// The CPU nanoseconds this thread has run, or `None` where the host
    /// does not say.
    fn now() -> Option<u64> {
        let text = std::fs::read_to_string("/proc/thread-self/schedstat").ok()?;
        text.split_whitespace().next()?.parse().ok()
    }

    pub(super) fn mark(name: &str) {
        let Some(now) = now() else {
            return;
        };
        let spent = now.saturating_sub(LAST.with(|last| last.replace(now)));
        let _ = writeln!(std::io::stderr().lock(), "phase {name} {}", spent / 1_000);
    }
}

/// Close the stage called `name`. With the `phase-times` feature, say on
/// standard error how much CPU time it took. Without the feature, nothing.
#[inline(always)]
pub fn mark(name: &str) {
    #[cfg(feature = "phase-times")]
    on::mark(name);
    #[cfg(not(feature = "phase-times"))]
    let _ = name;
}
