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
//! writes, the lock is left conflicted with no marker and the current side's
//! bytes in place, and in a configured clone the producer is named on standard
//! error. The descriptor is one record per entity (#1058), so it takes no
//! attribute and conflicts as text on the digest line both branches moved.
//! Without the step, the same merge writes conflict markers into both. The
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
        Tree::adopted_at(std::env::temp_dir().join(format!(
            "headwater-cli-merge-driver-{}-{label}",
            std::process::id()
        )))
    }

    /// An adopted tree at `at`, which a case places inside a directory of its own.
    fn adopted_at(at: PathBuf) -> Tree {
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
    assert!(
        override_.contains(&format!("{LOCK} merge=headwater-regenerate\n")),
        "the override names the driver for the lock:\n{override_}"
    );
    assert!(
        !override_.contains(DESCRIPTOR),
        "the descriptor is one record per entity, so nothing selects a driver for it:\n{override_}"
    );
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
    }
    let merged = tree.read(LOCK);
    assert!(
        !merged.contains("<<<<<<<"),
        "the lock carries no conflict marker:\n{merged}"
    );
    let current = tree.git(&["show", &format!("HEAD:{LOCK}")]);
    assert_eq!(merged, current, "the lock holds the current side's bytes");
    // The descriptor records the lock's digest on one line, which both
    // branches moved, so it conflicts as text: the record shape's ordinary
    // conflict, and `headwater generate` rewrites it below.
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
/// forge runs. Before #1058 `init --git` committed `merge=headwater-regenerate`,
/// and git reads a driver that no config defines as a text merge, so the lock
/// took conflict markers. With `-merge` committed, git keeps the current side,
/// writes no marker and records a conflict, and no driver or configuration is
/// needed for it.
///
/// The lock is the one fold an adopter's two verb producers write. Its digest
/// line moves on every change, so two branches that move it always conflict,
/// and what `-merge` changes is the file left behind: the current side, which
/// `headwater taxonomy resolve` reads, rather than a file with markers in it.
/// The quiet case, a fold that a text merge takes to exit 0, is a page under
/// `site/`, and `engine/crates/census/tests/merge_driver.rs` holds it.
#[test]
fn a_clone_with_the_committed_attributes_and_no_driver_config_conflicts_on_a_fold() {
    let tree = Tree::adopted("unconfigured");
    tree.headwater_ok(&["init", "--git"]);
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

    let merge = tree.diverge_and_merge();
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
        unmerged.iter().any(|listed| listed == LOCK),
        "the lock is left conflicted, and the conflicted paths are {unmerged:?}"
    );
    let merged = tree.read(LOCK);
    assert!(
        !merged.contains("<<<<<<<"),
        "the lock carries no conflict marker:\n{merged}"
    );
    let current = tree.git(&["show", &format!("HEAD:{LOCK}")]);
    assert_eq!(merged, current, "the lock holds the current side's bytes");
}

/// A shelf index is one record per entity, so it carries no attribute and a
/// text merge of it is what `headwater generate` writes over the merged tree.
///
/// Two branches each add one decision, far enough apart that their rows are
/// two hunks with an unchanged row between them. This is the pair #1058
/// measured over every generated file of this repository (1058-a): no pair
/// merged to bytes the producer does not write. A `-merge` here would stop
/// every pair of branches that each add a document.
#[test]
fn a_shelf_index_carries_no_attribute_and_merges_to_what_generate_writes() {
    const SHELF: &str = "docs/decisions/README.md";
    let tree = Tree::adopted("record-shaped");
    for seq in ["0001", "0003", "0005"] {
        tree.decide(seq);
    }
    tree.headwater_ok(&["generate"]);
    tree.headwater_ok(&["init", "--git"]);
    assert!(
        !tree.read(".gitattributes").contains(SHELF),
        "`init --git` declares no attribute for a record-shaped index:\n{}",
        tree.read(".gitattributes")
    );
    tree.git(&["add", "-A"]);
    tree.git(&["commit", "-q", "-m", "adopt headwater"]);

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
    assert!(
        merge.status.success(),
        "the two rows merge as text:\n{}",
        String::from_utf8_lossy(&merge.stderr)
    );
    tree.headwater_ok(&["generate", "--check"]);
}

