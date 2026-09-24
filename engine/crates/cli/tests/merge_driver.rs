// SPDX-License-Identifier: Apache-2.0
//! The merge driver as a verb of the binary, run by git over a tree that holds
//! no file of this repository.
//!
//! # What this target holds, and why it is not a case of `census`
//!
//! `engine/crates/census/tests/merge_driver.rs` holds `.githooks/merge-regenerate`,
//! which is a script of this repository and which no adopter receives.
//! [HW-DR-0077](../../../../docs/decisions/0077-the-consumer-surface-is-what-an-adopter-receives-runs-and-must-have-installed-and-it-is-a-closed-and-declared-list.md)
//! rules that merge safety ships as a verb and as configuration `headwater init`
//! writes ([#974](https://github.com/headwater-ai/headwater/issues/974)). So every
//! case here builds an adopter's tree out of the vendored package alone, and the
//! only executable git calls is the binary under test.
//!
//! # The decisive pair
//!
//! Two branches each move the taxonomy lock, and so the corpus descriptor that
//! records the lock digest, to different values. With the git step `init`
//! writes, the merge stops with both folds conflicted, no marker in either,
//! the current side's bytes in place and the producer named on standard error.
//! Without it, the same merge writes conflict markers into both folds. The
//! second arm is what makes the first one a measurement: a guard with no failing
//! arm beside it passes on a tree where it guards nothing.
//!
//! # What this does not reach
//!
//! Two branches that write one byte string to a fold are resolved by git at the
//! tree level, and no driver is called. The census target measures that, and the
//! contract of `headwater merge-driver` states the gap.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

fn binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_headwater"))
}

const LOCK: &str = ".headwater/taxonomy.lock";
const DESCRIPTOR: &str = ".headwater/corpus.json";

/// A scratch git repository that adopted `headwater/standard` by copying the
/// vendored package, and nothing else of this repository.
struct Tree {
    at: PathBuf,
}

impl Tree {
    /// `label` names the case, because cargo runs the cases of one target as
    /// threads of one process and a directory keyed on the pid alone races.
    fn adopted(label: &str) -> Tree {
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-merge-driver-{}-{label}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(at.join("docs")).expect("the corpus directory is made");
        std::fs::write(at.join("docs/one.md"), "# a document\n").expect("the document writes");
        copy_dir(
            &repository().join(".headwater/packages/headwater-standard"),
            &at.join(".headwater/packages/headwater-standard"),
        );
        let tree = Tree { at };
        tree.git(&["init", "-q", "-b", "main"]);
        tree.git(&["config", "user.email", "adopter@example.com"]);
        tree.git(&["config", "user.name", "An adopter"]);
        tree.headwater_ok(&["init"]);
        // The one answer `taxonomy resolve` refuses without: a namespace for
        // the decision identifier. It is the overlay's `INTERVIEW 2` question.
        let overlay = tree.read(".headwater/overlay.yml");
        assert!(
            overlay.ends_with("add: {}\n"),
            "the overlay `init` writes ends with an empty `add` block:\n{overlay}"
        );
        tree.write(
            ".headwater/overlay.yml",
            &overlay.replace(
                "add: {}\n",
                "add:\n  identifier_schemes.decision_id.namespace: ACME\n",
            ),
        );
        tree.headwater_ok(&["taxonomy", "resolve"]);
        tree.headwater_ok(&["generate"]);
        tree
    }

    fn read(&self, relative: &str) -> String {
        std::fs::read_to_string(self.at.join(relative))
            .unwrap_or_else(|error| panic!("{relative} reads: {error}"))
    }

    fn write(&self, relative: &str, body: &str) {
        std::fs::write(self.at.join(relative), body).expect("the file writes");
    }

