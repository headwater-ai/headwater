// SPDX-License-Identifier: Apache-2.0
//! Every row of `engine-ranges.tsv` against `release::satisfies`.
//!
//! The table is the one statement of the engine range syntax that three
//! evaluators answer to: this crate's, the Python one the two composite actions
//! run before they download an engine, and the `awk` one in `site/install.sh`.
//! This file holds the definition to the table, so a row the engine reads
//! differently fails here before either copy is asked about it.

use headwater_resolve::release::satisfies;

const TABLE: &str = include_str!("engine-ranges.tsv");

fn rows() -> Vec<(usize, &'static str, &'static str, &'static str)> {
    let mut rows = Vec::new();
    for (index, line) in TABLE.lines().enumerate() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        assert_eq!(
            fields.len(),
            3,
            "line {} of engine-ranges.tsv is not range<TAB>version<TAB>answer: {line:?}",
            index + 1
        );
        rows.push((index + 1, fields[0], fields[1], fields[2]));
    }
    rows
}

#[test]
fn every_row_of_the_range_table_is_what_the_engine_answers() {
    let rows = rows();
    assert!(
        rows.len() >= 20,
        "the table holds {} cases, and it held 39 when it was written",
        rows.len()
    );
    let mut wrong = Vec::new();
    for (line, range, version, want) in rows {
        let got = match satisfies(range, version) {
            Ok(true) => "yes",
            Ok(false) => "no",
            Err(_) => "error",
        };
        assert!(
            matches!(want, "yes" | "no" | "error"),
            "line {line} of engine-ranges.tsv answers {want:?}, which is not yes, no or error"
        );
        if got != want {
            wrong.push(format!(
                "line {line}: {range:?} against {version:?} is {got}, the table says {want}"
            ));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn the_table_holds_each_answer_and_the_case_a_string_comparison_gets_wrong() {
    let rows = rows();
    for answer in ["yes", "no", "error"] {
        assert!(
            rows.iter().any(|row| row.3 == answer),
            "no row of engine-ranges.tsv answers {answer}"
        );
    }
    // `0.10.0` sorts below `0.6` as text and above it as numbers, so an
    // evaluator that compares strings answers this row wrong.
    assert!(
        rows.iter()
            .any(|row| row.1 == ">=0.6" && row.2 == "0.10.0" && row.3 == "yes"),
        "the numeric-not-lexical row is gone from engine-ranges.tsv"
    );
}
