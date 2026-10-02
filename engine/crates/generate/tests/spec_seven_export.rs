// SPDX-License-Identifier: Apache-2.0
//! Spec 7's export section, held against [`headwater_generate::Grain`], and
//! held as the one home of the export rules.
//!
//! # Why this file exists
//!
//! The export rules used to live in spec 6, and #1572 moved them to spec 7
//! (`docs/spec/07-distribution-and-federation.md#exporting-what-leaves-a-corpus`).
//! A rule that two specification parts state drifts in one of them, which is
//! the defect the move removed. So this file holds three things:
//!
//! - the tombstone grain table of spec 7 names exactly the grains the engine
//!   reads, in both directions;
//! - spec 6 states neither the grain table nor the filter rule list again;
//! - spec 7 states the filter rules as a list of exactly six items.
//!
//! The private `_exhaustive` match makes a third `Grain` variant fail to
//! compile until [`ENGINE_GRAINS`] and the table change with it.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use headwater_generate::Grain;

/// The lead sentence of the filter rule list in spec 7. The list that
/// follows it is the list of the rules.
const FILTER_RULES_LEAD: &str = "rules make the filter honest";

/// The header line of the tombstone grain table.
const GRAIN_TABLE_HEADER: &str = "| Grain |";

/// Every grain the engine reads, in declaration order.
const ENGINE_GRAINS: [Grain; 2] = [Grain::Counted, Grain::Sealed];

/// A third variant fails to compile here until [`ENGINE_GRAINS`] and spec 7's
/// table name it.
fn _exhaustive(g: Grain) {
    match g {
        Grain::Counted | Grain::Sealed => {}
    }
}

fn spec(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../docs/spec").join(name)
}

fn spec_six() -> String {
    std::fs::read_to_string(spec("06-engine-architecture.md")).expect("read spec 6")
}

fn spec_seven() -> String {
    std::fs::read_to_string(spec("07-distribution-and-federation.md")).expect("read spec 7")
}

/// The backticked first-column values of the `| Grain |` table.
fn grain_table(text: &str) -> BTreeSet<String> {
    let mut lines = text.lines().skip_while(|l| !l.starts_with(GRAIN_TABLE_HEADER));
    assert!(lines.next().is_some(), "spec 7 has no `{GRAIN_TABLE_HEADER}` table");
    lines
        .skip(1) // the separator row
        .take_while(|l| l.starts_with('|'))
        .map(|row| {
            let cell = row.trim_start_matches('|').split('|').next().unwrap_or("").trim();
            assert!(
                cell.starts_with('`') && cell.ends_with('`') && cell.len() > 2,
                "a grain cell is one backticked name, found {cell:?}"
            );
            cell.trim_matches('`').to_owned()
        })
        .collect()
}

/// The top-level list items that follow the paragraph holding the lead.
fn filter_rules(text: &str) -> Vec<String> {
    let mut lines = text.lines().skip_while(|l| !l.contains(FILTER_RULES_LEAD));
    assert!(lines.next().is_some(), "spec 7 holds no sentence with {FILTER_RULES_LEAD:?}");
    lines
        .skip_while(|l| l.trim().is_empty())
        .take_while(|l| l.starts_with("- "))
        .map(str::to_owned)
        .collect()
}

#[test]
fn the_tombstone_grain_table_of_spec_7_names_exactly_the_grains_the_engine_reads() {
    let table = grain_table(&spec_seven());
    let engine: BTreeSet<String> = ENGINE_GRAINS.map(|g| g.name().to_owned()).into_iter().collect();
    assert_eq!(table, engine, "spec 7's grain table and `Grain` disagree");
}

#[test]
fn spec_6_states_no_grain_table_and_no_filter_rule_list() {
    let six = spec_six();
    for line in six.lines() {
        assert!(
            !line.starts_with("| `counted`") && !line.starts_with("| `sealed`"),
            "spec 6 states a grain row again: {line}"
        );
        assert!(!line.starts_with(GRAIN_TABLE_HEADER), "spec 6 states the grain table again");
    }
    assert!(
        !six.contains(FILTER_RULES_LEAD),
        "spec 6 states the filter rule list again; spec 7 is its one home"
    );
}

#[test]
fn spec_7_states_six_filter_rules() {
    let rules = filter_rules(&spec_seven());
    assert_eq!(rules.len(), 6, "spec 7 states {} filter rules, not six: {rules:#?}", rules.len());
}
