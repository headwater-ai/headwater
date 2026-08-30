// SPDX-License-Identifier: Apache-2.0
//! The palette [`HW-DR-0045`](../../../../docs/decisions/0045-coloring-the-cli-and-where-the-banner-goes.md)
//! rules on, and the pure functions every renderer applies it through.
//!
//! # Why this lives in `headwater-check` and not in `headwater-cli`
//!
//! [`Finding::render`](crate::finding::Finding::render) and
//! [`Run::render`](crate::Run::render) are in this crate, and
//! `headwater-query`'s `explain` and `headwater-sweep`'s `plan` and `intake`
//! each depend on this crate already. `headwater-cli` depends on all three the
//! other way, so a type or a function every one of them needs to color a
//! finding, a path or a heading can only live where every one of them can
//! reach it without a cycle — here, beside [`crate::fill`], which the same
//! four crates already share for the one fold this engine has.
//!
//! `engine/crates/cli/src/paint.rs` re-exports everything below under
//! `headwater_cli::paint::*`, so a caller who wrote `paint::Role::Error` before
//! this module existed still compiles unchanged, and `main.rs` stays the one
//! place that decides *whether* a stream renders color at all —
//! [`stdout_color`](../../../../engine/crates/cli/src/paint.rs) and its
//! `stderr` twin stay in `headwater-cli`, because the terminal a process is
//! attached to is a fact about the binary, not about a corpus.
//!
//! # `ColorMode` is a parameter, never a read
//!
//! Every function here is pure: given the same `Role` or the same
//! [`Severity`](crate::Severity) and the same [`ColorMode`], it returns the
//! same bytes. Nothing in this module opens a stream or reads an environment
//! variable, which is what makes `Finding::render`, `Run::render`, `explain`'s
//! own renderer and the two sweep renderers unit-testable with a `Case` table
//! and no real terminal, the pattern `paint::color_of`'s own tests already set
//! for `--wide`/`COLUMNS`.
//!
//! # `Plain` writes no escape sequence, ever
//!
//! [`HW-DR-0045`](../../../../docs/decisions/0045-coloring-the-cli-and-where-the-banner-goes.md)
//! is explicit that the fallback is "no escape sequence at all", and
//! `NO_COLOR_TEXT` (`engine/crates/cli/src/lib.rs`) promises the same thing to
//! every caller of `--no-color`. So [`paint`] and [`dim`] write `text` back
//! unchanged under [`ColorMode::Plain`], the same as before this module
//! existed. [`glyph`] is the other half of the fallback: a literal character,
//! never wrapped in an escape sequence, that a caller prints beside a severity
//! word so the distinction survives even where hue cannot carry it.

use crate::Severity;

/// Whether a stream renders the palette below, or its plain-text fallback.
///
/// Moved here from `headwater-cli` unchanged — see the module comment for why
/// the crate moved and `engine/crates/cli/src/paint.rs` for the re-export that
/// keeps every existing caller compiling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    Ansi,
    Plain,
}

/// The mode `--no-color`, `NO_COLOR` and a stream's own terminal state come
/// to, decided once so every caller reads the same answer the same way.
///
/// `--no-color` or a set `NO_COLOR` forces [`ColorMode::Plain`] regardless of
/// `is_terminal`. There is no third state: a caller who wants color forced
/// into a pipe has no lever here.
#[must_use]
pub fn color_of(no_color_flag: bool, no_color_env: bool, is_terminal: bool) -> ColorMode {
    if no_color_flag || no_color_env {
        return ColorMode::Plain;
    }
    match is_terminal {
        true => ColorMode::Ansi,
        false => ColorMode::Plain,
    }
}

