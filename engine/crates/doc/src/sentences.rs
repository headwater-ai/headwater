// SPDX-License-Identifier: Apache-2.0
//! Sentence segmentation, over the text this document's own author wrote.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#the-correctness-roots)
//! puts segmentation on the correctness-root list, and
//! [Q5](../../../../docs/spec/09-decisions.md#q5--voice-checking-depth) says why
//! it is there rather than the pattern sets: over this repository's own
//! specification, most errors of a lexical checker came from the decision about
//! which text is a sentence, and 32 of 58 sentence-length errors on one landing
//! were defects in the splitter. So this module owes conformance fixtures the
//! way a check owes a failing one.
//!
//! # What this splitter does not have to do
//!
//! `tools/ste-lint.py` is the same job on raw lines, and most of it is
//! Markdown removal: code spans to one token, link syntax to its text, emphasis
//! markers deleted, entities dropped. None of that is here, because
//! [`crate::body`] is a CommonMark parse and hands over the text a reader sees,
//! already split into runs that say who wrote each one. That is the whole of
//! the difference between the two, and it is the part Q5 measured as the larger
//! error source.
//!
//! # Three rules, and each one is a measured failure of the alternative
//!
//! **A sentence ends at a terminator that a new sentence follows.** A period,
//! a question mark or an exclamation mark, then space, then a character that
//! opens a sentence. A splitter that ends at every period merges nothing and
//! splits `cf. the projection` into two.
//!
//! **A terminator inside code is not a terminator.** `12.md` and `tools/x.py`
//! end no sentence, and the parser marks that run [`Ownership::Code`] without
//! any rule here knowing what a file name looks like.
//!
//! **A quoted block holds no sentence of this document.** [Spec 3](../../../../docs/spec/03-authoring-and-lifecycle.md#what-a-lexical-rule-gets-wrong-and-where-posture-comes-from)
//! rules that a quotation is outside every voice rule by construction, and this
//! is that construction: a block another author wrote produces no sentence, so
//! no rule downstream declares an exemption for one.

use crate::body::{Block, BlockKind, Body, Link, Ownership, Run};
use headwater_yaml::{Position, Span};

/// The abbreviations whose period ends no sentence.
///
/// The list is closed and short on purpose. Each entry is a string this corpus
/// writes, and an entry that nothing writes is a rule nobody can test.
const ABBREVIATIONS: [&str; 8] = [
    "e.g.", "i.e.", "cf.", "etc.", "vs.", "al.", "approx.", "no.",
];

/// One sentence of one document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sentence {
    /// The text a reader sees, code spans included. This is what a length rule
    /// counts, because a reader reads the code span too.
    pub text: String,
    /// The same sentence with every run that this document's author did not
    /// write as prose removed: code spans, and quoted text. This is what a
    /// voice rule reads, and the removal is the parser's rather than a rule's.
    pub authored: String,
    /// The word count ASD-STE100 rule 6.3 limits.
    ///
    /// A token counts when it holds a letter or a digit, so a lone dash counts
    /// as nothing. A code span counts as one word however many words it holds,
    /// and so does a parenthetical: both are the help text of the rule itself,
    /// and `tools/ste-lint.py` already counts this corpus that way.
    pub words: usize,
    pub span: Span,
    /// The kind of block this sentence came from.
    ///
    /// A rule about running prose is not a rule about a heading or a table
    /// cell, and nothing downstream can tell the three apart from the text.
    /// The parser knows, so it says.
    pub kind: BlockKind,
}

