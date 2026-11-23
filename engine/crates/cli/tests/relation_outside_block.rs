// SPDX-License-Identifier: Apache-2.0
//! A relation written as a top-level front-matter key, outside the
//! `relations:` block (#1639).
//!
//! The graph reads edges from one key of the front matter, `relations:` unless
//! the taxonomy configures another. A document that writes `conflicts_with:`
//! at the top level declares no edge, and until this rule no finding said so:
//! the key was an undeclared facet that every rule ignored. These cases drive
//! the binary, because the claim is about what `check` reports and what
//! `check --fix` writes, and both are the caller's.

use std::path::{Path, PathBuf};
use std::process::Command;

const RULE: &str = "relation.key.outside_block";

/// The pinned date every run here takes, so no case reads a clock.
const NOW: &str = "2026-11-23";

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

/// One finding of [`RULE`], as the JSON report states it.
#[derive(Debug)]
struct Seen {
    path: String,
    line: usize,
    message: String,
    remediation: String,
    fixable: bool,
}

impl Root {
    /// The label keeps two cases apart: cargo runs them as threads of one
    /// process, so the pid alone names one directory for both.
    fn new(label: &str) -> Root {
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-relation-outside-block-{}-{label}",
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

    /// Scaffold one decision and return its path and its identifier.
    fn decision(&self, title: &str) -> (PathBuf, String) {
        let made = self.run(&["new", "decision", "--title", title]);
        assert_eq!(made.code, Some(0), "the decision scaffolds\n{made:?}");
        let path = std::fs::read_dir(self.at.join("docs/decisions"))
            .expect("the shelf reads")
            .map(|entry| entry.expect("the entry reads").path())
            .filter(|path| path.extension().is_some_and(|kind| kind == "md"))
            .max_by_key(|path| {
                std::fs::metadata(path)
                    .and_then(|meta| meta.modified())
                    .expect("the file has a time")
            })
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

    /// Every finding of [`RULE`] in one JSON run.
    fn findings(&self) -> Vec<Seen> {
        let checked = self.run(&["check", "--format", "json", "--no-cache", "--now", NOW]);
        findings_of(&checked.out)
    }

    fn fix(&self) -> Ran {
        self.run(&["check", "--fix", "--no-cache", "--now", NOW])
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

/// Put `lines` into the front matter of `path`, before the block closes, and
/// return the 1-based line number of the first line put there.
fn into_front_matter(path: &Path, lines: &str) -> usize {
    let text = std::fs::read_to_string(path).expect("the document reads");
    let body = text
        .strip_prefix("---\n")
        .expect("the document opens a block");
    let (block, rest) = body.split_once("\n---\n").expect("the block closes");
    std::fs::write(path, format!("---\n{block}\n{lines}---\n{rest}")).expect("the document writes");
    // The opening `---`, then every line of the block, then the first new one.
    1 + block.lines().count() + 1
}

/// The value of one string member of a JSON object, read from the text that
/// follows the member's name. Escapes are left as written, which is enough for
/// the substrings these cases look for.
fn string_member(object: &str, name: &str) -> String {
    let opened = format!("\"{name}\": \"");
    let at = object
        .find(&opened)
        .unwrap_or_else(|| panic!("the finding carries `{name}`\n{object}"));
    let rest = &object[at + opened.len()..];
    let mut out = String::new();
    let mut escaped = false;
    for character in rest.chars() {
        match (escaped, character) {
            (true, _) => {
                out.push(character);
                escaped = false;
            }
            (false, '\\') => escaped = true,
            (false, '"') => break,
            (false, _) => out.push(character),
        }
    }
    out
}

fn bare_member<'a>(object: &'a str, name: &str) -> &'a str {
    let opened = format!("\"{name}\": ");
    let at = object
        .find(&opened)
        .unwrap_or_else(|| panic!("the finding carries `{name}`\n{object}"));
    object[at + opened.len()..]
        .split([',', '\n', '}'])
        .next()
        .expect("the member has a value")
        .trim()
}

/// Every finding of [`RULE`] in a JSON report. A finding opens with its rule
/// and then its severity, and the report's table of rules opens with the rule
/// and then its scope, so the table does not match.
fn findings_of(json: &str) -> Vec<Seen> {
    let opened = format!("\"rule\": \"{RULE}\",\n");
    json.match_indices(&opened)
        .filter_map(|(at, _)| {
            let object = &json[at..];
            let object = &object[..object.find("\n    }").unwrap_or(object.len())];
            if !object.contains("\"severity\":") {
                return None;
            }
            Some(Seen {
                path: string_member(object, "path"),
                line: bare_member(object, "line").parse().expect("a line number"),
                message: string_member(object, "message"),
                remediation: string_member(object, "remediation"),
                fixable: bare_member(object, "fixable") == "true",
            })
        })
        .collect()
}

fn relative(root: &Root, path: &Path) -> String {
    path.strip_prefix(&root.at)
        .expect("the path is under the root")
        .to_string_lossy()
        .replace('\\', "/")
}

/// The decisive case. A decision writes `conflicts_with:` at the top level and
/// no `relations:` block. `check` reports the key at its own line, names
/// `relations:` as the place a relation is written and marks the finding
/// fixable. `check --fix` moves the key under `relations:`, after which the
/// rule reports nothing and the graph holds the edge, and a second `--fix`
/// changes no byte.
#[test]
fn a_top_level_relation_key_is_reported_and_moved_under_relations() {
    let root = Root::new("decisive");
    let (target, target_id) = root.decision("The decision the other conflicts with");
    let (source, source_id) = root.decision("The decision that names a relation at the top");
    let _ = target;
    let line = into_front_matter(&source, &format!("conflicts_with: [{target_id}]\n"));

    let seen = root.findings();
    assert_eq!(seen.len(), 1, "one finding of the rule\n{seen:?}");
    let finding = &seen[0];
    assert_eq!(finding.path, relative(&root, &source), "{finding:?}");
    assert_eq!(finding.line, line, "the finding is at the key\n{finding:?}");
    assert!(
        finding.message.contains("`conflicts_with`") && finding.message.contains("no edge"),
        "the message names the key and says the graph read no edge from it\n{finding:?}"
    );
    assert!(
        finding.remediation.contains("`relations:`"),
        "the remediation names the block a relation is written in\n{finding:?}"
    );
    assert!(finding.fixable, "the finding carries a patch\n{finding:?}");

    let fixed = root.fix();
    assert_eq!(fixed.code, Some(0), "the fix runs\n{fixed:?}");
    let text = std::fs::read_to_string(&source).expect("the decision reads");
    assert!(
        !text.lines().any(|line| line.starts_with("conflicts_with:")),
        "no top-level `conflicts_with` remains\n{text}\n{}",
        fixed.err
    );
    assert!(
        text.contains(&format!(
            "relations:\n  conflicts_with:\n    - {target_id}\n"
        )),
        "`relations:` holds `conflicts_with` with the target\n{text}\n{}",
        fixed.err
    );

    assert!(
        root.findings().is_empty(),
        "the rule reports nothing after the fix"
    );
    let explained = root.run(&["explain", &source_id]);
    assert_eq!(explained.code, Some(0), "explain runs\n{explained:?}");
    assert!(
        explained.out.contains("conflicts_with") && explained.out.contains(&target_id),
        "the graph holds the edge\n{}",
        explained.out
    );

    let before = std::fs::read(&source).expect("the decision reads");
    let again = root.fix();
    assert_eq!(again.code, Some(0), "the second fix runs\n{again:?}");
    assert_eq!(
        before,
        std::fs::read(&source).expect("the decision reads"),
        "a second fix changes no byte\n{}",
        again.err
    );
}

/// A document that already has a `relations:` block with another relation:
/// the move merges into that block and leaves the other relation where it is.
#[test]
fn the_move_merges_into_a_relations_block_that_is_already_there() {
    let root = Root::new("merge");
    let (_, first) = root.decision("The first decision");
    let (_, second) = root.decision("The second decision");
    let (source, _) = root.decision("The decision with a block already");
    into_front_matter(
        &source,
        &format!("relations:\n  draws_on:\n    - {first}\nconflicts_with:\n  - {second}\n"),
    );

    let seen = root.findings();
    assert_eq!(seen.len(), 1, "one finding of the rule\n{seen:?}");
    assert!(seen[0].fixable, "{seen:?}");

    let fixed = root.fix();
    assert_eq!(fixed.code, Some(0), "the fix runs\n{fixed:?}");
    let text = std::fs::read_to_string(&source).expect("the decision reads");
    assert!(
        !text.lines().any(|line| line.starts_with("conflicts_with:")),
        "no top-level `conflicts_with` remains\n{text}\n{}",
        fixed.err
    );
    assert_eq!(
        text.matches("relations:").count(),
        1,
        "one block, not two\n{text}"
    );
    assert!(
        text.contains(&format!("  draws_on:\n    - {first}\n"))
            && text.contains(&format!("  conflicts_with:\n    - {second}\n")),
        "both relations sit under the one block\n{text}\n{}",
        fixed.err
    );
    assert!(root.findings().is_empty(), "the rule reports nothing after");
}

/// An inverse name is a relation name too: a half written as
/// `superseded_by:` at the top level is reported and moved.
#[test]
fn an_inverse_name_at_the_top_level_is_reported_and_moved() {
    let root = Root::new("inverse");
    let (_, newer) = root.decision("The newer decision");
    let (source, _) = root.decision("The older decision");
    let line = into_front_matter(&source, &format!("superseded_by: {newer}\n"));

    let seen = root.findings();
    assert_eq!(seen.len(), 1, "one finding of the rule\n{seen:?}");
    assert_eq!(seen[0].line, line, "{seen:?}");
    assert!(seen[0].message.contains("`superseded_by`"), "{seen:?}");
    assert!(seen[0].fixable, "{seen:?}");

    let fixed = root.fix();
    let text = std::fs::read_to_string(&source).expect("the decision reads");
    assert!(
        !text.lines().any(|line| line.starts_with("superseded_by:"))
            && text.contains(&format!("relations:\n  superseded_by:\n    - {newer}\n")),
        "the half sits under `relations:`\n{text}\n{}",
        fixed.err
    );
}

/// A top-level key that names no relation draws no finding: the rule reads
/// the declared relation names and nothing else.
#[test]
fn a_key_that_names_no_relation_draws_no_finding() {
    let root = Root::new("unrelated");
    let (source, _) = root.decision("A decision with an unknown key");
    into_front_matter(&source, "conflicts_with_nothing: [x]\n");
    assert!(
        root.findings().is_empty(),
        "no finding for a non-relation key"
    );
}

/// A value the move cannot express as halves is reported with no patch, and
/// `--fix` leaves the file byte for byte.
#[test]
fn a_mapping_value_is_reported_with_no_patch() {
    let root = Root::new("mapping");
    let (_, target) = root.decision("The target");
    let (source, _) = root.decision("The decision with a mapping");
    into_front_matter(&source, &format!("conflicts_with:\n  to: {target}\n"));

    let seen = root.findings();
    assert_eq!(seen.len(), 1, "one finding of the rule\n{seen:?}");
    assert!(!seen[0].fixable, "no patch for a mapping\n{seen:?}");
    assert!(seen[0].remediation.contains("`relations:`"), "{seen:?}");

    let before = std::fs::read(&source).expect("the decision reads");
    root.fix();
    assert_eq!(
        before,
        std::fs::read(&source).expect("the decision reads"),
        "the fix writes nothing for a value it cannot move"
    );
}
