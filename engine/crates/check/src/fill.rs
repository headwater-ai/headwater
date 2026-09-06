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
    // A line whose opening word leaves no room for the word after it is left
    // exactly as it arrived. See [`opens_past_the_room`] for why the test is
    // about the first two words rather than about the first one alone.
    if opens_past_the_room(line, width) {
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

/// The longest tail [`opens_past_the_room`] reads whole rather than by its
/// first word.
///
/// Two, because the widest identity tail in this report is a severity marker
/// under `ColorMode::Plain`: a glyph and a word. A read-set entry opens with
/// `input`, so its tail is measured by the path and this bound never binds it.
const TAIL: usize = 2;

/// Whether the opening word of `line` leaves no room for what follows it.
///
/// This is the one predicate that decides whether [`filled`] narrows a line or
/// hands it back untouched, and [`unfoldable`] is its public name. It reads
/// past the opening word rather than stopping at it, and what follows is the
/// whole point.
///
/// # Why one word is the wrong question
///
/// A greedy fill puts the first word on the opening line and wraps the next one
/// where it does not fit. When the first word alone reaches the width, the
/// opening line carries that word and nothing else, and **every** following word
/// wraps. For a two-word line that means the second word lands alone on a
/// continuation, which is the harm this guard exists to prevent rather than a
/// narrowing worth having.
///
/// The report is full of two-word lines, and each one is an identity beside the
/// token that classifies it: `<path>:<line>:<column> <severity>` is a finding's
/// location, `input <path> <sha256>` is a read-set entry. Splitting the second
/// word off one of those hands a reader half an identity. It also hands
/// `.githooks/pre-commit` a line whose whole content is `error`, which that
/// selector reads as the header of a new finding — so the rule line and the
/// `fix:` line under the real header are dropped and a refused commit explains
/// nothing.
///
/// A guard written as "the first word alone is past the width" is aimed one
/// boundary short of that harm: it misses every line whose first word *reaches*
/// the width without passing it. On this corpus that band held 38 of 334
/// documents. `crates/cli/tests/width.rs::no_finding_states_its_severity_on_a_line_of_its_own`
/// and the `a location line at the width boundary` case of `.githooks/fixtures.sh`
/// are what hold this now, and neither is a width assertion: the broken output
/// is two short lines, so counting columns cannot see it.
fn opens_past_the_room(line: &str, width: usize) -> bool {
    let indent = line.len() - line.trim_start_matches(' ').len();
    let mut words = line.split_whitespace();
    let Some(first) = words.next() else {
        return false;
    };
    let opening = indent + first.chars().count();
    let tail: Vec<&str> = words.collect();
    match tail.len() {
        // One word, so there is nothing to detach. It is left alone only when
        // no fill could narrow it.
        0 => opening > width,
        // An identity and the token that classifies it. The whole tail is
        // measured rather than its first word, because a severity marker is one
        // token under `ColorMode::Ansi` and two under `ColorMode::Plain`, where
        // `crate::paint::severity_word` writes a glyph, a space and the word. A
        // guard that measured the first word of the tail let the glyph ride and
        // wrapped the word alone, which is the harm above with an extra step in
        // front of it.
        1..=TAIL => {
            opening
                + tail
                    .iter()
                    .map(|word| 1 + word.chars().count())
                    .sum::<usize>()
                > width
        }
        // Prose. The opening word of a message line is short, so this arm is
        // reached with room to spare and the fill narrows the line as it should.
        _ => opening + 1 + tail[0].chars().count() > width,
    }
}

/// Whether [`filled`] leaves this line exactly as it arrived.
///
/// A line wider than `width` is either one this function names, or a defect in
/// whatever laid the text out. [`filled`] leaves none of the second kind behind,
/// so a caller can partition a report into the wide lines that are unavoidable
/// and the wide lines that are somebody's fault.
///
/// **Every line [`filled`] emits satisfies this or fits.** A line it built
/// greedily carries a second word only when that word fitted, so an over-width
/// output line holds exactly one word that no fill could narrow. A line it
/// handed back untouched is one this predicate already named. That equivalence
/// is what lets `engine/crates/cli/tests/width.rs` assert the avoidable count is
/// zero without re-implementing the fill.
pub fn unfoldable(line: &str, width: usize) -> bool {
    opens_past_the_room(line, width)
}

#[cfg(test)]
mod tests {
    use super::{block, filled, fold, fold_at, unfoldable, WIDEST, WIDTH};

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
    ///
    /// The line itself opens with `fix:`, which leaves room for the word after
    /// it, so the fill does narrow this line. What it cannot narrow is the
    /// continuation the path lands on, and that is the line [`unfoldable`]
    /// names — the predicate is about a line the fill emitted, not about the
    /// line it was handed.
    #[test]
    fn a_line_whose_one_word_is_past_the_room_is_left_whole() {
        let path = "docs/obligations/0146-the-stop-hook-reads-its-re-entry-guard.md";
        let line = format!("  fix: add a heading to {path}");
        let out = filled(&line, 40);
        assert!(out.contains(path), "the path arrived intact: {out}");
        let carrier = out
            .lines()
            .find(|one| one.contains(path))
            .expect("a line carries the path");
        assert_eq!(carrier.trim(), path, "the path is alone on its line");
        assert!(unfoldable(carrier, 40), "that line names its own reason");
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

    /// **The boundary, walked one column at a time.**
    ///
    /// A guard written as "the first word alone is past the width" is aimed one
    /// column short: a first word that *reaches* the width leaves the opening
    /// line full, so the second word wraps alone. This walks the opening width
    /// from well inside the room to well past it and asserts that a two-word
    /// line is either laid out with both words on the first line, or handed back
    /// whole — and never split one-and-one.
    #[test]
    fn a_two_word_line_is_never_split_one_word_to_a_line() {
        let width = 40;
        for opening in 20..=48 {
            // Two spaces of indent, a first word of `opening - 2`, then `warn`.
            let first = "p".repeat(opening - 2);
            let line = format!("  {first} warn");
            let out = filled(&line, width);
            let lines: Vec<&str> = out.lines().collect();
            assert!(
                lines.len() == 1,
                "at an opening of {opening} the line was split into {} lines:\n{out}",
                lines.len()
            );
            assert!(
                lines[0].ends_with(" warn"),
                "at an opening of {opening} the severity left its line: {out:?}"
            );
            // And the predicate the width tests read agrees with what happened.
            assert_eq!(
                unfoldable(&line, width),
                line.chars().count() > width && out == line,
                "at an opening of {opening} the predicate and the fill disagree"
            );
        }
    }

    /// The exact shape that broke `.githooks/pre-commit`, at 80 columns.
    ///
    /// `docs/decisions/0041-q41-whether-vale-becomes-a-declared-regime-backend.md`
    /// with a `:32:1` suffix is 79 columns at indent 2. Its severity used to
    /// wrap onto a line of its own, and the commit hook then read that bare
    /// `error` as the header of a new finding and dropped the real message.
    #[test]
    fn a_finding_location_at_the_width_keeps_its_severity() {
        let path = "docs/decisions/0041-q41-whether-vale-becomes-a-declared-regime-backend.md";
        // The opening word reaches the width exactly, which is the boundary the
        // old guard sat one column short of.
        let opening = 2 + path.chars().count() + ":32:1".chars().count();
        assert_eq!(opening, WIDTH, "this case is at the boundary it claims");
        let line = format!("  {path}:32:1 error");
        let out = filled(&line, WIDTH);
        assert_eq!(out, line, "the severity left its location line:\n{out}");
        assert_eq!(out.lines().count(), 1);
    }

    /// The same guard against a severity marker of two tokens.
    ///
    /// `crate::paint::severity_word` writes `warn` under `ColorMode::Ansi` and
    /// `▲ warn` under `ColorMode::Plain`, so a finding's location line
    /// holds three words in every recorded fixture and every piped run. The
    /// band where the glyph fits and the word does not is one column wide per
    /// opening, so a case that reads the corpus as it stands meets it only when
    /// a path happens to land there. This walks the opening instead.
    #[test]
    fn a_glyph_never_rides_alone_when_its_severity_word_wraps() {
        let width = 40;
        for opening in 20..=48 {
            let first = "p".repeat(opening - 2);
            let line = format!("  {first} ▲ warn");
            let out = filled(&line, width);
            let lines: Vec<&str> = out.lines().collect();
            assert!(
                !lines.iter().any(|line| line.trim() == "warn"),
                "at an opening of {opening} the severity word landed alone:\n{out}"
            );
            assert!(
                !lines.iter().any(|line| line.trim() == "▲"),
                "at an opening of {opening} the glyph landed alone:\n{out}"
            );
            assert_eq!(
                unfoldable(&line, width),
                line.chars().count() > width && out == line,
                "at an opening of {opening} the predicate and the fill disagree"
            );
        }
    }

    /// Every wide line the fill emits is one the predicate names.
    ///
    /// This is the equivalence `crates/cli/tests/width.rs` relies on to assert
    /// that the avoidable count is zero without re-implementing the fill.
    #[test]
    fn every_wide_line_the_fill_emits_names_itself() {
        let report = "  a short line\n  \
             docs/decisions/0041-q41-whether-vale-becomes-a-declared-regime-backend.md:32:1 error\n  \
             fix (mechanical): write `behavior` into \
             docs/obligations/0146-the-stop-hook-reads-its-re-entry-guard-with-an-interpreter.md\n  \
             a much longer line of ordinary prose that will certainly need to be laid out at eighty columns\n";
        for width in [40, WIDTH, WIDEST] {
            let out = filled(report, width);
            for line in out.lines() {
                if line.chars().count() > width {
                    assert!(
                        unfoldable(line, width),
                        "at {width} the fill emitted a wide line it could have narrowed: {line:?}"
                    );
                }
            }
            // And no emitted line is a bare severity word.
            for line in out.lines() {
                assert!(
                    !matches!(line.trim(), "error" | "warn" | "info"),
                    "at {width} a severity reached a line of its own:\n{out}"
                );
            }
        }
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
