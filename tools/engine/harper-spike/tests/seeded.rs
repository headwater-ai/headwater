// SPDX-License-Identifier: Apache-2.0
//! The seeded-defect set of #1013: one sentence for each category that
//! `language.rs` holds today, plus the two Harper categories the issue names.
//! The test prints what Harper found for each and asserts only what the
//! evaluation reports as found, so a Harper upgrade that changes a verdict
//! fails here.

const FIXTURE: &str = include_str!("fixtures/seeded.md");

fn rules_on_line(rows: &[harper_spike::Row], needle: &str) -> Vec<String> {
    let line = FIXTURE
        .lines()
        .position(|l| l.contains(needle))
        .expect("a seeded line")
        + 1;
    let mut rules: Vec<String> = rows
        .iter()
        .filter(|r| r.line == line)
        .map(|r| format!("{}:{}", r.rule, r.flagged))
        .collect();
    rules.sort();
    rules
}

#[test]
fn seeded_defects() {
    let mut linter = harper_spike::linter();
    let rows = harper_spike::authored("seeded.md", FIXTURE, &mut linter);
    for row in &rows {
        eprintln!("{}", row.tsv());
    }
    let contraction = rules_on_line(&rows, "doesn't");
    let british = rules_on_line(&rows, "organisation");
    let doubled = rules_on_line(&rows, "the the");
    let article = rules_on_line(&rows, "a element");
    let retired = rules_on_line(&rows, "leverage");
    let semicolon = rules_on_line(&rows, "reads it;");
    let long = rules_on_line(&rows, "census found");
    eprintln!("contraction {contraction:?}");
    eprintln!("british {british:?}");
    eprintln!("doubled {doubled:?}");
    eprintln!("article {article:?}");
    eprintln!("retired {retired:?}");
    eprintln!("semicolon {semicolon:?}");
    eprintln!("long {long:?}");

    assert!(doubled.iter().any(|r| r.starts_with("RepeatedWords")));
    assert!(article.iter().filter(|r| r.starts_with("AnA")).count() == 2);
    assert!(british.iter().any(|r| r.contains("organisation")));
}
