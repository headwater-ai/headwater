// SPDX-License-Identifier: Apache-2.0
//! The identity of the emitters this build carries.
//!
//! # Why a number
//!
//! `generate --check` compares bytes. It reads what is committed, it builds
//! what this corpus produces, and it reports the difference. That comparison
//! answers one question — "is this artifact current" — only while both sides
//! were produced by the same emitters. Two people on two engine versions break
//! that premise silently: A regenerates on one engine, B checks on another, and
//! `--check` says "stale, regenerate" to each of them in turn. B regenerates, A
//! then checks and is told the same thing, and the pair flip-flops forever over
//! a corpus that never moved.
//!
//! Nothing in the bytes distinguishes the two causes. A projection that differs
//! because the corpus moved and a projection that differs because an emitter
//! moved are the same diff. So the engine records which emitters wrote an
//! artifact, in the artifact set's own descriptor, and a `--check` reads that
//! number before it compares anything. On a difference it says so and it says
//! nothing about a remedy, because the remedy depends on which cause it is and
//! this run cannot tell.
//!
//! # Why it is maintained by hand
//!
//! This is [`headwater_resolve::rules::RULE_SET`] one layer over, and that
//! constant's doc comment carries the argument for the grain: a package version
//! moves on a change that touches no emitter, and it stays put on an emitter
//! moved between crates. Read it there rather than in a second copy here.
//!
//! Nothing derives this number and nothing catches a raise that was skipped.
//! That gap is [HW-OBL-0074](../../../../docs/obligations/0074-a-check-version-is-raised-by-hand-and-nothing-catches-a-stale.md),
//! recorded there against the check cache's rule digest. This constant inherits
//! the same gap rather than closing it.

/// The emitter set this build carries.
///
/// **Any edit that changes the bytes an emitter produces for a corpus it
/// produced other bytes for before must increment this number.** That is every
/// emitter this crate reaches: the shelf index, the shelf sections, the site
/// navigation, the verb index, the probe result, the register, each declared
/// export profile and the corpus descriptor itself. An edit that changes only
/// which corpora an emitter runs over does not need the raise, because the
/// bytes for a corpus it already ran over do not move. An edit to a comment,
/// to a name, or to the order in which the plan is built does not need it
/// either.
///
/// A raise moves `.headwater/corpus.json` in every repository that upgrades,
/// and [the interface contract](../../../../docs/interfaces/headwater-generate.md)
/// states that consequence for the adopter who meets it. `headwater_lock`'s
/// `FORMAT` already behaves this way for the same reason.
pub const EMITTER_SET: u32 = 1;
