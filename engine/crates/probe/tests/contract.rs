// SPDX-License-Identifier: Apache-2.0
//! The recorder contract, held to the intake it describes.
//!
//! # Why a document of the corpus is tested from a crate
//!
//! [Spec 15](../../../../docs/spec/15-the-recorder-contract.md) is the contract
//! a person reads before they write a recorder, and this repository holds no
//! recorder. So every reader of that document is somewhere else, and a key that
//! this engine started to refuse would reach them as a transcript that fails for
//! a reason the contract does not carry.
//!
//! Four closed key sets decide whether a transcript is read or refused whole,
//! and each one is an array in [`headwater_probe::intake`]. This test derives
//! the four tables of that document from the four arrays. A key added to an
//! array and not to a table fails here, and so does a key in a table that no
//! array holds.
//!
//! # The grain
//!
//! The tables are found by position rather than by the prose above them, so an
//! edit to a heading or to a paragraph moves nothing here. What this test reads
//! is the order of the tables, the first cell of each row, and the count word in
//! the heading over each table. Those three are the parts of the document that a
//! change to the engine invalidates.

use headwater_probe::intake::{CALL_KEYS, CONFIRMATIONS, EVENT_KEYS, IDENTITY_KEYS, PRODUCED_KEYS};
use std::path::{Path, PathBuf};

/// The document this test holds to the engine.
const CONTRACT: &str = "docs/spec/15-the-recorder-contract.md";

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root")
}

/// One table of the document: the heading over it, and the first cell of every
/// row whose first cell is one backticked token.
struct Table {
    heading: String,
    keys: Vec<String>,
}

/// Every table of the document, in the order it lists them.
fn tables(source: &str) -> Vec<Table> {
    let mut out: Vec<Table> = Vec::new();
    let mut heading = String::new();
    let mut inside = false;
    for line in source.lines() {
        if line.starts_with('#') {
            heading = line.trim_start_matches('#').trim().to_string();
            inside = false;
            continue;
        }
        if !line.starts_with('|') {
            inside = false;
            continue;
        }
        if !inside {
            out.push(Table {
                heading: heading.clone(),
                keys: Vec::new(),
            });
            inside = true;
        }
        // The first cell, which is a key where it is one backticked token.
        let cell = line
            .trim_start_matches('|')
            .split('|')
            .next()
            .unwrap_or("")
            .trim();
        if let Some(key) = cell.strip_prefix('`').and_then(|k| k.strip_suffix('`')) {
            if !key.is_empty() {
                out.last_mut().expect("a table").keys.push(key.to_string());
            }
        }
    }
    out
}

/// The English word a heading writes for a count, for the counts a closed set
/// of this contract reaches.
fn word(how_many: usize) -> &'static str {
    match how_many {
        1 => "one",
        2 => "two",
        3 => "three",
        4 => "four",
        5 => "five",
        6 => "six",
        7 => "seven",
        8 => "eight",
        9 => "nine",
        10 => "ten",
        11 => "eleven",
        12 => "twelve",
        other => panic!("no word for {other}: add it, or write the count as a digit"),
    }
}

/// Every key of every closed set is a row of the contract, and every row is a
/// key.
///
/// The comparison is over sorted lists rather than over sets, so a key written
/// twice fails as well as a key omitted. The order inside a table is the
/// author's, and the order of the four tables is not.
#[test]
fn the_four_tables_of_the_contract_are_the_four_closed_sets_of_the_intake() {
    let path = repository_root().join(CONTRACT);
    let source =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let tables = tables(&source);

    let declared: [(&str, &[&str]); 4] = [
        ("the run identity", &IDENTITY_KEYS),
        ("one event", &EVENT_KEYS),
        ("one tool call", &CALL_KEYS),
        ("one produced artifact", &PRODUCED_KEYS),
    ];
    assert_eq!(
        tables.len(),
        declared.len(),
        "{CONTRACT} holds {} tables and this test reads {}. The headings it found were {:?}",
        tables.len(),
        declared.len(),
        tables.iter().map(|t| &t.heading).collect::<Vec<_>>()
    );

    for (table, (what, keys)) in tables.iter().zip(declared) {
        let mut found = table.keys.clone();
        found.sort();
        let mut expected: Vec<String> = keys.iter().map(|key| key.to_string()).collect();
        expected.sort();
        assert_eq!(
            found, expected,
            "the table for {what} under `{}` is not the closed set the intake enforces",
            table.heading
        );
        let count = word(keys.len());
        assert!(
            table.heading.contains(count),
            "the heading `{}` does not say `{count}`, and the set it covers holds {} keys",
            table.heading,
            keys.len()
        );
    }
}

/// The five confirmations the contract numbers are the five this engine names
/// when it refuses a transcript.
///
/// Derived from the document rather than listed here, for the reason the four
/// tables above are: a report that names the confirmation a transcript failed
/// has to name it out of the same list the specification numbers, or a reader
/// who meets the name in a run and the name in the contract meets two strings
/// that drifted apart.
///
/// Each numbered item of the contract opens `**The taxonomy.**` and so on, and
/// the assertion is that the bolded opener of item *n* holds the *n*th member
/// of [`CONFIRMATIONS`] — case-folded, because a sentence opener is capitalized
/// and a report is not.
#[test]
fn the_five_confirmations_of_the_contract_are_the_five_this_engine_names() {
    let path = repository_root().join(CONTRACT);
    let source =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));

    let numbered: Vec<&str> = source
        .lines()
        .filter_map(|line| line.strip_prefix("1. **").or(line.strip_prefix("2. **")))
        .chain(source.lines().filter_map(|line| {
            line.strip_prefix("3. **")
                .or(line.strip_prefix("4. **"))
                .or(line.strip_prefix("5. **"))
        }))
        .filter_map(|rest| rest.split_once("**").map(|(opener, _)| opener))
        .collect();

    assert_eq!(
        numbered.len(),
        CONFIRMATIONS.len(),
        "{CONTRACT} numbers {} confirmations and this engine names {}. The openers it found were \
         {numbered:?}",
        numbered.len(),
        CONFIRMATIONS.len()
    );
    for (opener, confirmation) in numbered.iter().zip(CONFIRMATIONS) {
        let opener = opener.trim_end_matches('.').to_lowercase();
        assert_eq!(
            opener, confirmation,
            "the contract names `{opener}` and this engine reports `{confirmation}`"
        );
    }
}