/// A clone that ran the old `init --git --git-config` has the driver config
/// and no override. Running the step again selects the driver there, because
/// the committed `-merge` would otherwise deselect it in silence.
#[test]
fn the_git_step_selects_the_driver_in_a_clone_that_already_configured_it() {
    let tree = Tree::adopted("migrated");
    tree.git(&[
        "config",
        "merge.headwater-regenerate.driver",
        "headwater merge-driver %O %A %B %P",
    ]);
    tree.headwater_ok(&["init", "--git"]);
    let override_ = std::fs::read_to_string(tree.info_attributes())
        .expect("a configured clone gets the override without `--git-config`");
    assert!(
        override_.contains(&format!("{LOCK} merge=headwater-regenerate\n")),
        "the override selects the driver for the lock:\n{override_}"
    );
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
        vec![".headwater/taxonomy.lock -merge"],
        "the attribute lines unset the merge of each fold of the two verb producers, \
         and the descriptor is one record per entity:\n{attributes}"
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
    assert!(
        stdout.contains(&format!("{LOCK} merge=headwater-regenerate")),
        "the step prints the override line for the lock:\n{stdout}"
    );
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

/// The long description of `headwater init` states what `--git` writes as the
/// contract states it (`docs/interfaces/headwater-init.md`, the `--git` row):
/// a `-merge` line in `.gitattributes`, and the override that selects the
/// driver in the clone's own `info/attributes`.
///
/// The flag help already says so, so the case reads the description alone,
/// which is the text above `Usage:`. `help.rs` holds that text against the verb
/// table and would pass whatever the table says; this case holds the table
/// against the contract. Restore a description that has `--git` append
/// `merge=headwater-regenerate` to `.gitattributes`, and it fails.
#[test]
fn the_description_of_init_says_git_commits_unset_merge_and_selects_the_driver_per_clone() {
    let output = Command::new(binary())
        .args(["init", "--help"])
        .output()
        .expect("the binary runs");
    assert_eq!(
        output.status.code(),
        Some(0),
        "`headwater init --help` exits 0"
    );
    let help = String::from_utf8_lossy(&output.stdout).into_owned();
    let description = help
        .split("Usage:")
        .next()
        .expect("the help has a description")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        !description.contains("merge=headwater-regenerate"),
        "the description does not say `--git` commits a line that selects the driver, \
         because a line naming a driver no config defines is a text merge:\n{description}"
    );
    for words in [
        "`-merge`",
        "`.gitattributes`",
        "`info/attributes`",
        "`git config`",
    ] {
        assert!(
            description.contains(words),
            "the description names {words}, as the `--git` row of the contract does:\n{description}"
        );
    }
}

/// Every `headwater taxonomy resolve` command the merge hooks of this
/// repository print as a remedy is one the verb accepts (HW-OBL-0216).
///
/// A remedy that the verb refuses sends a merger from a stopped merge to an
/// "unexpected argument" error. The case reads the command out of each hook
/// and runs it on an adopted tree. Put `--write` back into either hook, and
/// the verb refuses it and this case fails.
#[test]
fn every_resolve_remedy_the_merge_hooks_print_is_one_the_verb_accepts() {
    const VERB: &str = "headwater taxonomy resolve";
    let tree = Tree::adopted("remedy");
    let mut ran = 0;
    for hook in [".githooks/merge-regenerate", ".githooks/merged-fold-check"] {
        let text = std::fs::read_to_string(repository().join(hook))
            .unwrap_or_else(|error| panic!("{hook} reads: {error}"));
        let mut found = 0;
        for (at, _) in text.match_indices(VERB) {
            let rest = &text[at..];
            let end = [
                rest.find('"'),
                rest.find('`'),
                rest.find(')'),
                rest.find("  "),
                rest.find('\n'),
            ]
            .into_iter()
            .flatten()
            .min()
            .unwrap_or(rest.len());
            let command = rest[..end].trim();
            let args: Vec<&str> = command.split_whitespace().skip(1).collect();
            let output = tree.headwater(&args);
            assert_eq!(
                output.status.code(),
                Some(0),
                "`{command}`, printed by {hook}, is a command the verb accepts:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
            found += 1;
        }
        assert!(found > 0, "{hook} prints a `{VERB}` remedy");
        ran += found;
    }
    assert!(
        ran >= 3,
        "the two hooks print three resolve commands, and {ran} ran"
    );
}

/// Inside a repository where git does not run, `headwater derived` says so and exits 1.
///
/// Git answers for every attribute file of the tree. Without it the verb can
/// read the root `.gitattributes` alone, and that file agrees here, so a verb
/// that read a failed spawn as "no repository" exits 0 and says nothing about
/// `info/attributes` or a nested file it could not read. Read the spawn failure
/// as `None` again, and this case fails on the exit status.
#[test]
fn inside_a_repository_where_git_does_not_run_derived_says_so() {
    let tree = Tree::adopted("no-git");
    tree.headwater_ok(&["init", "--git"]);
    tree.headwater_ok(&["derived"]);

    let empty = tree.at.with_extension("empty-path");
    std::fs::create_dir_all(&empty).expect("the empty directory is made");
    let output = Command::new(binary())
        .args(["derived", "--root"])
        .arg(&tree.at)
        .env("PATH", &empty)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env_remove("GIT_DIR")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_WORK_TREE")
        .output()
        .expect("the binary runs");
    let _ = std::fs::remove_dir_all(&empty);
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        output.status.code(),
        Some(1),
        "a repository whose git does not run is not a tree with no repository:\n{said}"
    );
    assert!(
        said.contains("git did not run"),
        "the report names that git did not run:\n{said}"
    );
}

