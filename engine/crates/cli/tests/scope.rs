// SPDX-License-Identifier: Apache-2.0
//! `headwater taxonomy audit` counts the governed scope over what git does
//! not ignore (#951, owner ruling 2026-09-25: "local and CI must agree, and
//! nobody governs a cache").
//!
//! The graph build counts the scope with nothing ignored, because spec 12
//! keeps git off the check loop, and the audit verb drops the ignored entries
//! afterward. This target holds that second step through the binary, so a
//! verb that stopped dropping them prints a different figure here.
//!
//! It also holds `taxonomy audit --json` to the text (#1573): the governed
//! scope figure a CI job uploads is the figure a person reads, for one tree.

mod common;
use common::Root;
use headwater_yaml::json::{count, field};
use std::process::Command;

/// The line of the scope section that names `pattern`.
fn line_for<'a>(out: &'a str, pattern: &str) -> &'a str {
    out.lines()
        .find(|line| line.contains(&format!("`{pattern}`")) && line.contains("in scope"))
        .unwrap_or_else(|| panic!("no scope line for `{pattern}` in:\n{out}"))
}

#[test]
fn the_audit_leaves_a_file_git_ignores_out_of_the_governed_scope() {
    let root = Root::new("scope-audit-ignored");
    let cache = root.at.join("tools/__pycache__/stub.cpython-312.pyc");
    std::fs::create_dir_all(cache.parent().expect("a parent")).expect("the cache is made");
    std::fs::write(&cache, "").expect("the cache writes");
    std::fs::write(root.at.join(".gitignore"), "__pycache__/\n").expect("the ignore file writes");

    // Outside a git repository nothing is ignored, so the cache counts.
    let before = root.run(&["taxonomy", "audit", "--now", "2026-09-25"]);
    assert_eq!(before.code, Some(0), "{before:?}");
    let line = line_for(&before.out, "tools/**");
    assert!(line.contains(" 2 in scope"), "{line}");

    let init = Command::new("git")
        .args(["init", "-q"])
        .current_dir(&root.at)
        .status()
        .expect("git runs");
    assert!(init.success());

    let after = root.run(&["taxonomy", "audit", "--now", "2026-09-25"]);
    assert_eq!(after.code, Some(0), "{after:?}");
    let line = line_for(&after.out, "tools/**");
    assert!(line.contains(" 1 in scope"), "{line}");
    assert!(!after.out.contains("__pycache__"), "{}", after.out);
}

/// The two counts of one scope line of the text: `(in scope, governed)`.
fn counts_of(line: &str) -> (usize, usize) {
    let words: Vec<&str> = line.split_whitespace().collect();
    let before = |noun: &str| {
        let at = words
            .iter()
            .position(|word| word.trim_end_matches(',') == noun)
            .unwrap_or_else(|| panic!("no `{noun}` in {line}"));
        words[at - 1].parse::<usize>().expect("a count")
    };
    let in_scope = words
        .windows(3)
        .find(|w| w[1] == "in" && w[2].starts_with("scope"))
        .map(|w| w[0].parse::<usize>().expect("a count"))
        .unwrap_or_else(|| panic!("no in-scope count in {line}"));
    (in_scope, before("governed"))
}

/// The text's total line: `(governed, in scope)`.
fn total_of(text: &str) -> (usize, usize) {
    let line = text
        .lines()
        .find(|line| line.trim_start().starts_with("in total "))
        .unwrap_or_else(|| panic!("no total line in {text}"));
    let words: Vec<&str> = line.split_whitespace().collect();
    (
        words[2].parse().expect("a count"),
        words[4].parse().expect("a count"),
    )
}

fn path(steps: &[&str]) -> Vec<String> {
    steps.iter().map(|step| step.to_string()).collect()
}

/// One scalar of the JSON document, or a panic naming the path.
fn at(document: &str, steps: &[&str]) -> String {
    field(document, &path(steps)).unwrap_or_else(|| panic!("no {steps:?} in {document}"))
}

fn number(document: &str, steps: &[&str]) -> usize {
    at(document, steps).parse().expect("a count")
}

/// The index of the `scope` element for `pattern`.
fn element_for(document: &str, pattern: &str) -> String {
    let elements = count(document, &path(&["scope"])).expect("`scope` is an array");
    (0..elements)
        .map(|index| index.to_string())
        .find(|index| at(document, &["scope", index, "pattern"]) == pattern)
        .unwrap_or_else(|| panic!("no `{pattern}` element in {document}"))
}

