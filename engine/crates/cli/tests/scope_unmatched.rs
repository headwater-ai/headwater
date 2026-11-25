// SPDX-License-Identifier: Apache-2.0
//! What `check --strict` makes of a governed scope pattern that matches no
//! entry of the tree (#2099).
//!
//! # The defect this target exists for
//!
//! `taxonomy validate` refuses a governed scope pattern that matches no entry
//! of the tree beside the taxonomy (#951), and `headwater check` read the same
//! scope and dropped the reason. So a taxonomy that `validate` refused could
//! pass `check --strict` at exit 0. An adopter wrote `mod-*/src/main/**`, which
//! opens with a wildcard and is refused before any walk, and `check --strict`
//! passed while the pattern admitted no file at all.
//!
//! The rule `relation.scope.unmatched` reports each such pattern as an error,
//! through the one function both verbs call
//! (`headwater_graph::scope::Scope::coverage`). Each case here drives the built
//! binary, because the claim is about an exit status and a report, and both
//! are the caller's.
//!
//! # The root each case runs over
//!
//! This repository's vendored package, its overlay and its consumer
//! declaration, copied with `docs/taxonomies` into a root that removes itself,
//! and one stub entry under each scope pattern the overlay declares. Over that
//! root `check --strict` exits 0, so a case can read the exit status as the
//! verdict of this rule alone. `common::Root` is not used: it copies the
//! package source with no pin and a docs tree that fails strict on its own,
//! for reasons this target is not about.

mod common;

use std::path::PathBuf;
use std::process::Command;

const RULE: &str = "relation.scope.unmatched";
const WILDCARD: &str = "mod-*/src/main/**";

/// A repository root that removes itself.
struct Root {
    at: PathBuf,
}

