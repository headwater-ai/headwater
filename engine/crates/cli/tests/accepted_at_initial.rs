// SPDX-License-Identifier: Apache-2.0
//! `warrant.acceptance.at_initial` through the binary, over a root that
//! carries this repository's own declaration (#1832).
//!
//! The check crate's fixture holds what the rule decides. Two clauses of the
//! issue need a whole root, and this file holds them:
//!
//! - **A template gets no instance.** This repository keeps templates out of
//!   the corpus with `corpus.exclude` in `.headwater/taxonomy.yml`, and no rule
//!   skips a template by itself. So the case puts one document at `draft` and
//!   `accepted` under an excluded templates path and under a decision shelf,
//!   and only the second is reported.
//! - **The rule and `warrant.promoted` disagree on an added document.** The
//!   promotion rule passes a document that the change adds, because an added
//!   document made no movement. A document added at `draft` and `accepted` in
//!   one change is therefore silent under that rule and reported by this one.

use std::path::{Path, PathBuf};
use std::process::Command;

const RULE: &str = "warrant.acceptance.at_initial";
const PROMOTED: &str = "warrant.promoted";

/// A decision at the initial state of its regime that says a person accepted
/// it. The identifier is free in the fixture corpus.
const DRAFTED: &str = "---
id: HW-DR-0002
title: The acceptance at draft
status: draft
status_since: 2026-08-01
last_verified: 2026-08-01
summary: A decision that stands at draft and says a person accepted it.
provenance:
  warrant: accepted
  agency: human
  accepted_by: a.person
  evidence_basis: unevidenced
---

# The acceptance at draft

## Context

The state says this decision is not proposed as finished.

## Decision

The warrant says a person accepted it as finished.

## Consequences

One rule reads both, and reports the contradiction.
";

const DRAFTED_PATH: &str = "docs/decisions/0002-the-acceptance-at-draft.md";
const TEMPLATE_PATH: &str = "docs/taxonomies/decision-record/templates/decision.md";

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

/// A scratch directory, removed when it is dropped. `label` names the case,
/// because cargo runs the cases of one target as threads of one process.
struct Scratch(PathBuf);

impl Scratch {
    fn new(label: &str) -> Scratch {
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-accepted-at-initial-{}-{label}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&at);
        Scratch(at)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// The fixture corpus plus this repository's lock and declaration, so that
/// `corpus.exclude` and the decision regime are the ones this repository runs.
fn root(label: &str) -> Scratch {
    let root = Scratch::new(label);
    copy(&fixtures().join("accepted-at-initial"), &root.0);
    let headwater = root.0.join(".headwater");
    std::fs::create_dir_all(&headwater).expect("the declaration directory is there");
    for name in ["taxonomy.lock", "taxonomy.yml"] {
        std::fs::copy(
            repository().join(".headwater").join(name),
            headwater.join(name),
        )
        .expect("the declaration copies");
    }
    root
}

fn copy(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("the directory is there");
    for entry in std::fs::read_dir(from).expect("the fixture directory reads") {
        let entry = entry.expect("the entry reads");
        let target = to.join(entry.file_name());
        match entry.file_type().expect("the file type reads").is_dir() {
            true => copy(&entry.path(), &target),
            false => {
                std::fs::copy(entry.path(), &target).expect("the fixture copies");
            }
        }
    }
}

fn write(root: &Path, relative: &str, text: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().expect("a parent")).expect("the directory is there");
    std::fs::write(path, text).expect("the document writes");
}

/// One `headwater check`, held to exit 0, with its standard output.
fn check(root: &Path, extra: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_headwater"))
        .args(["check", "--no-cache", "--now", "2026-08-01"])
        .args(extra)
        .arg("--root")
        .arg(root)
        .output()
        .expect("the binary runs");
    let out = String::from_utf8_lossy(&output.stdout).into_owned();
    assert_eq!(
        output.status.code(),
        Some(0),
        "{out}{}",
        String::from_utf8_lossy(&output.stderr)
    );
    out
}

/// The paths at which the text report places a finding of `rule`. A finding
/// is a location line, `  <path>:<line>:<column> ...`, and then the rule.
fn reported(out: &str, rule: &str) -> Vec<String> {
    let lines: Vec<&str> = out.lines().collect();
    let opening = format!("    {rule} (");
    lines
        .windows(2)
        .filter(|pair| pair[1].starts_with(&opening))
        .filter_map(|pair| pair[0].trim().split(':').next().map(str::to_string))
        .collect()
}

/// The number the report states for `rule` in its line of instance counts.
fn instances(out: &str, rule: &str) -> usize {
    let tail = format!(" instances of {rule}");
    out.lines()
        .find(|line| line.ends_with(&tail))
        .and_then(|line| line.trim().split(' ').next())
        .and_then(|count| count.parse().ok())
        .unwrap_or(0)
}

fn git(dir: &Path, arguments: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(arguments)
        .output()
        .expect("git runs");
    assert!(
        output.status.success(),
        "git {arguments:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

/// The same text under an excluded templates path gets no instance and no
/// finding, and under the decision shelf gets one finding.
#[test]
fn a_template_at_draft_and_accepted_is_not_read() {
    let root = root("template");
    write(&root.0, TEMPLATE_PATH, DRAFTED);

    let excluded = check(&root.0, &[]);
    assert!(
        reported(&excluded, RULE).is_empty(),
        "a template is not reported:\n{excluded}"
    );
    // The fixture's one decision is read, and the template is not.
    assert_eq!(instances(&excluded, RULE), 1, "{excluded}");

    write(&root.0, DRAFTED_PATH, DRAFTED);
    let shelved = check(&root.0, &[]);
    assert_eq!(
        reported(&shelved, RULE),
        vec![DRAFTED_PATH.to_string()],
        "the same text on a decision shelf is reported once:\n{shelved}"
    );
    assert_eq!(instances(&shelved, RULE), 2, "{shelved}");
}

/// A document added at `draft` and `accepted` in one change: `warrant.promoted`
/// evaluates it and reports nothing, and this rule reports it.
#[test]
fn an_added_document_is_silent_under_promotion_and_reported_here() {
    let root = root("added");
    git(&root.0, &["init", "-q"]);
    git(&root.0, &["config", "user.email", "fixtures@invalid"]);
    git(&root.0, &["config", "user.name", "fixtures"]);
    git(&root.0, &["add", "-A"]);
    git(&root.0, &["commit", "-q", "-m", "base"]);
    let base = git(&root.0, &["rev-parse", "HEAD"]);
    write(&root.0, DRAFTED_PATH, DRAFTED);

    let out = Scratch::new("added-out");
    let produced = Command::new(env!("CARGO_BIN_EXE_headwater"))
        .args(["change", &base])
        .arg(&out.0)
        .arg("--root")
        .arg(&root.0)
        .output()
        .expect("the binary runs");
    assert_eq!(
        produced.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&produced.stderr)
    );
    let manifest = out.0.join("manifest");
    let scoped = check(&root.0, &["--change", &manifest.display().to_string()]);

    // The promotion rule read the change rather than skipping it.
    assert!(
        !scoped.contains("change-scoped-only"),
        "warrant.promoted evaluated with the manifest:\n{scoped}"
    );
    assert!(
        !reported(&scoped, PROMOTED).contains(&DRAFTED_PATH.to_string()),
        "an added document is no promotion:\n{scoped}"
    );
    assert_eq!(
        reported(&scoped, RULE),
        vec![DRAFTED_PATH.to_string()],
        "the new rule reports the added document:\n{scoped}"
    );
}
