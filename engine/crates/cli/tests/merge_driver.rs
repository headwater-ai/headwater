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

/// Remove `path` and everything under it, and try again until it is gone.
///
/// One attempt is not enough. A git process that outlives the command that
/// started it, such as an automatic maintenance run, can still be writing under
/// the tree when the case ends, and `remove_dir_all` then fails on a directory
/// that is not empty. Each tree below turns automatic maintenance off, and this
/// is the second guard. CI's "The engine suite leaves nothing in its temporary
/// directory" step fails on anything left.
fn remove_all(path: &Path) {
    for _ in 0..50 {
        let _ = std::fs::remove_dir_all(path);
        if !path.exists() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

/// A scratch directory that is removed when this value is dropped: on the
/// success path, on a panic and on an early return (#1158).
struct Scratch(PathBuf);

impl Scratch {
    fn made(path: PathBuf) -> Scratch {
        remove_all(&path);
        let scratch = Scratch(path);
        std::fs::create_dir_all(&scratch.0).expect("the scratch directory is made");
        scratch
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        remove_all(&self.0);
    }
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
        remove_all(&at);
        Tree::adopted_in(at.clone(), &at)
    }

    /// An adopted tree at `at`, inside the git repository whose top is `top`,
    /// which is `at` itself or a directory above it.
    fn adopted_in(at: PathBuf, top: &Path) -> Tree {
        // The guard exists before the first byte is written, so a panic while
        // the tree is built removes what was built.
        let tree = Tree { at };
        std::fs::create_dir_all(tree.at.join("docs")).expect("the corpus directory is made");
        tree.write("docs/one.md", "# a document\n");
        copy_dir(
            &repository().join(".headwater/packages/headwater-standard"),
            &tree.at.join(".headwater/packages/headwater-standard"),
        );
        let init = tree.git_output(&["init", "-q", "-b", "main", &top.to_string_lossy()]);
        assert!(init.status.success(), "`git init` succeeds");
        tree.git(&["config", "user.email", "adopter@example.com"]);
        tree.git(&["config", "user.name", "An adopter"]);
        // No git process outlives the command that started it, so nothing
        // writes under the tree after the case removes it.
        tree.git(&["config", "maintenance.auto", "false"]);
        tree.git(&["config", "gc.auto", "0"]);
        tree.git(&["config", "gc.autoDetach", "false"]);
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
        remove_all(&self.at);
        remove_all(&self.change_dir());
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
/// The quiet case, a fold that a text merge takes to exit 0, is held by
/// `engine/crates/census/tests/merge_driver.rs`.
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

/// Where the graph export lands when a tree declares it as a committed
/// projection, as this repository did before #1251.
const EXPORT: &str = ".headwater/export.json";

/// The graph export this repository declared until #1251, as an `add_to` block
/// an adopter's overlay can carry.
const COMMITTED_EXPORT: &str =
    "add_to:\n  projections:\n    - kind: graph_export\n      profile: site\n      output: .headwater/export.json\n";

/// Every `graph_export` projection the resolved lock under `root` holds, as an
/// `add_to` block an adopter's overlay can carry, or nothing when it holds none.
///
/// The lock and not the overlay: an overlay entry may name its keys in any
/// order, and a bundle may declare an export too. `taxonomy resolve` folds all
/// of that into one list, and the lock is where the list is written. The lock is
/// read as YAML, so a key order the reader did not expect is not a missed entry
/// (`a_graph_export_is_found_in_the_lock_in_every_key_order`). An entry
/// whose value is not a scalar, such as a `filter`, is refused rather than
/// copied short.
fn graph_exports_in_lock(root: &Path) -> String {
    let source = std::fs::read_to_string(root.join(LOCK)).expect("the lock reads");
    let lock = headwater_yaml::load(&source).expect("the lock is YAML");
    let projections = lock
        .value
        .as_map()
        .and_then(|top| top.get("resolved"))
        .and_then(|resolved| resolved.value.as_map())
        .and_then(|resolved| resolved.get("projections"))
        .and_then(|projections| projections.value.as_seq())
        .map(<[_]>::to_vec)
        .unwrap_or_default();
    let mut entries = String::new();
    for projection in &projections {
        let Some(fields) = projection.value.as_map() else {
            continue;
        };
        let kind = fields
            .get("kind")
            .and_then(|kind| kind.value.as_scalar())
            .map(|kind| kind.text.as_str());
        if kind != Some("graph_export") {
            continue;
        }
        entries.push_str("    - kind: graph_export\n");
        for entry in fields.iter().filter(|entry| entry.key.value != "kind") {
            let value = entry.value.value.as_scalar().unwrap_or_else(|| {
                panic!(
                    "a graph_export's `{}` is not a scalar, and this reader copies scalars only",
                    entry.key.value
                )
            });
            entries.push_str(&format!("      {}: \"{}\"\n", entry.key.value, value.text));
        }
    }
    match entries.is_empty() {
        true => String::new(),
        false => format!("add_to:\n  projections:\n{entries}"),
    }
}

/// The graph exports this repository commits. Before #1251 it was one, at
/// `.headwater/export.json`, and the decisive case below was red on it.
fn this_repository_s_graph_exports() -> String {
    graph_exports_in_lock(&repository())
}

const GOVERNED: &str = "tools/run.sh";
const GOVERNING: &str = "docs/decisions/0001-governs.md";

impl Tree {
    /// The directory a case writes a change manifest into. It sits beside the
    /// tree and never inside it, because `headwater change` names a file under
    /// the tree that the index does not hold as a document the change adds.
    fn change_dir(&self) -> PathBuf {
        let mut name = self.at.as_os_str().to_owned();
        name.push("-change");
        PathBuf::from(name)
    }

    /// The governing decision, verified on `last_verified`, with one bare
    /// `governs` entry.
    fn governing(&self, last_verified: &str, entry: &str) {
        self.write(
            GOVERNING,
            &format!(
                "---\nid: ACME-DR-0001\ntitle: Governs\nstatus: draft\nstatus_since: 2026-01-01\nlast_verified: {last_verified}\nsummary: The decision that governs the run script.\nrelations:\n  governs:\n{entry}\n---\n\n# Governs\n\n## Context\n\nA script.\n\n## Decision\n\nIt runs.\n\n## Consequences\n\nIt ran.\n"
            ),
        );
    }

    /// A decision that governs one code path, with the digest recorded by
    /// `check --fix --change`, the one way a stamp is written without typing
    /// it. The change adds the decision, which states that its author read
    /// what it governs (#1259). The tree has no commit yet, so the manifest is
    /// written here rather than by `headwater change`.
    fn govern_and_stamp(&self) {
        std::fs::create_dir_all(self.at.join("docs/decisions")).expect("the shelf is made");
        std::fs::create_dir_all(self.at.join("tools")).expect("the directory is made");
        self.write(GOVERNED, "#!/bin/sh\necho one\n");
        self.governing("2026-01-02", &format!("    - {GOVERNED}"));
        let dir = self.change_dir();
        remove_all(&dir);
        std::fs::create_dir_all(&dir).expect("the change directory is made");
        let manifest = dir.join("manifest");
        std::fs::write(
            &manifest,
            format!("headwater change 1\nadded\t{GOVERNING}\n"),
        )
        .expect("the manifest writes");
        self.headwater_ok(&["check", "--fix", "--change", &manifest.to_string_lossy()]);
        remove_all(&dir);
    }

    /// Re-verify the governing decision and restamp its edge: move its
    /// `last_verified`, write the change against `HEAD` with `headwater
    /// change`, and run `check --fix` scoped to it, as an author does.
    fn re_verify_and_restamp(&self) {
        let body = self.read(GOVERNING);
        self.write(
            GOVERNING,
            &body.replace("last_verified: 2026-01-02", "last_verified: 2026-01-03"),
        );
        let dir = self.change_dir();
        remove_all(&dir);
        self.headwater_ok(&["change", "HEAD", &dir.to_string_lossy()]);
        let manifest = dir.join("manifest");
        self.headwater_ok(&["check", "--fix", "--change", &manifest.to_string_lossy()]);
        remove_all(&dir);
    }

    /// The recorded digest on the governing document's one `governs` entry.
    fn stamp(&self) -> String {
        let body = self.read(GOVERNING);
        let line = body
            .lines()
            .find(|line| line.trim_start().starts_with("verified_revision:"))
            .unwrap_or_else(|| panic!("`check --fix` recorded a stamp:\n{body}"));
        line.trim_start()
            .trim_start_matches("verified_revision:")
            .trim()
            .to_string()
    }

    /// Branch `a` edits the governed code path, restamps its edge and adds a
    /// decision. Branch `b` adds a specification, on the other shelf. Each
    /// branch runs `generate`, as a pull request does, and `b` then merges `a`
    /// in a clone with no driver config, which is the merge a forge runs.
    fn restamp_here_and_add_there(&self) -> Output {
        self.git(&["checkout", "-q", "-b", "a"]);
        self.write(GOVERNED, "#!/bin/sh\necho two\n");
        self.re_verify_and_restamp();
        // `headwater new`, as an adopter adds a document: it also appends one
        // reading to `.headwater/capture-cost.jsonl`, which both branches do.
        self.headwater_ok(&["new", "decision", "--title", "Second"]);
        self.headwater_ok(&["generate"]);
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "-m", "a restamp and a decision"]);

        self.git(&["checkout", "-q", "main"]);
        self.git(&["checkout", "-q", "-b", "b"]);
        self.headwater_ok(&["new", "specification", "--title", "Runs"]);
        self.headwater_ok(&["generate"]);
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "-m", "a specification"]);

        self.git_output(&["merge", "--no-edit", "a"])
    }
}