/// The `ungoverned` paths of one `scope` element.
fn ungoverned_of(document: &str, index: &str) -> Vec<String> {
    let paths =
        count(document, &path(&["scope", index, "ungoverned"])).expect("`ungoverned` is an array");
    (0..paths)
        .map(|entry| {
            at(
                document,
                &["scope", index, "ungoverned", &entry.to_string()],
            )
        })
        .collect()
}

/// The text and the JSON of one audit of `root`, both exiting 0.
fn read(root: &Root) -> (String, String) {
    let text = root.run(&["taxonomy", "audit", "--now", "2026-09-25"]);
    assert_eq!(text.code, Some(0), "{text:?}");
    let json = root.run(&["taxonomy", "audit", "--now", "2026-09-25", "--json"]);
    assert_eq!(json.code, Some(0), "{json:?}");
    let document = json.out.trim();
    assert!(
        document.starts_with('{') && document.ends_with('}') && document.lines().count() == 1,
        "one JSON document on one line: {document}"
    );
    (text.out, document.to_string())
}

/// `taxonomy audit --json` carries the governed-scope figure a later ratchet
/// reads (#1573), and it states the figures the text prints for the same tree.
/// A new in-scope file that no `governs` edge reaches is named in
/// `ungoverned`, and a `governs` edge onto it moves it out and raises
/// `governed` by one.
#[test]
fn the_audit_json_names_an_ungoverned_entry_and_drops_it_once_an_edge_reaches_it() {
    let root = Root::new("scope-audit-json");
    std::fs::write(root.at.join("tools/new-script.sh"), "").expect("the new entry writes");
    // A second ungoverned entry that sorts first, so the order of
    // `ungoverned` is a fact the case reads.
    std::fs::write(root.at.join("tools/aa-early.sh"), "").expect("the second entry writes");

    let (text, document) = read(&root);
    assert_eq!(at(&document, &["version"]), "1", "{document}");
    for member in ["package", "version", "lock"] {
        assert!(
            !at(&document, &["subject", member]).is_empty(),
            "{document}"
        );
    }
    // The subject is the header of the text, member by member.
    let header_lock = text
        .lines()
        .find_map(|line| line.trim_start().strip_prefix("lock "))
        .unwrap_or_else(|| panic!("no lock line in {text}"));
    assert_eq!(
        at(&document, &["subject", "lock"]),
        header_lock,
        "{document}"
    );
    assert_eq!(
        at(&document, &["subject", "now"]),
        "2026-09-25",
        "{document}"
    );

    // Every element agrees with its own text line.
    let elements = count(&document, &path(&["scope"])).expect("`scope` is an array");
    assert!(elements > 0, "{document}");
    for index in 0..elements {
        let index = index.to_string();
        let pattern = at(&document, &["scope", &index, "pattern"]);
        let (in_scope, governed) = counts_of(line_for(&text, &pattern));
        assert_eq!(
            number(&document, &["scope", &index, "in_scope"]),
            in_scope,
            "{pattern}"
        );
        assert_eq!(
            number(&document, &["scope", &index, "governed"]),
            governed,
            "{pattern}"
        );
        let ungoverned = ungoverned_of(&document, &index);
        assert_eq!(
            ungoverned.len(),
            in_scope - governed,
            "{pattern}: {document}"
        );
        let mut sorted = ungoverned.clone();
        sorted.sort();
        assert_eq!(ungoverned, sorted, "{pattern}: `ungoverned` is sorted");
        // The share is the percentage the text line prints.
        let line = line_for(&text, &pattern);
        let printed = line
            .split_whitespace()
            .find_map(|word| word.strip_suffix('%'))
            .unwrap_or_else(|| panic!("no percentage in {line}"));
        assert_eq!(
            at(&document, &["scope", &index, "share"]),
            printed,
            "{pattern}: {line}"
        );
        // Every pattern this repository's overlay scopes is on `code_path`.
        assert_eq!(
            at(&document, &["scope", &index, "anchor_kind"]),
            "code_path",
            "{pattern}"
        );
    }
    // The elements come in the order of the text lines, which is declaration order.
    let patterns: Vec<String> = (0..elements)
        .map(|index| at(&document, &["scope", &index.to_string(), "pattern"]))
        .collect();
    let lines: Vec<String> = text
        .lines()
        .filter(|line| line.contains(" in scope,"))
        .map(|line| {
            line.split('`')
                .nth(1)
                .expect("a pattern in backticks")
                .to_string()
        })
        .collect();
    assert_eq!(patterns, lines, "{document}\n{text}");
    let tools_ungoverned = ungoverned_of(&document, &element_for(&document, "tools/**"));
    assert!(tools_ungoverned.len() >= 2, "{tools_ungoverned:?}");
    // The total's share is the percentage the total line prints.
    let total_line = text
        .lines()
        .find(|line| line.trim_start().starts_with("in total "))
        .expect("a total line");
    assert_eq!(
        format!("{}%", at(&document, &["scope_total", "share"])),
        total_line.split_whitespace().last().expect("a share"),
        "{total_line}"
    );

    let tools = element_for(&document, "tools/**");
    let governed = number(&document, &["scope", &tools, "governed"]);
    let in_scope = number(&document, &["scope", &tools, "in_scope"]);
    assert!(
        ungoverned_of(&document, &tools).contains(&"tools/new-script.sh".to_string()),
        "{document}"
    );

    let total = (
        number(&document, &["scope_total", "governed"]),
        number(&document, &["scope_total", "in_scope"]),
    );
    assert_eq!(total, total_of(&text), "{document}\n{text}");

    govern(&root, "tools/new-script.sh");

    let (text, after) = read(&root);
    let tools = element_for(&after, "tools/**");
    assert_eq!(
        number(&after, &["scope", &tools, "governed"]),
        governed + 1,
        "{after}\n{text}"
    );
    assert_eq!(
        number(&after, &["scope", &tools, "in_scope"]),
        in_scope,
        "{after}"
    );
    assert!(
        !ungoverned_of(&after, &tools).contains(&"tools/new-script.sh".to_string()),
        "{after}"
    );
    let total_after = (
        number(&after, &["scope_total", "governed"]),
        number(&after, &["scope_total", "in_scope"]),
    );
    assert_eq!(total_after, (total.0 + 1, total.1), "{after}");
    assert_eq!(total_after, total_of(&text), "{after}\n{text}");
}

