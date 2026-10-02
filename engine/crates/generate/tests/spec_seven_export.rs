// SPDX-License-Identifier: Apache-2.0
//! Spec 7's export section, held against [`headwater_generate::Grain`], and
//! held as the one home of the export rules.
//!
//! # Why this file exists
//!
//! The export rules used to live in spec 6, and #1572 moved them to spec 7
//! (`docs/spec/07-distribution-and-federation.md#what-leaves-a-corpus`).
//! A rule that two specification parts state drifts in one of them, which is
//! the defect the move removed. So this file holds three things:
//!
//! - the tombstone grain table of spec 7 names exactly the grains the engine
//!   reads, in both directions;
//! - spec 6 states neither the grain table nor the filter rule list again, and
//!   shares no sentence of six words or more with spec 7's export section;
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
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../docs/spec")
        .join(name)
}

fn spec_six() -> String {
    std::fs::read_to_string(spec("06-engine-architecture.md")).expect("read spec 6")
}

fn spec_seven() -> String {
    std::fs::read_to_string(spec("07-distribution-and-federation.md")).expect("read spec 7")
}

/// The backticked first-column values of the `| Grain |` table.
fn grain_table(text: &str) -> BTreeSet<String> {
    let mut lines = text
        .lines()
        .skip_while(|l| !l.starts_with(GRAIN_TABLE_HEADER));
    assert!(
        lines.next().is_some(),
        "spec 7 has no `{GRAIN_TABLE_HEADER}` table"
    );
    lines
        .skip(1) // the separator row
        .take_while(|l| l.starts_with('|'))
        .map(|row| {
            let cell = row
                .trim_start_matches('|')
                .split('|')
                .next()
                .unwrap_or("")
                .trim();
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
    assert!(
        lines.next().is_some(),
        "spec 7 holds no sentence with {FILTER_RULES_LEAD:?}"
    );
    lines
        .skip_while(|l| l.trim().is_empty())
        .take_while(|l| l.starts_with("- "))
        .map(str::to_owned)
        .collect()
}

#[test]
fn the_tombstone_grain_table_of_spec_7_names_exactly_the_grains_the_engine_reads() {
    let table = grain_table(&spec_seven());
    let engine: BTreeSet<String> = ENGINE_GRAINS
        .map(|g| g.name().to_owned())
        .into_iter()
        .collect();
    assert_eq!(table, engine, "spec 7's grain table and `Grain` disagree");
}

/// The first cell of a table row, with its spacing and backticks gone, so
/// that `| \`counted\` |` and `|\`counted\`|` read the same.
fn first_cell(row: &str) -> Option<String> {
    let row = row.trim();
    let rest = row.strip_prefix('|')?;
    let cell = rest.split('|').next()?.trim().trim_matches('`').trim();
    Some(cell.to_lowercase())
}

#[test]
fn spec_6_states_no_grain_table_and_no_filter_rule_list() {
    let six = spec_six();
    let grains: Vec<&str> = ENGINE_GRAINS.iter().map(|g| g.name()).collect();
    for line in six.lines() {
        if let Some(cell) = first_cell(line) {
            assert!(
                !grains.contains(&cell.as_str()),
                "spec 6 states a grain row again: {line}"
            );
            assert!(
                cell != "grain",
                "spec 6 states the grain table again: {line}"
            );
        }
    }
    assert!(
        !six.contains(FILTER_RULES_LEAD),
        "spec 6 states the filter rule list again; spec 7 is its one home"
    );
}

/// The heading of the spec 7 section that holds every export rule.
const EXPORT_SECTION: &str = "## What leaves a corpus";

/// The fewest words a sentence fragment carries before a copy of it counts.
/// Shorter fragments, such as "Nothing else", recur in unrelated prose.
const MIN_FRAGMENT_WORDS: usize = 6;

/// Markdown reduced to the words a reader sees: no emphasis, no backticks,
/// link text kept and link targets dropped, lower case, one space between
/// words.
fn plain(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(i) = rest.find("](") {
        out.push_str(&rest[..i]);
        rest = &rest[i + 2..];
        rest = rest.find(')').map_or("", |j| &rest[j + 1..]);
    }
    out.push_str(rest);
    let out = out.replace(['*', '`', '['], "").to_lowercase();
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The body of spec 7's export section, from its heading to the next `## `.
fn export_section(text: &str) -> Vec<&str> {
    let mut lines = text.lines().skip_while(|l| *l != EXPORT_SECTION);
    assert!(lines.next().is_some(), "spec 7 has no `{EXPORT_SECTION}`");
    lines.take_while(|l| !l.starts_with("## ")).collect()
}

/// Every sentence fragment of the section that is long enough to count: a
/// line is cut at each table cell, each sentence end and each colon.
fn fragments(lines: &[&str]) -> BTreeSet<String> {
    lines
        .iter()
        .filter(|l| !l.starts_with('#') && !l.starts_with("|---"))
        .flat_map(|l| l.split('|'))
        .map(plain)
        .flat_map(|l| {
            l.split(['.', ':', ';'])
                .map(|s| s.trim().trim_start_matches("- ").to_owned())
                .collect::<Vec<_>>()
        })
        .filter(|s| s.split_whitespace().count() >= MIN_FRAGMENT_WORDS)
        .collect()
}

#[test]
fn spec_6_shares_no_sentence_with_the_export_section_of_spec_7() {
    let seven = spec_seven();
    let section = export_section(&seven);
    let six = plain(&spec_six().lines().collect::<Vec<_>>().join(" "));
    let copies: Vec<String> = fragments(&section)
        .into_iter()
        .filter(|f| six.contains(f.as_str()))
        .collect();
    assert!(
        copies.is_empty(),
        "spec 6 states again what spec 7's export section states: {copies:#?}"
    );
}

/// Words that only the moved export rules use. A line that names spec 6, read
/// with the two lines after it, may not carry one: the rule it credits now
/// lives in spec 7. The list is narrow, because a wider one ("census",
/// "native", "generated") fires on 26 lines that name text spec 6 keeps.
/// [`every_quote_of_spec_7_export_text_credits_spec_7`] holds the quotes this
/// list cannot see.
const MOVED_RULE_MARKERS: [&str; 22] = [
    "06-engine-architecture.md#an-export",
    "06-engine-architecture.md#what-a-filtered",
    "`counted`",
    "three classes",
    "projector defect",
    "one layer out",
    "the hole",
    "export\nis committed",
    "filtered export",
    "filtered view",
    "carry the warrant",
    "export is committed",
    "export is\n",
    "graph emitters",
    "an entry under `projections`",
    "projection like the others",
    "export that leaves",
    "tombstone",
    "withholding",
    "default-deny",
    "presents as total",
    "fails closed",
];

/// Lines that name spec 6 beside a marker and are right to: each names text
/// that stays in spec 6, or is not a credit at all. The second member is a
/// piece of the line itself, so an edit to that line drops it from this list.
const STAYS_IN_SPEC_6: [(&str, &str); 3] = [
    // A sentence splitter test input, not a credit.
    (
        "crates/doc/src/sentences.rs",
        "cf. the projection census of spec 6",
    ),
    // This file names spec 6 to say what spec 6 no longer holds.
    (
        "crates/generate/tests/spec_seven_export.rs",
        "spec 6 states the filter rule list again",
    ),
    (
        "crates/generate/tests/spec_seven_export.rs",
        "spec 6 states neither the grain table",
    ),
];

fn engine_sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("read an engine directory") {
        let path = entry.expect("read a directory entry").path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if path.is_dir() {
            if name != "target" && !name.starts_with('.') {
                engine_sources(&path, out);
            }
        } else if name.ends_with(".rs") || name.ends_with(".yml") {
            out.push(path);
        }
    }
}

#[test]
fn no_engine_source_credits_a_moved_export_rule_to_spec_6() {
    let engine = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut files = Vec::new();
    engine_sources(&engine.join("crates"), &mut files);
    assert!(files.len() > 50, "the walk found {} files", files.len());
    let mut stale = Vec::new();
    for file in files {
        let text = std::fs::read_to_string(&file).expect("read an engine source");
        let rel = file
            .strip_prefix(&engine)
            .unwrap_or(&file)
            .to_string_lossy()
            .replace('\\', "/");
        let lines: Vec<&str> = text.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if !line.to_lowercase().contains("spec 6") {
                continue;
            }
            let window = lines[i.saturating_sub(1)..lines.len().min(i + 3)]
                .iter()
                .map(|l| l.trim().trim_start_matches(['/', '!', '#']).trim())
                .collect::<Vec<_>>()
                .join("\n")
                .to_lowercase();
            let window = format!("{window}\n");
            let allowed = STAYS_IN_SPEC_6
                .iter()
                .any(|(path, text)| rel.ends_with(path) && line.contains(text));
            if let Some(marker) = MOVED_RULE_MARKERS.iter().find(|m| window.contains(**m)) {
                if !allowed {
                    stale.push(format!("{rel}:{}: {marker:?}: {}", i + 1, line.trim()));
                }
            }
        }
    }
    assert!(
        stale.is_empty(),
        "these lines credit to spec 6 an export rule that spec 7 now states: {stale:#?}"
    );
}

/// The text inside the first pair of double quotes after `at` in `text`,
/// where the quote opens within 80 bytes of `at`.
fn quote_after(text: &str, at: usize) -> Option<&str> {
    let rest = &text[at..];
    let open = rest.find('"')?;
    if open > 80 {
        return None;
    }
    let body = &rest[open + 1..];
    let close = body.find('"')?;
    Some(&body[..close])
}

/// A comment that quotes a sentence of spec 7's export section and credits it
/// to spec 6 points a reader at a part that no longer holds it. This reads
/// every comment that names spec 6 and then opens a quote, and fails when the
/// quote is text of that section. A quote of text spec 6 keeps is not this
/// test's business.
#[test]
fn every_quote_of_spec_7_export_text_credits_spec_7() {
    let engine = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let seven = spec_seven();
    let export = plain(&export_section(&seven).join(" "));
    let mut files = Vec::new();
    engine_sources(&engine.join("crates"), &mut files);
    let mut missing = Vec::new();
    let mut quotes = 0;
    for file in files {
        let text = std::fs::read_to_string(&file).expect("read an engine source");
        let rel = file
            .strip_prefix(&engine)
            .unwrap_or(&file)
            .to_string_lossy()
            .replace('\\', "/");
        let lines: Vec<&str> = text.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let Some(at) = line.to_lowercase().find("spec 6") else {
                continue;
            };
            // Only comments credit a sentence. A string literal that names
            // spec 6 is a message or a test input.
            let trimmed = line.trim_start();
            if !(trimmed.starts_with("//") || trimmed.starts_with('#')) {
                continue;
            }
            let joined = lines[i..lines.len().min(i + 6)]
                .iter()
                .map(|l| l.trim().trim_start_matches(['/', '!', '#']).trim())
                .collect::<Vec<_>>()
                .join(" ");
            let at = at - (line.len() - line.trim().len()).min(at);
            let at = joined.to_lowercase().find("spec 6").unwrap_or(at);
            let Some(quote) = quote_after(&joined, at) else {
                continue;
            };
            let quote = plain(quote);
            let quote = quote.trim_end_matches(['.', ',']);
            if quote.split_whitespace().count() < 4 {
                continue;
            }
            quotes += 1;
            if export.contains(quote) {
                missing.push(format!("{rel}:{}: {quote:?}", i + 1));
            }
        }
    }
    assert!(quotes > 0, "no engine comment quotes spec 6 at all");
    assert!(
        missing.is_empty(),
        "these lines credit to spec 6 a sentence of spec 7's export section: {missing:#?}"
    );
}

#[test]
fn spec_7_states_six_filter_rules() {
    let rules = filter_rules(&spec_seven());
    assert_eq!(
        rules.len(),
        6,
        "spec 7 states {} filter rules, not six: {rules:#?}",
        rules.len()
    );
}