/// The line `init --git` writes for the capture-cost store (#1263), and the
/// how-to tells an adopter initialized before that to add by hand: every
/// `headwater new` appends one reading to the end of the store, so two branches
/// that each run it conflict there without it.
const UNION: &str = ".headwater/capture-cost.jsonl merge=union\n";

/// A specification needs an identifier before `headwater new` writes one, and
/// the standard package declares none, so the adopted tree declares it.
const SPEC_ID: &str = "  identifier_schemes.spec_id: {pattern: \"{namespace}-SPEC-{slug}\", namespace: ACME, allocation: minted-once}\n  kinds.specification.identifier: {scheme: spec_id}\n";

/// An adopted tree with a governed code path, committed with `init --git` and
/// no driver config, which is every clone that has not run `git config`.
/// `init --git` writes the capture-cost union line, and the tree is checked
/// for it. Without `union` the line is taken out again, which is a tree
/// initialized before #1263 that never followed the how-to.
fn governed_tree(label: &str, projections: &str, union: bool) -> Tree {
    let tree = Tree::adopted(label);
    let overlay = tree.read(".headwater/overlay.yml");
    // Appended to the `add` block `Tree::adopted` left at the end of the file.
    // A comment above it quotes the namespace line too, so the block is found
    // by its `add:` line and not by the namespace line alone.
    let overlay = overlay.replace(
        "add:\n  identifier_schemes.decision_id.namespace: ACME\n",
        &format!("add:\n  identifier_schemes.decision_id.namespace: ACME\n{SPEC_ID}"),
    );
    tree.write(".headwater/overlay.yml", &format!("{overlay}{projections}"));
    tree.headwater_ok(&["taxonomy", "resolve"]);
    tree.govern_and_stamp();
    tree.headwater_ok(&["generate"]);
    tree.headwater_ok(&["init", "--git"]);
    let attributes = tree.read(".gitattributes");
    assert!(
        attributes.contains(UNION),
        "`init --git` writes the capture-cost union line:\n{attributes}"
    );
    if !union {
        tree.write(".gitattributes", &attributes.replace(UNION, ""));
    }
    tree.git(&["add", "-A"]);
    tree.git(&["commit", "-q", "-m", "adopt headwater"]);
    tree
}