/// A parenthetical counts as one word, however long it is.
fn collapse_parentheticals(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut depth = 0usize;
    for c in text.chars() {
        match c {
            '(' => {
                if depth == 0 {
                    out.push_str(" parenthetical ");
                }
                depth += 1;
            }
            ')' => depth = depth.saturating_sub(1),
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    out
}

/// Every sentence of a body, in document order.
pub fn of(body: &Body) -> Vec<Sentence> {
    body.blocks
        .iter()
        .flat_map(|block| of_block(block, &body.links))
        .collect()
}

/// The sentences of one block, and none for a block that holds no prose.
fn of_block(block: &Block, links: &[Link]) -> Vec<Sentence> {
    if block.quote_depth > 0 || matches!(block.kind, BlockKind::Code | BlockKind::Html) {
        return Vec::new();
    }
    let text: String = block.runs.iter().map(|run| run.text.as_str()).collect();
    let chars: Vec<char> = text.chars().collect();

    let mut sentences = Vec::new();
    let mut start = 0usize;
    let mut index = 0usize;
    while index < chars.len() {
        if ends_a_sentence(&chars, index, block, links) {
            // The break is after the terminator and any closing punctuation,
            // and the whitespace between the two sentences belongs to neither.
            let end = index + 1;
            push(&mut sentences, block, &chars, start, end);
            start = skip_space(&chars, end);
            index = start;
            continue;
        }
        index += 1;
    }
    push(&mut sentences, block, &chars, start, chars.len());
    sentences
}

/// Whether the character at `index` closes a sentence.
fn ends_a_sentence(chars: &[char], index: usize, block: &Block, links: &[Link]) -> bool {
    let c = chars[index];
    if !matches!(c, '.' | '!' | '?') {
        return false;
    }
    // A period inside a code span is part of a name, and the parser is what
    // knows that. See the module comment.
    if ownership_at(block, index) == Ownership::Code {
        return false;
    }
    // Closing quotation and brackets ride with the sentence that ends.
    let mut after = index + 1;
    while matches!(chars.get(after), Some('"' | '\'' | '”' | '’' | ')' | ']')) {
        after += 1;
    }
    // The end of the block ends the sentence in it.
    let Some(next) = chars.get(after) else {
        return true;
    };
    if !next.is_whitespace() {
        return false;
    }
    if c == '.' && closes_an_abbreviation(chars, index) {
        return false;
    }
    let opening = skip_space(chars, after);
    match chars.get(opening) {
        None => true,
        // A run of questions may continue in lower case, and a period may not:
        // splitting on `cf. the projection` is the failure that guard prevents.
        Some(_) if c != '.' => true,
        // A code span opens a sentence whatever letter it starts with. The
        // parser removed the backticks, so `\`scope()\` is a method` reaches
        // this rule as a lower-case `s`, and the case test alone would merge it
        // into the sentence before it. Found by differential against
        // `tools/ste-lint.py`, which stands a `CODE` token in its place and so
        // never met this.
        Some(_) if ownership_at(block, opening) == Ownership::Code => true,
        // A link opens a sentence on the same terms and for the same reason.
        // The parser removed the brackets, so `[spec 1](…) explicitly forswears
        // that` reaches this rule as a lower-case `s`.
        Some(_) if opens_a_link(block, opening, links) => true,
        Some(next) => opens_a_sentence(*next),
    }
}

/// Whether a character can open the next sentence.
///
/// A sentence of this corpus opens with a capital, a digit, a quotation, a
/// bracket, a code span or an emphasis marker. It never opens in lower case,
/// and that is the guard that keeps an abbreviation the list below misses from
/// splitting a sentence in two.
fn opens_a_sentence(c: char) -> bool {
    c.is_uppercase()
        || c.is_ascii_digit()
        || matches!(c, '"' | '“' | '(' | '[' | '`' | '*' | '_' | '§')
}

/// Whether the character at `index` is inside a prose link.
///
/// A link is the second construct whose markup the parse removed, and the two
/// have to be recognized here rather than by a rule that reads the text: a rule
/// downstream sees the link's text and nothing that says it was a link.
fn opens_a_link(block: &Block, index: usize, links: &[Link]) -> bool {
    let at = position(block, index).offset;
    links
        .iter()
        .any(|link| link.span.start.offset <= at && at < link.span.end.offset)
}

fn closes_an_abbreviation(chars: &[char], index: usize) -> bool {
    ABBREVIATIONS.iter().any(|abbreviation| {
        let letters: Vec<char> = abbreviation.chars().collect();
        if letters.len() > index + 1 {
            return false;
        }
        let from = index + 1 - letters.len();
        // A word boundary in front, so that `St.` does not match inside
        // `cost.` and hide a real sentence end.
        if from > 0 && chars[from - 1].is_alphanumeric() {
            return false;
        }
        letters
            .iter()
            .enumerate()
            .all(|(offset, letter)| chars[from + offset].eq_ignore_ascii_case(letter))
    })
}

fn skip_space(chars: &[char], from: usize) -> usize {
    let mut at = from;
    while matches!(chars.get(at), Some(c) if c.is_whitespace()) {
        at += 1;
    }
    at
}

/// The ownership of the run that holds the character at `index`.
fn ownership_at(block: &Block, index: usize) -> Ownership {
    let mut at = 0usize;
    for run in &block.runs {
        let length = run.text.chars().count();
        if index < at + length {
            return run.ownership;
        }
        at += length;
    }
    Ownership::Authored
}

fn push(sentences: &mut Vec<Sentence>, block: &Block, chars: &[char], start: usize, end: usize) {
    let text: String = chars[start.min(chars.len())..end.min(chars.len())]
        .iter()
        .collect();
    if text.trim().is_empty() {
        return;
    }
    sentences.push(Sentence {
        authored: authored_between(block, start, end),
        words: words_between(block, start, end),
        span: Span::new(position(block, start), position(block, end)),
        text: text.trim().to_string(),
        kind: block.kind,
    });
}

/// The words of one sentence, with each code span counted as one.
fn words_between(block: &Block, start: usize, end: usize) -> usize {
    let mut countable = String::new();
    for (run, from, to) in overlapping(block, start, end) {
        match run.ownership {
            Ownership::Code => countable.push_str(" code "),
            _ => countable.extend(run.text.chars().skip(from).take(to - from)),
        }
    }
    collapse_parentheticals(&countable)
        .split_whitespace()
        .filter(|token| token.chars().any(|c| c.is_alphanumeric()))
        .count()
}

/// Each run that overlaps a range, with the range inside the run's own text.
fn overlapping(block: &Block, start: usize, end: usize) -> Vec<(&Run, usize, usize)> {
    let mut out = Vec::new();
    let mut at = 0usize;
    for run in &block.runs {
        let (from, to) = (at, at + run.text.chars().count());
        at = to;
        let (lower, upper) = (from.max(start), to.min(end));
        if lower < upper {
            out.push((run, lower - from, upper - from));
        }
    }
    out
}

/// The part of one sentence that this document's own author wrote as prose.
fn authored_between(block: &Block, start: usize, end: usize) -> String {
    let mut out = String::new();
    for (run, from, to) in overlapping(block, start, end) {
        if run.ownership == Ownership::Authored {
            out.extend(run.text.chars().skip(from).take(to - from));
        }
    }
    out.trim().to_string()
}

/// The position of a character offset inside the text of one block.
///
/// A `Text` event of the CommonMark parse never spans a line: a break inside a
/// paragraph arrives as its own run. So a position inside a run is its start
/// advanced along one line, and no newline can be crossed without the run
/// changing.
fn position(block: &Block, index: usize) -> Position {
    let mut at = 0usize;
    let mut last = block.span.start;
    for run in &block.runs {
        let length = run.text.chars().count();
        if index < at + length {
            return advanced(run, index - at);
        }
        at += length;
        last = run.span.end;
    }
    last
}

fn advanced(run: &Run, characters: usize) -> Position {
    let bytes: usize = run.text.chars().take(characters).map(char::len_utf8).sum();
    // The run's own end is the ceiling. A source that escapes an entity holds
    // more bytes than the text does, and a position past the run would name a
    // line that is not the one the reader wants.
    if run.span.start.offset + bytes > run.span.end.offset {
        return run.span.end;
    }
    Position::new(
        run.span.start.line,
        run.span.start.col + characters,
        run.span.start.offset + bytes,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::body::scan;

    fn sentences(source: &str) -> Vec<Sentence> {
        of(&scan(source, source, 0))
    }

    /// The second half of the same defect: a link's text is usually lower case,
    /// and the brackets that would have said so are gone by the time a rule
    /// reads it.
    #[test]
    fn a_link_opens_a_sentence() {
        let texts = texts("It reasons about prose. [spec 1](01.md) forswears that.\n");
        assert_eq!(texts.len(), 2);
        assert_eq!(texts[1], "spec 1 forswears that.");
    }

    fn texts(source: &str) -> Vec<String> {
        sentences(source)
            .into_iter()
            .map(|sentence| sentence.text)
            .collect()
    }

    #[test]
    fn a_paragraph_splits_at_a_terminator_that_a_sentence_follows() {
        assert_eq!(
            texts("One thing holds. Another follows it.\n"),
            ["One thing holds.", "Another follows it."]
        );
    }

    /// The failure the abbreviation list exists for. `cf. the projection` is
    /// one sentence, and a splitter that ends at every period reports two.
    #[test]
    fn an_abbreviation_ends_no_sentence() {
        assert_eq!(
            texts("The loss set covers it, cf. the projection census of spec 6.\n").len(),
            1
        );
        assert_eq!(texts("A shelf collects them, e.g. a decision.\n").len(), 1);
    }

    /// The guard that catches every abbreviation the list misses: a sentence of
    /// this corpus never opens in lower case.
    #[test]
    fn a_period_before_lower_case_ends_no_sentence() {
        assert_eq!(texts("It reads spec 12. and stops there.\n").len(), 1);
    }

    /// A question may be followed by another in lower case, and that is the one
    /// place the rule above is relaxed.
    #[test]
    fn a_question_may_continue_in_lower_case() {
        assert_eq!(
            texts("Is it the same kind? does every check still pass?\n").len(),
            2
        );
    }

    /// The measured reason the scan is a parse. A file name inside a code span
    /// carries a period, and no rule here knows what a file name looks like.
    #[test]
    fn a_period_inside_a_code_span_ends_no_sentence() {
        assert_eq!(
            texts("The linter is `tools/ste-lint.py` today and it blocks at commit.\n").len(),
            1
        );
    }

    /// The defect the differential against `tools/ste-lint.py` found. The
    /// parser removes the backticks, so a sentence that opens with a code span
    /// opens in whatever case the code is written in.
    #[test]
    fn a_code_span_opens_a_sentence() {
        assert_eq!(
            texts("The reasoning binds it. `scope()` is a method it declares.\n"),
            [
                "The reasoning binds it.",
                "scope() is a method it declares."
            ]
        );
    }

    /// Spec 3 puts a quotation outside every prose rule by construction, and
    /// the construction is here rather than in a rule.
    #[test]
    fn a_quoted_block_holds_no_sentence_of_this_document() {
        let sentences = sentences("This one is ours.\n\n> This one is somebody else's.\n");
        assert_eq!(sentences.len(), 1);
        assert_eq!(sentences[0].text, "This one is ours.");
    }

    /// A code span is text a reader reads, so it counts toward the length, and
    /// it is not text the author wrote as prose, so a voice rule never sees it.
    #[test]
    fn a_code_span_counts_as_a_word_and_is_not_authored_prose() {
        let sentences = sentences("The rule is `will be` in this line.\n");
        assert_eq!(sentences.len(), 1);
        assert_eq!(sentences[0].words, 7);
        assert_eq!(sentences[0].authored, "The rule is  in this line.");
    }

    #[test]
    fn a_parenthetical_counts_as_one_word() {
        let one = &sentences("A rule holds (for every kind that declares it).\n")[0];
        assert_eq!(one.words, 4);
    }

    /// A finding anchors to a line, so the span has to be the sentence's own
    /// and not the block's.
    #[test]
    fn a_sentence_carries_the_position_it_starts_at() {
        let source = "First line here.\n\nOne holds. Two follows.\n";
        let sentences = sentences(source);
        assert_eq!(sentences[1].span.start.line, 3);
        assert_eq!(sentences[1].span.start.col, 1);
        assert_eq!(sentences[2].span.start.line, 3);
        assert_eq!(sentences[2].span.start.col, 12);
    }

    /// A break inside a paragraph is a run of its own, so the sentence after it
    /// reports the line it is really on.
    #[test]
    fn a_sentence_after_a_soft_break_reports_the_next_line() {
        let source = "One holds.\nTwo follows on the next line.\n";
        let second = &sentences(source)[1];
        assert_eq!(second.span.start.line, 2);
    }

    /// A list item is prose, and a heading is prose. Both answer to a length
    /// rule and to a voice rule, and neither ends in a period.
    #[test]
    fn a_heading_and_an_item_are_sentences() {
        let texts = texts("# The heading\n\n- one item\n- another item\n");
        assert_eq!(texts, ["The heading", "one item", "another item"]);
    }

    #[test]
    fn a_fenced_block_holds_no_sentence() {
        assert_eq!(
            texts("```\nfn main() { one(); two(); }\n```\n"),
            Vec::<String>::new()
        );
    }
}
