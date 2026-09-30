// SPDX-License-Identifier: Apache-2.0
//! `headwater new` over a relation whose far half it does not write, and the
//! line its report prints for each (#1187, HW-DR-0101).
//!
//! A symmetric relation is its own inverse, so the half in the new document
//! states the edge. The verb leaves the target's bytes alone and says why,
//! rather than "because reciprocity is required". A relation that declares
//! `on_target.set_state` gets a line that names the rule that reads the state
//! and the fix that writes it, rather than a claim that no rule reads a
//! transition.
//!
//! The base publishes no symmetric relation that a scaffold creates, so the
//! tree adds one to this repository's overlay, as an adopter would.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The relation this tree adds under the overlay's one `add:` block.
const PAIRS_WITH: &str = "add:
  relations.pairs_with:
    family: association
    from: [decision]
    to:   [decision]
    reciprocal: symmetric
    nuclearity: multinuclear
    created_by: scaffold
";

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
            "headwater-cli-new-symmetric-far-half-{}-{label}",
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
        std::fs::copy(
            repository.join(".headwater/taxonomy.yml"),
            at.join(".headwater/taxonomy.yml"),
        )
        .expect("the declaration copies");
        let overlay = std::fs::read_to_string(repository.join(".headwater/overlay.yml"))
            .expect("the overlay reads");
        assert!(
            overlay.contains("\nadd:\n"),
            "the overlay has one add block"
        );
        std::fs::write(
            at.join(".headwater/overlay.yml"),
            overlay.replacen("\nadd:\n", &format!("\n{PAIRS_WITH}"), 1),
        )
        .expect("the overlay writes");
        let root = Root { at };
        let resolved = root.run(&["taxonomy", "resolve"]);
        assert_eq!(resolved.code, Some(0), "the fixture resolves\n{resolved:?}");
        root
    }

    /// Scaffold one decision, move it to `current`, and return its path and
    /// its identifier. It is the live target of every case here.
    fn live_decision(&self) -> (PathBuf, String) {
        let made = self.run(&["new", "decision", "--title", "The live anchor"]);
        assert_eq!(made.code, Some(0), "the anchor scaffolds\n{made:?}");
        let path = std::fs::read_dir(self.at.join("docs/decisions"))
            .expect("the shelf reads")
            .map(|entry| entry.expect("the entry reads").path())
            .find(|path| {
                path.extension().is_some_and(|kind| kind == "md")
                    && path.file_name().is_some_and(|name| name != "README.md")
            })
            .expect("the scaffolder wrote a document");
        let text = std::fs::read_to_string(&path).expect("the document reads");
        assert!(text.contains("\nstatus: draft\n"), "{text}");
        std::fs::write(
            &path,
            text.replacen("\nstatus: draft\n", "\nstatus: current\n", 1),
        )
        .expect("the anchor is promoted");
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

#[test]
fn a_symmetric_relation_leaves_its_live_target_alone_and_says_why() {
    let root = Root::new("symmetric");
    let (anchor, id) = root.live_decision();
    let before = std::fs::read(&anchor).expect("the anchor reads");
    let relates = format!("pairs_with={id}");
    let made = root.run(&[
        "new",
        "decision",
        "--title",
        "A draft paired with the anchor",
        "--relates",
        &relates,
    ]);
    assert_eq!(made.code, Some(0), "stderr:\n{}", made.err);
    assert_eq!(
        std::fs::read(&anchor).expect("the anchor reads"),
        before,
        "the live anchor is not edited\n{made:?}"
    );
    assert!(
        made.out
            .contains("the relation is symmetric, so this half states the edge"),
        "{made:?}"
    );
    assert!(
        !made.out.contains("because reciprocity is required")
            && !made.out.contains("asks for no far half"),
        "{made:?}"
    );
}

#[test]
fn a_relation_that_sets_a_target_state_names_the_rule_that_reads_it() {
    let root = Root::new("supersedes");
    let (anchor, id) = root.live_decision();
    let before = std::fs::read(&anchor).expect("the anchor reads");
    let relates = format!("supersedes={id}");
    let made = root.run(&[
        "new",
        "decision",
        "--title",
        "A draft successor of the anchor",
        "--relates",
        &relates,
    ]);
    assert_eq!(made.code, Some(0), "stderr:\n{}", made.err);
    assert_eq!(
        std::fs::read(&anchor).expect("the anchor reads"),
        before,
        "a draft successor defers its far half (HW-DR-0086)\n{made:?}"
    );
    assert!(
        made.out.contains("the relation sets `superseded` on")
            && made.out.contains("`lifecycle.state.not_set_by_edge`")
            && made.out.contains("`headwater check --fix` writes it"),
        "{made:?}"
    );
    assert!(
        !made
            .out
            .contains("no rule of this engine reads a transition"),
        "{made:?}"
    );
}
