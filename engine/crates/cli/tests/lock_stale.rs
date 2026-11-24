// SPDX-License-Identifier: Apache-2.0
//! What `check --strict` makes of a lock that its sources no longer resolve to
//! (#1975).
//!
//! # The defect this target exists for
//!
//! `headwater check` takes its taxonomy from `.headwater/taxonomy.lock` alone.
//! An edit to `.headwater/overlay.yml`, to a vendored package file, or to the
//! selection in `.headwater/taxonomy.yml` reached no check until somebody ran
//! `headwater taxonomy resolve`, and `check --strict` passed at exit 0 in the
//! meantime. In the campaign's `change_taxonomy` scenario, 9 of 9 failures
//! edited the overlay and never resolved, and nothing told them.
//!
//! So the rule `taxonomy.lock.stale` holds the lock to its sources on every
//! run: one error on each source whose bytes moved, and one on the consumer
//! declaration where the selection moved. Each case here drives the built
//! binary, because the claim is about an exit status and a report, and both
//! are the caller's.

mod common;

use common::Root;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

const RULE: &str = "taxonomy.lock.stale";
const OVERLAY: &str = ".headwater/overlay.yml";
const CONSUMER: &str = ".headwater/taxonomy.yml";

/// The output with each run of whitespace folded to one space, so a finding
/// reads as one sentence wherever the report wrapped it.
fn flat(out: &str) -> String {
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// One more retired term in the house language regime: a real declaration
/// change, which a resolve would carry into the lock.
fn retire_a_term(at: &Path) {
    let path = at.join(OVERLAY);
    let text = std::fs::read_to_string(&path).expect("the overlay reads");
    let from = "    retired_terms:\n";
    assert!(text.contains(from), "the overlay lists retired terms");
    let to = format!(
        "{from}      - term: synergy\n        reason: it stands in for what the parts do together\n"
    );
    std::fs::write(&path, text.replacen(from, &to, 1)).expect("the overlay writes");
}

/// One finding of a run, as `--format json` writes it.
#[derive(Debug)]
struct Found {
    rule: String,
    severity: String,
    path: String,
    text: String,
}

/// Run `check --strict --format json` and read back its exit status and its
/// findings.
///
/// The scratch root fails strict on its own for reasons this file is not
/// about, so a case reads the errors by rule rather than the exit status
/// alone: the control carries every error the edited root carries except the
/// one this rule adds.
fn strict(root: &Root) -> (Option<i32>, Vec<Found>) {
    let ran = root.run(&["check", "--strict", "--format", "json"]);
    let parsed = headwater_yaml::load(&ran.out).expect("the report is JSON");
    let map = parsed.value.as_map().expect("the report is an object");
    let list = map
        .get("findings")
        .and_then(|node| node.value.as_seq())
        .expect("the report lists findings");
    let field = |finding: &headwater_yaml::Mapping, key: &str| {
        finding
            .get(key)
            .and_then(|node| node.value.as_scalar())
            .map(|scalar| scalar.text.clone())
            .unwrap_or_default()
    };
    let found = list
        .iter()
        .filter_map(|node| node.value.as_map())
        .map(|finding| Found {
            rule: field(finding, "rule"),
            severity: field(finding, "severity"),
            path: field(finding, "path"),
            text: flat(&format!(
                "{} {}",
                field(finding, "message"),
                field(finding, "remediation")
            )),
        })
        .collect();
    (ran.code, found)
}

/// The errors of a run that are not this rule's, by rule and path, so a case
/// can say the edit added this rule's error and nothing else.
fn other_errors(found: &[Found]) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = found
        .iter()
        .filter(|f| f.severity == "error" && f.rule != RULE)
        .map(|f| (f.rule.clone(), f.path.clone()))
        .collect();
    out.sort();
    out
}

fn stale(found: &[Found]) -> Vec<&Found> {
    found.iter().filter(|f| f.rule == RULE).collect()
}

/// The control has no finding of this rule, and the edited root has exactly
/// one, an error on `path` that names the verb that repairs it.
fn assert_one_error_on(control: &[Found], edited: &[Found], path: &str) {
    assert!(
        stale(control).is_empty(),
        "a freshly resolved root reports no stale lock: {:?}",
        stale(control)
    );
    let found = stale(edited);
    assert_eq!(found.len(), 1, "one stale-lock finding: {found:?}");
    assert_eq!(found[0].path, path, "on the moved file: {found:?}");
    assert_eq!(found[0].severity, "error", "and strict fails on it");
    assert!(
        found[0].text.contains("headwater taxonomy resolve"),
        "the finding names the verb that repairs it: {}",
        found[0].text
    );
    assert_eq!(
        other_errors(control),
        other_errors(edited),
        "the edit adds this rule's error and no other"
    );
}

