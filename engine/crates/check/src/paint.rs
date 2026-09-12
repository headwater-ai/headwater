// SPDX-License-Identifier: Apache-2.0
//! The palette of [`headwater_paint`], re-exported, plus the three functions
//! that read a [`Severity`] and so could not move down with it.
//!
//! # Where the primitives went, and why they left
//!
//! [`ColorMode`], [`Role`], [`ROLES`], [`color_of`], [`paint`] and [`dim`]
//! lived in this file until #479. They sat here, beside [`crate::fill`],
//! because every renderer that wanted color sat above this crate:
//! [`Finding::render`](crate::finding::Finding::render) and
//! [`Run::render`](crate::Run::render) are here, and `headwater-query`'s
//! `explain` and `headwater-sweep`'s `plan` and `intake` all depend on this
//! crate already.
//!
//! That reasoning was sound and it had an edge nobody had read. This crate
//! depends on `headwater-census`, `headwater-resolve` and `headwater-lock`, so
//! a renderer inside any of those three could never name this module: cargo
//! refuses the cycle. Six of the eleven command lines whose interface contract
//! promises terminal sensing render in exactly those crates, starting with
//! `headwater derived`. So the primitives moved to `headwater-paint`, a leaf
//! with no dependencies, and this module re-exports them under the paths they
//! already had. No import anywhere in this workspace moved.
//!
//! # What stayed
//!
//! [`glyph`], [`severity_role`] and [`severity_word`] read [`Severity`], which
//! is a type of the check layer. Moving them would have moved `Severity` into
//! a crate about a terminal palette, which is the swap the move was avoiding
//! in the other direction.
//!
//! `engine/crates/cli/src/paint.rs` re-exports this module in turn, so
//! `headwater_cli::paint::*` still answers, and `main.rs` stays the one place
//! that decides *whether* a stream renders color at all — `stdout_color` and
//! its `stderr` twin stay in `headwater-cli`, because the terminal a process
//! is attached to is a fact about the binary rather than about a corpus.
//!
//! # `Plain` writes no escape sequence, ever
//!
//! [`glyph`] is the half of that fallback this module still owns: a literal
//! character, never wrapped in an escape sequence, that a caller prints beside
//! a severity word so the distinction survives where hue cannot carry it.

use crate::Severity;

pub use headwater_paint::{color_of, dim, paint, ColorMode, Role, ROLES};

/// The role a severity renders under, so a caller never hand-maps the three
/// [`Severity`] variants onto [`Role`] a second time.
#[must_use]
pub fn severity_role(severity: Severity) -> Role {
    match severity {
        Severity::Error => Role::Error,
        Severity::Warn => Role::Warn,
        Severity::Info => Role::Info,
    }
}

/// The literal glyph a severity prints beside its word under [`ColorMode::Plain`].
///
/// `✗`, `▲` and `·`, in [`Severity`]'s own order. Never wrapped in an escape
/// sequence: the character alone is the whole of what carries the
/// distinction where hue cannot.
#[must_use]
pub fn glyph(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "✗",
        Severity::Warn => "▲",
        Severity::Info => "·",
    }
}

/// A severity word, in the shape every renderer prints it: colored under
/// `Ansi`, and a glyph beside the bare word under `Plain`.
///
/// One function rather than a `paint`/`glyph` pair at every call site, because
/// [`Finding::render`](crate::finding::Finding::render) and
/// [`crate::Run::render`]'s severity counts both need exactly this pairing and
/// a third copy of the pairing is the drift this module exists to refuse.
#[must_use]
pub fn severity_word(severity: Severity, mode: ColorMode) -> String {
    let word = severity.to_string();
    match mode {
        ColorMode::Ansi => paint(severity_role(severity), &word, mode),
        ColorMode::Plain => format!("{} {word}", glyph(severity)),
    }
}

#[cfg(test)]
mod tests {
    use super::{glyph, paint, severity_word, ColorMode};
    use crate::Severity;

    /// The three glyphs are distinct, so a reader who cannot see color still
    /// tells the three severities apart.
    #[test]
    fn every_severity_has_its_own_glyph() {
        let glyphs = [
            glyph(Severity::Error),
            glyph(Severity::Warn),
            glyph(Severity::Info),
        ];
        assert_eq!(glyphs, ["✗", "▲", "·"]);
    }

    /// Under `Plain` the severity word carries its glyph and no escape
    /// sequence. Under `Ansi` it carries color and no glyph — the glyph is the
    /// fallback for where hue cannot render, not a second signal on top of it.
    #[test]
    fn a_severity_word_carries_a_glyph_in_plain_and_color_in_ansi() {
        let plain = severity_word(Severity::Error, ColorMode::Plain);
        assert_eq!(plain, "✗ error");
        assert!(!plain.contains('\x1b'));

        let ansi = severity_word(Severity::Error, ColorMode::Ansi);
        assert!(ansi.contains("error"), "{ansi:?}");
        assert!(ansi.starts_with("\x1b["), "{ansi:?}");
        assert!(!ansi.contains('✗'), "{ansi:?}");
    }

    /// The re-export answers under the path every caller in this workspace
    /// already wrote, so the move below this crate is invisible to them.
    ///
    /// Without this case the move is held only by whatever else happens to
    /// import `headwater_check::paint`, which is the silent-success shape: a
    /// re-export deleted by hand would fail to compile in some other crate and
    /// name a file nobody was reading.
    #[test]
    fn the_moved_primitives_still_answer_under_this_module() {
        assert_eq!(
            paint(super::Role::Path, "docs/spec/05.md", ColorMode::Plain),
            "docs/spec/05.md"
        );
        for role in super::ROLES {
            assert_eq!(paint(role, "text", ColorMode::Plain), "text");
        }
        assert_eq!(super::color_of(false, false, true), ColorMode::Ansi);
        assert_eq!(super::dim("text", ColorMode::Plain), "text");
    }
}
