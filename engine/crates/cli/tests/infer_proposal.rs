// SPDX-License-Identifier: Apache-2.0
//! The overlay fragment `headwater infer` proposes from a tree, held to the
//! one test that matters: put in the overlay unchanged, it resolves and it
//! types the documents it read.
//!
//! # The defect this target exists for
//!
//! Spec 7 and HW-DR-0012 say one read of a tree gives three artifacts: an
//! overlay, a report of what no proposed shelf or kind explains, and the
//! adoption payload. Until #2019 the verb gave the payload alone, and an
//! outside trial typed 0 of 76 documents on its first run. A fragment that
//! prints and does not resolve, or resolves and types nothing, is that same 0
//! with more text in front of it, so the decisive case below resolves the
//! fragment and asks the engine what each document became.
//!
//! The setup copies the shape of `tests/init.rs` rather than importing it,
//! because one test target cannot name another's helpers.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

/// A scratch repository, removed when the case ends.
///
/// `label` names the case rather than the process, because cargo runs the
/// cases of one target as threads of one process.
struct Root {
    at: PathBuf,
    /// The overlay `init` wrote, which every `overlay` call starts from.
    written: String,
}

impl Drop for Root {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.at);
    }
}

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("the directory is there");
    for entry in std::fs::read_dir(from).expect("the package reads") {
        let entry = entry.expect("the entry reads");
        let target = to.join(entry.file_name());
        match entry.path().is_dir() {
            true => copy_tree(&entry.path(), &target),
            false => {
                std::fs::copy(entry.path(), &target).expect("the file copies");
            }
        }
    }
}

impl Root {
    /// A tree with the shipped package vendored, so that `init` pins a version
    /// that `taxonomy resolve` can read, and the files `files` names.
    fn over(label: &str, files: &[(&str, &str)]) -> Root {
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-infer-proposal-{}-{label}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&at);
        for (path, text) in files {
            let path = at.join(path);
            std::fs::create_dir_all(path.parent().expect("a file has a parent"))
                .expect("the directory is there");
            std::fs::write(path, text).expect("the document writes");
        }
        copy_tree(
            &repository().join(".headwater/packages/headwater-standard"),
            &at.join(".headwater/packages/headwater-standard"),
        );
        let mut root = Root {
            at,
            written: String::new(),
        };
        let init = root.run(&["init"]);
        assert_eq!(init.status.code(), Some(0), "init: {}", text(&init.stderr));
        root.written = std::fs::read_to_string(root.at.join(".headwater/overlay.yml"))
            .expect("init wrote the overlay");
        root
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_headwater"))
            .args(args)
            .arg("--root")
            .arg(&self.at)
            .output()
            .expect("the binary runs")
    }

    /// Replace the `add: {}` that `init` writes with `add:` and `lines`. The
    /// package leaves the namespace of `decision_id` to the corpus, so a
    /// resolvable overlay states one before anything else.
    fn overlay(&self, lines: &str) {
        let path = self.at.join(".headwater/overlay.yml");
        let head = self
            .written
            .strip_suffix("add: {}\n")
            .expect("init ends the overlay with `add: {}`");
        let body = format!("{head}add:\n  identifier_schemes.decision_id.namespace: ACME\n{lines}");
        std::fs::write(&path, body).expect("the overlay writes");
    }

    fn resolve(&self) {
        let resolve = self.run(&["taxonomy", "resolve"]);
        assert_eq!(
            resolve.status.code(),
            Some(0),
            "`headwater taxonomy resolve` accepts the overlay:\n{}\n{}",
            text(&resolve.stdout),
            text(&resolve.stderr)
        );
    }

    fn infer(&self) -> String {
        let infer = self.run(&["infer", "--now", "2026-01-15"]);
        assert_eq!(
            infer.status.code(),
            Some(0),
            "infer: {}",
            text(&infer.stderr)
        );
        text(&infer.stdout)
    }
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// The fragment, cut out of the report as a reader would cut it: the lines
/// between the line that introduces it and the first blank line after it.
fn fragment(report: &str) -> String {
    let mut lines = report
        .lines()
        .skip_while(|line| !line.contains("under the `add:` key"));
    lines.next().expect("the report introduces a fragment");
    lines
        .take_while(|line| !line.trim().is_empty())
        .map(|line| format!("{line}\n"))
        .collect()
}

