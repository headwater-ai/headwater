// SPDX-License-Identifier: Apache-2.0
//! What `check --strict` makes of a document whose discriminator names no kind.
//!
//! # The defect this target exists for
//!
//! HW-OBL-0191 measured a review written with `doc_type: review_recrod` on the
//! heterogeneous `reviews` shelf. The census counted it as untyped, so no
//! template instantiated over it, coverage reads typed rows alone, and a strict
//! run exited 0 over a document that had left every check.
//! [Spec 2](../../../../docs/spec/02-taxonomy-model.md#kind-resolution), step
//! 3, states the opposite: "A missing or unrecognized value is a finding". The
//! rule `shelf.discriminator.unresolved` is that finding.
//!
//! # Why these cases drive the binary
//!
//! The claim is about an exit status and a report, and both are the caller's,
//! on the terms `blank_facet.rs` states. Each case assembles a scratch
//! repository from this repository's taxonomy, writes one document onto the
//! `reviews` shelf, and runs the built binary over the result.
//!
//! # The controls
//!
//! The rule reads three stops of kind resolution and no others. A control for
//! each neighbor it must not reach is a case of its own: an admitted value, a
//! path that no shelf claims, and a file on the shelf with no front matter.

use std::path::{Path, PathBuf};
use std::process::Command;

const RULE: &str = "shelf.discriminator.unresolved";

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

/// A scratch repository holding this repository's taxonomy and nothing else.
struct Root {
    at: PathBuf,
}

impl Root {
    fn new(label: &str) -> Root {
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-discriminator-{}-{label}",
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
        // The stubs `blank_facet.rs` writes, for the reason it gives: the
        // overlay names these paths outside the corpus root, and a path that
        // matches no file is an error of a strict run.
        for entry in [
            "README.md",
            ".github/CONTRIBUTING.md",
            ".github/SECURITY.md",
            ".github/ISSUE_TEMPLATE/issue.md",
        ] {
            let to = at.join(entry);
            std::fs::create_dir_all(to.parent().expect("it has a parent"))
                .expect("the directory is there");
            std::fs::write(to, "# A stub\n\nThis file stands in for the real one.\n")
                .expect("the stub writes");
        }

        let root = Root { at };
        let resolved = root.run(&["taxonomy", "resolve"]);
        assert_eq!(
            resolved.code,
            Some(0),
            "the fixture resolves\n{}{}",
            resolved.out,
            resolved.err
        );
        root
    }

    fn write(&self, path: &str, text: &str) {
        let to = self.at.join(path);
        std::fs::create_dir_all(to.parent().expect("it has a parent"))
            .expect("the shelf directory is there");
        std::fs::write(to, text).expect("the document writes");
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

    fn strict(&self) -> Ran {
        self.run(&["check", "--strict", "--no-cache"])
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.at);
    }
}

/// The two streams held apart, on `blank_facet.rs`'s terms.
#[derive(Debug)]
struct Ran {
    code: Option<i32>,
    out: String,
    err: String,
}

impl Ran {
    /// Every finding of this rule, as the text report prints it: the line
    /// `<path>:<line>:<column> <severity>`, and the line `  <rule>: <message>`
    /// under it, joined by a newline.
    fn of_rule(&self) -> Vec<String> {
        let lines: Vec<&str> = self.out.lines().collect();
        lines
            .windows(2)
            .filter(|pair| pair[1].starts_with(&format!("  {RULE}: ")))
            .map(|pair| format!("{}\n{}", pair[0], pair[1]))
            .collect()
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

const PATH: &str = "docs/reviews/probe.md";

/// A review whose front matter opens with the line given, so that the
/// discriminator, where there is one, is on line 2.
fn review(first: &str) -> String {
    format!("---\n{first}title: A probe\n---\n\n# A probe\n\nThis is a probe.\n")
}

/// Exactly one finding of this rule, at the path and line given, in a strict
/// run that fails. Returns the finding line for the caller's own assertions.
fn refused(ran: &Ran, line: usize) -> String {
    assert_ne!(
        ran.code,
        Some(0),
        "a strict run passed a document that left every check\n{}{}",
        ran.out,
        ran.err
    );
    let found = ran.of_rule();
    assert_eq!(found.len(), 1, "one finding of {RULE}\n{}", ran.out);
    let at = format!("{PATH}:{line}:");
    assert!(
        found[0].starts_with(&at),
        "the finding anchors at {at}\n{}",
        ran.out
    );
    let head = found[0].lines().next().expect("the finding has a location");
    assert!(head.ends_with(" error"), "the finding is an error\n{head}");
    found[0].clone()
}

/// The reproduction of HW-OBL-0191: a misspelled kind on a heterogeneous
/// shelf is an error at the line of the discriminator, and it names the
/// value and the kinds the shelf admits.
#[test]
fn an_unknown_discriminator_value_is_an_error_at_its_line() {
    let root = Root::new("unknown");
    root.write(PATH, &review("doc_type: review_recrod\n"));
    let ran = root.strict();
    let finding = refused(&ran, 2);
    assert!(finding.contains("review_recrod"), "{finding}");
    assert!(
        finding.contains("review_prompt, review_record"),
        "the finding names the permitted kinds\n{finding}"
    );
}

/// No discriminator at all is the same stop, and its anchor is the document's
/// first line, because the derivation's span is the shelf's in the lock.
#[test]
fn a_missing_discriminator_is_an_error_at_line_one() {
    let root = Root::new("missing");
    root.write(PATH, &review(""));
    let ran = root.strict();
    let finding = refused(&ran, 1);
    assert!(finding.contains("doc_type"), "{finding}");
}

/// A sequence names no kind, so it is an unrecognized value.
#[test]
fn a_sequence_discriminator_is_an_error() {
    let root = Root::new("sequence");
    root.write(PATH, &review("doc_type: [review_record, review_prompt]\n"));
    let ran = root.strict();
    refused(&ran, 2);
}

/// The control: the same document with an admitted value raises nothing from
/// this rule.
#[test]
fn an_admitted_value_raises_nothing() {
    let root = Root::new("admitted");
    root.write(PATH, &review("doc_type: review_record\n"));
    let ran = root.strict();
    assert!(ran.of_rule().is_empty(), "{}", ran.out);
}

/// A path that no shelf claims is an ordinary state of a corpus, and it is
/// not this rule.
#[test]
fn a_path_no_shelf_claims_raises_nothing() {
    let root = Root::new("unshelved");
    root.write(
        "notes/loose.md",
        "---\ndoc_type: review_recrod\ntitle: Loose\n---\n\n# Loose\n",
    );
    let ran = root.strict();
    assert!(ran.of_rule().is_empty(), "{}", ran.out);
}

/// A file on the shelf with no front matter has no discriminator to read, and
/// the census accounts for it as a file with no block. It is not this rule.
#[test]
fn a_file_with_no_front_matter_raises_nothing() {
    let root = Root::new("bare");
    root.write(PATH, "# A probe\n\nThis is a probe.\n");
    let ran = root.strict();
    assert!(ran.of_rule().is_empty(), "{}", ran.out);
}
