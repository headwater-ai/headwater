// SPDX-License-Identifier: Apache-2.0
//! How wide the help is, and who lays it out.
//!
//! # `clap` cannot wrap in this workspace, and `term_width` will not make it
//!
//! `StyledStr::wrap` is `pub(crate) fn wrap(&mut self, _hard_width: usize) {}`
//! under `#[cfg(not(feature = "wrap_help"))]`, and this workspace takes `clap`
//! with `derive` alone. So `Command::term_width` sets a number every renderer
//! reads and nothing acts on, and every help string reached a caller on one
//! line however long it was — 1,126 columns at the widest.
//!
//! Taking the `wrap_help` feature is the route the crate offers and it is the
//! wrong one here. It pulls `terminal_size`, which measures the terminal the
//! process is attached to, and a width that depends on the terminal makes a
//! piped run and a run under a terminal write different bytes. Every recorded
//! fixture and every test that reads this help would then be reading the
//! terminal of whoever ran it.
//!
//! So the strings are folded here, before `clap` sees them, at a width this
//! module decides. Nothing in the path reads a terminal: `dimensions()` is
//! `(None, None)` without `wrap_help`, so `clap` asks no question about the
//! stream it is writing to, and neither does this.
//!
//! # The one indent, and how it is known
//!
//! [`painted`] declares `next_line_help` on every command of the tree, which
//! puts an argument's help on the line under the argument rather than in a
//! column whose width is a function of the longest argument at that node. The
//! indent is then `TAB` plus `NEXT_LINE_INDENT` — two spaces and eight — at
//! every node, so [`INDENT`] is a constant rather than a computation, and one
//! folded string is right wherever `clap` decides to print it.
//!
//! # What `clap` appends after a help string, and why folding has to know
//!
//! `HelpTemplate::help` writes the string this module folded and then appends
//! the spec values — `[default: 0]`, `[possible values: …]`, `[aliases: …]` —
//! on the same line after a space. A fold that did not account for them would
//! be right about the text and wrong about the line. [`reserved`] measures what
//! is coming and [`fold_at`] keeps the last word of the text and that suffix on
//! one line together.

use clap::{Arg, ArgAction, Command};

/// The width and the fill, which live in `headwater_check::fill` and are named
/// here so that a caller of this module keeps writing `paint::WIDTH`.
///
/// They moved down when the check report gained a layout: the report and the
/// help are laid out by one implementation, and `headwater-check` is the crate
/// every report composer can reach. Nothing is re-implemented here.
pub use headwater_check::fill::{fold, fold_at, WIDEST, WIDTH};

/// The column an argument's help starts at, at every node of the tree.
///
/// `clap`'s `TAB` is two spaces and its `NEXT_LINE_INDENT` is eight, and
/// [`painted`] declares `next_line_help` everywhere so that the pair is the
/// whole indent at every node.
pub const INDENT: usize = 10;

/// The width this process lays the help out at.
///
/// `COLUMNS` is read here and nowhere else in this binary, and only when the
/// command line carries `--wide`. The scan is over the raw arguments because
/// the answer is needed to build the tree that parses them: `clap` renders help
/// inside the parse, out of strings that were folded before it started.
pub fn width() -> usize {
    // The variable is read inside the `true` arm rather than beside the scan,
    // so that a run with no `--wide` on its command line makes no call at all.
    // Two interface contracts say what reaches this binary out of the
    // environment, and the honest sentence is shorter for it.
    match std::env::args_os().any(|one| one == "--wide") {
        false => WIDTH,
        true => width_of(true, std::env::var("COLUMNS").ok().as_deref()),
    }
}

/// The width a `--wide` and a `COLUMNS` reading come to.
///
/// Without `--wide` the answer is [`WIDTH`] and the reading is not consulted, so
/// a run in a 40-column terminal and a run piped into a file write the same
/// bytes. With it the reading is held to `[WIDTH, WIDEST]`: a narrower terminal
/// gets 80 because the text was written to be read at 80, and a wider one gets
/// 120 because a line of prose past that is harder to read rather than easier.
/// A `COLUMNS` that is absent or is not a number is the same answer as no
/// `--wide` at all.
pub fn width_of(wide: bool, columns: Option<&str>) -> usize {
    if !wide {
        return WIDTH;
    }
    match columns.and_then(|text| text.trim().parse::<usize>().ok()) {
        Some(number) => number.clamp(WIDTH, WIDEST),
        None => WIDTH,
    }
}

