// SPDX-License-Identifier: Apache-2.0
//! The adapter sections of `docs/subsystems/checks-and-cache.md`, held
//! against the code they describe, and held as the one home of the CI-adapter
//! rules.
//!
//! # Why this file exists
//!
//! The CI-adapter rules used to live in spec 6, under `### CI adapters`, and
//! #1572 moved them to the design document of the crate that implements them.
//! A rule that two documents state drifts in one of them, which is the defect
//! the move removed. So this file holds these things:
//!
//! - the severity table of checks-and-cache is what [`sarif::level`] writes,
//!   for every [`Severity`];
//! - the escape-class table of checks-and-cache is what [`sarif::kind`]
//!   writes, for every [`Escape`];
//! - the three census outcomes that checks-and-cache names are the outcome
//!   fields of [`Census`], and no other;
//! - spec 6 shares no run of eight words with the adapter sections, and it
//!   states neither of two moved sentences;
//! - no engine comment credits spec 6 with a sentence that now lives in the
//!   adapter sections.
//!
//! Each private `_exhaustive` function makes a new variant or a new field fail
//! to compile here until the list beside it changes.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use headwater_adapter::{sarif, Census, Escape};
use headwater_check::Severity;

/// The first heading of the adapter sections, which follow `### The adapter
/// crate, and why it is a crate` under *Design*.
const FIRST: &str = "### A renderer is what the engine ships";

/// The heading that ends the adapter sections.
const END: &str = "## Invariants";

/// The heading of the subsection that holds the loss set and its census.
const LOSS_SECTION: &str = "### Every format declares a loss set, and a census audits it";

/// The heading of the subsection that holds the severity table.
const LEVEL_SECTION: &str = "### Three severity scales, and the one that reaches SARIF";

/// The heading of the subsection that holds the escape-class table.
const SUPPRESSION_SECTION: &str = "### A suppressed finding is in the output, and it is marked";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn read(rel: &str) -> String {
    let path = root().join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn checks_and_cache() -> String {
    read("docs/subsystems/checks-and-cache.md")
}

fn spec_six() -> String {
    read("docs/spec/06-engine-architecture.md")
}

/// The adapter sections of checks-and-cache: every line from [`FIRST`] to
/// [`END`], headings included.
fn adapter_sections(text: &str) -> Vec<&str> {
    let lines: Vec<&str> = text
        .lines()
        .skip_while(|l| !l.starts_with(FIRST))
        .take_while(|l| *l != END)
        .collect();
    assert!(
        !lines.is_empty(),
        "docs/subsystems/checks-and-cache.md has no heading that opens `{FIRST}`"
    );
    lines
}

/// The lines of one `###` subsection, from its heading to the next heading.
fn subsection<'a>(text: &'a str, heading: &str) -> Vec<&'a str> {
    let mut lines = text.lines().skip_while(|l| *l != heading);
    assert!(
        lines.next().is_some(),
        "docs/subsystems/checks-and-cache.md has no `{heading}`"
    );
    lines.take_while(|l| !l.starts_with('#')).collect()
}

/// The body rows of the first table in `lines`, each cut into its cells with
/// backticks and spaces trimmed.
fn table(lines: &[&str]) -> Vec<Vec<String>> {
    let rows: Vec<Vec<String>> = lines
        .iter()
        .skip_while(|l| !l.starts_with('|'))
        .take_while(|l| l.starts_with('|'))
        .filter(|l| !l.starts_with("|---"))
        .skip(1)
        .map(|l| {
            l.trim()
                .trim_matches('|')
                .split('|')
                .map(|cell| cell.trim().trim_matches('`').to_string())
                .collect()
        })
        .collect();
    assert!(!rows.is_empty(), "the subsection holds no table with a body row");
    rows
}

/// Never called. A fourth severity fails to compile here until
/// [`SEVERITIES`] names it.
fn _exhaustive_severity(severity: Severity) {
    match severity {
        Severity::Error | Severity::Warn | Severity::Info => {}
    }
}