#[test]
fn an_overlay_edit_with_no_resolve_is_an_error_until_resolve_runs() {
    let root = Root::new("lock-stale-overlay");
    let (_, control) = strict(&root);

    retire_a_term(&root.at);
    let (code, edited) = strict(&root);
    assert_one_error_on(&control, &edited, OVERLAY);
    assert_eq!(code, Some(1), "strict fails");
    let digests: Vec<&str> = found_digests(&stale(&edited)[0].text);
    assert_eq!(
        digests.len(),
        2,
        "the message names the recorded and the computed digest: {}",
        stale(&edited)[0].text
    );
    assert_ne!(digests[0], digests[1]);

    let resolved = root.run(&["taxonomy", "resolve"]);
    assert_eq!(resolved.code, Some(0), "the root resolves: {resolved:?}");
    let (_, after) = strict(&root);
    assert!(stale(&after).is_empty(), "a resolve clears the finding");
}

/// Every `sha256:` digest a text names, in order.
fn found_digests(text: &str) -> Vec<&str> {
    text.split(|c: char| c.is_whitespace() || c == '`' || c == ',')
        .filter(|word| word.starts_with("sha256:"))
        .collect()
}

#[test]
fn a_dropped_bundle_is_an_error_on_the_consumer_declaration() {
    let root = Root::new("lock-stale-bundle");
    let (_, control) = strict(&root);
    let path = root.at.join(CONSUMER);
    let text = std::fs::read_to_string(&path).expect("the declaration reads");
    let from = ", requirement-verification]";
    assert!(text.contains(from), "the declaration selects the bundle");
    std::fs::write(&path, text.replacen(from, "]", 1)).expect("the declaration writes");

    let (_, edited) = strict(&root);
    assert_one_error_on(&control, &edited, CONSUMER);
}

#[test]
fn an_edit_that_changes_no_resolution_is_silent() {
    let root = Root::new("lock-stale-silent");
    let (_, control) = strict(&root);
    let path = root.at.join(CONSUMER);
    let text = std::fs::read_to_string(&path).expect("the declaration reads");
    let from = "package content, and a skeleton rather than prose.";
    assert!(text.contains(from), "the declaration carries the exclusion");
    std::fs::write(
        &path,
        text.replacen(from, "package content, and a skeleton and not prose.", 1),
    )
    .expect("the declaration writes");

    let (_, edited) = strict(&root);
    assert!(
        stale(&edited).is_empty(),
        "the reason of an exclusion moves no resolution: {:?}",
        stale(&edited)
    );
    assert_eq!(other_errors(&control), other_errors(&edited));
}

/// `headwater mcp` reruns the binary for its `check` tool, and keeps the
/// findings on files changed since it started. An overlay edited after the
/// server started is one of them, so the answer names the rule and the verb.
#[test]
fn the_mcp_check_tool_names_the_stale_lock_on_an_overlay_edited_after_start() {
    let root = Root::new("lock-stale-mcp");
    let mut child = root
        .command()
        .args(["mcp", "--now", "2026-11-25", "--root"])
        .arg(&root.at)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("the server runs");
    let mut stdin = child.stdin.take().expect("the standard input is piped");
    let mut stdout = BufReader::new(child.stdout.take().expect("the output is piped"));
    stdin
        .write_all(b"{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{}}\n")
        .expect("the request writes");
    let mut line = String::new();
    stdout.read_line(&mut line).expect("the server answers");
    assert!(line.contains("\"id\":1"), "initialize answers: {line}");

    // A modification time the server reads as after its start.
    std::thread::sleep(std::time::Duration::from_millis(1100));
    retire_a_term(&root.at);
    stdin
        .write_all(
            b"{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":\
              {\"name\":\"check\",\"arguments\":{\"format\":\"text\"}}}\n",
        )
        .expect("the request writes");
    drop(stdin);
    let mut answer = String::new();
    stdout.read_line(&mut answer).expect("the server answers");
    let _ = child.wait();
    assert!(answer.contains("\"id\":2"), "the call answers: {answer}");
    assert!(answer.contains(RULE), "the answer names the rule: {answer}");
    assert!(
        answer.contains("headwater taxonomy resolve"),
        "the answer names the verb that repairs it: {answer}"
    );
}