/// The decisive case of #1251: two pull requests on two shelves, one of which
/// restamps a governed edge, merge with no driver and no conflict.
///
/// The tree commits the graph exports this repository commits, read from its
/// overlay. Before #1251 that was one, at `.headwater/export.json`, and this
/// case was red on it: the export moves on any edit anywhere, so every pair of
/// branches conflicted on it, and a forge reads no merge attribute that could
/// stop that. After #1251 the export is computed where it is read, and the
/// merge is clean. The stamp moves on branch `a` alone, so it merges too.
#[test]
fn two_branches_on_different_shelves_merge_with_no_git_config_and_no_conflict_on_any_derived_path()
{
    let tree = governed_tree("two-shelves", &this_repository_s_graph_exports(), true);
    let before = tree.stamp();

    let merge = tree.restamp_here_and_add_there();
    assert!(
        merge.status.success(),
        "the two branches merge with no conflict, and the conflicted paths are {:?}:\n{}",
        tree.unmerged(),
        String::from_utf8_lossy(&merge.stdout)
    );
    assert_ne!(
        tree.stamp(),
        before,
        "the merged tree carries branch `a`'s restamp"
    );
    tree.headwater_ok(&["generate", "--check"]);
}

/// The arm that differs in one thing: the tree commits a graph export. The same
/// merge then conflicts on the export and on nothing else, which is what the
/// case above measures the absence of.
#[test]
fn the_same_merge_in_a_tree_that_commits_its_graph_export_conflicts_on_the_export() {
    let tree = governed_tree("committed-export", COMMITTED_EXPORT, true);
    assert!(
        tree.at.join(EXPORT).exists(),
        "`generate` writes the declared export"
    );

    let merge = tree.restamp_here_and_add_there();
    assert!(!merge.status.success(), "the merge stops on the export");
    assert_eq!(
        tree.unmerged(),
        vec![EXPORT.to_string()],
        "the export is the one conflicted path"
    );
}

/// The arm that differs in one thing: the tree lacks the capture-cost line that
/// `init --git` writes, as a tree initialized before #1263 does. Both branches
/// ran `headwater new`, each appended a reading at the end of the store, and
/// the merge conflicts there and nowhere else. `governed_tree` holds that
/// `init --git` wrote the line before the case took it out.
#[test]
fn the_same_merge_without_the_union_line_conflicts_on_the_capture_cost_store() {
    let tree = governed_tree("no-union", &this_repository_s_graph_exports(), false);
    let merge = tree.restamp_here_and_add_there();
    assert!(!merge.status.success(), "the merge stops on the store");
    assert_eq!(
        tree.unmerged(),
        vec![".headwater/capture-cost.jsonl".to_string()],
        "the capture-cost store is the one conflicted path"
    );
}

/// A `graph_export` is found in the resolved lock whatever order its overlay
/// entry names the keys in. `taxonomy resolve` keeps the overlay's key order, so
/// a reader that expects `kind` first misses the other two rows. A reader of the
/// overlay's text missed the profile-first row, and a grep of the lock for a
/// `- kind: graph_export` line missed the output-first row (#1253 verify).
#[test]
fn a_graph_export_is_found_in_the_lock_in_every_key_order() {
    for (label, entry) in [
        (
            "kind-first",
            "    - kind: graph_export\n      profile: public\n      output: site/graph.json\n",
        ),
        (
            "profile-first",
            "    - profile: public\n      kind: graph_export\n      output: site/graph.json\n",
        ),
        (
            "output-first",
            "    - output: site/graph.json\n      kind: graph_export\n",
        ),
    ] {
        let tree = Tree::adopted(&format!("order-{label}"));
        let overlay = tree.read(".headwater/overlay.yml");
        tree.write(
            ".headwater/overlay.yml",
            &format!("{overlay}add_to:\n  projections:\n{entry}"),
        );
        tree.headwater_ok(&["taxonomy", "resolve"]);
        let found = graph_exports_in_lock(&tree.at);
        assert!(
            found.contains("- kind: graph_export\n")
                && found.contains("output: \"site/graph.json\"\n"),
            "the {label} entry is read from the lock:\n{found}"
        );
    }
}

/// This repository declares no graph export, so `generate` commits none. This
/// is the decision CI's "The graph export is computed at build time and never
/// committed" step names: it reads the resolved lock as YAML, through the
/// reader the case above holds in every key order, rather than a line of text.
#[test]
fn this_repository_declares_no_graph_export() {
    let declared = this_repository_s_graph_exports();
    assert!(
        declared.is_empty(),
        "this repository's lock declares a graph export, which `generate` would commit; \
         compute it at build time instead (#1251):\n{declared}"
    );
}

/// How the second branch meets the first: `git merge` from the second branch,
/// or `git rebase` of the second branch onto the first. The two swap what
/// `--ours` and `--theirs` name, which is why the remedy names neither.
#[derive(Clone, Copy, Debug)]
enum Meet {
    Merge,
    Rebase,
}