/// `headwater derived` with `extra` set on the child alone, and `PATH` too
/// where `path` names one. Standard output and standard error, joined.
fn derived_with(tree: &Tree, path: Option<&Path>, extra: &[(&str, &str)]) -> (Option<i32>, String) {
    let mut command = Command::new(binary());
    command
        .args(["derived", "--root"])
        .arg(&tree.at)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env_remove("GIT_DIR")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_WORK_TREE");
    if let Some(path) = path {
        command.env("PATH", path);
    }
    for (key, value) in extra {
        command.env(key, value);
    }
    let output = command.output().expect("the binary runs");
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    (output.status.code(), said)
}

/// Inside a repository that git refuses for its owner, `headwater derived` says so and exits 1.
///
/// Git runs, and `git rev-parse` fails with "dubious ownership", which is
/// common where a container runs as another user than the one that owns the
/// checkout. `GIT_TEST_ASSUME_DIFFERENT_OWNER` is git's own knob for that
/// state. A `.git` entry says a repository is there, so the failed question is
/// a refusal and not "no repository". Read a failed `rev-parse` as `None`
/// again, and this case fails on the exit status.
#[test]
fn inside_a_repository_that_git_refuses_for_its_owner_derived_says_so() {
    let tree = Tree::adopted("dubious-owner");
    tree.headwater_ok(&["init", "--git"]);
    tree.headwater_ok(&["derived"]);

    let (code, said) = derived_with(&tree, None, &[("GIT_TEST_ASSUME_DIFFERENT_OWNER", "1")]);
    assert_eq!(
        code,
        Some(1),
        "a repository that git refuses is not a tree with no repository:\n{said}"
    );
    assert!(
        said.contains("git did not answer"),
        "the report names that git did not answer:\n{said}"
    );
}

/// Inside a linked worktree whose repository is gone, `headwater derived` says so and exits 1.
///
/// The `.git` file of a linked worktree names a directory, and once that
/// directory is pruned `git rev-parse` fails. The `.git` file still says the
/// tree is a work tree whose attributes only git reads.
#[test]
fn inside_a_worktree_whose_git_directory_is_gone_derived_says_so() {
    let tree = Tree::adopted("pruned-worktree");
    tree.headwater_ok(&["init", "--git"]);
    std::fs::remove_dir_all(tree.at.join(".git")).expect("the git directory is removed");
    let gone = tree.at.with_extension("pruned-gitdir");
    let _ = std::fs::remove_dir_all(&gone);
    tree.write(".git", &format!("gitdir: {}\n", gone.display()));

    let (code, said) = derived_with(&tree, None, &[]);
    assert_eq!(
        code,
        Some(1),
        "a worktree whose git directory is gone is not a tree with no repository:\n{said}"
    );
    assert!(
        said.contains("git did not answer"),
        "the report names that git did not answer:\n{said}"
    );
}

/// With no `.git` entry anywhere above the tree and no git on `PATH`, the root file answers with no refusal.
///
/// This is the other half of the two cases above. A tree that is not a
/// repository yet is read by the root-file reader, whether git is installed or
/// not. Read every failed spawn as a refusal, and this case fails.
#[test]
fn outside_a_repository_with_no_git_derived_reads_the_root_file_and_refuses_nothing() {
    let tree = Tree::adopted("no-git-no-repository");
    tree.headwater_ok(&["init", "--git"]);
    std::fs::remove_dir_all(tree.at.join(".git")).expect("the git directory is removed");
    assert!(
        !tree.at.ancestors().any(|dir| dir.join(".git").exists()),
        "no directory above the temporary tree holds a `.git` entry"
    );

    let empty = tree.at.with_extension("empty-path-no-repository");
    std::fs::create_dir_all(&empty).expect("the empty directory is made");
    let (code, said) = derived_with(&tree, Some(&empty), &[]);
    let _ = std::fs::remove_dir_all(&empty);
    assert!(
        !said.contains("git did not"),
        "a tree with no repository names no git failure:\n{said}"
    );
    assert_eq!(
        code,
        Some(0),
        "the root `.gitattributes` that `init --git` wrote agrees:\n{said}"
    );
}

/// A directory that holds an adopted tree at `tree/`, removed with everything in it.
struct Outer(PathBuf);