const SEVERITIES: [Severity; 3] = [Severity::Error, Severity::Warn, Severity::Info];

/// Never called. A third escape class fails to compile here until
/// [`ESCAPES`] names it.
fn _exhaustive_escape(escape: Escape) {
    match escape {
        Escape::MigrationPending | Escape::Suppression => {}
    }
}

const ESCAPES: [Escape; 2] = [Escape::MigrationPending, Escape::Suppression];

/// The severity table of checks-and-cache: one row for each severity a check
/// reports, and the SARIF `level` it maps to. The map is what
/// [`sarif::level`] writes, row by row and in both directions.
///
/// # Watched failing
///
/// On `main` at `a629c0be` this reddened because checks-and-cache had no
/// `LEVEL_SECTION`. Mapping `Severity::Info` to `"none"` in `sarif::level`
/// reddens the `info` row.
#[test]
fn the_level_table_of_checks_and_cache_is_what_sarif_writes() {
    let text = checks_and_cache();
    let rows = table(&subsection(&text, LEVEL_SECTION));
    let stated: BTreeMap<String, String> = rows
        .iter()
        .map(|r| (r[0].clone(), r.get(1).cloned().unwrap_or_default()))
        .collect();
    assert_eq!(stated.len(), rows.len(), "the level table names a severity twice: {rows:?}");
    let written: BTreeMap<String, String> = SEVERITIES
        .iter()
        .map(|s| (s.to_string(), sarif::level(*s).to_string()))
        .collect();
    assert_eq!(
        stated, written,
        "the level table of docs/subsystems/checks-and-cache.md is not what sarif::level writes"
    );
}

/// The escape-class table of checks-and-cache: one row for each class that
/// hides a finding, and the SARIF `suppression.kind` it maps to. Each row
/// that names a class the engine has is what [`sarif::kind`] writes, and every
/// class the engine has has a row. A `waiver` row may stand beside them: no
/// waiver mechanism exists, and the table states where one would go.
///
/// # Watched failing
///
/// On `main` at `a629c0be` this reddened because checks-and-cache had no
/// `SUPPRESSION_SECTION`. Mapping `Escape::MigrationPending` to `"inSource"`
/// in `sarif::kind` reddens the `migration-pending` row.
#[test]
fn the_suppression_kind_table_of_checks_and_cache_is_what_sarif_writes() {
    let text = checks_and_cache();
    let rows = table(&subsection(&text, SUPPRESSION_SECTION));
    let stated: BTreeMap<String, String> = rows
        .iter()
        .filter(|r| r[0] != "waiver")
        .map(|r| (r[0].clone(), r.last().cloned().unwrap_or_default()))
        .collect();
    let written: BTreeMap<String, String> = ESCAPES
        .iter()
        .map(|e| (e.name().to_string(), sarif::kind(*e).to_string()))
        .collect();
    assert_eq!(
        stated, written,
        "the escape-class table of docs/subsystems/checks-and-cache.md is not what sarif::kind \
         writes"
    );
    assert!(
        rows.len() <= ESCAPES.len() + 1,
        "the escape-class table has a row that is neither an engine class nor `waiver`: {rows:?}"
    );
}

/// The outcome fields of [`Census`]: what an entry of a loss set comes to.
/// The destructure has no `..`, so a new field of `Census` does not compile
/// here until it is placed in this list or among the finding counts.
fn census_outcomes(census: &Census) -> [&'static str; 3] {
    let Census {
        findings: _,
        carried: _,
        unaccounted: _,
        entries: _,
        held: _,
        adrift: _,
        unaudited: _,
    } = census;
    ["held", "adrift", "unaudited"]
}

