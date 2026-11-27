// SPDX-License-Identifier: Apache-2.0
//! A document `headwater new` wrote and nobody has written yet, checked over
//! this repository's own taxonomy (#1491, HW-OBL-0113).
//!
//! Before `scaffold.placeholder.unwritten`, `headwater check --strict` passed
//! such a document with no finding that named it. This is the adopter's path:
//! `new` with no `--summary`, then `check`. The MCP `new` tool writes through
//! the same composer, so this case covers it too.

use std::path::{Path, PathBuf};
use std::process::Command;

const RULE: &str = "scaffold.placeholder.unwritten";

/// The pinned date every run here takes.
const NOW: &str = "2026-11-27";

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

#[derive(Debug)]
struct Ran {
    code: Option<i32>,
    out: String,
    err: String,
}

struct Root {
    at: PathBuf,
}

impl Root {
    /// The label keeps two cases apart: cargo runs them as threads of one
    /// process, so the pid alone names one directory for both.
    fn new(label: &str) -> Root {
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-placeholder-unwritten-{}-{label}",
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

    fn decisions(&self) -> Vec<PathBuf> {
        match std::fs::read_dir(self.at.join("docs/decisions")) {
            Ok(entries) => entries
                .map(|entry| entry.expect("the entry reads").path())
                .filter(|path| path.extension().is_some_and(|kind| kind == "md"))
                .collect(),
            Err(_) => Vec::new(),
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

/// The findings of `RULE` in a JSON report, as the path each one names.
fn named(report: &str) -> Vec<String> {
    let parsed = headwater_yaml::load(report).expect("the report parses");
    member(&parsed.value, "findings")
        .and_then(headwater_yaml::Value::as_seq)
        .expect("the report carries findings")
        .iter()
        .filter(|finding| text(member(&finding.value, "rule")) == RULE)
        .map(|finding| text(member(&finding.value, "path")))
        .collect()
}

fn member<'a>(value: &'a headwater_yaml::Value, key: &str) -> Option<&'a headwater_yaml::Value> {
    value
        .as_map()
        .and_then(|map| map.get(key))
        .map(|spanned| &spanned.value)
}

fn text(value: Option<&headwater_yaml::Value>) -> String {
    value
        .and_then(headwater_yaml::Value::as_scalar)
        .map(headwater_yaml::core_schema::as_str)
        .expect("a scalar")
        .to_string()
}

/// `new` then `check`: the report names the new document once for each
/// section the scaffolder wrote and once for the summary prompt, and a strict
/// run still exits 0, because the finding is a warning.
#[test]
fn check_names_a_document_that_new_wrote_and_strict_still_passes() {
    let root = Root::new("named");
    let made = root.run(&[
        "new",
        "decision",
        "--title",
        "Every check may be disabled by an operator",
    ]);
    assert_eq!(made.code, Some(0), "the decision scaffolds\n{made:?}");
    let written = root.decisions();
    assert_eq!(written.len(), 1, "one document: {written:?}");
    let text = std::fs::read_to_string(&written[0]).expect("the document reads");
    let sections = text.matches("\n## ").count();
    let relative = written[0]
        .strip_prefix(&root.at)
        .expect("under the root")
        .to_string_lossy()
        .into_owned();

    let checked = root.run(&["check", "--no-cache", "--now", NOW, "--format", "json"]);
    let found = named(&checked.out);
    assert!(sections > 0, "the scaffolder wrote no section:\n{text}");
    assert_eq!(
        found,
        vec![relative.clone(); sections + 1],
        "one finding per section and one for the summary\n{checked:?}"
    );

    let text_report = root.run(&["check", "--no-cache", "--now", NOW]);
    assert!(
        text_report.out.contains(RULE) && text_report.out.contains(&relative),
        "the text report names the rule and the document\n{text_report:?}"
    );

    // Scoped to the new document, because this scratch root carries lock
    // findings of its own that have nothing to do with the document.
    let strict = root.run(&[
        "check",
        "--no-cache",
        "--now",
        NOW,
        "--strict",
        "--path",
        &relative,
    ]);
    assert_eq!(
        strict.code,
        Some(0),
        "a placeholder is a warning, so a strict run passes it\n{strict:?}"
    );
}

/// The control: once the summary and each section are written, the rule names
/// nothing.
#[test]
fn a_written_document_is_not_named() {
    let root = Root::new("written");
    let made = root.run(&[
        "new",
        "decision",
        "--title",
        "Every check may be disabled by an operator",
        "--summary",
        "An operator may disable any check, and the report says which",
    ]);
    assert_eq!(made.code, Some(0), "the decision scaffolds\n{made:?}");
    let path = &root.decisions()[0];
    let text = std::fs::read_to_string(path).expect("the document reads");
    std::fs::write(
        path,
        text.replace(
            headwater_check::placeholder::SECTION_PROMPT,
            "An operator disables a check by name.",
        ),
    )
    .expect("the document writes");

    let checked = root.run(&["check", "--no-cache", "--now", NOW, "--format", "json"]);
    assert_eq!(named(&checked.out), Vec::<String>::new(), "{checked:?}");
}
