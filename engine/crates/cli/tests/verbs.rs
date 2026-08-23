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
//! # The synopsis is held in both directions, and that is [#309](https://github.com/headwater-ai/headwater/issues/309)
//!
//! One direction is not enough for a list a reader treats as authoritative.
//! `.claude/skills/fixtures.sh` derived the set of shipping verbs by grepping
//! the rendered `--help` text for left-aligned `headwater …` lines, so a line
//! appended to `USAGE` naming nothing this binary dispatches widened the set it
//! accepted, and no suite, hook or CI job reported it. That suite now reads the
//! generated verb index, and the synopsis carries the assertion it lost: every
//! left-aligned line of `USAGE` names a command line `VERBS` carries, and every
//! command line `VERBS` carries appears in `USAGE`. A name in either one and not
//! the other fails, and the failure names it.
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

/// The `USAGE` literal, from its opening quote to its closing one.
fn usage(text: &str) -> &str {
    let start = text
        .find("const USAGE: &str = \"\\\n")
        .expect("the synopsis");
    let usage = &text[start..];
    let end = usage.find("\n\";").expect("the end of the synopsis");
    &usage[..end]
}

/// Every command line the synopsis states, one per left-aligned line.
///
/// A left-aligned `headwater …` line is a synopsis entry; a continuation line
/// and a description line are both indented, and the prose below the synopsis
/// never opens a line with the binary's name. The first word after the name is
/// the verb. The second is a second word only when it is a bare lower-case
/// word: `<expression>`, `[--check]` and `--out` are an argument and a flag, and
/// a verb that takes no second word is followed by one of those or by nothing.
fn synopsis_forms(usage: &str) -> Vec<String> {
    let prefix = format!("{} ", headwater_verbs::BINARY);
    let mut found = Vec::new();
    for line in usage.lines() {
        let Some(rest) = line.strip_prefix(&prefix) else {
            continue;
        };
        let mut words = rest.split_whitespace();
        let Some(name) = words.next() else {
            continue;
        };
        let second = words
            .next()
            .filter(|word| word.chars().all(|c| c.is_ascii_lowercase()));
        found.push(match second {
            Some(word) => format!("{prefix}{name} {word}"),
            None => format!("{prefix}{name}"),
        });
    }
    found
}

/// The synopsis is the surface a caller reads to find a verb, so a verb that is
/// not on it is a verb nobody types. `query` was the case that cost the most:
/// the verb states the wait that a declared name owes, and no caller could
/// reach the statement.
#[test]
fn every_verb_and_every_second_word_is_in_the_synopsis() {
    let text = source();
    let usage = usage(&text);
    for verb in headwater_verbs::VERBS {
        for form in verb.forms() {
            assert!(
                usage.contains(&form),
                "`{form}` is a command line this binary dispatches and the synopsis omits it"
            );
        }
    }
}

/// The other direction, and the one [#309](https://github.com/headwater-ai/headwater/issues/309)
/// was filed about: a synopsis line naming a command line nothing dispatches.
///
/// The measurement in that issue was a single appended line,
/// `headwater nonsense           [--root <path>]`, which every suite, hook and
/// CI job of this repository accepted. This is where it fails, and the message
/// names the line.
#[test]
fn every_synopsis_line_names_a_command_line_this_binary_dispatches() {
    let text = source();
    let forms = synopsis_forms(usage(&text));
    assert!(
        forms.len() > 20,
        "the synopsis scan found {} command lines, which is too few to be the synopsis",
        forms.len()
    );
    let dispatched: BTreeSet<String> = headwater_verbs::VERBS
        .iter()
        .flat_map(|verb| verb.forms())
        .collect();
    for form in &forms {
        assert!(
            dispatched.contains(form),
            "the synopsis states `{form}` and this binary dispatches no such command line"
        );
    }
}

/// The verb index is what `.claude/skills/fixtures.sh` reads to decide which
/// verbs ship, so the column it reads is held here as well as by
/// `headwater generate --check`.
///
/// The two are not one guard twice. `generate --check` holds every byte of the
/// file and runs in CI; this holds the one column the suite parses and runs in
/// `cargo test`, where a contributor meets it before the push. The commit gate
/// runs neither.
#[test]
fn the_verb_index_names_exactly_the_verbs_this_binary_dispatches() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../docs/interfaces/README.md");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let indexed: BTreeSet<&str> = text
        .lines()
        .filter_map(|line| line.strip_prefix("| `"))
        .filter_map(|rest| rest.split_once("` |"))
        .map(|(name, _)| name)
        .collect();
    let declared: BTreeSet<&str> = headwater_verbs::VERBS
        .iter()
        .map(|verb| verb.name)
        .collect();
    assert_eq!(
        declared, indexed,
        "the verb index and the dispatch table name different verbs; \
         `headwater generate` writes that file"
    );
}
