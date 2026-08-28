// SPDX-License-Identifier: Apache-2.0
//! The one fill in this engine, and the width every report is laid out at.
//!
//! # Why it lives here
//!
//! `engine/crates/conformance/src/render.rs` carried the only fill in the tree
//! and the instruction to move it: "when a later change fills the check report,
//! `WIDTH` and `block` move down into `headwater-check` and this crate calls
//! them there". This is that move. `headwater-conformance`, `headwater-adapter`
//! and `headwater-cli` all depend on this crate and this crate depends on none
//! of them, so one implementation is reachable from every report and a second
//! copy is not needed.
//!
//! # The three shapes a caller wants, and the difference between them
//!
//! [`fold`] and [`fold_at`] fill a run of prose that carries no indent of its
//! own. `headwater_cli::paint` folds a help string with them before `clap` sees
//! it, and [`fold_at`] leaves room on the last line for the `[default: 0]` that
//! `clap` appends after it.
//!
//! [`block`] writes one labelled block: `indent` spaces, a label, and the text
//! filled under it, with a continuation hanging at the width of the label. A
//! caller that is composing a report line by line reaches for this.
//!
//! [`filled`] lays out a report that is already composed. It reads each line's
//! own leading spaces as that line's indent, fills the rest, and hangs a
//! continuation two columns further in. A caller that is handed the finished
//! text of a foreign renderer — which is what `headwater_adapter::text` is
//! handed three times — reaches for this, because the alternative is a width
//! parameter threaded through every renderer below it.
//!
//! # What no fill here ever does
//!
//! **Nothing breaks inside a word.** A word longer than the room it lands in is
//! written past the width, whole. A digest, a path, a rule name and a flag name
//! all arrive here as one word, and every one of them is a thing a caller
//! retypes or a thing another program matches on: `headwater_adapter::census`
//! audits every format by `artifact.contains(path)`, so a path broken across a
//! fold point would make a clean run fail its own projection census.
//!
//! **The count is in `char`s.** `str::len()` is bytes, and a line carrying an em
//! dash of three bytes would be broken two characters early by a byte count. A
//! codepoint count claims no display width and no grapheme boundary, which is
//! the right claim for a report of ASCII identifiers and English prose.
//!
//! **A line that already fits comes back byte-identical.** [`filled`] returns a
//! short line untouched rather than re-joining its words, so a report whose
//! lines all fit is the same bytes before and after this function runs.

/// The width every report is laid out at, whatever the run is attached to.
///
/// It is a constant rather than the width of whatever terminal ran the verb. A
/// recorded block compared against the running terminal's width compares
/// nothing, and the engine names no `libc`, no `terminal_size` and no
/// `unicode-width` in its lock. `headwater_cli::paint::width` is the one place
/// a caller may state a different number, and [`WIDEST`] is the ceiling on it.
pub const WIDTH: usize = 80;

/// The widest a caller may ask for with `--wide`.
pub const WIDEST: usize = 120;

/// The continuation indent [`filled`] hangs a wrapped line at.
const HANG: usize = 2;

/// The text, folded to `width` columns, breaking only where there is a space.
pub fn fold(text: &str, width: usize) -> String {
    fold_at(text, width, 0)
}

/// The text folded to `width`, leaving room on the last line for what follows.
///
/// `tail` is the width of a run of text that something else will append after
/// this one, on the same line and after a space. It is kept with the last word
/// rather than added as a word of its own, so the last line either carries both
/// or carries neither.
///
/// A newline in the source is a break the author asked for and survives. A word
/// longer than `width` is written past it rather than cut: a fold that broke
/// inside a word would break an identifier, a path or a flag name, and every one
/// of those is a thing a caller retypes.
pub fn fold_at(text: &str, width: usize, tail: usize) -> String {
    let lines: Vec<&str> = text.split('\n').collect();
    let last = lines.len().saturating_sub(1);
    lines
        .iter()
        .enumerate()
        .map(|(at, line)| one_line(line, width, if at == last { tail } else { 0 }))
        .collect::<Vec<String>>()
        .join("\n")
}

fn one_line(text: &str, width: usize, tail: usize) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return String::new();
    }
    let mut widths: Vec<usize> = words.iter().map(|word| word.chars().count()).collect();
    if tail > 0 {
        let end = widths.len() - 1;
        widths[end] += 1 + tail;
    }

    let mut out = String::new();
    let mut used = 0;
    for (word, measure) in words.iter().zip(widths) {
        if used == 0 {
            out.push_str(word);
            used = measure;
        } else if used + 1 + measure <= width {
            out.push(' ');
            out.push_str(word);
            used += 1 + measure;
        } else {
            out.push('\n');
            out.push_str(word);
            used = measure;
        }
    }
    out
}

