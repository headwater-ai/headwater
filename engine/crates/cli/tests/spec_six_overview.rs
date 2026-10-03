// SPDX-License-Identifier: Apache-2.0
//! Spec 6 is an overview of the engine, and each rule it once stated alone now
//! lives in one home that it points at (#1572, slice 4b).
//!
//! The file keeps all 18 of its headings, because inbound links name them.
//! It keeps the pipeline, the subsystem map and one pointer for each rule.
//! The audit readings went to the `headwater taxonomy` contract, the declaring
//! rules of `exportable_as` to spec 12, and the waiting projection kinds and
//! the cycle of passes to the projections subsystem. Every other removed
//! paragraph already had a home that stated it.
//!
//! `spec_six_cli_rules.rs` holds the CLI section, and `spec_six_projections.rs`
//! holds the projection-kinds block. This file holds the rest.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn read(rel: &str) -> String {
    let path = repo().join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

const SPEC_SIX: &str = "docs/spec/06-engine-architecture.md";

fn spec_six() -> String {
    read(SPEC_SIX)
}

/// Text with Markdown link targets, emphasis and code ticks removed, in lower
/// case, with runs of whitespace folded to one space.
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

/// The fewest words in a row that spec 6's prose may not share with a home it
/// points at.
const RUN_WORDS: usize = 8;

/// The most prose words spec 6 may carry, outside its front matter and its
/// fences.
const WORD_BAR: usize = 2500;

/// The body of a document after its front matter.
fn body(text: &str) -> &str {
    let Some(rest) = text.strip_prefix("---\n") else {
        return text;
    };
    rest.find("\n---\n").map_or(rest, |i| &rest[i + 5..])
}

/// The lines of a body outside every fenced block.
fn prose_lines(body: &str) -> Vec<&str> {
    let mut fenced = false;
    let mut out = Vec::new();
    for line in body.lines() {
        if line.starts_with("```") {
            fenced = !fenced;
            continue;
        }
        if !fenced {
            out.push(line);
        }
    }
    assert!(!fenced, "an unterminated fence");
    out
}

/// The count the adjudication of slice 4b named: whitespace-separated fields
/// of every line after the front matter and outside every fence, as
/// `awk '/^```/{f=!f;next} !f{n+=NF}'` counts them.
fn prose_word_count(text: &str) -> usize {
    prose_lines(body(text))
        .iter()
        .map(|l| l.split_whitespace().count())
        .sum()
}