    /// Git, isolated from the configuration of the machine running the suite,
    /// with the binary under test first on the path so that a driver line that
    /// names `headwater` reaches it.
    fn git_output(&self, args: &[&str]) -> Output {
        let bin = binary();
        let dir = bin.parent().expect("the binary has a directory");
        let path = std::env::var_os("PATH").unwrap_or_default();
        let mut paths = vec![dir.to_path_buf()];
        paths.extend(std::env::split_paths(&path));
        Command::new("git")
            .args(args)
            .current_dir(&self.at)
            .env("PATH", std::env::join_paths(paths).expect("the path joins"))
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env_remove("GIT_DIR")
            .env_remove("GIT_INDEX_FILE")
            .env_remove("GIT_WORK_TREE")
            .output()
            .expect("git runs")
    }

    fn git(&self, args: &[&str]) -> String {
        let output = self.git_output(args);
        assert!(
            output.status.success(),
            "`git {}` succeeds:\n{}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).into_owned()
    }

    fn headwater(&self, args: &[&str]) -> Output {
        Command::new(binary())
            .args(args)
            .arg("--root")
            .arg(&self.at)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env_remove("GIT_DIR")
            .env_remove("GIT_INDEX_FILE")
            .env_remove("GIT_WORK_TREE")
            .output()
            .expect("the binary runs")
    }

    fn headwater_ok(&self, args: &[&str]) -> Output {
        let output = self.headwater(args);
        assert_eq!(
            output.status.code(),
            Some(0),
            "`headwater {}` exits 0:\n{}{}",
            args.join(" "),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        output
    }

    /// Two branches that each move the lock, and so the descriptor, to a
    /// different value, and the merge of one into the other.
    ///
    /// Branch `a` adds a scheme in the overlay and branch `b` selects a bundle
    /// in the declaration. The two edits are in two files, so git merges both
    /// sources cleanly and only the two folds are in question.
    fn diverge_and_merge(&self) -> Output {
        self.git(&["checkout", "-q", "-b", "a"]);
        let overlay = self.read(".headwater/overlay.yml");
        self.write(
            ".headwater/overlay.yml",
            &format!(
                "{overlay}  identifier_schemes.doc_id: {{pattern: \"{{namespace}}-DOC-{{slug}}\", namespace: ACME, allocation: minted-once}}\n"
            ),
        );
        self.headwater_ok(&["taxonomy", "resolve"]);
        self.headwater_ok(&["generate"]);
        self.git(&["commit", "-q", "-am", "a scheme"]);

        self.git(&["checkout", "-q", "main"]);
        self.git(&["checkout", "-q", "-b", "b"]);
        let declaration = self.read(".headwater/taxonomy.yml");
        assert!(
            declaration.contains("  bundles: []\n"),
            "the declaration selects no bundle:\n{declaration}"
        );
        self.write(
            ".headwater/taxonomy.yml",
            &declaration.replace("  bundles: []\n", "  bundles: [diataxis]\n"),
        );
        self.headwater_ok(&["taxonomy", "resolve"]);
        self.headwater_ok(&["generate"]);
        self.git(&["commit", "-q", "-am", "a bundle"]);

        self.git_output(&["merge", "--no-edit", "a"])
    }

    /// A decision record of the standard package, which is one row of the
    /// `decisions` shelf index that `headwater generate` writes.
    fn decide(&self, seq: &str) {
        std::fs::create_dir_all(self.at.join("docs/decisions")).expect("the shelf is made");
        self.write(
            &format!("docs/decisions/{seq}-d{seq}.md"),
            &format!(
                "---\nid: ACME-DR-{seq}\ntitle: Decision {seq}\nstatus: draft\nsummary: The decision numbered {seq}.\n---\n\n# Decision {seq}\n"
            ),
        );
    }

    /// Where git reads this clone's own attributes, which win over every
    /// `.gitattributes`. Asked of git, because a linked worktree's `.git` is a
    /// file and its `info/` lives in the common directory.
    fn info_attributes(&self) -> PathBuf {
        let path = self.git(&["rev-parse", "--git-path", "info/attributes"]);
        self.at.join(path.trim_end())
    }

    fn unmerged(&self) -> Vec<String> {
        let listed = self.git(&["diff", "--name-only", "--diff-filter=U"]);
        listed.lines().map(str::to_string).collect()
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.at);
    }
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("the directory is made");
    for entry in std::fs::read_dir(from).expect("the directory reads") {
        let entry = entry.expect("the entry reads");
        let target = to.join(entry.file_name());
        match entry.file_type().expect("the type reads").is_dir() {
            true => copy_dir(&entry.path(), &target),
            false => {
                std::fs::copy(entry.path(), &target).expect("the file copies");
            }
        }
    }
}

/// The decisive case: `init --git --git-config`, then a merge that moves both
/// folds, then the producers, then a clean `--check`.
#[test]
fn a_merge_that_moves_a_fold_stops_with_the_current_side_and_names_the_producer() {
    let tree = Tree::adopted("driver");
    tree.headwater_ok(&["init", "--git", "--git-config"]);
    let override_ = std::fs::read_to_string(tree.info_attributes())
        .expect("`--git-config` writes the clone's own attributes file");
    for path in [LOCK, DESCRIPTOR] {
        assert!(
            override_.contains(&format!("{path} merge=headwater-regenerate\n")),
            "the override names the driver for {path}:\n{override_}"
        );
    }
    tree.git(&["add", "-A"]);
    tree.git(&["commit", "-q", "-m", "adopt headwater"]);

    let merge = tree.diverge_and_merge();
    let stderr = String::from_utf8_lossy(&merge.stderr).into_owned();
    let stdout = String::from_utf8_lossy(&merge.stdout).into_owned();
    assert!(
        !merge.status.success(),
        "a merge that moves a fold to two values stops:\n{stdout}{stderr}"
    );
    let unmerged = tree.unmerged();
    for path in [LOCK, DESCRIPTOR] {
        assert!(
            unmerged.iter().any(|listed| listed == path),
            "{path} is left conflicted, and the conflicted paths are {unmerged:?}"
        );
        let merged = tree.read(path);
        assert!(
            !merged.contains("<<<<<<<"),
            "{path} carries no conflict marker:\n{merged}"
        );
        let current = tree.git(&["show", &format!("HEAD:{path}")]);
        assert_eq!(merged, current, "{path} holds the current side's bytes");
    }
    assert!(
        stderr.contains(DESCRIPTOR) && stderr.contains("headwater generate"),
        "the driver names the descriptor and `headwater generate`:\n{stderr}"
    );
    assert!(
        stderr.contains(LOCK) && stderr.contains("headwater taxonomy resolve"),
        "the driver names the lock and `headwater taxonomy resolve`:\n{stderr}"
    );

    tree.headwater_ok(&["taxonomy", "resolve"]);
    tree.headwater_ok(&["generate"]);
    tree.git(&["add", LOCK, DESCRIPTOR]);
    tree.git(&["commit", "-q", "--no-edit"]);
    tree.headwater_ok(&["taxonomy", "resolve", "--check"]);
    tree.headwater_ok(&["generate", "--check"]);
}

/// The arm that differs in one thing: no git step. The same merge writes
/// conflict markers into both folds, which is the defect the driver answers.
#[test]
fn the_same_merge_without_the_git_step_writes_markers_into_the_folds() {
    let tree = Tree::adopted("no-driver");
    tree.git(&["add", "-A"]);
    tree.git(&["commit", "-q", "-m", "adopt headwater"]);

    let merge = tree.diverge_and_merge();
    assert!(!merge.status.success(), "the merge conflicts");
    for path in [LOCK, DESCRIPTOR] {
        assert!(
            tree.read(path).contains("<<<<<<<"),
            "{path} carries a conflict marker when no driver is configured"
        );
    }
}

/// The decisive case of #1058: the committed attributes, and no driver config.
///
/// This is every clone that has not run `git config`, and it is the merge a
/// forge runs. Two branches each add one decision to one shelf, far enough
/// apart that their rows of the shelf index are two hunks with an unchanged row
/// between them. A text merge of that index exits 0 with no conflict, which is
/// the silent merge the committed attribute exists to stop. With `-merge`
/// committed, git keeps the current side, writes no marker and records a
/// conflict, and no driver or configuration is needed for it.
#[test]
fn a_clone_with_the_committed_attributes_and_no_driver_config_conflicts_on_a_fold() {
    const SHELF: &str = "docs/decisions/README.md";
    let tree = Tree::adopted("unconfigured");
    for seq in ["0001", "0003", "0005"] {
        tree.decide(seq);
    }
    tree.headwater_ok(&["generate"]);
    tree.headwater_ok(&["init", "--git"]);
    assert!(
        tree.read(".gitattributes").contains(&format!("{SHELF} ")),
        "`init --git` declares the shelf index it found:\n{}",
        tree.read(".gitattributes")
    );
    tree.git(&["add", "-A"]);
    tree.git(&["commit", "-q", "-m", "adopt headwater"]);
    let configured =
        tree.git_output(&["config", "--get-regexp", "^merge\\.headwater-regenerate\\."]);
    assert!(
        !configured.status.success(),
        "this clone configures no driver"
    );
    assert!(
        !tree.info_attributes().exists(),
        "this clone holds no attribute override"
    );

    tree.git(&["checkout", "-q", "-b", "a"]);
    tree.decide("0002");
    tree.headwater_ok(&["generate"]);
    tree.git(&["add", "-A"]);
    tree.git(&["commit", "-q", "-m", "a decision"]);
    tree.git(&["checkout", "-q", "main"]);
    tree.git(&["checkout", "-q", "-b", "b"]);
    tree.decide("0006");
    tree.headwater_ok(&["generate"]);
    tree.git(&["add", "-A"]);
    tree.git(&["commit", "-q", "-m", "another decision"]);

    let merge = tree.git_output(&["merge", "--no-edit", "a"]);
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&merge.stdout),
        String::from_utf8_lossy(&merge.stderr)
    );
    assert!(
        !merge.status.success(),
        "a merge that moves a fold in an unconfigured clone stops:\n{said}"
    );
    let unmerged = tree.unmerged();
    assert!(
        unmerged.iter().any(|listed| listed == SHELF),
        "{SHELF} is left conflicted, and the conflicted paths are {unmerged:?}"
    );
    let merged = tree.read(SHELF);
    assert!(
        !merged.contains("<<<<<<<"),
        "{SHELF} carries no conflict marker:\n{merged}"
    );
    let current = tree.git(&["show", &format!("HEAD:{SHELF}")]);
    assert_eq!(merged, current, "{SHELF} holds the current side's bytes");

    tree.headwater_ok(&["generate"]);
    tree.git(&["add", SHELF]);
    tree.git(&["commit", "-q", "--no-edit"]);
    tree.headwater_ok(&["generate", "--check"]);
}

