// SPDX-License-Identifier: Apache-2.0
//! The dispatch table, the arms and the synopsis, held against each other.
//!
//! [#257](https://github.com/headwater-ai/headwater/issues/257) measured four
//! hand-kept copies of the verb list and found three of them wrong. The message
//! a caller reads on an unknown verb listed fifteen names against seventeen
//! arms, and it omitted `probe` and `query`. The `USAGE` synopsis omitted
//! `query` and `taxonomy migrate`. Spec 6 kept a fourth copy as the CLI grammar.
//!
//! `headwater_verbs::VERBS` ended the first of those, and it ends it in one
//! direction by construction: `main` resolves the first word against the table
//! before it enters the arms, so **an arm the table does not carry never runs**.
//! Nothing enforces the other direction, and these tests are it. A name in the
//! table with no arm, and a name in either the arms or the table that the
//! synopsis omits, fail here.
//!
//! # Reading the arms out of the source is the point rather than a shortcut
//!
//! A test that listed the arms would be a fifth copy. This one reads them, so
//! the only way to add a verb without failing a test is to add it everywhere.
//! `crates/check/src/fragment.rs` walks the comments of every source file of
//! this engine for the same class of reason.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn source() -> String {
    let path: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/main.rs");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// Every `["word", …]` match arm of the dispatch, as `(first, second)`.
///
/// A pattern whose first element is a binding rather than a literal is not an
/// arm for a named verb: `[]`, `[other, ..]` and `[one]` are the catch-alls,
/// and they carry no name to compare. A `USAGE` line opens `[--`, which is not
/// a string literal either, so the same test excludes it.
fn arms(text: &str) -> Vec<(String, Option<String>)> {
    let mut found = Vec::new();
    for line in text.lines() {
        let Some(rest) = line.trim_start().strip_prefix("[\"") else {
            continue;
        };
        let Some((first, rest)) = rest.split_once('"') else {
            continue;
        };
        let second = rest
            .trim_start_matches([',', ' '])
            .strip_prefix('"')
            .and_then(|rest| rest.split_once('"'))
            .map(|(word, _)| word.to_string());
        found.push((first.to_string(), second));
    }
    found
}

#[test]
fn the_arms_and_the_dispatch_table_carry_the_same_verbs() {
    let text = source();
    let arms = arms(&text);
    assert!(
        arms.len() > 20,
        "the arm scan found {} patterns, which is too few to be the dispatch",
        arms.len()
    );
    let dispatched: BTreeSet<&str> = arms.iter().map(|(first, _)| first.as_str()).collect();
    let declared: BTreeSet<&str> = headwater_verbs::VERBS
        .iter()
        .map(|verb| verb.name)
        .collect();
    assert_eq!(
        declared, dispatched,
        "the dispatch table and the match arms of `main.rs` name different verbs"
    );
}

#[test]
fn the_second_words_of_each_verb_are_the_second_words_of_its_arms() {
    let text = source();
    let arms = arms(&text);
    for verb in headwater_verbs::VERBS {
        let dispatched: BTreeSet<&str> = arms
            .iter()
            .filter(|(first, _)| first == verb.name)
            .filter_map(|(_, second)| second.as_deref())
            .collect();
        let declared: BTreeSet<&str> = verb.words.iter().copied().collect();
        assert_eq!(
            declared, dispatched,
            "`{}` declares different second words from the ones its arms carry",
            verb.name
        );
    }
}

/// The synopsis is the surface a caller reads to find a verb, so a verb that is
/// not on it is a verb nobody types. `query` was the case that cost the most:
/// the verb states the wait that a declared name owes, and no caller could
/// reach the statement.
#[test]
fn every_verb_and_every_second_word_is_in_the_synopsis() {
    let text = source();
    let start = text
        .find("const USAGE: &str = \"\\\n")
        .expect("the synopsis");
    let usage = &text[start..];
    let end = usage.find("\n\";").expect("the end of the synopsis");
    let usage = &usage[..end];
    for verb in headwater_verbs::VERBS {
        for form in verb.forms() {
            assert!(
                usage.contains(&form),
                "`{form}` is a command line this binary dispatches and the synopsis omits it"
            );
        }
    }
}
