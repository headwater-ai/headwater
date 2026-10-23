// SPDX-License-Identifier: Apache-2.0
//! The BERT uncased WordPiece tokenizer the pinned model was trained with.
//!
//! Hand-written rather than the `tokenizers` crate, whose default build links a
//! C regex library. The model's `tokenizer_config.json` states `do_lower_case`
//! and a null `strip_accents`, and BERT reads a null there as "strip when
//! lowercasing", so the text is lowercased, decomposed, and loses its combining
//! marks before it is split.

use std::collections::HashMap;
use unicode_normalization::UnicodeNormalization;

/// A word longer than this is one unknown token, as BERT's own tokenizer does.
const LONGEST_WORD: usize = 100;

#[derive(Debug)]
pub struct WordPiece {
    vocab: HashMap<String, i64>,
    cls: i64,
    sep: i64,
    unk: i64,
}

impl WordPiece {
    /// A vocabulary is one token per line, and the line number is its id.
    pub fn parse(vocab: &str) -> Result<WordPiece, String> {
        let vocab: HashMap<String, i64> = vocab
            .lines()
            .zip(0_i64..)
            .map(|(token, id)| (token.to_string(), id))
            .collect();
        let special = |name: &str| {
            vocab
                .get(name)
                .copied()
                .ok_or_else(|| format!("the vocabulary has no `{name}` token"))
        };
        Ok(WordPiece {
            cls: special("[CLS]")?,
            sep: special("[SEP]")?,
            unk: special("[UNK]")?,
            vocab,
        })
    }

    /// The ids of a text, opened by `[CLS]`, closed by `[SEP]`, and at most
    /// `limit` long.
    pub fn encode(&self, text: &str, limit: usize) -> Vec<i64> {
        let mut ids = vec![self.cls];
        for word in words(text) {
            self.pieces(&word, &mut ids);
        }
        ids.truncate(limit.saturating_sub(1).max(1));
        ids.push(self.sep);
        ids
    }

    /// The ids and the token type ids of a (query, document) pair, as a BERT
    /// cross-encoder reads it: `[CLS] query [SEP] document [SEP]`, with type 0
    /// for `[CLS]`, the query and the first `[SEP]`, and type 1 for the rest.
    /// The whole is at most `limit` long. When it is longer, one piece at a
    /// time leaves the end of the longer side, and the document side on a
    /// tie, which is the Hugging Face `longest_first` truncation.
    pub fn encode_pair(&self, query: &str, document: &str, limit: usize) -> (Vec<i64>, Vec<i64>) {
        let mut first = Vec::new();
        for word in words(query) {
            self.pieces(&word, &mut first);
        }
        let mut second = Vec::new();
        for word in words(document) {
            self.pieces(&word, &mut second);
        }
        let budget = limit.saturating_sub(3);
        while first.len() + second.len() > budget {
            if first.len() > second.len() {
                first.pop();
            } else {
                second.pop();
            }
        }
        let mut ids = Vec::with_capacity(first.len() + second.len() + 3);
        ids.push(self.cls);
        ids.extend(first);
        ids.push(self.sep);
        let query_side = ids.len();
        ids.extend(second);
        ids.push(self.sep);
        let types = (0..ids.len())
            .map(|at| i64::from(at >= query_side))
            .collect();
        (ids, types)
    }

    /// Greedy longest match from the left, with `##` on every piece but the
    /// first. A word with any unmatched stretch is one unknown token.
    fn pieces(&self, word: &str, ids: &mut Vec<i64>) {
        let chars: Vec<char> = word.chars().collect();
        if chars.len() > LONGEST_WORD {
            ids.push(self.unk);
            return;
        }
        let mut found = Vec::new();
        let mut start = 0;
        while start < chars.len() {
            let mut end = chars.len();
            let mut piece = None;
            while start < end {
                let text: String = chars[start..end].iter().collect();
                let key = match start {
                    0 => text,
                    _ => format!("##{text}"),
                };
                if let Some(id) = self.vocab.get(&key) {
                    piece = Some(*id);
                    break;
                }
                end -= 1;
            }
            let Some(id) = piece else {
                ids.push(self.unk);
                return;
            };
            found.push(id);
            start = end;
        }
        ids.extend(found);
    }
}

