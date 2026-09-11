// SPDX-License-Identifier: Apache-2.0
//! The CommonMark scan.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#the-correctness-roots) names
//! the parser's spans a correctness root, and the reason is a silent pass: a
//! mis-parsed heading lets a section contract hold with no finding anywhere. A
//! scan by regular expression fails that way on the first `# heading` inside a
//! fenced code block, and this repository's own specification has several.
//!
//! So the scan is a real CommonMark parse, and it keeps three things:
//!
//! - **Blocks**, in document order, each with the span a finding points at.
//! - **Links**, with the destination a later milestone resolves to an
//!   identifier. Prose-link extraction is on the correctness-root list too.
//! - **Ownership**, per run of text. Spec 12 puts the author-owned span there
//!   on measured grounds: over this repository's own specification, the
//!   remaining structural share of a lexical checker's errors came from text
//!   which quotes another author. A voice rule that reads a block quote as the
//!   document's own prose reports the quoted author's grammar as a defect, and
//!   no exemption written into the rule can fix that — the rule cannot see the
//!   quote. The parser can.

use crate::lines::Lines;
use headwater_yaml::{Position, Span};
use pulldown_cmark::{Event, HeadingLevel, LinkType, Options, Parser, Tag, TagEnd};

/// Who wrote a run of text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ownership {
    /// The author of this document. Only this text answers to a voice rule.
    Authored,
    /// Another author, quoted. Structurally a block quote.
    Quoted,
    /// Not prose: a fenced or indented code block, an inline code span, or raw
    /// HTML. Nothing that reads sentences reads this.
    Code,
}

impl Ownership {
    pub fn name(self) -> &'static str {
        match self {
            Ownership::Authored => "authored",
            Ownership::Quoted => "quoted",
            Ownership::Code => "code",
        }
    }
}

/// A run of text inside a block, with the span it occupies in the file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Run {
    pub text: String,
    pub span: Span,
    pub ownership: Ownership,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockKind {
    Paragraph,
    /// An ATX or a setext heading. Both reach here as the same thing, which is
    /// one reason the scan is a parse: `Title` over `-----` is a heading, and
    /// nothing that matches a leading `#` sees it.
    Heading(u8),
    /// A list item that holds its text directly, which is what a tight list
    /// produces. A loose list puts a paragraph inside the item, and that
    /// paragraph is the block.
    Item,
    TableCell,
    Code,
    /// Raw HTML at block level. Kept so that nothing counting the corpus loses
    /// it, and marked as code so that nothing reading prose finds it.
    Html,
}

impl BlockKind {
    pub fn name(self) -> &'static str {
        match self {
            BlockKind::Paragraph => "paragraph",
            BlockKind::Heading(_) => "heading",
            BlockKind::Item => "item",
            BlockKind::TableCell => "cell",
            BlockKind::Code => "code",
            BlockKind::Html => "html",
        }
    }
}

/// A block of the body that holds text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub kind: BlockKind,
    pub span: Span,
    /// How many block quotes enclose it. Zero is the document's own author.
    pub quote_depth: usize,
    pub runs: Vec<Run>,
    /// Where the source broke this block over a line without ending it: one
    /// span per CommonMark soft break, in document order.
    ///
    /// Every rule that reads prose reads a soft break as a space, and that is
    /// correct for prose. It is the whole of the fact for a rule about the
    /// source form, because a paragraph that a later commit reflows reads the
    /// same and is written differently. Nothing except the parser can tell a
    /// soft break from a backslash hard break, so the parser keeps the
    /// difference here rather than leaving each rule to guess it from a line
    /// count.
    pub soft_breaks: Vec<Span>,
}

impl Block {
    /// The text a reader sees, with the inline markup removed.
    pub fn text(&self) -> String {
        self.runs.iter().map(|run| run.text.as_str()).collect()
    }

    /// The heading level, for a heading.
    pub fn level(&self) -> Option<u8> {
        match self.kind {
            BlockKind::Heading(level) => Some(level),
            _ => None,
        }
    }
}