/// One semantic role [`HW-DR-0045`](../../../../docs/decisions/0045-coloring-the-cli-and-where-the-banner-goes.md)'s
/// palette names.
///
/// `Verb` carries two rows of the decision's table at once — "a verb name or a
/// `fix:` label" is one role, green and bold, so a second variant for the
/// second noun would be a distinction the palette itself does not draw.
/// `Heading` and structural or already-stated text are the two rows with no
/// hue at all — a heading is bold in the default color and the other is dim in
/// it — so the second reaches no variant here: [`dim`] is a sibling function
/// rather than a seventh role, the same shape `engine/crates/cli/src/paint.rs`
/// already set before this module existed.
#[derive(Debug, Clone, Copy)]
pub enum Role {
    Error,
    Warn,
    Info,
    /// A file path or a flag name.
    Path,
    /// A verb name or a `fix:` label.
    Verb,
    /// An obligation or adoption-task identifier (`OB-…`, `AD-…`).
    Obligation,
    /// A section heading.
    Heading,
}

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

/// `text`, painted for `role` under `mode`.
///
/// `Ansi` writes the standard SGR codes the decision names, which are
/// remapped by whatever theme the caller's terminal already runs — the reason
/// the decision refuses a truecolor hex. `Plain` writes `text` back unchanged:
/// no escape sequence, on the rule the module comment states.
#[must_use]
pub fn paint(role: Role, text: &str, mode: ColorMode) -> String {
    let (open, close) = match (role, mode) {
        (Role::Error, ColorMode::Ansi) => ("\x1b[1;31m", "\x1b[0m"),
        (Role::Warn, ColorMode::Ansi) => ("\x1b[1;33m", "\x1b[0m"),
        (Role::Info, ColorMode::Ansi) => ("\x1b[34m", "\x1b[0m"),
        (Role::Path, ColorMode::Ansi) => ("\x1b[36m", "\x1b[0m"),
        (Role::Verb, ColorMode::Ansi) => ("\x1b[1;32m", "\x1b[0m"),
        (Role::Obligation, ColorMode::Ansi) => ("\x1b[35m", "\x1b[0m"),
        (Role::Heading, ColorMode::Ansi) => ("\x1b[1m", "\x1b[0m"),
        (_, ColorMode::Plain) => ("", ""),
    };
    format!("{open}{text}{close}")
}

/// Dim weight, the one part of the `Plain` fallback that is not color.
#[must_use]
pub fn dim(text: &str, mode: ColorMode) -> String {
    match mode {
        ColorMode::Ansi => format!("\x1b[2m{text}\x1b[0m"),
        ColorMode::Plain => text.to_string(),
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
    use super::{color_of, dim, glyph, paint, severity_word, ColorMode, Role};
    use crate::Severity;

    #[test]
    fn color_is_plain_off_a_terminal_and_ansi_on_one_unless_overridden() {
        assert_eq!(color_of(false, false, false), ColorMode::Plain);
        assert_eq!(color_of(false, false, true), ColorMode::Ansi);
        assert_eq!(
            color_of(true, false, true),
            ColorMode::Plain,
            "--no-color wins"
        );
        assert_eq!(
            color_of(false, true, true),
            ColorMode::Plain,
            "NO_COLOR wins"
        );
        assert_eq!(color_of(true, true, false), ColorMode::Plain);
    }

    /// `Plain` never writes an escape sequence, for every role.
    #[test]
    fn plain_writes_no_escape_sequence_for_any_role() {
        for role in [
            Role::Error,
            Role::Warn,
            Role::Info,
            Role::Path,
            Role::Verb,
            Role::Obligation,
            Role::Heading,
        ] {
            assert_eq!(paint(role, "text", ColorMode::Plain), "text");
        }
        assert_eq!(dim("text", ColorMode::Plain), "text");
    }

    /// `Ansi` wraps the text in an SGR pair that closes with a reset, and
    /// changes no byte of the text itself.
    #[test]
    fn ansi_wraps_every_role_in_an_opening_and_a_reset() {
        for role in [
            Role::Error,
            Role::Warn,
            Role::Info,
            Role::Path,
            Role::Verb,
            Role::Obligation,
            Role::Heading,
        ] {
            let written = paint(role, "text", ColorMode::Ansi);
            assert!(written.starts_with("\x1b["), "{written:?}");
            assert!(written.ends_with("\x1b[0m"), "{written:?}");
            assert!(written.contains("text"), "{written:?}");
        }
        let dimmed = dim("text", ColorMode::Ansi);
        assert_eq!(dimmed, "\x1b[2mtext\x1b[0m");
    }

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
}