/// The backticked words of the loss-set subsection that name an outcome are
/// the outcome fields of `Census`, and the subsection names all of them.
///
/// # Watched failing
///
/// On `main` at `a629c0be` this reddened because checks-and-cache had no
/// `LOSS_SECTION`. Deleting `` `adrift` `` from the subsection reddens it
/// naming `adrift`.
#[test]
fn the_census_outcomes_checks_and_cache_names_are_the_fields_census_counts() {
    let text = checks_and_cache();
    let body = subsection(&text, LOSS_SECTION).join("\n");
    let census = Census {
        findings: 0,
        carried: 0,
        unaccounted: Vec::new(),
        entries: 0,
        held: 0,
        adrift: Vec::new(),
        unaudited: 0,
    };
    let outcomes = census_outcomes(&census);
    let missing: Vec<&str> = outcomes
        .iter()
        .copied()
        .filter(|o| !body.contains(&format!("`{o}`")))
        .collect();
    assert!(
        missing.is_empty(),
        "the loss-set subsection of docs/subsystems/checks-and-cache.md does not name the census \
         outcomes {missing:?}"
    );
    // The three outcomes sum to the entries declared, and the census states it.
    assert!(
        census.accounts(),
        "an empty census does not account for its entries"
    );
}

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

/// Plain text cut into words, with punctuation gone.
fn words(text: &str) -> Vec<String> {
    plain(text)
        .split(|c: char| !(c.is_alphanumeric() || c == '\'' || c == '-' || c == '_'))
        .filter(|w| !w.is_empty())
        .map(str::to_owned)
        .collect()
}

/// The fewest words in a row that spec 6 may not share with the adapter
/// sections.
const RUN_WORDS: usize = 8;

/// Two sentences that moved, each of which spec 6 stated word for word.
const MOVED_SENTENCES: [&str; 2] = [
    "A suppressed finding is in the output, and it is marked",
    "An entry is held when the artifact agrees with the run",
];

/// Spec 6 shares no run of eight words with the adapter sections, so that no
/// moved rule is stated in both places.
///
/// # Watched failing
///
/// On `main` at `a629c0be` this reddened because checks-and-cache had no
/// adapter sections. With the sections written and spec 6 unedited, it
/// reddens naming the runs both state.
#[test]
fn spec_6_shares_no_run_of_eight_words_with_the_adapter_sections_of_checks_and_cache() {
    let text = checks_and_cache();
    let six = spec_six();
    let six_words = format!(" {} ", words(&six).join(" "));
    let mut runs = BTreeSet::new();
    for line in adapter_sections(&text) {
        if line.starts_with('#') {
            continue;
        }
        for cell in line.split('|') {
            let w = words(cell);
            for run in w.windows(RUN_WORDS) {
                let run = run.join(" ");
                if six_words.contains(&format!(" {run} ")) {
                    runs.insert(run);
                }
            }
        }
    }
    assert!(
        runs.is_empty(),
        "spec 6 shares these runs of {RUN_WORDS} words with the adapter sections of \
         checks-and-cache: {runs:#?}"
    );
    let plain_six = plain(&six);
    for sentence in MOVED_SENTENCES {
        assert!(
            !plain_six.contains(&plain(sentence)),
            "spec 6 still states {sentence:?}, which checks-and-cache states"
        );
    }
}

/// The body of spec 6's `### CI adapters`, from its heading to the next
/// heading.
fn ci_adapters(six: &str) -> Vec<&str> {
    let mut lines = six.lines().skip_while(|l| *l != "### CI adapters");
    assert!(lines.next().is_some(), "spec 6 has no `### CI adapters`");
    lines.take_while(|l| !l.starts_with('#')).collect()
}

/// Spec 6's `### CI adapters` is one short paragraph that points at
/// checks-and-cache, and it keeps its heading, which inbound links name.
///
/// # Watched failing
///
/// On `main` at `a629c0be` the section was 1,143 words long, and this
/// reddened.
#[test]
fn spec_6_keeps_a_short_ci_adapters_section_that_points_at_checks_and_cache() {
    let six = spec_six();
    let body = ci_adapters(&six).join("\n");
    let count = body.split_whitespace().count();
    assert!(
        count <= 120,
        "spec 6's `### CI adapters` is {count} words, and the bar is 120"
    );
    assert!(
        body.contains("../subsystems/checks-and-cache.md#"),
        "spec 6's `### CI adapters` does not link checks-and-cache"
    );
}

