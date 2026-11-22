// SPDX-License-Identifier: Apache-2.0
//! What `headwater new` leaves in the identifier claim store when the document
//! it claimed for cannot be created.
//!
//! [#1972](https://github.com/headwater-ai/headwater/issues/1972) measured a
//! title of 5,000 letters: the claim landed, the document create failed with
//! `File name too long`, and the claim stayed behind with no document, so the
//! number was spent and the message said no other file was written. The run
//! now withdraws the claim it wrote, and only that claim, because `create_new`
//! returned `Ok` for it in this run and so no other claimant's record can be at
//! that path. The claim is about a tree and a message, which are the caller's,
//! so the cases drive the built binary over a scratch repository, as
//! `warrant_value.rs` does.

use std::path::{Path, PathBuf};
use std::process::Command;

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

const FIRST_CLAIM: &str = ".headwater/ids/decision_id/HW-DR-0001";
const SECOND_CLAIM: &str = ".headwater/ids/decision_id/HW-DR-0002";

/// A scratch repository holding one scaffolded decision record, `HW-DR-0001`,
/// whose claim is the other claimant that must survive.
struct Root {
    at: PathBuf,
}

impl Root {
    fn new(label: &str) -> Root {
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-new-withdraws-{}-{label}",
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

        let root = Root { at };
        let resolved = root.run(&["taxonomy", "resolve"]);
        assert_eq!(
            resolved.code,
            Some(0),
            "the fixture resolves\n{}{}",
            resolved.out,
            resolved.err
        );
        let made = root.run(&["new", "decision", "--title", "First"]);
        assert_eq!(
            made.code,
            Some(0),
            "the first decision scaffolds\n{}{}",
            made.out,
            made.err
        );
        assert_eq!(
            root.read(FIRST_CLAIM),
            "docs/decisions/0001-first.md\n",
            "the first claim is on disk before the case starts"
        );
        root
    }

    fn read(&self, relative: &str) -> String {
        std::fs::read_to_string(self.at.join(relative)).expect("the file reads")
    }

    fn holds(&self, relative: &str) -> bool {
        self.at.join(relative).exists()
    }

    fn decisions(&self) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(self.at.join("docs/decisions"))
            .expect("the decisions shelf is there")
            .map(|entry| {
                entry
                    .expect("the entry reads")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .filter(|name| name.ends_with(".md"))
            .collect();
        names.sort();
        names
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

    /// The state every case asserts after the failed run: no second claim, no
    /// second document, the first claim as it was, and a next run that mints
    /// the number the failed run did not spend.
    fn withdrawn(&self, refused: &Ran) {
        assert_ne!(refused.code, Some(0), "the run refuses\n{refused:?}");
        assert!(
            !self.holds(SECOND_CLAIM),
            "the claim of a document that was never created is withdrawn\n{refused:?}"
        );
        assert_eq!(
            self.decisions(),
            vec!["0001-first.md".to_string()],
            "no second document is on the shelf\n{refused:?}"
        );
        assert_eq!(
            self.read(FIRST_CLAIM),
            "docs/decisions/0001-first.md\n",
            "the other claimant's record is untouched"
        );
        assert!(
            refused
                .err
                .contains(&format!("The claim {SECOND_CLAIM} this run wrote was removed.")),
            "the message names the claim it removed\n{refused:?}"
        );
        assert!(
            !refused.err.contains("could not be removed"),
            "{refused:?}"
        );
    }

    fn mints_the_second(&self) {
        let made = self.run(&["new", "decision", "--title", "Second"]);
        assert_eq!(made.code, Some(0), "the next run scaffolds\n{made:?}");
        assert_eq!(
            self.read(SECOND_CLAIM),
            "docs/decisions/0002-second.md\n",
            "the number the failed run claimed was not spent\n{made:?}"
        );
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        let shelf = self.at.join("docs/decisions");
        if let Ok(metadata) = std::fs::metadata(&shelf) {
            let mut permissions = metadata.permissions();
            #[allow(clippy::permissions_set_readonly_false)]
            permissions.set_readonly(false);
            let _ = std::fs::set_permissions(&shelf, permissions);
        }
        let _ = std::fs::remove_dir_all(&self.at);
    }
}

/// The two streams held apart.
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

/// The measurement of #1972. A file name of 5,000 bytes fails the create with
/// `File name too long` after the claim has landed.
#[test]
fn a_title_too_long_for_a_file_name_leaves_no_claim_behind() {
    let root = Root::new("long-title");
    let title = "a".repeat(5_000);
    let refused = root.run(&["new", "decision", "--title", &title, "--summary", "s"]);
    root.withdrawn(&refused);
    root.mints_the_second();
}

/// A failure that no check of the path before the claim could predict: the
/// shelf is there and the file name is short, and the create still fails,
/// because the shelf directory refuses a new entry.
#[test]
fn a_shelf_that_refuses_the_document_leaves_no_claim_behind() {
    let root = Root::new("locked-shelf");
    let shelf = root.at.join("docs/decisions");
    let mut permissions = std::fs::metadata(&shelf)
        .expect("the shelf is there")
        .permissions();
    permissions.set_readonly(true);
    std::fs::set_permissions(&shelf, permissions).expect("the shelf locks");
    // Root writes into a read-only directory, so the case has nothing to say
    // there.
    let probe = shelf.join("probe");
    if std::fs::write(&probe, "").is_ok() {
        let _ = std::fs::remove_file(&probe);
        return;
    }

    let refused = root.run(&["new", "decision", "--title", "Locked out"]);
    root.withdrawn(&refused);

    let mut permissions = std::fs::metadata(&shelf)
        .expect("the shelf is there")
        .permissions();
    #[allow(clippy::permissions_set_readonly_false)]
    permissions.set_readonly(false);
    std::fs::set_permissions(&shelf, permissions).expect("the shelf unlocks");
    root.mints_the_second();
}
