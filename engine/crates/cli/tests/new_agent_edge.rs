// SPDX-License-Identifier: Apache-2.0
//! `headwater new` over the edges an agent writes (#1560, HW-DR-0104).
//!
//! The scaffold transcript holds what the library decides. This file holds
//! what the binary prints and writes from it: the anchor a target bound to,
//! the comment a `cited_in` file still owes, and a refusal that leaves the
//! tree unchanged. The tree is this repository's own taxonomy, which declares
//! `governs` onto `code_path` and `cited_in` onto `test_site`.

use std::path::{Path, PathBuf};
use std::process::Command;

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
            "headwater-cli-new-agent-edge-{}-{label}",
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
        std::fs::create_dir_all(at.join("src")).expect("the source directory is made");
        std::fs::write(at.join("src/widget.rs"), "pub fn widget() {}\n").expect("a source file");
        std::fs::write(at.join("src/gadget.rs"), "pub fn gadget() {}\n").expect("a source file");
        std::fs::write(at.join("logo.png"), b"\x89PNG\r\n\x1a\n\xff\xfe\x00")
            .expect("a binary file");
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

    /// Every document under `docs/` other than the taxonomy bundles.
    fn documents(&self) -> Vec<PathBuf> {
        let mut found = Vec::new();
        walk(&self.at.join("docs"), &mut found);
        found
            .into_iter()
            .filter(|path| !path.starts_with(self.at.join("docs/taxonomies")))
            .collect()
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.at);
    }
}

fn walk(directory: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        match path.is_dir() {
            true => walk(&path, found),
            false => found.push(path),
        }
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

#[test]
fn a_cited_in_file_is_written_and_the_report_names_the_comment_it_owes() {
    let root = Root::new("cited-in");
    let made = root.run(&[
        "new",
        "verification",
        "--title",
        "A citation the widget owes",
        "--relates",
        "cited_in=src/widget.rs",
    ]);
    assert_eq!(made.code, Some(0), "{made:?}");
    assert!(
        made.out
            .contains("`created_by: agent`, so an agent pays for it"),
        "{}",
        made.out
    );
    assert!(
        made.out
            .contains("the target binds as the anchor `test_site`, and it reaches 1 entry"),
        "{}",
        made.out
    );
    assert!(
        made.out
            .contains("`test_site` binds `src/widget.rs` once a Rust `//` or `/* */` comment in it cites `"),
        "the report names the comment the file owes\n{}",
        made.out
    );
}

#[test]
fn a_governs_pattern_is_written_with_its_reach_and_one_onto_nothing_is_refused() {
    let root = Root::new("governs");
    let refused = root.run(&[
        "new",
        "decision",
        "--title",
        "A pattern onto nothing",
        "--relates",
        "governs=src/nowhere/**",
    ]);
    assert_eq!(refused.code, Some(1), "{refused:?}");
    assert!(refused.err.contains("binds to nothing"), "{}", refused.err);
    assert!(root.documents().is_empty(), "nothing is written");

    let made = root.run(&[
        "new",
        "decision",
        "--title",
        "A pattern onto the sources",
        "--relates",
        "governs=src/*.rs",
    ]);
    assert_eq!(made.code, Some(0), "{made:?}");
    assert!(
        made.out
            .contains("the target binds as the anchor `code_path`, and it reaches 2 entries"),
        "{}",
        made.out
    );
}

#[test]
fn a_cited_in_pattern_is_refused_because_comment_scan_opens_one_named_file() {
    let root = Root::new("cited-in-pattern");
    let refused = root.run(&[
        "new",
        "verification",
        "--title",
        "A citation in a pattern",
        "--relates",
        "cited_in=src/*.rs",
    ]);
    assert_eq!(refused.code, Some(1), "{refused:?}");
    assert!(refused.err.contains("binds to nothing"), "{}", refused.err);
    assert!(root.documents().is_empty(), "nothing is written");
}

/// `comment-scan` reads the file as text, so a file that is not UTF-8 can
/// never carry a citation the check reads, and the verb refuses it rather
/// than promising a comment that would not bind.
#[test]
fn a_cited_in_file_that_is_not_text_is_refused() {
    let root = Root::new("cited-in-binary");
    let refused = root.run(&[
        "new",
        "verification",
        "--title",
        "A citation in an image",
        "--relates",
        "cited_in=logo.png",
    ]);
    assert_eq!(refused.code, Some(1), "{refused:?}");
    assert!(refused.err.contains("binds to nothing"), "{}", refused.err);
    assert!(root.documents().is_empty(), "nothing is written");
}
