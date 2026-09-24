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
        vec![
            ".headwater/corpus.json merge=headwater-regenerate",
            ".headwater/taxonomy.lock merge=headwater-regenerate",
        ],
        "the attribute lines are the outputs of the two verb producers:\n{attributes}"
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

/// A producer that only this repository holds is neither named by
/// `headwater derived` nor written by `init --git` in an adopter's tree.
///
/// `derived` knows four producers, and two of them are a script and a blessing
/// run that only the repository maintaining this engine holds. Each row below
/// plants a file that producer would claim here, and the file whose presence
/// makes a tree hold it. In the adopter's tree the file is claimed by nobody,
/// so the report names neither the file nor the command, and `init --git`
/// writes no line. Once the tree holds the producer, the same file is claimed
/// and written, which is what makes the first half a measurement of the
/// predicate rather than of a filter that admits nothing.
#[test]
fn the_git_step_writes_no_line_for_a_producer_the_adopter_does_not_hold() {
    let rows: [(&str, &str, &str, &str, &str); 2] = [
        (
            "script-producer",
            "site/index.html",
            "<p>The corpus holds <span data-figure=\"census.seen\">1</span> files.</p>\n",
            "sh tools/site/refresh-figures.sh",
            "tools/site/refresh-figures.sh",
        ),
        (
            "blessing-producer",
            "engine/crates/a/fixtures/corpus.a",
            "426 files\nsha256:0a1b\n",
            "HEADWATER_BLESS=1 cargo test",
            "engine/Cargo.toml",
        ),
    ];
    for (label, path, body, command, holder) in rows {
        let tree = Tree::adopted(label);
        let plant = |relative: &str, text: &str| {
            let at = tree.at.join(relative);
            std::fs::create_dir_all(at.parent().expect("a parent")).expect("the directory is made");
            std::fs::write(at, text).expect("the file writes");
        };
        plant(path, body);

        let derived = tree.headwater(&["derived"]);
        let report = String::from_utf8_lossy(&derived.stdout).into_owned();
        assert!(
            !report.contains(path) && !report.contains(command),
            "`headwater derived` names {path} or `{command}` in a tree without \
             {holder}:\n{report}"
        );
        tree.headwater_ok(&["init", "--git"]);
        let attributes = tree.read(".gitattributes");
        assert!(
            !attributes.contains(path),
            "`init --git` writes a line for {path}, whose producer the adopter does \
             not hold:\n{attributes}"
        );
        assert!(
            attributes.contains(".headwater/corpus.json merge=headwater-regenerate"),
            "the verb producers' outputs are still written:\n{attributes}"
        );

        plant(holder, "held\n");
        let derived = tree.headwater(&["derived"]);
        let report = String::from_utf8_lossy(&derived.stdout).into_owned();
        assert!(
            report.contains(path) && report.contains(command),
            "with {holder} present, `headwater derived` claims {path} for \
             `{command}`:\n{report}"
        );
        tree.headwater_ok(&["init", "--git"]);
        let attributes = tree.read(".gitattributes");
        assert!(
            attributes.contains(&format!("{path} merge=headwater-regenerate")),
            "with {holder} present, `init --git` writes {path}:\n{attributes}"
        );
    }
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
