// SPDX-License-Identifier: Apache-2.0
//! Four inputs that took time in proportion to their square, through the
//! binary (#1968).
//!
//! One document of a few hundred kilobytes made `check --strict` run for
//! seconds or minutes, because a reader searched everything it had already
//! read once for each new item: the loader for a repeated key, the graph and
//! the edge scope for a repeated relation, the sentence reader for the run
//! that owns a position and for an open link, and the line table for a
//! column. Each case below builds one such document and holds the whole run
//! under a ceiling that the code before the fix exceeded by more than ten
//! times in the same test profile.
//!
//! The ceilings are wall time in the `dev` profile (no optimization), so each
//! leaves the fixed code a wide margin on a loaded runner. The doc comment on
//! each case records the time measured before and after the fix.

mod common;
use common::{Ran, Root};
use std::time::{Duration, Instant};

const NAME: &str = "9999-linear-scale.md";

/// Front matter that `check` accepts, with `extra` before `provenance:` and
/// `body` after the closing fence.
fn document(extra: &str, body: &str) -> String {
    format!(
        "---\nid: HW-DR-9999\ntitle: Linear scale\nstatus: current\nstatus_since: 2026-08-01\nlast_verified: 2026-08-01\nsummary: A document with many items of one kind.\n{extra}provenance:\n  warrant: accepted\n  agency: human\n  accepted_by: a.person\n  evidence_basis: unevidenced\n---\n\n# Linear scale\n\n{body}\n"
    )
}

/// Each case takes its own label: the root directory is keyed on the process
/// and the label, and the cases of one file run as threads of one process.
fn root(label: &str, text: &str) -> Root {
    Root::shaped(label, |at| {
        let path = at.join("docs/decisions").join(NAME);
        std::fs::create_dir_all(path.parent().expect("it has a parent"))
            .expect("the shelf directory is there");
        std::fs::write(path, text).expect("the document writes");
    })
}

fn tail(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    lines[lines.len().saturating_sub(20)..].join("\n")
}

/// `check --strict` ended with a status, under the ceiling, and read the
/// document.
fn assert_in_time(label: &str, text: &str, ceiling: Duration) {
    let root = root(label, text);
    let started = Instant::now();
    let ran: Ran = root.run(&["check", "--strict"]);
    let took = started.elapsed();
    eprintln!("{label}: check --strict took {took:?} (ceiling {ceiling:?})");
    assert!(
        matches!(ran.code, Some(0) | Some(1)),
        "`check --strict` did not end with a status (a signal reads as None): {:?}\nstderr:\n{}",
        ran.code,
        tail(&ran.err)
    );
    assert!(
        took < ceiling,
        "`check --strict` took {took:?} on {label}, over the ceiling of {ceiling:?}"
    );
}

/// Unknown front-matter keys, each one distinct. The loader searched every
/// key already read for a duplicate of each new one.
#[test]
fn many_unknown_front_matter_keys_check_in_linear_time() {
    let extra: String = (0..KEYS).map(|i| format!("k{i}: v\n")).collect();
    assert_in_time("linear-keys", &document(&extra, "One paragraph."), CEILING);
}

/// Distinct relation targets that resolve to nothing. The graph searched the
/// triples of the document for a repeat of each edge, and the edge scope
/// searched its groups for each edge once for each edge-scoped rule.
#[test]
fn many_unresolved_relation_targets_check_in_linear_time() {
    let targets: String = (0..TARGETS)
        .map(|i| format!("    - HW-DR-{:05}\n", i + 10_000))
        .collect();
    let extra = format!("relations:\n  traces_to:\n{targets}");
    assert_in_time(
        "linear-targets",
        &document(&extra, "One paragraph."),
        CEILING,
    );
}

/// Sentences in one paragraph. The sentence reader walked every run of the
/// block, and counted the characters of each, for each position it asked.
#[test]
fn many_sentences_in_one_paragraph_check_in_linear_time() {
    let body = "This is a sentence. ".repeat(SENTENCES);
    assert_in_time("linear-sentences", &document("", &body), CEILING);
}

/// Links on one line. The line table counted the characters from the start
/// of the line for each column, and the sentence reader scanned every link
/// of the body for each terminator.
#[test]
fn many_links_on_one_line_check_in_linear_time() {
    let body = "[a](0001-the-warrant-a-person-set.md) ".repeat(LINKS);
    assert_in_time("linear-links", &document("", &body), CEILING);
}

const KEYS: usize = 40_000;
const TARGETS: usize = 20_000;
const SENTENCES: usize = 20_000;
const LINKS: usize = 20_000;
const CEILING: Duration = Duration::from_secs(600);