/// BERT's basic split: whitespace separates, and each punctuation mark and
/// each CJK ideograph is a word of its own.
fn words(text: &str) -> Vec<String> {
    let folded: String = text
        .to_lowercase()
        .nfd()
        .filter(|c| !unicode_normalization::char::is_combining_mark(*c))
        .collect();
    let mut words = Vec::new();
    let mut current = String::new();
    for c in folded.chars() {
        if c.is_whitespace() || c.is_control() {
            flush(&mut current, &mut words);
        } else if is_punctuation(c) || is_cjk(c) {
            flush(&mut current, &mut words);
            words.push(c.to_string());
        } else {
            current.push(c);
        }
    }
    flush(&mut current, &mut words);
    words
}

fn flush(current: &mut String, words: &mut Vec<String>) {
    if !current.is_empty() {
        words.push(std::mem::take(current));
    }
}

fn is_punctuation(c: char) -> bool {
    c.is_ascii_punctuation() || (!c.is_alphanumeric() && !c.is_whitespace() && !c.is_control())
}

fn is_cjk(c: char) -> bool {
    matches!(
        u32::from(c),
        0x4E00..=0x9FFF
            | 0x3400..=0x4DBF
            | 0x20000..=0x2A6DF
            | 0x2A700..=0x2B73F
            | 0x2B740..=0x2B81F
            | 0x2B820..=0x2CEAF
            | 0xF900..=0xFAFF
            | 0x2F800..=0x2FA1F
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vocab() -> WordPiece {
        WordPiece::parse("[PAD]\n[UNK]\n[CLS]\n[SEP]\nhead\n##water\nnaive\n,\n.\nroute\n'\ns\n")
            .expect("the fixture vocabulary holds the special tokens")
    }

    #[test]
    fn a_word_splits_into_the_longest_pieces_the_vocabulary_holds() {
        assert_eq!(vocab().encode("Headwater", 256), vec![2, 4, 5, 3]);
    }

    #[test]
    fn accents_are_stripped_and_punctuation_is_a_word_of_its_own() {
        assert_eq!(vocab().encode("Naïve, route.", 256), vec![2, 6, 7, 9, 8, 3]);
    }

    #[test]
    fn a_word_with_an_unmatched_stretch_is_one_unknown_token() {
        assert_eq!(vocab().encode("headwaterx route", 256), vec![2, 1, 9, 3]);
    }

    #[test]
    fn the_limit_counts_both_special_tokens() {
        assert_eq!(vocab().encode("route route route", 3), vec![2, 9, 3]);
    }

    #[test]
    fn a_pair_carries_type_zero_for_the_query_and_type_one_for_the_document() {
        let (ids, types) = vocab().encode_pair("route", "Headwater.", 512);
        assert_eq!(ids, vec![2, 9, 3, 4, 5, 8, 3]);
        assert_eq!(types, vec![0, 0, 0, 1, 1, 1, 1]);
    }

    #[test]
    fn a_long_pair_loses_pieces_from_the_longer_side_first() {
        // Query 5 pieces, document 2: a limit of 8 leaves 5 for both sides, so
        // two pieces leave the query and the document keeps both of its own.
        let (ids, types) = vocab().encode_pair("route route route route route", "head s", 8);
        assert_eq!(ids, vec![2, 9, 9, 9, 3, 4, 11, 3]);
        assert_eq!(types, vec![0, 0, 0, 0, 0, 1, 1, 1]);
        // Both sides 3 long and a budget of 5: the tie takes from the document,
        // so the query keeps three and the document two.
        let (ids, _) = vocab().encode_pair("route route route", "head head head", 8);
        assert_eq!(ids, vec![2, 9, 9, 9, 3, 4, 4, 3]);
    }

    #[test]
    fn a_vocabulary_without_the_special_tokens_is_refused() {
        assert!(WordPiece::parse("head\n").is_err());
    }
}
