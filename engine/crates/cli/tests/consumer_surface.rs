// SPDX-License-Identifier: Apache-2.0
//! `surface.local_path.instructed`, held against the manifest it reads.
//!
//! # What this target is evidence of
//!
//! [HW-DR-0077](../../../../docs/decisions/0077-the-consumer-surface-is-what-an-adopter-receives-runs-and-must-have-installed-and-it-is-a-closed-and-declared-list.md)
//! rules that no page for an adopter instructs a file only this repository
//! holds, and that such a page may cite one in a passage that says so.
//! [#933](https://github.com/headwater-ai/headwater/issues/933) refused a rule
//! that read a hard-coded list, because it reports zero by construction. So the
//! decisive case is three documents that hold the same two paths and differ in
//! one thing each:
//!
//! - `x.md` is on the manifest's adopter list and instructs both paths. Both
//!   are reported.
//! - `y.md` is on the list and holds the same paths under a directive that
//!   marks the passage as how this repository does it. Nothing is reported.
//! - `z.md` holds the same paths and is **not** on the list. Nothing is
//!   reported, and that is the case that proves the rule reads the manifest.
//!
//! Every case runs against this repository's own resolved taxonomy, with the
//! adopter list narrowed by an `override` so that the three documents share a
//! shelf, a kind and every other declaration.

use std::path::{Path, PathBuf};
use std::process::Command;

const RULE: &str = "surface.local_path.instructed";

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
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

struct Root {
    at: PathBuf,
}

impl Root {
    /// A scratch corpus with this repository's taxonomy, and an adopter list
    /// of exactly the documents `adopter` names.
    fn new(label: &str, adopter: &[&str]) -> Root {
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-surface-{}-{label}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).expect("the root is made");
        let repository = repository();
        copy(
            &repository.join(headwater_resolve::package::PACKAGES),
            &at.join(headwater_resolve::package::PACKAGES),
        );
        copy(&repository.join("docs/taxonomies"), &at.join("docs/taxonomies"));
        std::fs::copy(
            repository.join(".headwater/taxonomy.yml"),
            at.join(".headwater/taxonomy.yml"),
        )
        .expect("the declaration copies");
        let mut overlay = std::fs::read_to_string(repository.join(".headwater/overlay.yml"))
            .expect("the overlay reads");
        assert!(
            !overlay.lines().any(|line| line == "override:"),
            "the overlay grew an `override` block, so this case must merge into it"
        );
        overlay.push_str(&format!(
            "\noverride:\n  surface.adopter_documents: [{}]\n",
            adopter.join(", ")
        ));
        std::fs::write(at.join(".headwater/overlay.yml"), overlay).expect("the overlay writes");

        let root = Root { at };
        let resolved = root.run(&["taxonomy", "resolve"]);
        assert_eq!(resolved.0, Some(0), "the fixture resolves\n{}{}", resolved.1, resolved.2);
        root
    }

    fn contract(&self, slug: &str, body: &str) {
        let path = self.at.join(format!("docs/interfaces/{slug}.md"));
        std::fs::create_dir_all(path.parent().expect("it has a parent"))
            .expect("the shelf directory is there");
        let mut text = format!(
            "---\n\
             id: HW-IFACE-{slug}\n\
             status: draft\n\
             status_since: 2026-08-16\n\
             summary: \"What this verb takes, what it prints, and what its exit status means.\"\n\
             last_verified: 2026-08-16\n\
             title: \"{slug}\"\n\
             governs:\n  - .githooks/pre-commit\n\
             ---\n\n\
             # {slug}\n"
        );
        for section in [
            "Synopsis",
            "Description",
            "Preconditions",
            "Options",
            "Exit status",
            "Environment",
            "Files",
            "See also",
        ] {
            text.push_str(&format!("\n## {section}\n\nNone.\n"));
        }
        text.push_str(body);
        std::fs::write(&path, text).expect("the contract writes");
    }

    fn run(&self, arguments: &[&str]) -> (Option<i32>, String, String) {
        let output = Command::new(env!("CARGO_BIN_EXE_headwater"))
            .args(arguments)
            .arg("--root")
            .arg(&self.at)
            .output()
            .expect("the binary runs");
        (
            output.status.code(),
            String::from_utf8_lossy(&output.stdout).into_owned(),
            String::from_utf8_lossy(&output.stderr).into_owned(),
        )
    }

    /// The message of each finding of this rule on one document that no escape
    /// hides, read from the JSON report, which prints one member per line.
    fn findings(&self, slug: &str) -> Vec<String> {
        let (_, out, _) = self.run(&["check", "--format", "json"]);
        let path = format!("\"path\": \"docs/interfaces/{slug}.md\",");
        let lines: Vec<&str> = out.lines().map(str::trim).collect();
        let mut messages = Vec::new();
        for (at, line) in lines.iter().enumerate() {
            if *line != format!("\"rule\": \"{RULE}\",") {
                continue;
            }
            // One finding runs to the next `rule` member. A finding the runner
            // filtered is still in the report, with its escape named, and only
            // an unescaped one is a verdict a reader meets.
            let end = lines[at + 1..]
                .iter()
                .position(|l| l.starts_with("\"rule\""))
                .map_or(lines.len(), |next| at + 1 + next);
            let finding = &lines[at..end];
            if finding.contains(&path.as_str())
                && finding.iter().any(|l| l.starts_with("\"escape\": \"none\""))
            {
                if let Some(message) = finding.iter().find(|l| l.starts_with("\"message\"")) {
                    messages.push(message.to_string());
                }
            }
        }
        messages
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.at);
    }
}

const INSTRUCTS: &str = "\nRun the gate before you commit.\n\n```sh\nsh tools/foo.sh\n```\n";

const SAYS_SO: &str = "\n<!-- headwater allow=surface.local_path.instructed scope=block until=2099-12-31 reason=accepted_deviation note=how this repository does it -->\nThis repository runs `tools/foo.sh` before it commits.\n\n<!-- headwater allow=surface.local_path.instructed scope=file until=2099-12-31 reason=accepted_deviation note=the governs edge names how this repository does it -->\n";

#[test]
fn the_rule_reads_the_manifest_and_honors_a_passage_that_says_so() {
    let root = Root::new(
        "three",
        &["docs/interfaces/x.md", "docs/interfaces/y.md"],
    );
    root.contract("x", INSTRUCTS);
    root.contract("y", SAYS_SO);
    root.contract("z", INSTRUCTS);

    let (_, out, err) = root.run(&["check"]);
    let x = root.findings("x");
    assert_eq!(
        x.len(),
        2,
        "the adopter page names `.githooks/pre-commit` in `governs` and `tools/foo.sh` in a \
         code block, and each is one finding\n{x:#?}\n{out}{err}"
    );
    assert!(x.iter().any(|f| f.contains("tools/foo.sh")), "{x:#?}");
    assert!(x.iter().any(|f| f.contains(".githooks/pre-commit")), "{x:#?}");
    assert_eq!(
        root.findings("y"),
        Vec::<String>::new(),
        "a passage marked as how this repository does it passes"
    );
    assert_eq!(
        root.findings("z"),
        Vec::<String>::new(),
        "a page the manifest does not list is not an adopter's page, and the rule reads the \
         manifest rather than a list of its own"
    );
}
