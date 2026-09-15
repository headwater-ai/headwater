// SPDX-License-Identifier: Apache-2.0
//! What `check --strict` makes of a facet that is declared and carries nothing.
//!
//! # The defect this target exists for
//!
//! [#541](https://github.com/headwater-ai/headwater/issues/541) measured a
//! `title` written as `title:` reaching a rendered shelf index as `[~]`, at
//! exit 0, through every gate this engine has. A missing `title` was an error
//! and a blank one was silent, which is the asymmetry the rule
//! `facet.value.blank` closes.
//!
//! # Why these cases drive the binary
//!
//! The claim is about an exit status and a report, and both are the caller's.
//! A case that constructed a view and called `evaluate` would prove the
//! predicate and not the gate, and the gate is what the issue measured green.
//! So each case here assembles a scratch repository, scaffolds one document
//! into it, writes one blank shape onto that document and runs the built
//! binary over the result.
//!
//! # Why the first case writes `~` rather than `""`
//!
//! `title:` with nothing after the colon is handed back by the parser as a
//! plain scalar whose text is the literal `~`. An implementation that tested
//! `trim().is_empty()` alone would pass that family through and still satisfy
//! a case written against `""`, so `~` is the arm that separates a rule that
//! reads the core schema from one that does not.

use std::path::{Path, PathBuf};
use std::process::Command;

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

/// A scratch repository holding one scaffolded decision record.
struct Root {
    at: PathBuf,
    document: PathBuf,
}

impl Root {
    fn new(label: &str) -> Root {
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-blank-facet-{}-{label}",
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
        for name in ["taxonomy.yml", "overlay.yml"] {
            let to = at.join(".headwater").join(name);
            std::fs::create_dir_all(to.parent().expect("it has a parent"))
                .expect("the declaration directory is there");
            std::fs::copy(repository.join(".headwater").join(name), to)
                .expect("the declaration copies");
        }

        let mut root = Root {
            at,
            document: PathBuf::new(),
        };
        let resolved = root.run(&["taxonomy", "resolve"]);
        assert_eq!(
            resolved.code,
            Some(0),
            "the fixture resolves\n{}{}",
            resolved.out,
            resolved.err
        );

        // Scaffolded rather than written out here, so a taxonomy that adds a
        // required facet fails this fixture rather than typing the document as
        // nothing and raising the finding for another reason.
        let made = root.run(&["new", "decision", "--title", "A scratch ruling"]);
        assert_eq!(
            made.code,
            Some(0),
            "the corpus document scaffolds\n{}{}",
            made.out,
            made.err
        );
        root.document = std::fs::read_dir(root.at.join("docs/decisions"))
            .expect("the decisions shelf is there")
            .map(|entry| entry.expect("the entry reads").path())
            .find(|path| path.extension().is_some_and(|kind| kind == "md"))
            .expect("the scaffolder wrote a document");
        root
    }

    /// The path the check layer names, relative to the corpus root.
    fn named(&self) -> String {
        let name = self.document.file_name().expect("the document has a name");
        format!("docs/decisions/{}", name.to_string_lossy())
    }

    /// Replace the whole `title:` line with one written out here.
    fn title(&self, line: &str) {
        let text = std::fs::read_to_string(&self.document).expect("the document reads");
        let found = text
            .lines()
            .find(|line| line.starts_with("title:"))
            .expect("the scaffolder wrote a title")
            .to_string();
        std::fs::write(&self.document, text.replacen(&found, line, 1))
            .expect("the document writes");
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

/// The two streams held apart. `headwater check` reports on one and states a
/// run statistic on the other, and a case that merged them reads the wrong one.
#[derive(Debug)]
struct Ran {
    code: Option<i32>,
    out: String,
    err: String,
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

/// Run one shape and assert the strict verdict, the rule and the message.
fn refuses(label: &str, line: &str, complaint: &str) {
    let root = Root::new(label);
    root.title(line);
    let ran = root.run(&["check", "--strict", "--no-cache"]);
    assert_eq!(
        ran.code,
        Some(1),
        "`{line}` passed a strict run\n{}{}",
        ran.out,
        ran.err
    );
    let named = root.named();
    let reported = ran
        .out
        .split(&named)
        .skip(1)
        .any(|after| after.contains("facet.value.blank") && after.contains(complaint));
    assert!(
        reported,
        "no `facet.value.blank` finding on {named} saying {complaint}\n{}",
        ran.out
    );
}

/// The reproduction of #541, with the shape that a partial implementation
/// passes.
#[test]
fn a_title_declared_with_no_value_stops_a_strict_run() {
    refuses("plain-null", "title:", "declared with no value");
}

/// `~` and `null` are the same family, written out. They are here because an
/// author meets all three spellings and a rule that knew one of them would be
/// two thirds silent.
#[test]
fn the_two_written_spellings_of_null_are_the_same_finding() {
    refuses("tilde", "title: ~", "declared with no value");
    refuses("null", "title: null", "declared with no value");
}

/// The family the issue names, and the one a quoted empty string produces.
#[test]
fn a_title_declared_as_empty_text_stops_a_strict_run() {
    refuses("empty-string", "title: \"\"", "declared as empty text");
    refuses("blank-string", "title: \"   \"", "declared as empty text");
}

/// The declaration says `string`, and neither of these is one.
#[test]
fn a_title_that_is_not_a_scalar_stops_a_strict_run() {
    refuses("sequence", "title: []", "declared as a sequence");
    refuses("mapping", "title: {}", "declared as a mapping");
}

/// The negative arm, and the one that decides whether this rule is worth
/// having. A required check that reports correct input is a check the first
/// person it annoys turns off, after which it guards nothing.
#[test]
fn a_title_a_person_wrote_is_not_reported() {
    let root = Root::new("control");
    let ran = root.run(&["check", "--strict", "--no-cache"]);
    assert_eq!(
        ran.code,
        Some(0),
        "the scaffolded corpus does not pass a strict run\n{}{}",
        ran.out,
        ran.err
    );
    assert!(
        !ran.out.contains("facet.value.blank:"),
        "the rule reported a document whose facets a person wrote\n{}",
        ran.out
    );
}