/// The report section that names what no proposed shelf or kind explains.
fn leftovers(report: &str) -> Vec<String> {
    report
        .lines()
        .skip_while(|line| !line.contains("no proposed shelf or kind explains"))
        .skip(1)
        .take_while(|line| line.starts_with("    "))
        .map(|line| line.trim().to_owned())
        .collect()
}

const RUNBOOK: &str = "---\ntitle: Restart the API\nowner: ops\nservice: api\n---\n\n# Restart the API\n\n## Steps\n\nRestart it.\n\n## Rollback\n\nStart the old build.\n";
const RUNBOOK_NO_SERVICE: &str = "---\ntitle: Rotate a key\nowner: ops\n---\n\n# Rotate a key\n\n## Steps\n\nRotate it.\n\n## Rollback\n\nRestore the old key.\n";

#[test]
fn the_fragment_infer_proposes_types_the_documents_it_read() {
    let root = Root::over(
        "types",
        &[
            ("docs/runbooks/a.md", RUNBOOK),
            ("docs/runbooks/b.md", RUNBOOK),
            ("docs/runbooks/c.md", RUNBOOK_NO_SERVICE),
            (
                "docs/notes/lone.md",
                "---\ntitle: A lone note\n---\n\n# A lone note\n",
            ),
            ("docs/plain/x.md", "# X\n"),
            ("docs/plain/y.md", "# Y\n"),
            // An empty block is no front matter either, and a runbook that
            // has none stays left over when the runbooks shelf claims it.
            ("docs/plain/z.md", "---\n---\n\n# Z\n"),
            ("docs/runbooks/draft.md", "# A draft\n"),
        ],
    );
    root.overlay("");
    root.resolve();

    // Before: the bound taxonomy types none of the three runbooks.
    let before = root.infer();
    for runbook in ["a", "b", "c"] {
        let explained = text(
            &root
                .run(&["explain", &format!("docs/runbooks/{runbook}.md")])
                .stdout,
        );
        assert!(
            !explained.contains("\n  kind "),
            "the base types no runbook:\n{explained}"
        );
    }

    let proposed = fragment(&before);
    // The covered count is the documents the fragment's shelves claim: the
    // three runbooks with front matter, and not the draft without it, the
    // lone note or the plain files.
    assert!(
        before.contains("this fragment proposes a shelf and a kind for 3 of them."),
        "the fragment covers the 3 runbooks:\n{before}"
    );
    assert!(
        proposed.contains("  shelves.runbooks:\n"),
        "a shelf for the runbooks:\n{before}"
    );
    assert!(
        proposed.contains("  kinds.runbooks:\n"),
        "a kind for the runbooks:\n{before}"
    );
    assert!(
        !proposed.contains("notes"),
        "a lone file is no evidence of a kind:\n{proposed}"
    );
    assert!(
        !proposed.contains("plain"),
        "a file with no front matter is none:\n{proposed}"
    );
    assert!(
        proposed.contains("  # docs/runbooks: 3 documents"),
        "the evidence:\n{proposed}"
    );

    // The fragment goes in unchanged, and resolves.
    root.overlay(&proposed);
    root.resolve();

    for runbook in ["a", "b", "c"] {
        let explained = text(
            &root
                .run(&["explain", &format!("docs/runbooks/{runbook}.md")])
                .stdout,
        );
        assert!(
            explained.contains("\n  kind runbooks\n"),
            "{runbook} is a runbook:\n{explained}"
        );
        let required = explained
            .lines()
            .find(|line| line.contains("requires the facets"))
            .expect("the kind requires facets");
        assert!(
            required.contains("owner"),
            "every runbook writes `owner`: {required}"
        );
        assert!(
            !required.contains("service"),
            "`c` writes no `service`: {required}"
        );
        assert!(
            explained.contains("requires the sections Steps, Rollback"),
            "the headings every runbook writes:\n{explained}"
        );
    }

    // What the check still finds is the debt of the base kind and nothing the
    // fragment caused: each runbook lacks the four facets `governed_document`
    // requires and the identifier the base kind has no scheme for. A missing
    // section, or a required key the fragment named wrong, is a finding more.
    let check = text(&root.run(&["check", "--format", "json"]).stdout);
    let findings = check
        .split("\"findings\": [")
        .nth(1)
        .expect("the report lists findings");
    let mut rules: Vec<&str> = findings
        .lines()
        .filter_map(|line| line.trim().strip_prefix("\"rule\": \""))
        .map(|rule| rule.trim_end_matches(['"', ',']))
        .collect();
    rules.sort_unstable();
    rules.dedup_by(|a, b| a == b);
    assert_eq!(
        rules,
        ["facet.required.missing", "identifier.unusable"],
        "{findings}"
    );
    assert_eq!(
        findings.matches("\"facet.required.missing\"").count(),
        12,
        "{findings}"
    );

    let after = root.infer();
    let left = leftovers(&after);
    assert_eq!(
        left,
        [
            "docs/notes/lone.md  alone in its directory",
            "docs/plain/x.md  no front matter",
            "docs/plain/y.md  no front matter",
            "docs/plain/z.md  no front matter",
            "docs/runbooks/draft.md  no front matter",
        ],
        "{after}"
    );
    assert_eq!(
        left,
        leftovers(&before),
        "the report before the fragment names the same files"
    );
}