/// The tree, with every string folded and every node laid out the same way.
///
/// It walks the whole tree rather than the verbs, so a second word and an
/// argument `clap` propagated are folded by the same code as a verb, and an
/// argument added later is folded without anybody remembering to.
pub fn painted(command: Command, width: usize) -> Command {
    let mut one = command.next_line_help(true);
    if let Some(about) = one.get_about().map(ToString::to_string) {
        one = one.about(fold(&about, width));
    }
    one = one.mut_args(|arg| {
        let Some(help) = arg.get_help().map(ToString::to_string) else {
            return arg;
        };
        let room = width.saturating_sub(INDENT);
        let tail = reserved(&arg);
        let folded = fold_at(&help, room, tail);
        arg.help(folded)
    });
    let names: Vec<String> = one
        .get_subcommands()
        .map(|inner| inner.get_name().to_string())
        .collect();
    for name in names {
        one = one.mut_subcommand(name, |inner| painted(inner, width));
    }
    one
}

/// The width of what `clap` appends after an argument's help, with its space.
///
/// It is the `spec_vals` of `HelpTemplate`, measured rather than rendered. The
/// `env` feature is not compiled in, so the environment-variable clause of that
/// function cannot occur here and is not measured. Every other clause is, and
/// the whole surface is held to [`WIDTH`] by `tests/width.rs`, so a clause
/// measured wrong is reported as a wide line rather than passing quietly.
fn reserved(arg: &Arg) -> usize {
    let mut parts: Vec<usize> = Vec::new();

    // `clap` prints a default and a possible-value set for an argument that
    // takes a value and for no other. A flag declared `SetTrue` carries
    // `true`/`false` on its value parser and `false` as its default, and
    // neither reaches a caller. `get_num_args` is `None` until
    // `Command::build` and this runs before that, so the action is what
    // answers here. An alias is printed for a flag as well and is not gated.
    let takes_a_value = matches!(arg.get_action(), ArgAction::Set | ArgAction::Append);

    let defaults = arg.get_default_values();
    if takes_a_value && !defaults.is_empty() && !arg.is_hide_default_value_set() {
        let written: Vec<String> = defaults
            .iter()
            .map(|value| value.to_string_lossy().into_owned())
            .collect();
        parts.push("[default: ]".chars().count() + written.join(" ").chars().count());
    }

    let mut aliases: Vec<usize> = Vec::new();
    aliases.extend(
        arg.get_visible_short_aliases()
            .unwrap_or_default()
            .iter()
            .map(|_| 2),
    );
    aliases.extend(
        arg.get_visible_aliases()
            .unwrap_or_default()
            .iter()
            .map(|name| 2 + name.chars().count()),
    );
    if !aliases.is_empty() {
        let plural = if aliases.len() == 1 { 0 } else { 2 };
        let separators = 2 * (aliases.len() - 1);
        parts.push(
            "[alias: ]".chars().count() + plural + separators + aliases.iter().sum::<usize>(),
        );
    }

    if takes_a_value && !arg.is_hide_possible_values_set() {
        // `get_visible_quoted_name` is `clap`'s and is private, so its two
        // rules are read off it here: a hidden value is not printed, and a name
        // holding a space is printed in quotes.
        let possible: Vec<usize> = arg
            .get_possible_values()
            .iter()
            .filter(|value| !value.is_hide_set())
            .map(|value| {
                let name = value.get_name();
                let quotes = usize::from(name.contains(char::is_whitespace)) * 2;
                name.chars().count() + quotes
            })
            .collect();
        if !possible.is_empty() {
            let separators = 2 * (possible.len() - 1);
            parts.push(
                "[possible values: ]".chars().count() + separators + possible.iter().sum::<usize>(),
            );
        }
    }

    match parts.is_empty() {
        true => 0,
        // `spec_vals` joins its parts with one space, and one more space
        // separates the whole of it from the help text before it.
        false => parts.iter().sum::<usize>() + parts.len(),
    }
}

