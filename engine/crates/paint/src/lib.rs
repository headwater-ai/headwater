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

/// `text` with every control character written as a visible backslash form.
///
/// A file name, an identifier and every other value a document supplies reach
/// a terminal, a CI log and a Markdown table through the prose formats, and a
/// control character in one of them is an instruction to the reader's tool
/// rather than text (#1970). An escape sequence recolors or clears a terminal,
/// and a line feed opens a new line, which GitHub Actions reads as a workflow
/// command when it starts `::`. So every renderer that prints such a value as
/// prose passes it through here first.
///
/// The set is the whole category [`char::is_control`] names: C0, DEL and C1.
/// C1 is included because U+009B is a control sequence introducer on some
/// terminals, the same reason `headwater infer` quotes by category rather than
/// by character. A line feed and a tab are in the set: a value never carries a
/// legitimate line break, and a tab moves a column. `\n`, `\r` and `\t` keep
/// their usual names, and every other character is written `\xHH` below U+0100
/// and `\u{HHHH}` above it.
///
/// A backslash is not escaped, so a clean path is written back unchanged and
/// borrowed. The cost is one ambiguity: a name that holds the two characters
/// `\` and `n` reads the same as a name that holds a line feed. A reader who
/// needs the exact value reads JSON or SARIF, which escape with their own
/// string syntax and give it back exactly when parsed.
#[must_use]
pub fn visible(text: &str) -> std::borrow::Cow<'_, str> {
    if !text.chars().any(char::is_control) {
        return std::borrow::Cow::Borrowed(text);
    }
    let mut out = String::with_capacity(text.len() + 8);
    for char in text.chars() {
        match char {
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            control if control.is_control() && (control as u32) < 0x100 => {
                out.push_str(&format!("\\x{:02x}", control as u32));
            }
            control if control.is_control() => {
                out.push_str(&format!("\\u{{{:04x}}}", control as u32));
            }
            other => out.push(other),
        }
    }
    std::borrow::Cow::Owned(out)
}

#[cfg(test)]
mod tests {
    use super::{color_of, dim, paint, visible, ColorMode, Role, ROLES};

    /// Each control character is written in its visible form, and a value
    /// with none comes back borrowed and unchanged (#1970).
    #[test]
    fn visible_writes_every_control_character_as_backslash_text() {
        let cases = [
            ("docs/a.md", "docs/a.md"),
            ("a\\nb", "a\\nb"),
            ("a\nb", "a\\nb"),
            ("a\rb", "a\\rb"),
            ("a\tb", "a\\tb"),
            ("\u{1b}[31m", "\\x1b[31m"),
            ("\u{0}", "\\x00"),
            ("\u{7f}", "\\x7f"),
            ("\u{9b}", "\\x9b"),
            ("caf\u{e9}", "caf\u{e9}"),
            ("x\u{1b}[2J\u{1b}[31my", "x\\x1b[2J\\x1b[31my"),
        ];
        for (given, written) in cases {
            assert_eq!(visible(given), written, "{given:?}");
        }
        assert!(matches!(
            visible("docs/a.md"),
            std::borrow::Cow::Borrowed(_)
        ));
    }

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
