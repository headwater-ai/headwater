// SPDX-License-Identifier: Apache-2.0
//! The patch a check offers beside its finding.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#fixability): "A check may
//! return a patch alongside a finding… A fix is offered only when it is
//! **mechanical and total** — one correct outcome, derivable without judgment."
//! This module is that patch, and [`crate::Finding::fixable`] reads it rather
//! than carrying a second opinion beside it.
//!
//! # The flag this type replaced, and the obligation that asked for it
//!
//! [HW-OBL-0087](../../../../docs/obligations/0087-fixable-has-two-readings-inside-one-engine.md)
//! records that `fixable` carried two readings at once. One says the defect has
//! a mechanical remedy, and the other says the engine will apply it. The record
//! says where they meet: "`check --fix` is where the two meet, and until it
//! exists a reader of the report cannot tell which sense a `fixable` flag
//! carries."
//!
//! The ruling is the second reading, and it is the presence of this value that
//! carries it. A finding is fixable when a patch rides with it and never
//! otherwise, so no report can claim a capability that no code path has. A
//! defect whose remedy is mechanical and that this engine does not write is an
//! **error** by severity and carries remediation prose, which is where
//! `CLAUDE.md` already puts the first reading.
//!
//! # Three shapes, and only the first one edits prose
//!
//! [`Patch::Text`] replaces a byte range of one file. [`Patch::Half`] declares
//! one edge half in a document's front matter, and the writer for it is
//! [`headwater_scaffold::write::splice`] rather than anything here. Two writers
//! into one `relations:` block would disagree the first time a document nested
//! differently, and the splice already carries the read-back guard that makes a
//! wrong guess a refusal.
//!
//! [`Patch::Create`] makes a file that does not exist. It is the one shape that
//! touches no document: the file it writes is a claim of
//! [`crate::claim`]'s store, which holds no prose and which nothing in this
//! engine ever modifies. **Its writer opens with create-new semantics and
//! refuses an occupied path**, and that is a bar rather than a convenience. A
//! claim file is the only record of which document minted an identifier, the
//! tree does not hold it, and no rule repairs a claim that was overwritten. So
//! a fixer that truncated one would destroy a fact a person cannot
//! reconstruct, which is a worse failure than any wrong report this module can
//! produce.
//!
//! # Nothing here writes, and nothing here reads a file
//!
//! A check receives one document through [`crate::scope::DocumentView`] and no
//! filesystem. So a patch states an offset and **what the check believes lies
//! at it**, and the applier holds the belief against the bytes before it moves
//! one. `expect` is that statement. It is the first of the three guards
//! [`headwater_scaffold::fix`] runs.

use headwater_doc::body::{Body, Ownership};
use headwater_doc::Sentence;

/// One mechanical correction, as a check states it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Patch {
    /// Replace `start..end` of `path` with `replacement`.
    ///
    /// Byte offsets into the file, from the spans the parser kept. `expect` is
    /// the text the check read at that range, and the applier refuses when the
    /// bytes disagree.
    Text {
        path: String,
        start: usize,
        end: usize,
        expect: String,
        replacement: String,
    },
    /// Declare `relation: id` under `relations:` in `path`.
    ///
    /// No offset, because the far document's shape decides where the half goes
    /// and [`headwater_scaffold::write::splice`] is the one thing that knows.
    Half {
        path: String,
        relation: String,
        id: String,
    },
    /// Make `path`, holding `contents`, where no file stands.
    ///
    /// No offset and no `expect`, because there is nothing to hold a belief
    /// against: the belief is that the path is free, and the create-new syscall
    /// is what answers it. An occupied path is a refusal and never an
    /// overwrite. See the module comment.
    Create { path: String, contents: String },
}

impl Patch {
    /// The file this patch writes, which is not always the file the finding is
    /// reported against. A reciprocal half is owed by the document at the far
    /// end of the half that exists.
    pub fn path(&self) -> &str {
        match self {
            Patch::Text { path, .. } => path,
            Patch::Half { path, .. } => path,
            Patch::Create { path, .. } => path,
        }
    }
}

