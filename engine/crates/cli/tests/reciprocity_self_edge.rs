// SPDX-License-Identifier: Apache-2.0
//! `check --fix` over a document whose `relations:` block names the document
//! itself under a relation that requires both halves (#1335).
//!
//! The reciprocity rule reads the far end of the half that exists as the
//! document that owes the other half. For an entry that names its own
//! document, the far end is the same file, so a fix would write a second entry
//! that names the document itself. That is a second defect and not a remedy,
//! so the rule offers no fix and the verb writes nothing. The finding stays,
//! because the edge is still a defect the author has to retarget or delete.

use std::path::{Path, PathBuf};
use std::process::Command;

const RULE: &str = "relation.reciprocity.missing";

/// The pinned date every run here takes, so the fix cannot read a clock.
const NOW: &str = "2026-09-29";

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

struct Root {
    at: PathBuf,
}

#[derive(Debug)]
struct Ran {
    code: Option<i32>,
    out: String,
    err: String,
}

impl Root {
    /// The label keeps two cases apart: cargo runs them as threads of one
    /// process, so the pid alone names one directory for both.
    fn new(label: &str) -> Root {
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-reciprocity-self-edge-{}-{label}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).expect("the root is made");
        let repository = repository();
        copy(
            &repository.join(headwater_resolve::package::PACKAGES),
            &at.join(headwater_resolve::package::PACKAGES),
        );
        copy(
            &repository.join("docs/taxonomies"),
            &at.join("docs/taxonomies"),
        );
        for file in [".headwater/taxonomy.yml", ".headwater/overlay.yml"] {
            std::fs::copy(repository.join(file), at.join(file)).expect("the declaration copies");
        }
        let root = Root { at };
        let resolved = root.run(&["taxonomy", "resolve"]);
        assert_eq!(resolved.code, Some(0), "the fixture resolves\n{resolved:?}");
        root
    }

    /// Scaffold one evaluation and return its path and its identifier.
    fn evaluation(&self, title: &str) -> (PathBuf, String) {
        let made = self.run(&["new", "evaluation", "--title", title]);
        assert_eq!(made.code, Some(0), "the evaluation scaffolds\n{made:?}");
        let path = std::fs::read_dir(self.at.join("docs/evaluations"))
            .expect("the shelf reads")
            .map(|entry| entry.expect("the entry reads").path())
            .find(|path| path.extension().is_some_and(|kind| kind == "md"))
            .expect("the scaffolder wrote a document");
        let text = std::fs::read_to_string(&path).expect("the document reads");
        let id = text
            .lines()
            .find_map(|line| line.strip_prefix("id: "))
            .expect("the scaffolder wrote an identifier")
            .trim()
            .to_string();
        (path, id)
    }

    fn run(&self, arguments: &[&str]) -> Ran {
        let output = Command::new(env!("CARGO_BIN_EXE_headwater"))
            .args(arguments)
            .arg("--root")
            .arg(&self.at)
            .output()
            .expect("the binary runs");
        Ran {
            code: output.status.code(),
            out: String::from_utf8_lossy(&output.stdout).into_owned(),
            err: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.at);
    }
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

/// Write `key: value` into the front matter of `path`, replacing the line
/// that opens with `key:` or adding one before the block closes.
fn set_facet(path: &Path, key: &str, value: &str) {
    let text = std::fs::read_to_string(path).expect("the document reads");
    let body = text
        .strip_prefix("---\n")
        .expect("the document opens a block");
    let (block, rest) = body.split_once("\n---\n").expect("the block closes");
    let prefix = format!("{key}:");
    let mut lines: Vec<String> = block.lines().map(str::to_string).collect();
    match lines.iter().position(|line| line.starts_with(&prefix)) {
        Some(at) => lines[at] = format!("{key}: {value}"),
        None => lines.push(format!("{key}: {value}")),
    }
    std::fs::write(path, format!("---\n{}\n---\n{rest}", lines.join("\n")))
        .expect("the document writes");
}

/// Put `lines` into the front matter of `path`, before the block closes.
fn into_front_matter(path: &Path, lines: &str) {
    let text = std::fs::read_to_string(path).expect("the document reads");
    let body = text
        .strip_prefix("---\n")
        .expect("the document opens a block");
    let (block, rest) = body.split_once("\n---\n").expect("the block closes");
    std::fs::write(path, format!("---\n{block}\n{lines}---\n{rest}")).expect("the document writes");
}

/// The `fixable` member of every reciprocity finding in a JSON report, read
/// with the whitespace taken out so the layout of the report does not matter.
fn fixable_of_reciprocity_findings(json: &str) -> Vec<String> {
    let dense: String = json.split_whitespace().collect();
    // A finding opens with its rule and its severity. The report's table of
    // rules opens with the rule and its scope, so it does not match.
    let opened = format!("\"rule\":\"{RULE}\",\"severity\":");
    dense
        .match_indices(&opened)
        .map(|(at, _)| {
            let after = &dense[at..];
            let member = after
                .find("\"fixable\":")
                .expect("a finding carries a fixable member");
            let value = &after[member + "\"fixable\":".len()..];
            value
                .split([',', '}'])
                .next()
                .expect("the member has a value")
                .to_string()
        })
        .collect()
}

/// A live evaluation writes `cited_by: <its own id>`. `check` reports the
/// missing half with no fix, and `check --fix` leaves the file byte for byte.
#[test]
fn check_fix_writes_nothing_for_a_half_that_names_its_own_document() {
    let root = Root::new("fix");
    let (evaluation, id) = root.evaluation("An evaluation that cites itself");
    set_facet(&evaluation, "status", "current");
    set_facet(&evaluation, "status_since", "2026-09-01");
    into_front_matter(
        &evaluation,
        &format!("relations:\n  cited_by:\n    - {id}\n"),
    );
    let before = std::fs::read(&evaluation).expect("the evaluation reads");

    let fixed = root.run(&["check", "--fix", "--no-cache", "--now", NOW]);
    let after = std::fs::read(&evaluation).expect("the evaluation reads");
    let text = String::from_utf8_lossy(&after);
    assert!(
        !text.contains("cites_evidence"),
        "the fix wrote a second entry that names the document itself\n{text}\n{}",
        fixed.err
    );
    assert_eq!(
        before, after,
        "the fix wrote into the document\n{text}\n{}",
        fixed.err
    );

    let checked = root.run(&["check", "--format", "json", "--no-cache", "--now", NOW]);
    let fixable = fixable_of_reciprocity_findings(&checked.out);
    assert_eq!(
        fixable,
        vec!["false".to_string()],
        "the self-edge is still reported once, with no fix\n{}{}",
        checked.out,
        checked.err
    );
    // `cited_by` is an evidence relation, so the self-target rule reports the
    // same entry, and the remediation says so rather than asking for a half.
    assert!(
        checked.out.contains("names its own document")
            && checked
                .out
                .contains("(`relation.target.is_source` reports it too)"),
        "the remediation says to retarget or delete, and names the self-target rule\n{}",
        checked.out
    );
}