impl Drop for Outer {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

impl Outer {
    /// An adopted tree at `<outer>/tree` that holds no `.git` of its own, so
    /// what git finds above it is what the case puts in `<outer>`.
    fn with_tree(label: &str) -> (Outer, Tree) {
        let outer = Outer(std::env::temp_dir().join(format!(
            "headwater-cli-merge-driver-{}-{label}",
            std::process::id()
        )));
        let _ = std::fs::remove_dir_all(&outer.0);
        std::fs::create_dir_all(&outer.0).expect("the outer directory is made");
        let tree = Tree::adopted_at(outer.0.join("tree"));
        tree.headwater_ok(&["init", "--git"]);
        std::fs::remove_dir_all(tree.at.join(".git")).expect("the git directory is removed");
        (outer, tree)
    }

    /// A real repository in the outer directory, made by git itself.
    fn git_init(&self) {
        let output = Command::new("git")
            .args(["init", "-q"])
            .arg(&self.0)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .output()
            .expect("git runs");
        assert!(output.status.success(), "`git init` succeeds");
    }
}

/// Where git looks above the tree and finds no repository, nothing is refused.
///
/// Each of these three trees has a `.git` entry above it, and git says there
/// is no repository: a `.git` file that is not a gitfile ("invalid gitfile
/// format"), an empty `.git` directory, and a repository above a directory
/// that `GIT_CEILING_DIRECTORIES` names. The root file answers, as it does for
/// a tree with no `.git` entry at all. Count every `.git` entry above the tree
/// as a repository, and these cases fail.
fn no_repository_above(label: &str, shape: impl Fn(&Outer), extra: &[(&str, &str)]) {
    let (outer, tree) = Outer::with_tree(label);
    shape(&outer);
    let (code, said) = derived_with(&tree, None, extra);
    assert!(
        !said.contains("git did not"),
        "git finds no repository here, so nothing is refused:\n{said}"
    );
    assert_eq!(
        code,
        Some(0),
        "the root `.gitattributes` that `init --git` wrote agrees:\n{said}"
    );
}

#[test]
fn a_file_named_git_that_is_not_a_gitfile_above_the_tree_is_no_repository() {
    no_repository_above(
        "plain-git-file",
        |outer| std::fs::write(outer.0.join(".git"), "hello\n").expect("the file writes"),
        &[],
    );
}

#[test]
fn an_empty_directory_named_git_above_the_tree_is_no_repository() {
    no_repository_above(
        "empty-git-directory",
        |outer| std::fs::create_dir(outer.0.join(".git")).expect("the directory is made"),
        &[],
    );
}

#[test]
fn a_repository_beyond_a_ceiling_directory_is_no_repository() {
    let label = "ceiling";
    let ceiling = std::env::temp_dir().join(format!(
        "headwater-cli-merge-driver-{}-{label}",
        std::process::id()
    ));
    let ceiling = ceiling.to_string_lossy().into_owned();
    no_repository_above(
        label,
        Outer::git_init,
        &[("GIT_CEILING_DIRECTORIES", &ceiling)],
    );
}

/// In a subdirectory of a repository that git refuses for its owner, `headwater derived` says so.
///
/// The tree holds no `.git` of its own, and the repository is the directory
/// above it. Look for a repository at the root of the tree alone, and this
/// case fails on the exit status.
#[test]
fn below_a_repository_that_git_refuses_for_its_owner_derived_says_so() {
    let (outer, tree) = Outer::with_tree("dubious-owner-above");
    outer.git_init();
    let (code, said) = derived_with(&tree, None, &[("GIT_TEST_ASSUME_DIFFERENT_OWNER", "1")]);
    assert_eq!(
        code,
        Some(1),
        "a tree inside a repository that git refuses is not a tree with no repository:\n{said}"
    );
    assert!(
        said.contains("git did not answer"),
        "the report names that git did not answer:\n{said}"
    );
}

/// `init --git` writes no root line for a fold that a nested file already declares.
///
/// The lock is declared in `.headwater/.gitattributes`, and git answers
/// `unset` for it. The step asks git rather than reading the root file, so it
/// finds every artifact declared and writes nothing, on each of two runs.
#[test]
fn the_git_step_writes_no_line_for_a_fold_a_nested_file_declares() {
    let tree = Tree::adopted("nested");
    tree.write(".headwater/.gitattributes", "taxonomy.lock -merge\n");
    for run in ["first", "second"] {
        let output = tree.headwater_ok(&["init", "--git"]);
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        assert!(
            stdout.contains("already declares all"),
            "the {run} run finds every derived artifact declared:\n{stdout}"
        );
        let root = std::fs::read_to_string(tree.at.join(".gitattributes")).unwrap_or_default();
        assert!(
            !root.contains("taxonomy.lock"),
            "the {run} run wrote a root line for the lock, which the nested file declares:\n{root}"
        );
    }
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
            attributes.contains(".headwater/taxonomy.lock -merge"),
            "the verb producers' folds are still written:\n{attributes}"
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
            attributes.contains(&format!("{path} -merge")),
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