/// A block of text folded to `width` and indented, with its closing newline.
///
/// The whole block is indented, first line included, which is what separates it
/// from [`row`]: a row hangs under a name and this stands under a heading.
pub fn fold_indented(text: &str, width: usize, at: usize) -> String {
    let indent = " ".repeat(at);
    let folded = fold(text, width.saturating_sub(at));
    let body = folded.replace('\n', &format!("\n{indent}"));
    format!("{indent}{body}\n")
}

/// One row of a two-column list, folded so that no line passes `width`.
///
/// `at` is the column the second field starts at. A row whose text does not fit
/// is continued under itself rather than under the name, which is what keeps a
/// list of verbs readable when one summary is long.
pub fn row(name: &str, text: &str, at: usize, width: usize) -> String {
    let pad = at.saturating_sub(2 + name.chars().count()).max(1);
    let folded = fold(text, width.saturating_sub(at));
    let indent = " ".repeat(at);
    let body = folded.replace('\n', &format!("\n{indent}"));
    format!("  {name}{}{body}\n", " ".repeat(pad))
}

#[cfg(test)]
mod tests {
    use super::{fold, fold_at, fold_indented, row, width_of, INDENT, WIDEST, WIDTH};

    #[test]
    fn nothing_reads_columns_until_a_caller_asks_for_it() {
        assert_eq!(width_of(false, Some("500")), WIDTH);
        assert_eq!(width_of(false, Some("40")), WIDTH);
        assert_eq!(width_of(false, None), WIDTH);
    }

    #[test]
    fn a_width_a_caller_asks_for_is_held_to_the_band() {
        assert_eq!(width_of(true, Some("40")), WIDTH);
        assert_eq!(width_of(true, Some("100")), 100);
        assert_eq!(width_of(true, Some("500")), WIDEST);
        assert_eq!(width_of(true, Some("80")), WIDTH);
        assert_eq!(width_of(true, Some("120")), WIDEST);
    }

    /// A reading that is not a number is the width every other run takes.
    #[test]
    fn a_columns_that_is_not_a_number_is_the_default_width() {
        assert_eq!(width_of(true, None), WIDTH);
        assert_eq!(width_of(true, Some("")), WIDTH);
        assert_eq!(width_of(true, Some("wide")), WIDTH);
        assert_eq!(width_of(true, Some("-1")), WIDTH);
    }

    /// The fill this module re-exports is the one in `headwater-check`.
    ///
    /// Its own cases live beside it, in `crates/check/src/fill.rs`. This one
    /// holds the re-export: a second implementation appearing here would pass
    /// every case there and lay the help out differently.
    #[test]
    fn the_fold_this_module_names_is_the_one_the_check_layer_owns() {
        let text = "see docs/spec/06-engine-architecture.md#the-command-line for it";
        assert_eq!(fold(text, 20), headwater_check::fill::fold(text, 20));
        assert_eq!(
            fold_at(text, 20, 6),
            headwater_check::fill::fold_at(text, 20, 6)
        );
        assert_eq!(WIDTH, headwater_check::fill::WIDTH);
        assert_eq!(WIDEST, headwater_check::fill::WIDEST);
    }

    #[test]
    fn a_row_that_does_not_fit_is_continued_under_itself() {
        let written = row(
            "check",
            "run the pipeline over the corpus, against the lock",
            15,
            40,
        );
        let lines: Vec<&str> = written.trim_end().lines().collect();
        assert_eq!(lines[0], "  check        run the pipeline over the");
        for line in &lines[1..] {
            assert!(line.starts_with(&" ".repeat(15)), "{line:?}");
        }
        for line in &lines {
            assert!(line.chars().count() <= 40, "{line:?}");
        }
    }

    #[test]
    fn an_indented_block_holds_every_line_inside_the_width() {
        let written = fold_indented("run the checks, and fail on an error", 20, 6);
        assert!(written.ends_with('\n'));
        for line in written.lines() {
            assert!(line.starts_with("      "), "{line:?}");
            assert!(line.chars().count() <= 20, "{line:?}");
        }
    }

    /// The indent is `clap`'s two-space `TAB` and its eight-space next-line
    /// indent, and the whole point of declaring `next_line_help` is that it is
    /// the same number at every node.
    #[test]
    fn the_indent_is_the_pair_clap_writes() {
        assert_eq!(INDENT, "  ".len() + "        ".len());
    }
}