/// Every claim under `.headwater/ids/` names a file that is on the tree.
///
/// A claim is written once and holds the path of the document that minted the
/// identifier ([HW-DR-0054](../../../../docs/decisions/0054-the-upper-bound-of-a-reconcile-first-allocator-is-the-corpus-and-a-claim-store.md)).
/// `identifier.claim.stale` is advisory, so `check --strict` passes a claim
/// that names a path which is gone, and this assertion is what refuses it.
fn assert_every_claim_names_a_document(tree: &Tree) {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).expect("the claim directory reads") {
            let path = entry.expect("the entry reads").path();
            match path.is_dir() {
                true => walk(&path, out),
                false => out.push(path),
            }
        }
    }
    let mut claims = Vec::new();
    walk(&tree.at.join(".headwater/ids"), &mut claims);
    assert!(!claims.is_empty(), "the tree holds claims");
    for claim in claims {
        let named = std::fs::read_to_string(&claim).expect("the claim reads");
        assert!(
            tree.at.join(named.trim_end()).is_file(),
            "{} names {}, which is not on the tree",
            claim.display(),
            named.trim_end()
        );
    }
}

/// The residual #1251 leaves open, and its remedy, over a merge and a rebase.
///
/// Two branches that each run `headwater new decision` both mint the next
/// number, `ACME-DR-0002`. Meeting them conflicts on the shelf index, because
/// both insert a row at one position, and on the claim file of the identifier,
/// by design (HW-DR-0054). Held exactly, so the set cannot grow without this
/// case moving.
///
/// Regenerating is not the remedy: it leaves two documents with one identifier,
/// and `check --strict` refuses the tree. The remedy the how-to states takes
/// the claim from the branch that landed first by its name, never by `--ours`
/// or `--theirs`: under a rebase `--theirs` is the branch being rebased, and it
/// would rewrite the landed claim to name a path that no longer exists, which
/// `check --strict` and `generate --check` both pass (#1253 verify 3). Then the
/// second branch's document takes the next free number and `check --fix` writes
/// its claim.
#[test]
fn two_decisions_minted_on_two_branches_conflict_on_the_index_and_the_claim_and_a_renumber_resolves_them(
) {
    for meet in [Meet::Merge, Meet::Rebase] {
        mint_twice_and_renumber(meet);
    }
}

fn mint_twice_and_renumber(meet: Meet) {
    const CLAIM: &str = ".headwater/ids/decision_id/ACME-DR-0002";
    let label = format!("same-shelf-{meet:?}").to_lowercase();
    let tree = governed_tree(&label, &this_repository_s_graph_exports(), true);
    tree.git(&["checkout", "-q", "-b", "a"]);
    tree.headwater_ok(&["new", "decision", "--title", "Alpha"]);
    tree.headwater_ok(&["generate"]);
    tree.git(&["add", "-A"]);
    tree.git(&["commit", "-q", "-m", "a decision"]);
    tree.git(&["checkout", "-q", "main"]);
    tree.git(&["checkout", "-q", "-b", "b"]);
    tree.headwater_ok(&["new", "decision", "--title", "Beta"]);
    tree.headwater_ok(&["generate"]);
    tree.git(&["add", "-A"]);
    tree.git(&["commit", "-q", "-m", "another decision"]);

    let met = match meet {
        Meet::Merge => tree.git_output(&["merge", "--no-edit", "a"]),
        Meet::Rebase => tree.git_output(&["rebase", "a"]),
    };
    assert!(!met.status.success(), "{meet:?}: the two mints conflict");
    assert_eq!(
        tree.unmerged(),
        vec![CLAIM.to_string(), "docs/decisions/README.md".to_string()],
        "{meet:?}: the claim and the shelf index are the conflicted paths"
    );

    // The claim of the branch that landed first, taken by its name.
    tree.git(&["checkout", "a", "--", CLAIM]);
    assert_eq!(
        tree.read(CLAIM),
        "docs/decisions/0002-alpha.md\n",
        "{meet:?}: the landed claim names the landed document"
    );
    tree.headwater_ok(&["generate"]);
    tree.git(&["add", "-A"]);
    let strict = tree.headwater(&["check", "--strict"]);
    assert_eq!(
        strict.status.code(),
        Some(1),
        "{meet:?}: two documents hold ACME-DR-0002, so a strict check refuses the tree"
    );

    // The renumber: the second branch's document takes the next free number.
    tree.git(&[
        "mv",
        "docs/decisions/0002-beta.md",
        "docs/decisions/0003-beta.md",
    ]);
    let moved = tree.read("docs/decisions/0003-beta.md");
    tree.write(
        "docs/decisions/0003-beta.md",
        &moved.replace("id: ACME-DR-0002\n", "id: ACME-DR-0003\n"),
    );
    tree.headwater_ok(&["check", "--fix"]);
    assert!(
        tree.at
            .join(".headwater/ids/decision_id/ACME-DR-0003")
            .exists(),
        "{meet:?}: `check --fix` writes the claim of the new number"
    );
    tree.headwater_ok(&["generate"]);
    tree.git(&["add", "-A"]);
    match meet {
        Meet::Merge => tree.git(&["commit", "-q", "--no-edit"]),
        Meet::Rebase => {
            let continued = tree.git_output(&["-c", "core.editor=true", "rebase", "--continue"]);
            assert!(
                continued.status.success(),
                "{meet:?}: the rebase continues:\n{}",
                String::from_utf8_lossy(&continued.stderr)
            );
            String::new()
        }
    };
    tree.headwater_ok(&["check", "--strict"]);
    tree.headwater_ok(&["generate", "--check"]);
    assert_every_claim_names_a_document(&tree);
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
        vec![
            ".headwater/taxonomy.lock -merge",
            ".headwater/capture-cost.jsonl merge=union",
            ".headwater/adoption.jsonl merge=union",
        ],
        "the attribute lines unset the merge of each fold of the two verb producers, \
         the descriptor is one record per entity, and each append-only store takes \
         the union:\n{attributes}"
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
        !stdout.contains(".jsonl merge=headwater-regenerate"),
        "the step prints no override line for an append-only store, which no producer \
         rebuilds:\n{stdout}"
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

    // Which file gets which line. Each file is named in a sentence of its
    // own, so a sentence that names both cannot hand one file's line to the
    // other. The `.gitattributes` sentence carries `-merge` and no driver, and
    // the `info/attributes` sentence selects the driver and carries no
    // `-merge`. A description that swaps the two files fails here.
    let sentences: Vec<&str> = description.split(". ").collect();
    let committed: Vec<&str> = sentences
        .iter()
        .copied()
        .filter(|sentence| sentence.contains("`.gitattributes`"))
        .collect();
    let per_clone: Vec<&str> = sentences
        .iter()
        .copied()
        .filter(|sentence| sentence.contains("`info/attributes`"))
        .collect();
    assert!(
        !committed.is_empty() && !per_clone.is_empty(),
        "the description names each file in a sentence:\n{description}"
    );
    for sentence in &committed {
        assert!(
            sentence.contains("`-merge` line to `.gitattributes`")
                && !sentence.contains("`info/attributes`")
                && !sentence.contains("driver"),
            "the sentence that names `.gitattributes` gives it the `-merge` line and \
             nothing that selects the driver:\n{sentence}"
        );
    }
    for sentence in &per_clone {
        assert!(
            sentence.contains("`info/attributes` lines that select the driver")
                && !sentence.contains("`.gitattributes`")
                && !sentence.contains("`-merge`"),
            "the sentence that names `info/attributes` gives it the lines that select \
             the driver and no `-merge` line:\n{sentence}"
        );
    }
}