impl Root {
    /// The taxonomy, `docs/taxonomies`, and no entry under any scope pattern.
    fn copy_only(label: &str) -> Root {
        let at = common::scratch_base().join(format!(
            "headwater-cli-scope-unmatched-{}-{label}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).expect("the root is made");
        let repository = common::repository();
        common::copy(
            &repository.join(".headwater/packages/headwater-standard"),
            &at.join(".headwater/packages/headwater-standard"),
        );
        common::copy(
            &repository.join("docs/taxonomies"),
            &at.join("docs/taxonomies"),
        );
        for name in ["taxonomy.yml", "overlay.yml"] {
            std::fs::copy(
                repository.join(".headwater").join(name),
                at.join(".headwater").join(name),
            )
            .expect("the declaration copies");
        }
        Root { at }
    }

    /// The copy with one entry under each scope pattern and each path the
    /// house regime lists outside the corpus root.
    fn new(label: &str) -> Root {
        let root = Root::copy_only(label);
        for entry in [
            "engine/crates/stub/src/lib.rs",
            "engine/crates/cli/harness/stub",
            ".githooks/stub",
            ".claude/hooks/stub",
            ".claude/agents/stub",
            ".claude/skills/stub",
            "tools/stub",
            "site/stub",
        ] {
            root.write(entry, "");
        }
        root.with_outside_root_stubs()
    }

    /// The four files `ste_house` lists outside the corpus root
    /// (HW-DR-0084). `check` names a pattern there that matches no file, on
    /// its own rule, so each gets a stub that obeys the regime. Each is a file
    /// at the top or under a dot-directory, so none of them is a tree.
    fn with_outside_root_stubs(self) -> Root {
        for entry in [
            "README.md",
            ".github/CONTRIBUTING.md",
            ".github/SECURITY.md",
            ".github/ISSUE_TEMPLATE/issue.md",
        ] {
            self.write(entry, "# A stub\n");
        }
        self
    }

    fn write(&self, entry: &str, text: &str) {
        let to = self.at.join(entry);
        std::fs::create_dir_all(to.parent().expect("it has a parent"))
            .expect("the directory is there");
        std::fs::write(to, text).expect("the entry writes");
    }

    /// Add one governed scope pattern after `site/**` in the overlay.
    fn add_scope_pattern(&self, pattern: &str) {
        let overlay = self.at.join(".headwater/overlay.yml");
        let text = std::fs::read_to_string(&overlay).expect("the overlay reads");
        let anchor = "    - site/**\n";
        assert!(text.contains(anchor), "the overlay still declares site/**");
        std::fs::write(
            &overlay,
            text.replacen(anchor, &format!("{anchor}    - {pattern}\n"), 1),
        )
        .expect("the overlay writes");
    }

    fn run(&self, arguments: &[&str]) -> Ran {
        let mut command = Command::new(env!("CARGO_BIN_EXE_headwater"));
        common::fence(&mut command, &self.at);
        let output = command
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

    /// Resolve the lock from the sources as they are now, which every case
    /// does before it checks.
    fn resolve(&self) {
        let ran = self.run(&["taxonomy", "resolve"]);
        assert_eq!(ran.code, Some(0), "the root resolves: {ran:?}");
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

/// The output with each run of whitespace folded to one space, so a finding
/// reads as one sentence wherever the report wrapped it.
fn flat(out: &str) -> String {
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The control: over the root with every pattern matched, `check --strict`
/// exits 0 and the rule is silent. Without it, an exit 1 in the cases below
/// could be any rule's.
#[test]
fn check_strict_passes_where_every_governed_scope_pattern_matches() {
    let root = Root::new("control");
    root.resolve();
    let ran = root.run(&["check", "--strict"]);
    assert_eq!(ran.code, Some(0), "{ran:?}");
    assert!(!ran.out.contains(RULE), "{ran:?}");
    assert!(!ran.out.contains("governed scope:"), "{ran:?}");
}

/// The adopter's case. A pattern that opens on a wildcard is refused before
/// any walk, and `check --strict` exits 1 on it, naming the pattern and the
/// rule and no other pattern. `taxonomy validate` refuses the same pattern
/// over the same root, so one case holds both verbs to one answer.
#[test]
fn check_strict_refuses_a_governed_scope_pattern_that_opens_on_a_wildcard() {
    let root = Root::new("wildcard");
    root.add_scope_pattern(WILDCARD);
    root.resolve();

    let ran = root.run(&["check", "--strict"]);
    assert_eq!(ran.code, Some(1), "{ran:?}");
    let report = flat(&ran.out);
    assert!(report.contains(RULE), "{ran:?}");
    assert!(
        report.contains(&format!(
            "governed scope pattern `{WILDCARD}` matches no entry of the tree: `{WILDCARD}` \
             opens with a wildcard"
        )),
        "{ran:?}"
    );
    assert!(!report.contains("`site/**`"), "{ran:?}");
    assert_eq!(
        ran.out.matches(&format!("{RULE} (")).count(),
        1,
        "one finding, on the one pattern: {ran:?}"
    );

    let validated = root.run(&["taxonomy", "validate"]);
    assert_eq!(validated.code, Some(1), "{validated:?}");
    assert!(
        validated.err.contains(&format!(
            "governed scope pattern `{WILDCARD}` matches no entry of the tree: `{WILDCARD}` \
             opens with a wildcard"
        )),
        "{validated:?}"
    );
}

/// A misspelled pattern with a literal root is reported too, so the rule is
/// not about wildcards: it reports whatever the resolver refuses.
#[test]
fn check_strict_refuses_a_governed_scope_pattern_whose_literal_root_is_missing() {
    let root = Root::new("literal");
    root.add_scope_pattern("nowhere/**");
    root.resolve();
    let ran = root.run(&["check", "--strict"]);
    assert_eq!(ran.code, Some(1), "{ran:?}");
    let report = flat(&ran.out);
    assert!(report.contains(RULE), "{ran:?}");
    assert!(
        report.contains("governed scope pattern `nowhere/**` matches no entry of the tree"),
        "{ran:?}"
    );
}

/// A taxonomy with no tree beside it, as one published as its own repository
/// has, is not refused for its scope (owner ruling 2026-09-25, #951 and
/// #1103). `check --strict` exits 0, reports no finding of this rule, and
/// prints the one-line notice `taxonomy validate` prints.
///
/// The root keeps the four files the house regime lists outside the corpus
/// root. Each is a file at the top or under `.github/`, so none of them is a
/// tree, and without them `language.outside_root.refused` fails the run for a
/// reason this case is not about.
#[test]
fn check_skips_the_governed_scope_with_a_notice_where_no_tree_is_beside_the_taxonomy() {
    let root = Root::copy_only("no-tree").with_outside_root_stubs();
    std::fs::remove_dir_all(root.at.join("docs")).expect("the copied docs go");
    root.resolve();
    let ran = root.run(&["check", "--strict"]);
    assert_eq!(ran.code, Some(0), "{ran:?}");
    assert!(!ran.out.contains(RULE), "{ran:?}");
    let notices: Vec<&str> = ran
        .out
        .lines()
        .filter(|line| line.contains("governed scope"))
        .collect();
    assert_eq!(notices.len(), 1, "{ran:?}");
    assert!(notices[0].contains("no tree"), "{notices:?}");
    assert!(notices[0].contains("skipped"), "{notices:?}");

    // The same notice `taxonomy validate` prints, word for word.
    let validated = root.run(&["taxonomy", "validate"]);
    assert_eq!(validated.code, Some(0), "{validated:?}");
    let said = validated
        .out
        .lines()
        .find(|line| line.contains("governed scope"))
        .expect("validate prints the notice");
    assert_eq!(notices[0].trim(), said.trim(), "{ran:?}");

    // JSON carries no notice and no finding of the rule.
    let json = root.run(&["check", "--strict", "--format", "json"]);
    assert_eq!(json.code, Some(0), "{json:?}");
    assert!(!json.out.contains(RULE), "{json:?}");
}