/// One block of a report: `indent` spaces, then `label`, then `text` filled to
/// `width`.
///
/// A continuation line is indented to `indent + label.chars().count()`, so the
/// label hangs the text under itself and no caller names a second indent. Where
/// a line opens with an identifying token — `fix: `, a level name, a rule name —
/// that token is the label, and the text under it lines up.
///
/// `text` is split on `'\n'` first and each piece is filled on its own, so a
/// break the author wrote survives. The fill is greedy on whitespace runs, and a
/// whitespace run becomes one space; that normalization is invisible in the
/// YAML-folded strings a rule set ships.
pub fn block(out: &mut String, indent: usize, label: &str, text: &str, width: usize) {
    let cont = " ".repeat(indent + label.chars().count());
    let mut opening = format!("{}{label}", " ".repeat(indent));
    for paragraph in text.split('\n') {
        let mut line = opening.clone();
        let mut written = false;
        for word in paragraph.split_whitespace() {
            match written {
                false => {
                    line.push_str(word);
                    written = true;
                }
                true => match line.chars().count() + 1 + word.chars().count() <= width {
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

/// A composed report, every line laid out at its own leading indent.
///
/// A line's leading spaces are its indent and are kept. What follows them is
/// filled greedily to `width`, and a continuation sits at the indent plus two,
/// so a wrapped line is visibly a continuation of the line above rather than a
/// new one. A blank line stays blank, with no indent written into it: a blank
/// line carrying trailing whitespace would put it in an artifact a test compares
/// byte for byte.
///
/// A line that already fits inside `width` is returned untouched. That is what
/// makes this safe to run over text somebody else composed: a renderer that
/// aligns a column with two spaces keeps its alignment, because the line was
/// never taken apart.
///
/// A trailing newline on the input is a trailing newline on the output, and no
/// newline is added to text that had none.
pub fn filled(text: &str, width: usize) -> String {
    if text.is_empty() {
        return String::new();
    }
    let ends = text.ends_with('\n');
    let body = match ends {
        true => &text[..text.len() - 1],
        false => text,
    };
    let mut out: String = body
        .split('\n')
        .map(|line| one_indented(line, width))
        .collect::<Vec<String>>()
        .join("\n");
    if ends {
        out.push('\n');
    }
    out
}

/// One line of a composed report, kept at its own indent and filled under it.
fn one_indented(line: &str, width: usize) -> String {
    // A line that fits is returned as it arrived, so this function is the
    // identity over a report whose lines are all short enough.
    if line.chars().count() <= width {
        return line.to_string();
    }
    let indent = line.len() - line.trim_start_matches(' ').len();
    let rest = &line[indent..];
    // A line whose only content is whitespace stays blank rather than becoming
    // an indent with nothing under it.
    if rest.trim().is_empty() {
        return String::new();
    }
    // A line whose first word is already past the room is left exactly as it
    // arrived. Nothing the fill can do narrows it, and wrapping what follows
    // would detach a word from a line that overflowed before that word was
    // reached. The report is full of lines shaped `<path> <one word>` — a
    // finding's location and its severity, a read-set input and its digest —
    // and splitting the second word off one of those hands a reader half an
    // identity and hands `.githooks/pre-commit` a finding it cannot select.
    let first = rest.split_whitespace().next().unwrap_or(rest);
    if indent + first.chars().count() > width {
        return line.to_string();
    }
    let opening = " ".repeat(indent);
    let cont = " ".repeat(indent + HANG);

    let mut out = String::new();
    let mut line_out = opening;
    let mut written = false;
    for word in rest.split_whitespace() {
        match written {
            false => {
                line_out.push_str(word);
                written = true;
            }
            true => match line_out.chars().count() + 1 + word.chars().count() <= width {
                true => {
                    line_out.push(' ');
                    line_out.push_str(word);
                }
                false => {
                    out.push_str(&line_out);
                    out.push('\n');
                    line_out = cont.clone();
                    line_out.push_str(word);
                }
            },
        }
    }
    out.push_str(&line_out);
    out
}

/// The lines of `text` that are wider than `width` and could not be narrower.
///
/// A line is unfoldable when one of its words, standing at the line's own
/// indent, is already past `width`. Every other wide line is a defect in
/// whatever laid the text out, and [`filled`] leaves none of them behind.
/// Where the **first** word is the one past the room, [`filled`] leaves the
/// whole line alone rather than wrapping what follows it, so a caller reading
/// this partition should also ask whether the rest of such a line would have
/// fitted without its long word.
/// `engine/crates/cli/tests/width.rs` is the caller: it partitions the report
/// this way so that the avoidable count is asserted to be zero and the
/// unavoidable one names itself.
pub fn unfoldable(line: &str, width: usize) -> bool {
    let indent = line.len() - line.trim_start_matches(' ').len();
    line.split_whitespace()
        .any(|word| indent + word.chars().count() > width)
}

#[cfg(test)]
mod tests {
    use super::{block, filled, fold, fold_at, unfoldable, WIDTH};

    #[test]
    fn a_folded_line_is_never_wider_than_the_width() {
        let text = "the manifest of the change this run is scoped to, and every line of it \
                    names one document the change carries";
        for width in [20, 40, 70, 80] {
            for line in fold(text, width).lines() {
                assert!(
                    line.chars().count() <= width,
                    "{width}: {line:?} is {} columns",
                    line.chars().count()
                );
            }
        }
        assert_eq!(fold(text, 200), text);
    }

    /// A word wider than the fold is written past it rather than cut in half.
    #[test]
    fn a_word_longer_than_the_width_is_never_broken() {
        let text = "see docs/spec/06-engine-architecture.md#the-command-line for it";
        let folded = fold(text, 20);
        assert!(folded.contains("docs/spec/06-engine-architecture.md#the-command-line"));
        assert_eq!(folded.split('\n').next(), Some("see"));
    }

    #[test]
    fn a_break_the_author_wrote_survives_the_fold() {
        assert_eq!(fold("one two\nthree four", 40), "one two\nthree four");
    }

    /// The last word and the suffix `clap` appends are on one line or on none.
    #[test]
    fn the_suffix_that_follows_the_text_is_left_room_for() {
        let width = 30;
        let tail = "[default: 0]".chars().count();
        let text = "the rotation seed, which is a member of the run identity";
        let folded = fold_at(text, width, tail);
        let last = folded.lines().last().expect("the fold wrote a line");
        assert!(
            last.chars().count() + 1 + tail <= width,
            "{last:?} plus the suffix is past {width}"
        );
        // The same text with no suffix fills the line the suffix vacated.
        assert_ne!(fold(text, width), folded);
    }

    /// The label hangs the text under itself, at the label's own width.
    #[test]
    fn a_block_hangs_its_text_under_its_label() {
        let mut out = String::new();
        block(
            &mut out,
            2,
            "fix: ",
            "rewrite the sentence so that it states one claim and no more",
            40,
        );
        let lines: Vec<&str> = out.trim_end().lines().collect();
        assert_eq!(lines[0], "  fix: rewrite the sentence so that it");
        for line in &lines[1..] {
            assert!(line.starts_with(&" ".repeat(7)), "{line:?}");
        }
        for line in &lines {
            assert!(line.chars().count() <= 40, "{line:?}");
        }
        assert!(out.ends_with('\n'));
    }

    /// A report whose every line already fits is returned byte for byte.
    #[test]
    fn a_report_that_already_fits_comes_back_unchanged() {
        let report = "census\n  36 documents\n\n  4 excluded\nchecks\n  0 findings\n";
        assert_eq!(filled(report, WIDTH), report);
    }

    /// The indent a line arrived with is the indent it keeps, and a wrapped
    /// line sits two columns further in.
    #[test]
    fn a_wrapped_line_keeps_its_indent_and_hangs_two_columns_in() {
        let line = format!("    {}", "word ".repeat(30).trim_end());
        let out = filled(&line, 40);
        let lines: Vec<&str> = out.lines().collect();
        assert!(lines.len() > 1, "the line wrapped");
        assert!(lines[0].starts_with("    w"), "{:?}", lines[0]);
        for one in &lines[1..] {
            assert!(one.starts_with("      w"), "{one:?}");
        }
        for one in &lines {
            assert!(one.chars().count() <= 40, "{one:?}");
        }
    }

    /// A blank line stays blank, and a trailing newline survives.
    #[test]
    fn a_blank_line_stays_blank_and_the_closing_newline_survives() {
        let text = "one\n\ntwo\n";
        assert_eq!(filled(text, 10), text);
        assert_eq!(filled("one\n\ntwo", 10), "one\n\ntwo");
        assert_eq!(filled("", 10), "");
    }

    /// A word past the room is written past it, whole, and says so.
    #[test]
    fn a_line_whose_one_word_is_past_the_room_is_left_whole() {
        let path = "docs/obligations/0146-the-stop-hook-reads-its-re-entry-guard.md";
        let line = format!("  fix: add a heading to {path}");
        let out = filled(&line, 40);
        assert!(out.contains(path), "the path arrived intact: {out}");
        assert!(unfoldable(&line, 40), "the line names its own reason");
        assert!(!unfoldable("  a short line", 40));
    }

    /// A line whose *first* word is past the room is left exactly as it came.
    ///
    /// `<path> <severity>` is the shape of every finding's location line, and
    /// `<path> <digest>` is the shape of every read-set input. Wrapping the
    /// second word of one of those detaches an identity from the thing it
    /// identifies, and narrows nothing: the line was already over the width
    /// when the first word landed.
    #[test]
    fn a_line_that_opens_past_the_room_is_not_wrapped_after_it() {
        let path = "docs/obligations/0146-the-stop-hook-reads-its-re-entry-guard.md";
        let line = format!("  {path}:12:3 error");
        assert_eq!(filled(&line, 40), line);
        assert_eq!(filled(&format!("  {path}"), 40), format!("  {path}"));
    }

    /// The fill is idempotent: laying out a laid-out report changes nothing.
    #[test]
    fn laying_out_a_laid_out_report_changes_nothing() {
        let report = "checks\n  a very long sentence that will certainly need to wrap at forty \
                      columns and then some more\n  docs/one.md:1:1 warn\n";
        let once = filled(report, 40);
        assert_eq!(filled(&once, 40), once);
    }
}