/// What `init --git` writes, read byte for byte: attribute lines and comments,
/// and no file that needs an interpreter, a toolchain or a script.
#[test]
fn the_git_step_writes_attributes_for_the_two_verb_producers_and_prints_the_config() {
    let tree = Tree::adopted("written");
    let before = tree.git(&["status", "--porcelain", "--untracked-files=all"]);
    let output = tree.headwater_ok(&["init", "--git"]);
    let after = tree.git(&["status", "--porcelain", "--untracked-files=all"]);
    let added: Vec<&str> = after
        .lines()
        .filter(|line| !before.lines().any(|seen| seen == *line))
        .collect();
    assert_eq!(
        added,
        vec!["?? .gitattributes"],
        "the git step writes one file, `.gitattributes`"
    );

    let attributes = tree.read(".gitattributes");
    let declared: Vec<&str> = attributes
        .lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .collect();
    assert_eq!(
        declared,
        vec![".headwater/corpus.json -merge", ".headwater/taxonomy.lock -merge",],
        "the attribute lines unset the merge of each output of the two verb producers:\n{attributes}"
    );
    assert!(
        !tree.info_attributes().exists(),
        "without `--git-config` no attribute override is written, because an \
         override that names a driver no config defines is a text merge again"
    );
    for word in ["tools/", ".githooks", "cargo", "python", ".sh"] {
        assert!(
            !attributes.contains(word),
            "`.gitattributes` names nothing an adopter does not hold, and it names `{word}`:\n{attributes}"
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    assert!(
        stdout.contains(
            "git config merge.headwater-regenerate.driver \"headwater merge-driver %O %A %B %P\""
        ),
        "the step prints the driver line:\n{stdout}"
    );
    assert!(
        stdout.contains("git config merge.headwater-regenerate.name"),
        "the step prints the name line:\n{stdout}"
    );
    for path in [LOCK, DESCRIPTOR] {
        assert!(
            stdout.contains(&format!("{path} merge=headwater-regenerate")),
            "the step prints the override line for {path}:\n{stdout}"
        );
    }
    assert!(
        stdout.contains("git rev-parse --git-path info/attributes"),
        "the step names where the override goes:\n{stdout}"
    );
    let configured = tree.git_output(&["config", "--get", "merge.headwater-regenerate.driver"]);
    assert!(
        !configured.status.success(),
        "without `--git-config` nothing is written to git's configuration"
    );

    // Run twice, the step appends nothing.
    tree.headwater_ok(&["init", "--git"]);
    assert_eq!(
        tree.read(".gitattributes"),
        attributes,
        "a second run writes no line the first one wrote"
    );
}

/// A producer output of this repository's own script is reported by
/// `headwater derived` and never written by `init --git`.
///
/// `derived` asks four producers, and two of them are a script and a test run
/// that only the repository maintaining this engine holds. A page under `site/`
/// carrying a `data-figure` element is what the script claims, so the tree below
/// holds one. The first assertion is what makes the second a measurement: a
/// page `derived` did not claim would be absent from `.gitattributes` whatever
/// the filter did.
#[test]
fn the_git_step_writes_no_line_for_a_producer_the_adopter_does_not_hold() {
    let tree = Tree::adopted("script-producer");
    const PAGE: &str = "site/index.html";
    std::fs::create_dir_all(tree.at.join("site")).expect("the site directory is made");
    tree.write(
        PAGE,
        "<p>The corpus holds <span data-figure=\"census.seen\">1</span> files.</p>\n",
    );

    let derived = tree.headwater(&["derived"]);
    let report = String::from_utf8_lossy(&derived.stdout).into_owned();
    assert!(
        report.contains(PAGE),
        "`headwater derived` claims {PAGE} for the figure producer:\n{report}"
    );

    tree.headwater_ok(&["init", "--git"]);
    let attributes = tree.read(".gitattributes");
    assert!(
        !attributes.contains(PAGE),
        "`init --git` writes no line for a producer an adopter does not hold:\n{attributes}"
    );
    assert!(
        attributes.contains(".headwater/corpus.json -merge"),
        "the verb producers' outputs are still written:\n{attributes}"
    );
}

/// The verb, called as git calls it, with nothing of git around it.
#[test]
fn the_driver_leaves_the_current_side_and_exits_non_zero() {
    let dir = std::env::temp_dir().join(format!(
        "headwater-cli-merge-driver-{}-bare",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("the directory is made");
    for (name, body) in [("o", "ancestor\n"), ("a", "current\n"), ("b", "other\n")] {
        std::fs::write(dir.join(name), body).expect("the side writes");
    }
    for (path, producer) in [
        (LOCK, "headwater taxonomy resolve"),
        (DESCRIPTOR, "headwater generate"),
        ("docs/README.md", "headwater generate"),
    ] {
        let output = Command::new(binary())
            .arg("merge-driver")
            .arg(dir.join("o"))
            .arg(dir.join("a"))
            .arg(dir.join("b"))
            .arg(path)
            .output()
            .expect("the binary runs");
        assert_eq!(output.status.code(), Some(1), "the driver refuses {path}");
        assert!(
            output.stdout.is_empty(),
            "standard output during a merge is git's"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains(path) && stderr.contains(producer),
            "the driver names {path} and `{producer}`:\n{stderr}"
        );
        assert_eq!(
            std::fs::read_to_string(dir.join("a")).expect("the current side reads"),
            "current\n",
            "the current side is left byte for byte"
        );
    }
    let _ = std::fs::remove_dir_all(&dir);
}