/// The rules of the findings a check reports, each once, sorted.
fn rules_found(check: &str) -> Vec<String> {
    let findings = check
        .split("\"findings\": [")
        .nth(1)
        .expect("the report lists findings");
    let mut rules: Vec<String> = findings
        .lines()
        .filter_map(|line| line.trim().strip_prefix("\"rule\": \""))
        .map(|rule| rule.trim_end_matches(['"', ',']).to_owned())
        .collect();
    rules.sort_unstable();
    rules.dedup();
    rules
}

/// The slug rule's three steps, and the keys a fragment must not name, on one
/// tree. Two directories end in `notes`, so each takes its whole path. The
/// base package declares `decisions`, so a `decisions` directory outside its
/// shelf takes its whole path too. Every document writes `relations`, which
/// holds edges, and a key with a dot, which no dotted overlay path can name,
/// and a heading that YAML would read as a mapping if it went unquoted.
#[test]
fn the_fragment_names_each_directory_once_and_resolves_where_names_collide() {
    const NOTE: &str = "---\ntitle: A note\nrelations:\n  traces_to: [ACME-1]\n\"odd.key\": 1\n\"bad\\nkey\": 1\ntags: [x, y]\n---\n\n## Why: \"this\" heading\n\nBecause.\n";
    const OLD: &str = "---\ntitle: An old decision\n---\n\n## Context\n\nThen.\n";
    const SETEXT: &str = "---\ntitle: Two lines\n---\n\nWhy this\nheading\n===\n\nText.\n";
    let root = Root::over(
        "collide",
        &[
            ("docs/a/notes/1.md", NOTE),
            ("docs/a/notes/2.md", NOTE),
            ("docs/b/notes/1.md", NOTE),
            ("docs/b/notes/2.md", NOTE),
            ("docs/old/decisions/1.md", OLD),
            ("docs/old/decisions/2.md", OLD),
            ("docs/My-Guides/1.md", OLD),
            ("docs/My-Guides/2.md", OLD),
            ("docs/r1.md", OLD),
            ("docs/r2.md", OLD),
            ("docs/lone/x.md", "---\ntitle: Lone\n---\n\n# Lone\n"),
            ("docs/setext/1.md", SETEXT),
            ("docs/setext/2.md", SETEXT),
        ],
    );
    root.overlay("");
    root.resolve();
    let report = root.infer();
    // Six directories of two documents each, and the lone file is not one.
    assert!(
        report.contains("this fragment proposes a shelf and a kind for 12 of them."),
        "the fragment covers 12 documents in 6 directories:\n{report}"
    );
    let proposed = fragment(&report);
    for name in [
        "docs_a_notes",
        "docs_b_notes",
        "docs_old_decisions",
        "my_guides",
        "setext",
        "docs",
    ] {
        assert!(
            proposed.contains(&format!("  kinds.{name}:\n")),
            "{name}:\n{proposed}"
        );
        assert!(
            proposed.contains(&format!("  shelves.{name}:\n")),
            "{name}:\n{proposed}"
        );
    }
    assert_eq!(
        proposed.matches("  facets.tags:").count(),
        1,
        "one declaration:\n{proposed}"
    );
    assert!(!proposed.contains("facets.relations"), "{proposed}");
    assert!(
        !proposed.contains("require: [title, relations"),
        "{proposed}"
    );
    assert!(!proposed.contains("facets.odd"), "{proposed}");
    assert!(
        proposed.contains("It leaves out relations, odd.key"),
        "{proposed}"
    );

    root.overlay(&proposed);
    root.resolve();
    for (path, kind) in [
        ("docs/a/notes/1.md", "docs_a_notes"),
        ("docs/b/notes/2.md", "docs_b_notes"),
        ("docs/old/decisions/1.md", "docs_old_decisions"),
        ("docs/My-Guides/2.md", "my_guides"),
    ] {
        let explained = text(&root.run(&["explain", path]).stdout);
        assert!(
            explained.contains(&format!("\n  kind {kind}\n")),
            "{path}:\n{explained}"
        );
    }
    let check = text(&root.run(&["check", "--format", "json"]).stdout);
    assert_eq!(
        rules_found(&check),
        ["facet.required.missing", "identifier.unusable"],
        "{check}"
    );
    // The corpus root's own shelf claims its own files and no subdirectory, so
    // the lone file below it stays left over rather than typed and failing
    // the root kind's section contract.
    assert!(proposed.contains("    path: \"docs/*.md\"\n"), "{proposed}");
    let after = root.infer();
    assert_eq!(
        leftovers(&after),
        ["docs/lone/x.md  alone in its directory"],
        "{after}"
    );
}