/// The store `headwater new` appends a capture-cost reading to, and the one
/// `headwater taxonomy audit` appends an adoption reading to. Each line depends
/// on no other line, so a union of two branches is what either writer would
/// have written (`docs/interfaces/headwater-derived.md`).
const CAPTURE_COST: &str = ".headwater/capture-cost.jsonl";
const ADOPTION: &str = ".headwater/adoption.jsonl";

/// An adopter who ran `init --git` and then `headwater new` on two branches
/// merges them without a conflict on a file neither of them edited by hand.
///
/// Both branches create the capture-cost store from nothing, which is an
/// add/add conflict under a text merge. Branch `b` holds a decision the other
/// does not, so the two runs mint two identifiers, neither the claim store
/// (HW-DR-0054) nor a path collides, and the store is the only file in
/// question. Take the union line out of
/// `init --git` and the store is unmerged here.
#[test]
fn two_branches_that_each_run_new_merge_the_capture_cost_store_without_a_conflict() {
    let tree = Tree::adopted("two-news");
    tree.headwater_ok(&["init", "--git"]);
    tree.git(&["add", "-A"]);
    tree.git(&["commit", "-q", "-m", "adopted"]);

    tree.git(&["checkout", "-q", "-b", "a"]);
    tree.headwater_ok(&["new", "decision", "--title", "A ruling on branch a"]);
    tree.git(&["add", "-A"]);
    tree.git(&["commit", "-q", "-m", "a ruling"]);

    tree.git(&["checkout", "-q", "main"]);
    tree.git(&["checkout", "-q", "-b", "b"]);
    // A decision already on `b` moves its allocator past `a`'s, so the two
    // runs mint two identifiers and the claim store does not collide.
    tree.decide("0005");
    tree.git(&["add", "-A"]);
    tree.git(&["commit", "-q", "-m", "a decision by hand"]);
    tree.headwater_ok(&["new", "decision", "--title", "A ruling on branch b"]);
    tree.git(&["add", "-A"]);
    tree.git(&["commit", "-q", "-m", "a ruling"]);

    let merged = tree.git_output(&["merge", "--no-edit", "a"]);
    assert_eq!(
        (merged.status.code(), tree.unmerged()),
        (Some(0), Vec::<String>::new()),
        "the merge of two branches that each ran `new` exits 0 with nothing unmerged:\n{}{}",
        String::from_utf8_lossy(&merged.stdout),
        String::from_utf8_lossy(&merged.stderr)
    );
    let store = tree.read(CAPTURE_COST);
    let readings: Vec<&str> = store.lines().collect();
    assert_eq!(
        readings.len(),
        2,
        "the store holds one reading from each branch:\n{store}"
    );
    assert!(
        readings
            .iter()
            .any(|line| line.contains("a-ruling-on-branch-a"))
            && readings
                .iter()
                .any(|line| line.contains("a-ruling-on-branch-b")),
        "one reading names each document:\n{store}"
    );
}

/// The union lines are appended only where `.gitattributes` says nothing of the
/// store, and they never reach the override that selects the driver.
///
/// An adopter who already declared one store `-merge` and the other
/// `merge=union` keeps both lines byte for byte, and gets no line after either
/// that would win or repeat it. The step reads any treatment as a declaration,
/// not only `-merge`. Under `--git-config` the override names the lock alone,
/// because a driver line on a store would hand a file no producer writes to the
/// regenerate driver.
#[test]
fn the_git_step_leaves_a_declared_store_alone_and_writes_no_override_for_a_store() {
    let tree = Tree::adopted("declared-store");
    let own = format!("# the adopter's own lines\n{CAPTURE_COST} -merge\n{ADOPTION} merge=union\n");
    tree.write(".gitattributes", &own);
    tree.headwater_ok(&["init", "--git", "--git-config"]);

    let attributes = tree.read(".gitattributes");
    assert!(
        attributes.starts_with(&own),
        "the adopter's lines are kept byte for byte:\n{attributes}"
    );
    for (store, line) in [
        (CAPTURE_COST, format!("{CAPTURE_COST} -merge")),
        (ADOPTION, format!("{ADOPTION} merge=union")),
    ] {
        let naming: Vec<&str> = attributes
            .lines()
            .filter(|seen| seen.starts_with(store))
            .collect();
        assert_eq!(
            naming,
            vec![line.as_str()],
            "the adopter's line is the one line that names {store}:\n{attributes}"
        );
    }

    let over = std::fs::read_to_string(tree.info_attributes()).expect("the override is written");
    for store in [CAPTURE_COST, ADOPTION] {
        assert!(
            !over.contains(store),
            "the override names no store:\n{over}"
        );
    }
    assert!(
        over.contains(&format!("{LOCK} merge=headwater-regenerate")),
        "the override still names the lock:\n{over}"
    );
}