fn engine_sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("read an engine directory") {
        let path = entry.expect("read a directory entry").path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if path.is_dir() {
            if name != "target" && !name.starts_with('.') {
                engine_sources(&path, out);
            }
        } else if name.ends_with(".rs") {
            out.push(path);
        }
    }
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

/// Sentences of the moved rules that a comment can credit to spec 6 without
/// quoting them. Each one is a clause of a rule, never a generic term such as
/// "loss set" or "suppression", so a comment that names the term and credits
/// spec 6 for something else does not match.
const MOVED_CLAIMS: [&str; 6] = [
    "a run reports what it evaluated",
    "emits what it evaluated",
    "never orders what lands",
    "asks a run to report",
    "a suppressed finding is in the output",
    "an entry names its members",
];

/// A comment that credits spec 6 with a sentence of the adapter sections
/// points a reader at a part that no longer holds it.
///
/// It reads every comment line that names spec 6, joined with the five lines
/// after it, and fails when the text from `spec 6` on opens a quote that the
/// adapter sections hold, or states one of [`MOVED_CLAIMS`] within 240 bytes.
/// A quote of text that spec 6 keeps is not this test's business, and neither
/// is a string literal, which is a message or a test input. This file is
/// skipped, because it names the claims it looks for.
///
/// # Watched failing
///
/// On `main` at `a629c0be` this reddened naming the comments in `lib.rs`,
/// `sarif.rs`, `text.rs`, `markdown.rs`, `json.rs`, `tests/fixtures.rs` of
/// this crate and `lock/src/lib.rs`.
#[test]
fn no_comment_quotes_moved_adapter_text_as_spec_6() {
    let engine = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let sections = plain(&adapter_sections(&checks_and_cache()).join(" "));
    let kept = plain(&ci_adapters(&spec_six()).join(" "));
    let mut files = Vec::new();
    engine_sources(&engine.join("crates"), &mut files);
    let mut credited = Vec::new();
    let mut read = 0;
    for file in files {
        if file.file_name().and_then(|n| n.to_str()) == Some("checks_and_cache_adapters.rs") {
            continue;
        }
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
            let trimmed = line.trim_start();
            if !trimmed.starts_with("//") {
                continue;
            }
            let joined = lines[i..lines.len().min(i + 6)]
                .iter()
                .map(|l| l.trim().trim_start_matches(['/', '!']).trim())
                .collect::<Vec<_>>()
                .join(" ");
            let Some(at) = joined.to_lowercase().find("spec 6") else {
                continue;
            };
            read += 1;
            if let Some(quote) = quote_after(&joined, at) {
                let quote = plain(quote);
                let quote = quote.trim_end_matches(['.', ',']);
                if quote.split_whitespace().count() >= 4 && sections.contains(quote) {
                    credited.push(format!("{rel}:{}: {quote:?}", i + 1));
                    continue;
                }
                // A quote under a link to spec 6's section is text that the
                // section still holds, or the link points at nothing.
                if quote.split_whitespace().count() >= 4
                    && joined.contains("#ci-adapters")
                    && !kept.contains(quote)
                {
                    credited.push(format!("{rel}:{}: {quote:?} (not in spec 6)", i + 1));
                    continue;
                }
            }
            let window: String = plain(&joined[at..]).chars().take(240).collect();
            if let Some(claim) = MOVED_CLAIMS.iter().find(|c| window.contains(*c)) {
                credited.push(format!("{rel}:{}: {claim:?}", i + 1));
            }
        }
    }
    assert!(read > 0, "no engine comment names spec 6 at all");
    assert!(
        credited.is_empty(),
        "these comments credit spec 6 with a rule that docs/subsystems/checks-and-cache.md now \
         states: {credited:#?}"
    );
}