#[test]
fn infer_proposes_nothing_where_every_file_is_shelved() {
    let root = Root::over(
        "shelved",
        &[(
            "docs/decisions/0001-one.md",
            "---\nstatus: current\nstatus_since: 2026-01-01\nlast_verified: 2026-01-01\nsummary: One choice.\n---\n\n# One\n\n## Context\n\nA.\n\n## Decision\n\nB.\n\n## Consequences\n\nC.\n",
        )],
    );
    root.overlay("");
    root.resolve();
    let report = root.infer();
    assert!(
        report.contains("every file the census walked classified, so there is nothing to propose"),
        "{report}"
    );
    assert!(!report.contains("under the `add:` key"), "{report}");
}

/// A document a declared shelf reaches but cannot type is left over with the
/// reason resolution stopped on, and no fragment claims it, although each one
/// carries front matter and the three share a directory. Each of the three
/// discriminator stops is here: no value, a value the shelf does not admit,
/// and a sequence. The other two stops on a shelf, a tie and an abstract kind,
/// are refused by `taxonomy resolve`, so `propose.rs` holds them in-process.
#[test]
fn a_shelved_file_the_census_cannot_type_is_left_with_its_own_reason() {
    const DOC: &str = "---\ntitle: A page\nowner: ops\n---\n\n# A page\n";
    let root = Root::over(
        "shelved-untyped",
        &[
            ("docs/mixed/none.md", DOC),
            (
                "docs/mixed/other.md",
                "---\ntitle: A page\ntier: nonsense\n---\n\n# A page\n",
            ),
            (
                "docs/mixed/seq.md",
                "---\ntitle: A page\ntier: [decision]\n---\n\n# A page\n",
            ),
        ],
    );
    root.overlay(concat!(
        "  facets.tier: {volatility: mutable}\n",
        "  shelves.mixed: {path: \"docs/mixed/*.md\", homogeneous: false, discriminator: tier, kinds: [decision]}\n",
    ));
    root.resolve();
    let report = root.infer();
    assert!(!report.contains("under the `add:` key"), "{report}");
    assert_eq!(
        leftovers(&report),
        [
            "docs/mixed/none.md  discriminator",
            "docs/mixed/other.md  discriminator",
            "docs/mixed/seq.md  discriminator",
        ],
        "{report}"
    );
}