/// Spec 6's prose with its `### CLI` section left out, which
/// `spec_six_cli_rules.rs` holds against the verb contracts.
fn prose_outside_cli(text: &str) -> String {
    let mut out = String::new();
    let mut in_cli = false;
    for line in prose_lines(body(text)) {
        if line.starts_with('#') {
            in_cli = line == "### CLI";
        }
        if !in_cli {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

/// Each heading line of spec 6 at `1b12d20c`, which inbound links name: the
/// title and the 18 section headings the adjudication counted.
const HEADINGS: [&str; 19] = [
    "# 6 — Engine architecture",
    "## Why one engine",
    "## Pipeline",
    "### Subsystems",
    "## Nothing stores the graph",
    "## Checks",
    "## Projections",
    "### A verb index reads the command surface of the engine",
    "### An export is a projection, and it declares what it dropped",
    "### An export profile carries a filter",
    "### What a filtered export claims, and what it does not",
    "## Interfaces",
    "### CLI",
    "### `taxonomy validate` versus `taxonomy audit`",
    "### Library",
    "### MCP server",
    "### CI adapters",
    "## Performance targets",
    "## Implementation constraints",
];

/// One lead sentence of each paragraph that left spec 6, because a home states
/// its rule.
const MOVED_LEADS: [&str; 23] = [
    // `taxonomy validate` versus `taxonomy audit`, to the `headwater taxonomy` contract.
    "Nine readings run",
    "Two of the readings this section named do not run",
    "A wait that a string literal states",
    "A count copied into prose",
    "The warrant reading walks the closed set of four",
    "The layout reading holds apart",
    "One bar is declared",
    "`audit` also writes one line",
    "The grain of the creator reading",
    // MCP server, to `headwater-mcp.md` and queries-and-explain.
    "The server walks the corpus once",
    "A call that moves a byte of that tree ends the server",
    // Checks, to spec 12.
    "Checks come from five origins",
    "The last three are why a native engine exists at all",
    "The last column is a **set of emitter targets**",
    "The declaration is per rule, and it lives on the check as `EXPORTABLE_AS`",
    // Nothing stores the graph, to HW-DR-0006 and spec 12.
    "| The in-memory graph | one run |",
    "The cache is disposable, and a test says so",
    // Projections, to projections-and-export and the generate contract.
    "One write is one run",
    "A shelf index and a shelf sections file each carry",
    "A relation view carries decision lineage",
    "A run over any corpus names every waiting kind",
    "Both hold that standing for one reason",
    // Implementation constraints: contradiction 5.
    "Usable as a library from an editor plugin",
];

/// One sentence of each rule that moved to a home that did not state it, as
/// the home states it now.
const HOMES: [(&str, &str); 14] = [
    (
        "docs/interfaces/headwater-taxonomy.md",
        "**The report takes nine readings of the corpus.**",
    ),
    (
        "docs/interfaces/headwater-taxonomy.md",
        "**Two readings that the report names do not run, and each one says what it waits on.**",
    ),
    (
        "docs/interfaces/headwater-taxonomy.md",
        "**The report states each figure, and no document copies one.**",
    ),
    (
        "docs/interfaces/headwater-taxonomy.md",
        "**The warrant reading has one row for each of the four warrant values that",
    ),
    (
        "docs/interfaces/headwater-taxonomy.md",
        "**The layout reading reports a shelf that it cannot measure apart from a name that drifted.**",
    ),
    (
        "docs/interfaces/headwater-taxonomy.md",
        "**Two findings carry a verdict, and every other reading is a distribution.**",
    ),
    (
        "docs/interfaces/headwater-taxonomy.md",
        "**`--record` writes one line that is not a document.**",
    ),
    (
        "docs/interfaces/headwater-taxonomy.md",
        "**The creator reading has one row for each relation, and none for each edge.**",
    ),
    (
        "docs/interfaces/headwater-taxonomy.md",
        "**`audit` gates nothing.**",
    ),
    (
        "docs/spec/12-check-layer.md",
        "`facet.required.missing` and `facet.value.not_permitted`",
    ),
    (
        "docs/spec/12-check-layer.md",
        "The other three origins are why a native engine exists at all, because LinkML and SHACL cannot express a check of any of them.",
    ),
    (
        "docs/subsystems/projections-and-export.md",
        "A relation view would carry the lineage of decisions and a traceability matrix.",
    ),
    (
        "docs/subsystems/projections-and-export.md",
        "the declarations form a cycle, and one more pass would not settle it",
    ),
    (
        "docs/subsystems/projections-and-export.md",
        "A template would carry the permitted relations, facets and sections of one kind.",
    ),
];

/// The documents spec 6 points at for a rule it no longer states.
const POINTED_HOMES: [&str; 7] = [
    "docs/interfaces/headwater-taxonomy.md",
    "docs/interfaces/headwater-mcp.md",
    "docs/subsystems/queries-and-explain.md",
    "docs/subsystems/projections-and-export.md",
    "docs/interfaces/headwater-generate.md",
    "docs/spec/12-check-layer.md",
    "docs/decisions/0006-where-the-corpus-graph-lives-at-rest.md",
];

/// Spec 6 is an overview, and an overview of 4,601 words is the specification
/// it points at under another heading.
///
/// # Watched failing
///
/// On `1b12d20c` this read 4,601.
#[test]
fn spec_6_is_an_overview_of_at_most_2500_words() {
    let count = prose_word_count(&spec_six());
    assert!(
        count <= WORD_BAR,
        "{SPEC_SIX} carries {count} prose words outside its front matter and fences, \
         and the bar is {WORD_BAR}"
    );
}

/// The counter above counts what the adjudication's `awk` command counts:
/// fields, with the front matter and every fence skipped.
#[test]
fn the_prose_count_skips_front_matter_and_fences() {
    let text =
        "---\nid: X\ntitle: \"a b c\"\n---\n\n# T\n\nOne two, three.\n\n```\nfour five\n```\nsix\n";
    assert_eq!(prose_word_count(text), 6);
}

/// Every heading survives byte for byte, so that no inbound anchor breaks.
#[test]
fn every_heading_of_spec_6_survives() {
    let six = spec_six();
    let lines: BTreeSet<&str> = six.lines().collect();
    for heading in HEADINGS {
        assert!(
            lines.contains(heading),
            "{SPEC_SIX} lost the heading line `{heading}`, which inbound links name"
        );
    }
    let count = six
        .lines()
        .filter(|l| l.starts_with('#') && !l.starts_with("#!"))
        .count();
    assert_eq!(
        count,
        HEADINGS.len(),
        "{SPEC_SIX} has {count} heading lines, and it had 19 at `1b12d20c`"
    );
}

/// A rule that moved to its home is not left behind in spec 6, so it has one
/// statement and not two.
///
/// # Watched failing
///
/// On `1b12d20c` every lead below was present.
#[test]
fn no_moved_rule_is_left_in_spec_6() {
    let six = plain(&spec_six());
    let left: Vec<&str> = MOVED_LEADS
        .iter()
        .copied()
        .filter(|lead| six.contains(&plain(lead)))
        .collect();
    assert!(
        left.is_empty(),
        "{SPEC_SIX} still states these rules, which their homes now state:\n{}",
        left.join("\n")
    );
}

/// Each rule that left spec 6 for a home that did not state it is now stated
/// in that home.
///
/// # Watched failing
///
/// On `1b12d20c` no home carried these sentences.
#[test]
fn every_moved_rule_is_stated_in_its_home() {
    let missing: Vec<String> = HOMES
        .iter()
        .filter(|(home, sentence)| !plain(&read(home)).contains(&plain(sentence)))
        .map(|(home, sentence)| format!("{home}: {sentence}"))
        .collect();
    assert!(
        missing.is_empty(),
        "these moved rules are stated in no home:\n{}",
        missing.join("\n")
    );
}

/// The runs of `RUN_WORDS` words in a row of a text.
fn runs(text: &str) -> BTreeSet<Vec<String>> {
    words(text)
        .windows(RUN_WORDS)
        .map(<[String]>::to_vec)
        .collect()
}

/// Spec 6 points at a home rather than repeating it, so the two share no run
/// of eight words. The CLI section is left to `spec_six_cli_rules.rs`.
#[test]
fn spec_6_shares_no_run_of_eight_words_with_a_home() {
    let six = runs(&prose_outside_cli(&spec_six()));
    let mut shared = Vec::new();
    for home in POINTED_HOMES {
        let home_runs = runs(body(&read(home)));
        for run in six.intersection(&home_runs) {
            shared.push(format!("{home}: {}", run.join(" ")));
        }
    }
    assert!(
        shared.is_empty(),
        "{SPEC_SIX} repeats a home it points at:\n{}",
        shared.join("\n")
    );
}

/// Words that deny what the sentence they stand in names.
const NEGATIONS: [&str; 9] = [
    "no", "not", "never", "none", "without", "absent", "lacks", "nor", "cannot",
];

/// The sentences of a text, cut at a full stop, a question mark or an
/// exclamation mark followed by a space, after `plain()`.
fn sentences(text: &str) -> Vec<String> {
    let flat = plain(text);
    flat.split(". ")
        .flat_map(|s| s.split("? "))
        .flat_map(|s| s.split("! "))
        .map(str::to_owned)
        .collect()
}

/// The commit hook runs `check --strict --change`, so no sentence of spec 6
/// may deny that a change-scoped check exists, however it is worded.
#[test]
fn no_sentence_of_spec_6_denies_the_change_scoped_check() {
    let denials: Vec<String> = sentences(&prose_lines(body(&spec_six())).join("\n"))
        .into_iter()
        .filter(|s| s.contains("change-scoped") || s.contains("change scoped"))
        .filter(|s| {
            let w = words(s);
            NEGATIONS.iter().any(|n| w.iter().any(|x| x == n))
        })
        .collect();
    assert!(
        denials.is_empty(),
        "{SPEC_SIX} denies a change-scoped check that `.githooks/pre-commit` runs:\n{}",
        denials.join("\n")
    );
}

/// The commit-hook row of the Performance table names the mode the hook runs
/// and the record that says why it narrows nothing.
#[test]
fn the_commit_hook_row_names_the_change_scoped_run() {
    let six = spec_six();
    let row = six
        .lines()
        .find(|l| l.starts_with("| Commit hook"))
        .unwrap_or_else(|| panic!("{SPEC_SIX} has no `| Commit hook` row"));
    for needle in ["check --strict --change", "0080-changed-only"] {
        assert!(
            row.contains(needle),
            "the commit-hook row of {SPEC_SIX} does not name `{needle}`:\n{row}"
        );
    }
}

/// The two sentences that contradicted the engine are gone, and the
/// `**Embeddable.**` item cites the decision it is now true of.
///
/// `.githooks/pre-commit` runs `check --strict --change`, so a change-scoped
/// mode exists and narrows nothing. HW-DR-0102 makes an editor a client of
/// `headwater mcp`, so an editor spawns a process.
///
/// # Watched failing
///
/// On `1b12d20c` both phrases were present and the item cited nothing.
#[test]
fn spec_6_does_not_contradict_itself() {
    let six = spec_six();
    let flat = plain(&six);
    for phrase in [
        "no change-scoped check exists",
        "no need to spawn subprocesses",
    ] {
        assert!(
            !flat.contains(phrase),
            "{SPEC_SIX} still says `{phrase}`, which the engine contradicts"
        );
    }
    let embeddable = six
        .lines()
        .find(|l| l.starts_with("- **Embeddable.**"))
        .unwrap_or_else(|| panic!("{SPEC_SIX} has no `**Embeddable.**` item"));
    assert!(
        embeddable.contains("client of `headwater mcp`"),
        "the `**Embeddable.**` item of {SPEC_SIX} does not make an editor a client of \
         `headwater mcp`:\n{embeddable}"
    );
    assert!(
        embeddable.contains("decisions/0102-"),
        "the `**Embeddable.**` item of {SPEC_SIX} does not cite HW-DR-0102:\n{embeddable}"
    );
}