/// How a link was written. A check that rewrites one has to put it back in the
/// form it found, and a check that counts prose links should not count an
/// autolink in a footer as one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkForm {
    Inline,
    Reference,
    Collapsed,
    Shortcut,
    Autolink,
    Email,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Link {
    /// The destination as written, before anything normalizes it. Normalization
    /// belongs to the external-anchor resolvers, which are their own
    /// correctness root and their own milestone.
    pub destination: String,
    /// The link text, with markup removed.
    pub text: String,
    /// The whole link, from its first bracket to its last parenthesis.
    pub span: Span,
    pub form: LinkForm,
    /// Whether the link is an image rather than a link.
    pub image: bool,
    /// Whether it sits inside a block quote, and so belongs to another author.
    pub quoted: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Body {
    pub blocks: Vec<Block>,
    pub links: Vec<Link>,
}

impl Body {
    /// The headings, in document order.
    pub fn headings(&self) -> impl Iterator<Item = &Block> {
        self.blocks
            .iter()
            .filter(|block| matches!(block.kind, BlockKind::Heading(_)))
    }

    /// The blocks whose text this document's own author wrote.
    pub fn authored(&self) -> impl Iterator<Item = &Block> {
        self.blocks.iter().filter(|block| block.quote_depth == 0)
    }

    /// Every sentence this document's own author wrote, in document order.
    ///
    /// [Spec 12](../../../../docs/spec/12-check-layer.md#the-correctness-roots)
    /// holds segmentation here rather than in a rule, with the spans and the
    /// author-owned text. See [`crate::sentences`].
    pub fn sentences(&self) -> Vec<crate::sentences::Sentence> {
        crate::sentences::of(self)
    }
}

/// The extensions the scan enables, and no others.
///
/// Tables are here because thirty-seven files in this corpus use them, and a
/// parser without the extension reads a table as a paragraph of pipes — one
/// paragraph where there are twenty cells, with every prose rule then reading
/// the whole table as one sentence. Strikethrough is here because the same
/// authors use it and a bare `~~` would otherwise survive into the text a voice
/// rule reads.
///
/// Nothing else is enabled. An extension changes what a source *means*, so
/// turning one on for a corpus that does not use it can only reinterpret text
/// that already parsed.
fn options() -> Options {
    Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH
}

/// Scan a body.
///
/// `offset` is where the body begins in the file, because the CommonMark
/// dependency reports offsets into the text it was handed and a finding has to
/// name a line of the file.
pub fn scan(source: &str, body: &str, offset: usize) -> Body {
    let lines = Lines::new(source);
    let span_of = |range: std::ops::Range<usize>| {
        lines.span(source, (range.start + offset)..(range.end + offset))
    };

    let mut out = Body::default();
    // The block that text runs attach to. Only the innermost open block that
    // can hold text is ever the target, so a paragraph inside a list item
    // inside a block quote collects into the paragraph.
    let mut open: Vec<Block> = Vec::new();
    let mut quote_depth = 0usize;
    // A link can nest inside emphasis and emphasis inside a link, so the text
    // of a link is collected on a stack of its own rather than from the block.
    let mut links: Vec<Link> = Vec::new();

    let push_text = |open: &mut Vec<Block>,
                     links: &mut Vec<Link>,
                     text: &str,
                     range: std::ops::Range<usize>,
                     ownership: Ownership| {
        for link in links.iter_mut() {
            link.text.push_str(text);
        }
        if let Some(block) = open.last_mut() {
            block.runs.push(Run {
                text: text.to_string(),
                span: span_of(range),
                ownership,
            });
        }
    };

    for (event, range) in Parser::new_ext(body, options()).into_offset_iter() {
        match event {
            Event::Start(tag) => match tag {
                Tag::BlockQuote(_) => quote_depth += 1,
                Tag::Paragraph => {
                    open.push(block(BlockKind::Paragraph, span_of(range), quote_depth));
                }
                Tag::Heading { level, .. } => open.push(block(
                    BlockKind::Heading(level_of(level)),
                    span_of(range),
                    quote_depth,
                )),
                Tag::Item => open.push(block(BlockKind::Item, span_of(range), quote_depth)),
                Tag::TableCell => {
                    open.push(block(BlockKind::TableCell, span_of(range), quote_depth));
                }
                Tag::CodeBlock(_) => open.push(block(BlockKind::Code, span_of(range), quote_depth)),
                Tag::Link {
                    link_type,
                    dest_url,
                    ..
                } => links.push(Link {
                    destination: dest_url.to_string(),
                    text: String::new(),
                    span: span_of(range),
                    form: form_of(link_type),
                    image: false,
                    quoted: quote_depth > 0,
                }),
                Tag::Image {
                    link_type,
                    dest_url,
                    ..
                } => links.push(Link {
                    destination: dest_url.to_string(),
                    text: String::new(),
                    span: span_of(range),
                    form: form_of(link_type),
                    image: true,
                    quoted: quote_depth > 0,
                }),
                _ => {}
            },
            Event::End(tag) => match tag {
                TagEnd::BlockQuote(_) => quote_depth = quote_depth.saturating_sub(1),
                TagEnd::Paragraph
                | TagEnd::Heading(_)
                | TagEnd::Item
                | TagEnd::TableCell
                | TagEnd::CodeBlock => close(&mut open, &mut out),
                TagEnd::Link | TagEnd::Image => {
                    if let Some(link) = links.pop() {
                        out.links.push(link);
                    }
                }
                _ => {}
            },
            Event::Text(text) => {
                let ownership = ownership(&open, quote_depth);
                push_text(&mut open, &mut links, &text, range, ownership);
            }
            // An inline code span is not prose, whoever it sits inside.
            Event::Code(text) => {
                push_text(&mut open, &mut links, &text, range, Ownership::Code);
            }
            Event::Html(text) | Event::InlineHtml(text) => {
                if open.is_empty() {
                    let mut html = block(BlockKind::Html, span_of(range.clone()), quote_depth);
                    html.runs.push(Run {
                        text: text.to_string(),
                        span: span_of(range),
                        ownership: Ownership::Code,
                    });
                    out.blocks.push(html);
                } else {
                    push_text(&mut open, &mut links, &text, range, Ownership::Code);
                }
            }
            // A break inside a paragraph is whitespace to every rule that reads
            // the paragraph as prose, so it arrives as a space rather than as a
            // newline that a sentence splitter would have to know about. The
            // soft break is also recorded, because a rule about the source form
            // reads the break itself and a backslash hard break is deliberate.
            Event::SoftBreak => {
                let ownership = ownership(&open, quote_depth);
                let span = span_of(range.clone());
                if let Some(block) = open.last_mut() {
                    block.soft_breaks.push(span);
                }
                push_text(&mut open, &mut links, " ", range, ownership);
            }
            Event::HardBreak => {
                let ownership = ownership(&open, quote_depth);
                push_text(&mut open, &mut links, " ", range, ownership);
            }
            _ => {}
        }
    }

    // A source that ends inside an unclosed construct cannot arrive from this
    // parser, which balances its own events. Draining is cheap insurance
    // against a future extension that does not.
    while !open.is_empty() {
        close(&mut open, &mut out);
    }

    out.blocks
        .sort_by_key(|block| (block.span.start, block.span.end));
    out.links
        .sort_by_key(|link| (link.span.start, link.span.end));
    out
}

fn block(kind: BlockKind, span: Span, quote_depth: usize) -> Block {
    Block {
        kind,
        span,
        quote_depth,
        runs: Vec::new(),
        soft_breaks: Vec::new(),
    }
}

/// Close the innermost block, and keep it only if it holds something.
///
/// A loose list produces an `Item` that holds a paragraph and no text of its
/// own. Emitting that empty item would give every prose rule a block with no
/// prose in it, and every count over the corpus a denominator that depends on
/// which list style an author used.
fn close(open: &mut Vec<Block>, out: &mut Body) {
    if let Some(mut block) = open.pop() {
        if !block.runs.is_empty() {
            mark_inline_quotations(&mut block);
            out.blocks.push(block);
        }
    }
}

/// Mark the inline quotations inside one closed block.
///
/// [Spec 3](../../../../docs/spec/03-authoring-and-lifecycle.md#what-a-lexical-rule-gets-wrong-and-where-posture-comes-from)
/// rules that a quotation is outside every voice rule "by construction", and
/// that this is "not an exemption that a rule declares". A block quotation
/// already arrives marked, because CommonMark has a token for it. A quotation
/// inside a sentence has no token, so this pass reads the quotation marks out
/// of the text the parse produced and splits the runs at them. No rule in the
/// check layer learns what a quotation mark is.
///
/// It runs over a closed block rather than inside the event loop, because a
/// quotation opens and closes across events. A quotation that holds a code
/// span arrives as three events, and this corpus has one.
///
/// **The rules. Each is a case in `mod tests` below, and together they are the
/// open half of HW-OBL-0086.**
///
/// - A pair marks the text between its two marks, and the marks with it.
/// - Straight `"` pairs in order of appearance. Nothing in the source says
///   which of two straight marks opens, so a straight quotation does not nest:
///   the second mark closes the first.
/// - Curly `“` and `”` pair by direction, and they do nest.
/// - An apostrophe marks nothing, straight or curly. `'` is a possessive at
///   least as often as a quotation here and the parse cannot tell the two
///   apart.
/// - A quotation may open in one run and close in a later run of the same
///   block, so a code span, a soft break and a wrapped source line are all
///   inside it. A code span between the marks stays `Code`, because a code
///   span is not prose whoever wrote it.
/// - **An unclosed mark suppresses nothing.** At the end of the block every
///   run marked since the last unmatched opening goes back to `Authored`, and
///   the runs it split are put back together. A stray mark therefore leaves
///   the block exactly as the parse produced it.
/// - A quotation never crosses a block boundary, because the state is per
///   block and is dropped with it.
///
/// A run whose span does not measure one line of the same length as its text
/// is left alone. That is the entity reference case: `&amp;` is five source
/// columns and one character, so no offset inside it can be derived by
/// counting characters, and marking nothing is the safe answer.
fn mark_inline_quotations(block: &mut Block) {
    /// A quotation that has opened and not yet closed.
    struct Open {
        /// `None` for a straight mark, which does not nest. `Some(depth)` for
        /// a curly one, which does.
        curly: Option<usize>,
        /// Where in `out` the marked runs of this quotation begin.
        from: usize,
    }

    const STRAIGHT: char = '"';
    const CURLY_OPEN: char = '\u{201C}';
    const CURLY_CLOSE: char = '\u{201D}';

    if !block
        .runs
        .iter()
        .any(|run| run.ownership == Ownership::Authored && run.text.contains(marks))
    {
        return;
    }

    // Each entry is the run it came from, so that sub-runs of one original run
    // can be put back together and sub-runs of two never are. Merging two runs
    // the parse emitted separately would change what every recorded parse says
    // about a soft break.
    let mut out: Vec<(usize, Run)> = Vec::new();
    let mut open: Option<Open> = None;

    for (origin, run) in block.runs.iter().enumerate() {
        if run.ownership != Ownership::Authored {
            out.push((origin, run.clone()));
            continue;
        }
        // A run with no mark in it needs no offset, so it takes the state
        // whole. This is the soft break and the wrapped source line: the run
        // between two marks carries no mark of its own.
        if !run.text.contains(marks) {
            let mut run = run.clone();
            if open.is_some() {
                run.ownership = Ownership::Quoted;
            }
            out.push((origin, run));
            continue;
        }
        // A run that holds a mark and cannot be measured is left alone, which
        // marks nothing rather than marking the wrong span.
        if !splittable(run) {
            out.push((origin, run.clone()));
            continue;
        }
        let chars: Vec<char> = run.text.chars().collect();
        let mut cut = 0usize;
        let push = |out: &mut Vec<(usize, Run)>, from: usize, to: usize, quoted: bool| {
            if from < to {
                out.push((origin, sub_run(run, &chars, from, to, quoted)));
            }
        };
        for (index, &c) in chars.iter().enumerate() {
            match open.as_mut() {
                None if c == STRAIGHT || c == CURLY_OPEN => {
                    push(&mut out, cut, index, false);
                    cut = index;
                    open = Some(Open {
                        curly: (c == CURLY_OPEN).then_some(1),
                        from: out.len(),
                    });
                }
                None => {}
                Some(state) => match state.curly.as_mut() {
                    None if c == STRAIGHT => {
                        push(&mut out, cut, index + 1, true);
                        cut = index + 1;
                        open = None;
                    }
                    Some(depth) if c == CURLY_OPEN => *depth += 1,
                    Some(depth) if c == CURLY_CLOSE => {
                        *depth -= 1;
                        if *depth == 0 {
                            push(&mut out, cut, index + 1, true);
                            cut = index + 1;
                            open = None;
                        }
                    }
                    _ => {}
                },
            }
        }
        push(&mut out, cut, chars.len(), open.is_some());
    }

    // An unclosed mark suppresses nothing.
    if let Some(state) = open {
        for (_, run) in out.iter_mut().skip(state.from) {
            if run.ownership == Ownership::Quoted {
                run.ownership = Ownership::Authored;
            }
        }
    }

    block.runs = coalesce(out);
}

/// Whether a character can open or close an inline quotation.
fn marks(c: char) -> bool {
    matches!(c, '"' | '\u{201C}' | '\u{201D}')
}

/// Whether an offset inside a run can be turned into a position by counting
/// characters. A `Text` event never spans a line, so a run that does, or one
/// whose text is shorter than the columns it covers, is a decoded entity
/// reference and no character count locates anything inside it.
fn splittable(run: &Run) -> bool {
    run.span.start.line == run.span.end.line
        && run.span.end.col.saturating_sub(run.span.start.col) == run.text.chars().count()
}

fn sub_run(run: &Run, chars: &[char], from: usize, to: usize, quoted: bool) -> Run {
    Run {
        text: chars[from..to].iter().collect(),
        span: Span::new(advanced(run, chars, from), advanced(run, chars, to)),
        ownership: if quoted {
            Ownership::Quoted
        } else {
            run.ownership
        },
    }
}

fn advanced(run: &Run, chars: &[char], characters: usize) -> Position {
    let bytes: usize = chars[..characters].iter().map(|c| c.len_utf8()).sum();
    Position::new(
        run.span.start.line,
        run.span.start.col + characters,
        run.span.start.offset + bytes,
    )
}

/// Put back together the sub-runs of one original run that ended up with the
/// same ownership, so that a block this pass changed nothing in comes out
/// byte for byte as the parse produced it.
fn coalesce(parts: Vec<(usize, Run)>) -> Vec<Run> {
    let mut out: Vec<(usize, Run)> = Vec::with_capacity(parts.len());
    for (origin, run) in parts {
        match out.last_mut() {
            Some((last_origin, last))
                if *last_origin == origin && last.ownership == run.ownership =>
            {
                last.text.push_str(&run.text);
                last.span.end = run.span.end;
            }
            _ => out.push((origin, run)),
        }
    }
    out.into_iter().map(|(_, run)| run).collect()
}

fn ownership(open: &[Block], quote_depth: usize) -> Ownership {
    if matches!(open.last().map(|b| b.kind), Some(BlockKind::Code)) {
        Ownership::Code
    } else if quote_depth > 0 {
        Ownership::Quoted
    } else {
        Ownership::Authored
    }
}

fn level_of(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn form_of(link_type: LinkType) -> LinkForm {
    match link_type {
        LinkType::Inline => LinkForm::Inline,
        LinkType::Reference | LinkType::ReferenceUnknown => LinkForm::Reference,
        LinkType::Collapsed | LinkType::CollapsedUnknown => LinkForm::Collapsed,
        LinkType::Shortcut | LinkType::ShortcutUnknown => LinkForm::Shortcut,
        LinkType::Autolink => LinkForm::Autolink,
        LinkType::Email => LinkForm::Email,
        LinkType::WikiLink { .. } => LinkForm::Reference,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every run of the first block, as `ownership:text`. The shape the
    /// inline-quotation rules are stated in.
    fn marked(source: &str) -> Vec<String> {
        body_of(source).blocks[0]
            .runs
            .iter()
            .map(|run| format!("{}:{}", run.ownership.name(), run.text))
            .collect()
    }

    /// The case the issue exists for. The words between the marks are another
    /// author's, and `Sentence::authored` drops a `Quoted` run, so no voice
    /// rule and no lexical rule reads them.
    #[test]
    fn a_pair_of_straight_marks_marks_what_is_between_them() {
        assert_eq!(
            marked("The judgment \"we no longer describe it\" was prose.\n"),
            [
                "authored:The judgment ",
                "quoted:\"we no longer describe it\"",
                "authored: was prose."
            ]
        );
    }

    /// The other half of the pair. A fix that dropped the whole sentence would
    /// pass the case above and fail this one.
    #[test]
    fn the_same_words_outside_the_marks_stay_this_authors() {
        assert_eq!(
            marked("We no longer describe it that way, and \"so\" is short.\n"),
            [
                "authored:We no longer describe it that way, and ",
                "quoted:\"so\"",
                "authored: is short."
            ]
        );
    }

    /// An unclosed mark suppresses nothing, and it leaves the block exactly as
    /// the parse produced it rather than split at the mark. This is the rule
    /// HW-OBL-0086 asked for by name, and the conservative half of it: a stray
    /// mark can never hide prose from a rule.
    #[test]
    fn an_unclosed_mark_suppresses_nothing() {
        assert_eq!(
            marked("The width is 3\" and nothing closes it here.\n"),
            ["authored:The width is 3\" and nothing closes it here."]
        );
    }

    /// A straight mark does not nest, because nothing in the source says which
    /// of two straight marks opens. The second mark closes the first, and the
    /// third opens again.
    #[test]
    fn straight_marks_pair_in_order_and_do_not_nest() {
        assert_eq!(
            marked("He said \"she said \"yes\" to it\" once.\n"),
            [
                "authored:He said ",
                "quoted:\"she said \"",
                "authored:yes",
                "quoted:\" to it\"",
                "authored: once."
            ]
        );
    }

    /// A curly mark says which end it is, so a curly quotation nests and the
    /// whole of it is marked once.
    #[test]
    fn curly_marks_nest_by_direction() {
        assert_eq!(
            marked("He said \u{201C}she said \u{201C}yes\u{201D} to it\u{201D} once.\n"),
            [
                "authored:He said ",
                "quoted:\u{201C}she said \u{201C}yes\u{201D} to it\u{201D}",
                "authored: once."
            ]
        );
    }

    /// An apostrophe marks nothing. `'` is a possessive here at least as often
    /// as it is a quotation, and the parse cannot tell the two apart, so a rule
    /// that read it would suppress the rest of every sentence that owns
    /// something.
    #[test]
    fn an_apostrophe_marks_nothing() {
        assert_eq!(
            marked("The author's own sentence, and 'this' as well.\n"),
            ["authored:The author's own sentence, and 'this' as well."]
        );
    }

    /// A quotation opens in one run and closes in another. A code span inside
    /// it stays code, because a code span is not prose whoever wrote it, and
    /// the text on either side of the span is the quoted author's.
    ///
    /// This is not hypothetical. `docs/evaluations/n8n-worked-example.md:427`
    /// quotes a sentence of another project that holds a code span, and it
    /// carried a `language.controlled.not_met` suppression for that reason.
    #[test]
    fn a_quotation_closes_in_a_later_run_and_keeps_a_code_span_as_code() {
        assert_eq!(
            marked("It reads \"do not link `testing/`\", which is wrong.\n"),
            [
                "authored:It reads ",
                "quoted:\"do not link ",
                "code:testing/",
                "quoted:\"",
                "authored:, which is wrong."
            ]
        );
    }

    /// A `Text` event never spans a line, so a quotation over a wrapped source
    /// line arrives as three runs and the soft break between them is inside the
    /// quotation.
    #[test]
    fn a_quotation_survives_a_wrapped_source_line() {
        assert_eq!(
            marked("It reads \"one phrase\nand another\" here.\n"),
            [
                "authored:It reads ",
                "quoted:\"one phrase",
                "quoted: ",
                "quoted:and another\"",
                "authored: here."
            ]
        );
    }

    /// A quotation never crosses a block boundary. The state is per block, so
    /// an unclosed mark in one paragraph cannot reach into the next.
    #[test]
    fn a_mark_does_not_reach_the_next_block() {
        let body = body_of("An open \" mark here.\n\nAnd the next paragraph.\n");
        assert_eq!(body.blocks[1].runs[0].ownership, Ownership::Authored);
        assert_eq!(body.blocks[1].runs[0].text, "And the next paragraph.");
    }

    /// Inside a block quotation every run is already another author's, so this
    /// pass has nothing to mark and does not split the block at a mark.
    #[test]
    fn a_mark_inside_a_block_quotation_splits_nothing() {
        assert_eq!(
            marked("> He said \"yes\" to it.\n"),
            ["quoted:He said \"yes\" to it."]
        );
    }

    fn body_of(source: &str) -> Body {
        scan(source, source, 0)
    }

    /// The fact a source-form rule reads. A paragraph written over two lines
    /// holds one soft break, and the break points at the line the source wrapped
    /// onto.
    /// The fact a source-form rule reads. A paragraph written over two lines
    /// holds one soft break, and the break ends where the source wrapped onto.
    ///
    /// The block span is no substitute. It ends on the line after the last one
    /// the paragraph writes, because the range the parser reports runs to the
    /// start of the next block, and a rule that compared its two line numbers
    /// would report every paragraph in the corpus.
    #[test]
    fn a_wrapped_paragraph_records_the_break() {
        let body = body_of("One sentence that the source\nbroke over two lines.\n");
        let paragraph = &body.blocks[0];
        assert_eq!(paragraph.kind, BlockKind::Paragraph);
        assert_eq!(paragraph.soft_breaks.len(), 1);
        assert_eq!(paragraph.soft_breaks[0].end.line, 2);
        assert_eq!(paragraph.soft_breaks[0].end.col, 1);
        assert_eq!((paragraph.span.start.line, paragraph.span.end.line), (1, 3));
    }

    /// The distinction the parser exists to make here. A trailing backslash is a
    /// break the author asked for, and `CLAUDE.md` allows it. Nothing that counts
    /// lines can tell the two apart, and this can.
    #[test]
    fn a_backslash_break_is_not_a_soft_break() {
        let body = body_of("One line the author broke\\\non purpose.\n");
        let paragraph = &body.blocks[0];
        assert_eq!((paragraph.span.start.line, paragraph.span.end.line), (1, 3));
        assert!(paragraph.soft_breaks.is_empty());
    }

    /// One line is one block with nothing recorded, which is the shape of every
    /// paragraph this repository writes.
    #[test]
    fn a_paragraph_on_one_line_records_nothing() {
        let body = body_of("One sentence on one line.\n");
        assert!(body.blocks[0].soft_breaks.is_empty());
    }

    /// A fenced block is verbatim, and a rule about the source form has no
    /// business inside it. The scan keeps its breaks on the code block, where the
    /// rule that skips code never reads them.
    #[test]
    fn a_code_block_keeps_its_breaks_on_the_code_block() {
        let body = body_of("```\nfirst\nsecond\n```\n");
        assert_eq!(body.blocks[0].kind, BlockKind::Code);
    }
}