/// A `governs` edge onto `entry`, from one decision the fixture did not hold.
///
/// The shape is the decision `governed_pipe.rs` writes, because a decision
/// is a kind that may carry `governs`.
fn govern(root: &Root, entry: &str) {
    let decision = format!(
        "---\nid: HW-DR-0002\ntitle: The decision that governs a script\nstatus: \
         current\nstatus_since: 2026-08-01\nlast_verified: 2026-08-01\nsummary: One \
         decision that governs a script under the tools directory.\nprovenance:\n  \
         warrant: asserted\n  agency: human\n  evidence_basis: unevidenced\nrelations:\n  \
         governs:\n    - {entry}\n---\n\n# The decision that governs a script\n\n## \
         Context\n\nA fixture.\n\n## Decision\n\nIt governs a script.\n\n## \
         Consequences\n\nThe audit reads it.\n"
    );
    std::fs::write(
        root.at
            .join("docs/decisions/0002-the-decision-that-governs-a-script.md"),
        decision,
    )
    .expect("the decision writes");
}

/// `--json --record` still appends the adoption reading, and the line that
/// says so goes to standard error, so standard output is the one document.
#[test]
fn the_audit_json_with_record_appends_and_keeps_standard_output_one_document() {
    let root = Root::new("scope-audit-json-record");
    let store = root.at.join(".headwater/adoption.jsonl");
    let before = std::fs::read_to_string(&store)
        .unwrap_or_default()
        .lines()
        .count();
    let ran = root.run(&[
        "taxonomy",
        "audit",
        "--now",
        "2026-09-25",
        "--json",
        "--record",
    ]);
    assert_eq!(ran.code, Some(0), "{ran:?}");
    let after = std::fs::read_to_string(&store).expect("the store is written");
    assert_eq!(after.lines().count(), before + 1, "{after}");
    assert!(ran.err.contains("appended one adoption reading"), "{ran:?}");
    let document = ran.out.trim();
    assert_eq!(document.lines().count(), 1, "{document}");
    assert_eq!(at(document, &["version"]), "1", "{document}");
}

/// Under `--json` a refusal writes nothing on standard output, and its account
/// is one sentence on standard error (HW-DR-0043).
#[test]
fn a_refused_audit_json_writes_no_document() {
    let root = Root::new("scope-audit-json-refused");
    std::fs::remove_file(root.at.join(".headwater/taxonomy.lock")).expect("the lock is removed");
    let refused = root.run(&["taxonomy", "audit", "--now", "2026-09-25", "--json"]);
    assert_ne!(refused.code, Some(0), "{refused:?}");
    assert_eq!(refused.out, "", "{refused:?}");
    assert!(refused.err.starts_with("headwater: "), "{refused:?}");
}