/// The patch for a substitution inside one sentence of authored prose.
///
/// This is the one place a prose rule turns a word into an offset, so the rules
/// that share the shape — a contraction, a British spelling and a retired term
/// — cannot disagree about which occurrence they meant.
///
/// # What it refuses, and why each refusal is a `None` rather than a guess
///
/// The word is looked for in the **authored** runs that lie inside the
/// sentence, in the order the parser produced them. That is the same text the
/// rule read, because [`headwater_doc::Sentence::authored`] is the
/// concatenation of exactly those runs. So the first match here is the first
/// match there.
///
/// A run that does not occupy as many bytes as it reads is skipped. A source
/// that writes an entity or a backslash escape holds more bytes than its text
/// does, so an offset inside such a run names a position the reader never sees.
/// This is the strongest test available here, because a check receives no
/// filesystem. `expect` carries the belief to the applier, which holds it
/// against the bytes before it moves one.
///
/// A code span and a quoted run are skipped for the reason
/// [spec 3](../../../../docs/spec/03-authoring-and-lifecycle.md#what-a-lexical-rule-gets-wrong-and-where-posture-comes-from)
/// gives: neither is this author's prose, and a patch into one would rewrite
/// somebody else's words or a file name.
///
/// A word this function cannot place produces no patch, and the finding still
/// reports with its remediation prose. A finding with a wrong offset is the
/// failure this whole path exists to prevent, and a finding with no patch costs
/// one hand edit.
pub fn substitution(
    path: &str,
    body: &Body,
    sentence: &Sentence,
    word: &str,
    replacement: &str,
) -> Option<Patch> {
    if word.is_empty() || word == replacement {
        return None;
    }
    for block in &body.blocks {
        if block.quote_depth > 0 {
            continue;
        }
        for run in &block.runs {
            if run.ownership != Ownership::Authored {
                continue;
            }
            let (from, to) = (run.span.start.offset, run.span.end.offset);
            // The intersection, because one run holds every sentence of a
            // paragraph and a sentence is a window on it rather than a run of
            // its own.
            let low = from.max(sentence.span.start.offset);
            let high = to.min(sentence.span.end.offset);
            if low >= high {
                continue;
            }
            // The assumption stated and then tested, the way the splice tests
            // its indentation. An offset inside a run is a source offset only
            // when the run reads every byte that it spans.
            if to - from != run.text.len() {
                continue;
            }
            let Some(at) = word_at(&run.text, word, (low - from)..(high - from)) else {
                continue;
            };
            return Some(Patch::Text {
                path: path.to_string(),
                start: from + at,
                end: from + at + word.len(),
                expect: word.to_string(),
                replacement: replacement.to_string(),
            });
        }
    }
    None
}

/// A replacement raised to the case the author wrote.
///
/// A lexicon holds its entries in lower case and a sentence opens in upper
/// case, so a substitution that wrote the entry as declared would put a
/// lower-case word at the head of the sentence it repaired. The case is only
/// ever raised: `I` stays `I` where the author wrote `i`, because a table entry
/// that opens in upper case opens that way for a reason of its own.
pub fn matching_case(written: &str, replacement: &str) -> String {
    let raised = written.chars().next().is_some_and(char::is_uppercase)
        && !replacement.starts_with(char::is_uppercase);
    if !raised {
        return replacement.to_string();
    }
    let mut characters = replacement.chars();
    match characters.next() {
        None => replacement.to_string(),
        Some(first) => first.to_uppercase().collect::<String>() + characters.as_str(),
    }
}

