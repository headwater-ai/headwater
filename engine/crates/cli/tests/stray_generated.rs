// SPDX-License-Identifier: Apache-2.0
//! A marked file that no projection writes, told two states by two relations.
//!
//! # The defect this target exists for
//!
//! [#1137](https://github.com/headwater-ai/headwater/issues/1137):
//! `lifecycle.state.set_twice` said "`headwater generate` writes `X` onto this
//! page" about a file that carries the generated marker and that no projection
//! declares. The census classifies a file as generated from the marker alone,
//! and generate lists such a file as orphaned and writes nothing onto it. So
//! the finding promised a write that never happens.
//!
//! # Why this case drives the binary
//!
//! The check crate cannot build the projection plan, because the generate
//! crate depends on it. The CLI builds the plan and injects the orphaned paths
//! into the check run. A test inside the check crate proves the rule reads the
//! set. Only a run of the binary proves the set reaches the rule. So this case
//! assembles a scratch repository from this repository's taxonomy, adds a
//! second setter relation so the rule can fire, and hand-places one marked
//! decision that two other decisions tell two states.

use std::path::{Path, PathBuf};
use std::process::Command;

const RULE: &str = "lifecycle.state.set_twice";

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

/// The overlay addition: a second relation that writes a state, and a state
/// different from the one `supersedes` writes, so a clash is possible.
const RETIRES: &str = concat!(
    "  relations.retires:\n",
    "    family: succession\n",
    "    from: [governed_document]\n",
    "    to: [governed_document]\n",
    "    on_target: {set_state: deprecated}\n",
    "    created_by: agent\n",
);

struct Root {
    at: PathBuf,
}

impl Root {
    fn new() -> Root {
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-stray-generated-{}",
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
            "the overlay opens an `add` block"
        );
        std::fs::write(
            at.join(".headwater/overlay.yml"),
            overlay.replacen("\nadd:\n", &format!("\nadd:\n{RETIRES}\n"), 1),
        )
        .expect("the overlay writes");

        let root = Root { at };
        let resolved = root.run(&["taxonomy", "resolve"]);
        assert_eq!(resolved.code, Some(0), "the fixture resolves\n{resolved:?}");
        root
    }

    /// Scaffold one decision and return its path under the root and its
    /// identifier.
    fn decision(&self, title: &str) -> (PathBuf, String) {
        let before = self.decisions();
        let made = self.run(&["new", "decision", "--title", title]);
        assert_eq!(made.code, Some(0), "the decision scaffolds\n{made:?}");
        let path = self
            .decisions()
            .into_iter()
            .find(|path| !before.contains(path))
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

    fn decisions(&self) -> Vec<PathBuf> {
        match std::fs::read_dir(self.at.join("docs/decisions")) {
            Ok(entries) => entries
                .map(|entry| entry.expect("the entry reads").path())
                .filter(|path| path.extension().is_some_and(|kind| kind == "md"))
                .collect(),
            Err(_) => Vec::new(),
        }
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

/// Put `lines` into the front matter of `path`, before the block closes.
fn into_front_matter(path: &Path, lines: &str) {
    let text = std::fs::read_to_string(path).expect("the document reads");
    let body = text
        .strip_prefix("---\n")
        .expect("the document opens a block");
    let (block, rest) = body.split_once("\n---\n").expect("the block closes");
    std::fs::write(path, format!("---\n{block}\n{lines}---\n{rest}")).expect("the document writes");
}

/// The page is marked and no projection writes it, so generate lists it as
/// orphaned and the rule says nothing about a write to it.
#[test]
fn a_marked_page_no_projection_writes_draws_no_set_twice_finding() {
    let root = Root::new();
    let (page, page_id) = root.decision("A page nobody generates");
    let (superseder, _) = root.decision("A ruling that supersedes the page");
    let (retirer, _) = root.decision("A ruling that retires the page");
    into_front_matter(
        &superseder,
        &format!("relations:\n  supersedes:\n    - {page_id}\n"),
    );
    into_front_matter(
        &retirer,
        &format!("relations:\n  retires:\n    - {page_id}\n"),
    );
    // Hand-placed: the marker names a projection nothing declares.
    let text = std::fs::read_to_string(&page).expect("the page reads");
    std::fs::write(
        &page,
        text.replacen(
            "---\n",
            "---\n\"headwater:generated\": \"stray_pages. Placed by hand.\"\n",
            1,
        ),
    )
    .expect("the page writes");
    let named = format!(
        "docs/decisions/{}",
        page.file_name()
            .expect("the page has a name")
            .to_string_lossy()
    );

    let generated = root.run(&["generate", "--check"]);
    let orphans = generated
        .out
        .split_once("marked, and written by no declaration\n")
        .map(|(_, section)| section.split("\n\n").next().unwrap_or(""))
        .unwrap_or("");
    assert!(
        orphans.lines().any(|line| line.trim() == named),
        "generate lists {named} as orphaned\n{generated:?}"
    );

    let checked = root.run(&["check", "--no-cache"]);
    // The rule ran: the taxonomy has two setters that name two states, so the
    // gate admits its one corpus instance, and a pass is a verdict rather than
    // a rule that never looked.
    assert!(
        checked.out.contains(&format!("1 instances of {RULE}\n")),
        "the rule runs one instance\n{}{}",
        checked.out,
        checked.err
    );
    // A finding prints its location on one line and `<rule>: <message>` on the
    // next. The page is the only document two setters reach, so any finding
    // of this rule is a finding at it.
    let fired: Vec<&str> = checked
        .out
        .lines()
        .filter(|line| line.trim_start().starts_with(&format!("{RULE}: ")))
        .collect();
    assert!(fired.is_empty(), "no {RULE} finding at {named}: {fired:?}");
}
