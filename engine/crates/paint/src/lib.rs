// SPDX-License-Identifier: Apache-2.0
//! The palette [`HW-DR-0045`](../../../../docs/decisions/0045-coloring-the-cli-and-where-the-banner-goes.md)
//! rules on, and the pure functions every renderer applies it through.
//!
//! # Why this is a leaf crate and not a module of `headwater-check`
//!
//! These primitives lived in `headwater-check` until #479, beside
//! `headwater_check::fill`, because the four crates that needed them then —
//! `headwater-check` itself, `headwater-query`, `headwater-sweep` and
//! `headwater-cli` — all sit above that crate and reach it without a cycle.
//! That placement held for as long as every renderer that wanted color sat
//! above `headwater-check`, and it stopped holding the moment one did not.
//!
//! `headwater-check` depends on `headwater-census`, `headwater-resolve` and
//! `headwater-lock`. A renderer inside any of those three cannot name
//! `headwater_check::paint` at all: cargo refuses the cycle before a line
//! compiles. Six of the eleven command lines whose interface contract still
//! promises terminal sensing render inside exactly those three crates —
//! `headwater derived` in `headwater-census`, and `taxonomy validate`,
//! `resolve`, `publish`, `vendor` and `migrate` in `headwater-resolve` and
//! `headwater-lock`. Wiring any of them was not expensive, it was impossible,
//! and nothing recorded that until the dependency graph was read against
//! [HW-OBL-0180](../../../../docs/obligations/0180-a-renderer-s-color-mode-is-wired-at-a-call-site-that-no-type-forbids-from-being-wrong.md).
//!
//! So the palette moved to the bottom of the graph, where every renderer of
//! this engine reaches it. This crate depends on nothing, for the reason
//! `headwater-hash` and `headwater-mark` depend on nothing: a rule that two
//! components on opposite sides of the engine both read must have one
//! implementation, and a second copy of a palette is two palettes.
//!
//! # What stayed in `headwater-check`, and why
//!
//! `Severity` is a type of the check layer, so `glyph`, `severity_role` and
//! `severity_word` stayed with it rather than dragging `Severity` down here
//! behind them. `headwater_check::paint` re-exports everything below
//! unchanged, so every caller that wrote `headwater_check::paint::Role` before
//! the move still compiles — the same re-export chain
//! `engine/crates/cli/src/paint.rs` already ran one level up.
//!
//! Deciding *whether* a stream renders color stays in `headwater-cli`:
//! `stdout_color` and its `stderr` twin read the terminal a process is
//! attached to, which is a fact about the binary rather than about a corpus.
//!
//! # `ColorMode` is a parameter, never a read
//!
//! Every function here is pure: given the same [`Role`] and the same
//! [`ColorMode`], it returns the same bytes. Nothing here opens a stream or
//! reads an environment variable, which is what makes every renderer that
//! takes a mode unit-testable with a case table and no real terminal.
//!
//! That purity is also the defect [HW-OBL-0180](../../../../docs/obligations/0180-a-renderer-s-color-mode-is-wired-at-a-call-site-that-no-type-forbids-from-being-wrong.md)
//! records: a renderer threaded with a mode and a call site that hands it
//! [`ColorMode::Plain`] forever passes every headless test there is. What
//! catches that is `tools/engine/color-fixtures.sh`, which attaches a
//! pseudo-terminal and counts escape bytes, and nothing else in this
//! repository can.
//!
//! # `Plain` writes no escape sequence, ever
//!
//! [`HW-DR-0045`](../../../../docs/decisions/0045-coloring-the-cli-and-where-the-banner-goes.md)
//! is explicit that the fallback is "no escape sequence at all", and
//! `NO_COLOR_TEXT` (`engine/crates/cli/src/lib.rs`) promises the same thing to
//! every caller of `--no-color`. So [`paint`] and [`dim`] write `text` back
//! unchanged under [`ColorMode::Plain`].

/// Whether a stream renders the palette below, or its plain-text fallback.
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
/// rather than a seventh role.
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

/// Every [`Role`], for a case table that would otherwise hand-keep its own copy
/// of the variants.
///
/// [HW-OBL-0172](../../../../docs/obligations/0172-nine-hand-kept-constants-enumerate-an-enum-and-nothing-holds-one-against-the-variants.md)
/// records the shape this is: a hand-typed array whose length is part of its
/// type, so a new variant leaves it short and compiles. It ships discharged
/// under that record's own Discharge clause — `roles_carries_every_variant_once`
/// maps this array through an exhaustive `match` on [`Role`], so a variant
/// added to the enum and left out of here stops the crate compiling. One held
/// list replaces the two unheld ones the tests below used to type by hand, and
/// a renderer's color case table reads this rather than growing a third.
pub const ROLES: [Role; 7] = [
    Role::Error,
    Role::Warn,
    Role::Info,
    Role::Path,
    Role::Verb,
    Role::Obligation,
    Role::Heading,
];

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

#[cfg(test)]
mod tests {
    use super::{color_of, dim, paint, ColorMode, Role, ROLES};

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
        for role in ROLES {
            assert_eq!(paint(role, "text", ColorMode::Plain), "text");
        }
        assert_eq!(dim("text", ColorMode::Plain), "text");
    }

    /// `Ansi` wraps the text in an SGR pair that closes with a reset, and
    /// changes no byte of the text itself.
    #[test]
    fn ansi_wraps_every_role_in_an_opening_and_a_reset() {
        for role in ROLES {
            let written = paint(role, "text", ColorMode::Ansi);
            assert!(written.starts_with("\x1b["), "{written:?}");
            assert!(written.ends_with("\x1b[0m"), "{written:?}");
            assert!(written.contains("text"), "{written:?}");
        }
        let dimmed = dim("text", ColorMode::Ansi);
        assert_eq!(dimmed, "\x1b[2mtext\x1b[0m");
    }

    /// [`ROLES`] carries every variant of [`Role`], once each.
    ///
    /// The `match` below is the discharge HW-OBL-0172 names, and it works at
    /// compile time rather than here: a variant added to [`Role`] leaves this
    /// `match` non-exhaustive and the crate stops building. What this case
    /// itself adds is the other half — that no variant is written twice and
    /// none is silently dropped for a duplicate.
    #[test]
    fn roles_carries_every_variant_once() {
        let mut seen = [false; ROLES.len()];
        for role in ROLES {
            let index = match role {
                Role::Error => 0,
                Role::Warn => 1,
                Role::Info => 2,
                Role::Path => 3,
                Role::Verb => 4,
                Role::Obligation => 5,
                Role::Heading => 6,
            };
            assert!(!seen[index], "{role:?} appears twice in ROLES");
            seen[index] = true;
        }
        assert!(
            seen.iter().all(|one| *one),
            "ROLES misses a variant of Role: {seen:?}"
        );
    }
}
