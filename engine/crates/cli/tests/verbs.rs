// SPDX-License-Identifier: Apache-2.0
//! The dispatch table and the parser, held against each other.
//!
//! [#257](https://github.com/headwater-ai/headwater/issues/257) measured four
//! hand-kept copies of the verb list and found three of them wrong. The message
//! a caller reads on an unknown verb listed fifteen names against seventeen
//! arms, and it omitted `probe` and `query`. The `USAGE` synopsis omitted
//! `query` and `taxonomy migrate`. Spec 6 kept a fourth copy as the CLI grammar.
//!
//! [`headwater_verbs::VERBS`] ended the first of those. This file ends the
//! second, in both directions: a name in the table that the parser does not
//! answer to fails here, and a command line the parser answers to that the
//! table does not carry fails here, and each failure names the command line.
//!
//! # What replaced the source-text scrape, and why it is stronger
//!
//! Until
//! [HW-DR-0033](../../../../docs/decisions/0033-q33-whether-the-command-line-is-derived-and-who-a-flag-belongs-to.md)
//! four of the five cases in this file read `main.rs` as **text**: one function
//! scraped the `["word", …]` match arms out of it and another scraped the
//! `USAGE` literal. Both were parsers of Rust and of a string constant that
//! nothing held, and both went blind the moment either surface stopped being
//! written by hand — which is what the parser migration did to them.
//!
//! What they are replaced by is the command tree
//! [`headwater_cli::command`] hands back, walked to its leaves. That is the
//! parse itself
//! rather than a reading of the source that declares it, so a discrepancy this
//! file reports is one a caller would meet.
//!
//! # The third direction needs no case, because it does not compile
//!
//! A verb in the parser with no arm behind it was the direction the arm scrape
//! covered. It is now held by the compiler: `dispatch` in `main.rs` matches
//! `headwater_cli::Verb` exhaustively, so a variant added to the parser with no
//! arm is a build failure rather than a finding. **The wrong state is
//! unrepresentable**, which is why no case below looks for it.

use headwater_cli::command;
use std::collections::BTreeSet;
use std::path::Path;

/// Every command line the parser answers to, one per leaf of the command tree.
///
/// The walk is recursive rather than two levels deep. This surface is two deep
/// today, and a third level added under one of the three grouped verbs would be
/// reported by the same case rather than silently unread.
///
/// `build()` is called first, because the tree `clap` hands back before it is
/// built carries none of the commands `clap` adds on its own. A `help`
/// subcommand is the one this binary would meet, and the parser disables it —
/// with this call, re-enabling it fails here rather than passing unread. The
/// injected one is not the `headwater help` this binary carries: it brings a
/// copy of the whole tree under itself, and `headwater_cli::Verb::Help` is one
/// leaf with one positional.
fn parsed_forms() -> BTreeSet<String> {
    fn walk(prefix: &str, command: &clap::Command, found: &mut BTreeSet<String>) {
        let mut leaf = true;
        for word in command.get_subcommands() {
            leaf = false;
            walk(&format!("{prefix} {}", word.get_name()), word, found);
        }
        if leaf {
            found.insert(prefix.to_string());
        }
    }

    let mut command = command();
    command.build();
    let mut found = BTreeSet::new();
    walk(headwater_verbs::BINARY, &command, &mut found);
    found
}

/// The name the parser answers to is the name the table prints.
///
/// Every message of this binary opens with [`headwater_verbs::BINARY`], and so
/// does every form the verb index carries. A parser that called itself
/// something else would put a second name on the usage line alone, where
/// nothing reads it.
#[test]
fn the_parser_and_the_table_call_this_binary_the_same_thing() {
    assert_eq!(command().get_name(), headwater_verbs::BINARY);
}

/// The whole surface, both ways.
///
/// # This case was watched failing, in both directions and separately
///
/// A subcommand added to `headwater_cli::Verb` and not to
/// [`headwater_verbs::VERBS`] fails the second assertion, naming
/// `headwater <name>`. A [`headwater_verbs::Verb`] added to the table with no
/// variant behind it fails the first, naming the same form. Neither failure is
/// a count: each prints the command lines that are on one side and not the
/// other.
#[test]
fn the_parser_answers_to_exactly_the_command_lines_the_table_carries() {
    let parsed = parsed_forms();
    assert!(
        parsed.len() > 20,
        "the walk of the command tree found {} command lines, which is too few to be this surface",
        parsed.len()
    );
    let declared: BTreeSet<String> = headwater_verbs::VERBS
        .iter()
        .flat_map(|verb| verb.forms())
        .collect();

    let unparsed: Vec<&String> = declared.difference(&parsed).collect();
    assert!(
        unparsed.is_empty(),
        "the dispatch table carries {unparsed:?} and the parser answers to no such command line"
    );
    let undeclared: Vec<&String> = parsed.difference(&declared).collect();
    assert!(
        undeclared.is_empty(),
        "the parser answers to {undeclared:?} and the dispatch table carries no such command line"
    );
}

/// The second words of each grouped verb, held against the same tree.
///
/// The case above compares whole command lines, so a second word moved from one
/// verb to another fails there too. This one fails with the verb's name in the
/// message, which is what a reader of a failure wants first, and it is the
/// direct successor of the case that read the second element of each match arm.
#[test]
fn the_second_words_of_each_verb_are_the_second_words_the_parser_answers_to() {
    let mut command = command();
    command.build();
    for verb in headwater_verbs::VERBS {
        let found = command
            .get_subcommands()
            .find(|one| one.get_name() == verb.name);
        let parsed: BTreeSet<&str> = found
            .into_iter()
            .flat_map(|one| one.get_subcommands())
            .map(clap::Command::get_name)
            .collect();
        let declared: BTreeSet<&str> = verb.words.iter().map(|word| word.name).collect();
        assert_eq!(
            declared, parsed,
            "`{}` declares different second words from the ones the parser answers to",
            verb.name
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