/// Every form of a hand-written merge line for a store, in any file and by any
/// pattern, keeps its effect through `init --git`, with the store absent and
/// with it present.
///
/// Git obeys the later of two lines for a path, so a union line appended after
/// an adopter's `merge=ours` overrides it with exit 0 and no message. The step
/// asks git whether anything names the store's merge attribute, which reads a
/// glob, a nested file, `!merge` and a store that does not exist yet as the
/// merge itself does. Read the root file's literal `-merge` lines alone, and
/// the rows other than those go red.
#[test]
fn the_git_step_leaves_every_form_of_a_merge_line_for_a_store_in_effect() {
    let rows: [(&str, &str, &str); 9] = [
        (
            "ours",
            ".gitattributes",
            ".headwater/capture-cost.jsonl merge=ours\n",
        ),
        (
            "text",
            ".gitattributes",
            ".headwater/capture-cost.jsonl merge=text\n",
        ),
        (
            "bang",
            ".gitattributes",
            ".headwater/capture-cost.jsonl !merge\n",
        ),
        (
            "set",
            ".gitattributes",
            ".headwater/capture-cost.jsonl merge\n",
        ),
        ("glob", ".gitattributes", ".headwater/*.jsonl -merge\n"),
        (
            "anchored",
            ".gitattributes",
            "/.headwater/capture-cost.jsonl -merge\n",
        ),
        (
            "binary",
            ".gitattributes",
            ".headwater/capture-cost.jsonl binary\n",
        ),
        (
            "nested",
            ".headwater/.gitattributes",
            "capture-cost.jsonl merge=ours\n",
        ),
        (
            "nested-bang",
            ".headwater/.gitattributes",
            "capture-cost.jsonl !merge\n",
        ),
    ];
    let mut wrong: Vec<String> = Vec::new();
    for (label, file, line) in rows {
        for present in [false, true] {
            let tree = Tree::adopted(&format!("form-{label}-{present}"));
            if present {
                tree.write(CAPTURE_COST, "{\"a\":1}\n");
            }
            tree.write(file, line);
            let before = tree.git(&["check-attr", "merge", "--", CAPTURE_COST]);
            tree.headwater_ok(&["init", "--git"]);
            let after = tree.git(&["check-attr", "merge", "--", CAPTURE_COST]);
            let root = tree.read(".gitattributes");
            let appended = root.contains(&format!("{CAPTURE_COST} merge=union"));
            let second = tree.headwater_ok(&["init", "--git"]);
            let settled = String::from_utf8_lossy(&second.stdout).contains("already declares all");
            if after != before || appended || !settled {
                wrong.push(format!(
                    "{label} (`{}` in {file}, store present: {present}): before `{}`, after \
                     `{}`, union line appended: {appended}, second run wrote nothing: {settled}",
                    line.trim_end(),
                    before.trim_end(),
                    after.trim_end()
                ));
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "the adopter's line keeps its effect, the step writes no union line for the store, \
         and a second run writes nothing:\n{}",
        wrong.join("\n")
    );
}

/// Two branches that each record an adoption reading merge without a conflict
/// on the adoption store, and it holds both readings.
///
/// `headwater taxonomy audit --record` appends one line to
/// `.headwater/adoption.jsonl`. Both branches create it from nothing, which is
/// an add/add conflict under a text merge. Drop the adoption store from the
/// union lines, and this case goes red.
#[test]
fn two_branches_that_each_record_an_audit_merge_the_adoption_store_without_a_conflict() {
    let tree = Tree::adopted("two-audits");
    tree.headwater_ok(&["init", "--git"]);
    tree.git(&["add", "-A"]);
    tree.git(&["commit", "-q", "-m", "adopted"]);

    tree.git(&["checkout", "-q", "-b", "a"]);
    tree.headwater_ok(&["taxonomy", "audit", "--record", "--now", "2026-01-01"]);
    tree.git(&["add", "-A"]);
    tree.git(&["commit", "-q", "-m", "a reading"]);

    tree.git(&["checkout", "-q", "main"]);
    tree.git(&["checkout", "-q", "-b", "b"]);
    tree.headwater_ok(&["taxonomy", "audit", "--record", "--now", "2026-02-02"]);
    tree.git(&["add", "-A"]);
    tree.git(&["commit", "-q", "-m", "b reading"]);

    let merged = tree.git_output(&["merge", "--no-edit", "a"]);
    assert_eq!(
        (merged.status.code(), tree.unmerged()),
        (Some(0), Vec::<String>::new()),
        "the merge of two branches that each recorded an audit exits 0 with nothing unmerged:\n{}{}",
        String::from_utf8_lossy(&merged.stdout),
        String::from_utf8_lossy(&merged.stderr)
    );
    let store = tree.read(ADOPTION);
    let readings: Vec<&str> = store.lines().collect();
    assert_eq!(
        readings.len(),
        2,
        "the store holds one reading from each branch:\n{store}"
    );
    assert!(
        readings.iter().any(|line| line.contains("2026-01-01"))
            && readings.iter().any(|line| line.contains("2026-02-02")),
        "one reading carries each date:\n{store}"
    );
}

/// With `--root` a subdirectory of the repository, each store gets its union
/// line unless a line there already names it, with the store absent and present.
///
/// Git reads the paths of an attributes file that no directory holds from the
/// top of the work tree, so a check that spelled the store from `--root` would
/// never match it in this tree. The step must then not read "no answer" as "a
/// line names the store" and write nothing in silence.
#[test]
fn below_the_top_of_the_repository_the_git_step_writes_each_store_line_it_owes() {
    let mut wrong: Vec<String> = Vec::new();
    for owned in [None, Some(CAPTURE_COST), Some(ADOPTION)] {
        for present in [false, true] {
            let label = format!(
                "below-{}-{present}",
                owned.map_or("none", |store| if store == ADOPTION {
                    "adoption"
                } else {
                    "cost"
                })
            );
            let top = std::env::temp_dir().join(format!(
                "headwater-cli-merge-driver-{}-{label}",
                std::process::id()
            ));
            let _ = std::fs::remove_dir_all(&top);
            let _top = Tree { at: top.clone() };
            let tree = Tree::adopted_in(top.join("handbook"), &top);
            for store in [CAPTURE_COST, ADOPTION] {
                if present {
                    tree.write(store, "{\"a\":1}\n");
                }
            }
            if let Some(store) = owned {
                tree.write(".gitattributes", &format!("{store} merge=ours\n"));
            }
            let output = tree.headwater(&["init", "--git"]);
            let attributes =
                std::fs::read_to_string(tree.at.join(".gitattributes")).unwrap_or_default();
            for store in [CAPTURE_COST, ADOPTION] {
                let answer = tree.git(&["check-attr", "merge", "--", store]);
                let expected = if owned == Some(store) {
                    "ours"
                } else {
                    "union"
                };
                let lines = attributes
                    .lines()
                    .filter(|line| line.starts_with(store))
                    .count();
                if !answer.trim_end().ends_with(&format!(": {expected}")) || lines != 1 {
                    wrong.push(format!(
                        "{label}, {store}: git answers `{}`, {lines} lines name it, exit {:?}",
                        answer.trim_end(),
                        output.status.code()
                    ));
                }
            }
            if output.status.code() != Some(0) {
                wrong.push(format!(
                    "{label}: exit {:?}:\n{}",
                    output.status.code(),
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "below the top of the repository, each store the file does not name gets its union \
         line, and a named store keeps its line:\n{}",
        wrong.join("\n")
    );
}

/// A corpus below a directory whose name holds a character that means
/// something in an attributes file gets both union lines, and a line of its own
/// for a store is left alone, with exit 0.
///
/// The probe line is C-quoted with its glob characters escaped, so it names the
/// store as it is spelled. Spell it unquoted, and a space ends the pattern, a
/// `[1]` matches `1`, and the probe matches nothing: the step fails closed and
/// exits 1 on every run in such a tree.
#[test]
fn below_a_directory_with_a_special_name_the_git_step_probes_the_store_as_spelled() {
    let names = [
        ("space", "my corpus"),
        ("hash", "#notes"),
        ("bang", "!bang"),
        ("bracket", "br[1]"),
        ("star", "st*r"),
        ("quote", "quo\"te"),
        ("backslash", "back\\slash"),
        ("newline", "new\nline"),
    ];
    let mut wrong: Vec<String> = Vec::new();
    for (label, name) in names {
        for owned in [false, true] {
            let top = std::env::temp_dir().join(format!(
                "headwater-cli-merge-driver-{}-named-{label}-{owned}",
                std::process::id()
            ));
            remove_all(&top);
            let _top = Tree { at: top.clone() };
            let tree = Tree::adopted_in(top.join(name), &top);
            if owned {
                tree.write(".gitattributes", &format!("{CAPTURE_COST} merge=ours\n"));
            }
            let output = tree.headwater(&["init", "--git"]);
            let attributes =
                std::fs::read_to_string(tree.at.join(".gitattributes")).unwrap_or_default();
            for store in [CAPTURE_COST, ADOPTION] {
                let expected = if owned && store == CAPTURE_COST {
                    "ours"
                } else {
                    "union"
                };
                let answer = tree.git(&["check-attr", "merge", "--", store]);
                let lines = attributes
                    .lines()
                    .filter(|line| line.starts_with(store))
                    .count();
                if !answer.trim_end().ends_with(&format!(": {expected}")) || lines != 1 {
                    wrong.push(format!(
                        "{label}, owned {owned}, {store}: git answers `{}`, {lines} lines name it",
                        answer.trim_end()
                    ));
                }
            }
            if output.status.code() != Some(0) {
                wrong.push(format!(
                    "{label}, owned {owned}: exit {:?}: {}",
                    output.status.code(),
                    String::from_utf8_lossy(&output.stderr).trim_end()
                ));
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "below each directory name, the step writes the union line each store is owed, leaves \
         the adopter's line alone and exits 0:\n{}",
        wrong.join("\n")
    );
}

/// Where git is there and the probe gets no answer, the step exits 1, names
/// why, writes no store line, and says truthfully on a second run that it
/// wrote none.
///
/// The probe file goes in the temporary directory, so a `TMPDIR` that does not
/// exist is a probe with no answer. Fall back to the root file in silence
/// there, and the union lines appear and the exit is 0. Count an unanswered
/// store as declared, and the second run says `.gitattributes` declares it.
#[test]
fn a_probe_with_no_answer_exits_1_writes_no_store_line_and_says_so_again() {
    let tree = Tree::adopted("no-answer");
    let missing = tree.at.join("no-such-temporary-directory");
    let run = || {
        Command::new(binary())
            .args(["init", "--git", "--root"])
            .arg(&tree.at)
            .env("TMPDIR", &missing)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env_remove("GIT_DIR")
            .env_remove("GIT_INDEX_FILE")
            .env_remove("GIT_WORK_TREE")
            .output()
            .expect("the binary runs")
    };
    for attempt in ["first", "second"] {
        let output = run();
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        assert_eq!(
            output.status.code(),
            Some(1),
            "the {attempt} run exits 1:\n{stdout}{stderr}"
        );
        assert!(
            stderr.contains("wrote no line for the append-only stores")
                && stderr.contains("cannot write the probe file"),
            "the {attempt} run names why on standard error:\n{stderr}"
        );
        let attributes = tree.read(".gitattributes");
        for store in [CAPTURE_COST, ADOPTION] {
            assert!(
                !attributes.contains(store),
                "the {attempt} run writes no line for {store}:\n{attributes}"
            );
        }
        assert!(
            !stdout.contains("append-only stores\n") && !stdout.contains("and append-only stores"),
            "the {attempt} run does not count the stores as declared:\n{stdout}"
        );
        assert!(
            stdout.contains("no line for the 2 append-only stores"),
            "the {attempt} run says on standard output that it wrote no store line:\n{stdout}"
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

    let empty = Scratch::made(tree.at.with_extension("empty-path"));
    let output = Command::new(binary())
        .args(["derived", "--root"])
        .arg(&tree.at)
        .env("PATH", &empty.0)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env_remove("GIT_DIR")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_WORK_TREE")
        .output()
        .expect("the binary runs");
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

    let empty = Scratch::made(tree.at.with_extension("empty-path-no-repository"));
    let (code, said) = derived_with(&tree, Some(&empty.0), &[]);
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
        remove_all(&self.0);
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
        // The first run writes the union lines of the two append-only stores,
        // which the nested file does not name. The second finds everything
        // declared.
        assert!(
            !stdout.contains(&format!("{LOCK} -merge")),
            "the {run} run reports no root line for the lock:\n{stdout}"
        );
        if run == "second" {
            assert!(
                stdout.contains("already declares all"),
                "the {run} run finds every derived artifact declared:\n{stdout}"
            );
        }
        let root = std::fs::read_to_string(tree.at.join(".gitattributes")).unwrap_or_default();
        assert!(
            !root.contains("taxonomy.lock"),
            "the {run} run wrote a root line for the lock, which the nested file declares:\n{root}"
        );
    }
}

/// `init --git` inside a repository whose git does not run names the refusal,
/// writes no root line that the unread nested file could already declare, and
/// exits 1.
///
/// This is the case above with git gone from `PATH`. The census falls back to
/// the root `.gitattributes` alone, which does not read
/// `.headwater/.gitattributes`, so a step that trusted the fallback appended a
/// duplicate lock line and exited 0 in silence (#1119). Drop the refusal print
/// or the nested-file filter, and this case fails.
#[test]
fn the_git_step_without_git_names_the_refusal_and_writes_no_line_a_nested_file_could_declare() {
    let tree = Tree::adopted("nested-no-git");
    tree.write(".headwater/.gitattributes", "taxonomy.lock -merge\n");
    let empty = Scratch::made(tree.at.with_extension("nested-no-git-empty-path"));
    for run in ["first", "second"] {
        let output = Command::new(binary())
            .args(["init", "--git", "--root"])
            .arg(&tree.at)
            .env("PATH", &empty.0)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env_remove("GIT_DIR")
            .env_remove("GIT_INDEX_FILE")
            .env_remove("GIT_WORK_TREE")
            .output()
            .expect("the binary runs");
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        assert!(
            stderr.contains("git did not run"),
            "the {run} run names that git did not run, on standard error:\n{stderr}\n--- stdout\n{stdout}"
        );
        assert!(
            stderr.contains(".headwater/.gitattributes"),
            "the {run} run names the nested file it could not read:\n{stderr}"
        );
        let root = std::fs::read_to_string(tree.at.join(".gitattributes")).unwrap_or_default();
        assert!(
            !root.contains("taxonomy.lock"),
            "the {run} run wrote a root line for the lock, which the unread nested file declares:\n{root}"
        );
        assert_eq!(
            output.status.code(),
            Some(1),
            "the {run} run exits 1, because git did not give the merge attributes:\n{stderr}"
        );
    }
}

/// A producer that only this repository holds is neither named by
/// `headwater derived` nor written by `init --git` in an adopter's tree.
///
/// `derived` knows three producers, and one of them is a blessing run that only
/// the repository maintaining this engine holds. The row below plants a file
/// that producer would claim here, and the file whose presence makes a tree
/// hold it. Until #1273 a second row planted a page under `site/` for the
/// figure refresh, which is no longer a producer. In the adopter's tree the file is claimed by nobody,
/// so the report names neither the file nor the command, and `init --git`
/// writes no line. Once the tree holds the producer, the same file is claimed
/// and written, which is what makes the first half a measurement of the
/// predicate rather than of a filter that admits nothing.
#[test]
fn the_git_step_writes_no_line_for_a_producer_the_adopter_does_not_hold() {
    let rows: [(&str, &str, &str, &str, &str); 1] = [(
        "blessing-producer",
        "engine/crates/a/fixtures/corpus.a",
        "426 files\nsha256:0a1b\n",
        "HEADWATER_BLESS=1 cargo test",
        "engine/Cargo.toml",
    )];
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
    let outer = Outer(std::env::temp_dir().join(format!(
        "headwater-cli-merge-driver-{}-bare",
        std::process::id()
    )));
    let dir = &outer.0;
    let _ = std::fs::remove_dir_all(dir);
    std::fs::create_dir_all(dir).expect("the directory is made");
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
}