/// The first occurrence of `word` inside `window`, at a word boundary.
///
/// A boundary rather than a substring, because `licence` sits inside
/// `licences` and a rule that reported the plural would then patch the
/// singular's letters out of the middle of it. The boundary is read from the
/// whole run rather than from the window, so a window that opens mid-word
/// still refuses.
fn word_at(text: &str, word: &str, window: std::ops::Range<usize>) -> Option<usize> {
    if !text.is_char_boundary(window.start) || !text.is_char_boundary(window.end) {
        return None;
    }
    let mut from = window.start;
    while let Some(offset) = text.get(from..window.end)?.find(word) {
        let at = from + offset;
        let before = text[..at].chars().next_back();
        let after = text[at + word.len()..].chars().next();
        let bounded = |c: Option<char>| match c {
            None => true,
            Some(c) => !c.is_alphanumeric() && c != '\'' && c != '\u{2019}',
        };
        if bounded(before) && bounded(after) {
            return Some(at);
        }
        from = at + word.len();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use headwater_doc::body::scan;

    fn sentence_patch(source: &str, word: &str, replacement: &str) -> Option<Patch> {
        let body = scan(source, source, 0);
        let sentences = body.sentences();
        sentences
            .iter()
            .find_map(|sentence| substitution("d.md", &body, sentence, word, replacement))
    }

    #[test]
    fn a_word_in_authored_prose_gets_the_offset_it_occupies() {
        let source = "The behaviour of a check.\n";
        let Some(Patch::Text {
            start,
            end,
            expect,
            replacement,
            ..
        }) = sentence_patch(source, "behaviour", "behavior")
        else {
            panic!("no patch");
        };
        assert_eq!(&source[start..end], "behaviour");
        assert_eq!(expect, "behaviour");
        assert_eq!(replacement, "behavior");
    }

    /// The hazard this whole module exists for. A code span reaches a rule with
    /// its backticks removed, so an offset taken from the text a reader sees
    /// lands one character early inside the source.
    #[test]
    fn a_word_inside_a_code_span_gets_no_patch() {
        assert_eq!(
            sentence_patch("The `behaviour` flag.\n", "behaviour", "behavior"),
            None
        );
    }

    /// A quoted block is another author's words, and no rule reads one. The
    /// guard is here as well, because a patch is the irreversible half.
    #[test]
    fn a_word_inside_a_block_quote_gets_no_patch() {
        assert_eq!(
            sentence_patch("> The behaviour they describe.\n", "behaviour", "behavior"),
            None
        );
    }

    /// A link's text is authored prose, and the parser removed the brackets. So
    /// the offset has to come from the run rather than from the sentence, and
    /// this is the fixture that says it does.
    #[test]
    fn a_word_inside_a_links_text_gets_the_offset_of_the_text() {
        let source = "Read [the behaviour note](x.md) first.\n";
        let Some(Patch::Text { start, end, .. }) = sentence_patch(source, "behaviour", "behavior")
        else {
            panic!("no patch");
        };
        assert_eq!(&source[start..end], "behaviour");
        assert_eq!(start, source.find("behaviour").unwrap());
    }

    /// One paragraph is one run and several sentences, so the sentence is a
    /// window on the run. A patch that searched the whole run would report the
    /// second sentence's word against the first sentence's finding.
    #[test]
    fn a_sentence_is_a_window_on_the_run_that_holds_it() {
        let source = "First is fine. The behaviour follows.\n";
        let body = scan(source, source, 0);
        let sentences = body.sentences();
        assert_eq!(sentences.len(), 2);
        assert_eq!(
            substitution("d.md", &body, &sentences[0], "behaviour", "behavior"),
            None
        );
        let Some(Patch::Text { start, .. }) =
            substitution("d.md", &body, &sentences[1], "behaviour", "behavior")
        else {
            panic!("no patch for the sentence that writes it");
        };
        assert_eq!(start, source.find("behaviour").unwrap());
    }

    #[test]
    fn a_word_that_is_only_part_of_a_longer_word_gets_no_patch() {
        assert_eq!(
            sentence_patch("Two licences here.\n", "licence", "license"),
            None
        );
    }

    /// A source that escapes an entity spends more bytes than its text reads,
    /// and the offsets still land. The parser gives the entity a run of its
    /// own, so the run that holds the word is its own source bytes. A change
    /// that merged the two runs would move this patch, which is what the
    /// fixture is here to catch.
    #[test]
    fn an_entity_beside_a_word_does_not_move_the_words_offset() {
        let source = "Ampersand &amp; behaviour together.\n";
        let Some(Patch::Text { start, end, .. }) = sentence_patch(source, "behaviour", "behavior")
        else {
            panic!("no patch");
        };
        assert_eq!(&source[start..end], "behaviour");
    }

    /// The guard itself, over a run built by hand. Nothing in this corpus
    /// produces one, and a patch is the irreversible half, so the refusal is
    /// tested rather than assumed.
    #[test]
    fn a_run_that_reads_fewer_bytes_than_it_spans_gets_no_patch() {
        use headwater_doc::body::{Block, BlockKind, Run};
        use headwater_doc::{Position, Span};
        let run = Run {
            text: "the behaviour of it".to_string(),
            // Two bytes wider than the text, which is what an escape does.
            span: Span::new(Position::new(1, 1, 0), Position::new(1, 22, 21)),
            ownership: Ownership::Authored,
        };
        let block = Block {
            kind: BlockKind::Paragraph,
            span: run.span,
            quote_depth: 0,
            runs: vec![run],
            soft_breaks: Vec::new(),
        };
        let body = Body {
            blocks: vec![block],
            links: Vec::new(),
        };
        let sentence = &body.sentences()[0];
        assert_eq!(
            substitution("d.md", &body, sentence, "behaviour", "behavior"),
            None
        );
    }
}
