// SPDX-License-Identifier: Apache-2.0
//! The one crate of this workspace that runs a version control command.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#temporal-inputs-the-clock-and-the-prior-version)
//! and [`headwater_check::change`] both state the boundary this crate sits
//! outside of: the check-evaluation path "runs no version control command, and
//! opens no file that the [manifest] does not name". A change manifest still
//! has to come from somewhere, and until this crate existed the only producer
//! was `.githooks/change-manifest`, a shell script that belonged to this
//! repository alone. [HW-DR-0072](../../../../docs/decisions/0072-the-binary-is-the-only-interface-an-adopter-must-run-and-every-integration-point-outside-it-is-declared.md)
//! rules that an adopter reaches the whole governed loop through the binary,
//! and names the missing producer as the one place that promise failed.
//!
//! [`produce`] is the whole of what moves. It is the git plumbing the shell
//! script ran, carried into a library so `headwater change` can be the verb
//! that runs it, and the git commands it shells out to are exactly the ones
//! the hook already ran: `rev-parse`, `diff --name-status`, `show`,
//! `ls-files`. Nothing here reads a corpus, resolves a taxonomy, or parses a
//! document; that reading happens once the manifest this crate writes reaches
//! [`headwater_check::change::Unbound::at`].
//!
//! [manifest]: https://github.com/headwater-ai/headwater/blob/main/docs/spec/12-check-layer.md#temporal-inputs-the-clock-and-the-prior-version

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The manifest header this crate writes, and the one [`headwater_check::change::FORMAT`]
/// reads back. Held here too, rather than as a dependency on that crate, so
/// this crate stays what its module comment claims: the git plumbing, and
/// nothing that reads a corpus.
const FORMAT: &str = "headwater change 1";

/// Write a change manifest for `base..working tree`, anchored at `root`, into
/// `out`.
///
/// `out/manifest` is the file `headwater check --change` reads, and
/// `out/prior/<n>` holds the bytes of the `n`th prior version the manifest
/// names. The caller owns `out` and removes it; this function only ever
/// creates inside it. The path of the manifest is the return value, which is
/// what a caller prints — `.githooks/change-manifest` used to print it on
/// standard output, and `headwater change` does the same.
///
/// `root` is resolved to its git top-level first, the way the shell script
/// resolved it from its own working directory, so a `--root` that names a
/// subdirectory of a larger repository still anchors every path this function
/// writes at the repository root rather than at the subdirectory.
pub fn produce(root: &Path, base: &str, out: &Path) -> Result<PathBuf, String> {
    let toplevel = show_toplevel(root)?;

    if !commit_exists(&toplevel, base) {
        return Err(format!(
            "this repository does not hold `{base}`, so there is no prior version to name. A \
             shallow clone is the usual cause."
        ));
    }

    let prior_dir = out.join("prior");
    fs::create_dir_all(&prior_dir).map_err(|error| format!("{}: {error}", prior_dir.display()))?;
    let manifest_path = out.join("manifest");
    let mut manifest = format!("{FORMAT}\n");
    let mut priors = 0usize;

    let mut emit = |status: Status, old: &str, new: &str| -> Result<(), String> {
        if new.contains('\t') {
            return Err(format!(
                "`{new}` holds a tab and a manifest line is tab separated, so this change cannot \
                 be described without corrupting it."
            ));
        }
        if status == Status::Added {
            manifest.push_str(&format!("added\t{new}\n"));
            return Ok(());
        }
        let source_path = if old.is_empty() { new } else { old };
        priors += 1;
        let file = prior_dir.join(priors.to_string());
        let bytes = show(&toplevel, base, source_path).ok_or_else(|| {
            format!(
                "`{base}:{source_path}` did not read, and the engine is told which files moved \
                 rather than what they became. Nothing is written."
            )
        })?;
        fs::write(&file, bytes).map_err(|error| format!("{}: {error}", file.display()))?;
        manifest.push_str(&format!("prior\t{new}\t{}\n", file.display()));
        Ok(())
    };

    for change in diff_name_status(&toplevel, base)? {
        match change {
            Changed::Renamed { old, new } => emit(Status::Modified, &old, &new)?,
            Changed::Added { path } => emit(Status::Added, "", &path)?,
            Changed::Other { path } => emit(Status::Modified, "", &path)?,
        }
    }
    for path in untracked(&toplevel) {
        emit(Status::Added, "", &path)?;
    }

    fs::write(&manifest_path, manifest)
        .map_err(|error| format!("{}: {error}", manifest_path.display()))?;
    Ok(manifest_path)
}

#[derive(PartialEq, Eq)]
enum Status {
    Added,
    Modified,
}

/// One entry of `git diff --name-status`, read into the three shapes the
/// producer treats differently. A rename or a copy carries the path the prior
/// version stood at; every other status is read at one path, the way the
/// shell script's state machine folds `M`, `D`, `T` and everything else it
/// does not special-case into one arm.
enum Changed {
    Renamed { old: String, new: String },
    Added { path: String },
    Other { path: String },
}

fn show_toplevel(root: &Path) -> Result<PathBuf, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|error| format!("git did not run: {error}"))?;
    if !output.status.success() {
        return Err("not inside a git repository.".to_string());
    }
    let text = String::from_utf8_lossy(&output.stdout);
    Ok(PathBuf::from(text.trim()))
}

fn commit_exists(root: &Path, base: &str) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args([
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("{base}^{{commit}}"),
        ])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn show(root: &Path, base: &str, path: &str) -> Option<Vec<u8>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("show")
        .arg(format!("{base}:{path}"))
        .output()
        .ok()?;
    output.status.success().then_some(output.stdout)
}

/// `git diff --name-status --find-renames -z <base> --`, parsed the way the
/// shell script's `emit` state machine parsed it: `-z` so a path holding a
/// special character reaches this function unquoted, and NUL as the one
/// separator left, on the terms the script's own comment states.
fn diff_name_status(root: &Path, base: &str) -> Result<Vec<Changed>, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["diff", "--name-status", "--find-renames", "-z", base, "--"])
        .output()
        .map_err(|error| format!("git did not run: {error}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    let fields: Vec<String> = split_nul(&output.stdout);
    let mut changes = Vec::new();
    let mut index = 0;
    while index < fields.len() {
        let status = fields[index].clone();
        index += 1;
        let renamed = status.starts_with('R') || status.starts_with('C');
        if renamed {
            let old = fields
                .get(index)
                .cloned()
                .ok_or_else(|| "a rename entry named no prior path".to_string())?;
            index += 1;
            let new = fields
                .get(index)
                .cloned()
                .ok_or_else(|| "a rename entry named no new path".to_string())?;
            index += 1;
            changes.push(Changed::Renamed { old, new });
        } else {
            let path = fields
                .get(index)
                .cloned()
                .ok_or_else(|| "a diff entry named no path".to_string())?;
            index += 1;
            if status == "A" {
                changes.push(Changed::Added { path });
            } else {
                changes.push(Changed::Other { path });
            }
        }
    }
    Ok(changes)
}

/// `git ls-files --others --exclude-standard -z`: the files the working tree
/// holds that the index does not, which `git diff` never reports because it
/// reads what the index tracks. Disjoint from the set above by construction.
fn untracked(root: &Path) -> Vec<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "--others", "--exclude-standard", "-z"])
        .output();
    match output {
        Ok(output) if output.status.success() => split_nul(&output.stdout),
        _ => Vec::new(),
    }
}

/// Every path under `root` that git's ignore rules exclude, read with
/// `git ls-files --others --ignored --exclude-standard --directory -z`: a
/// whole directory collapses to one entry ending in `/` where everything
/// under it is ignored, and an individual file is listed on its own where
/// only it is. Paths come back relative to `root`, the same posture
/// [`untracked`] takes and for the same reason: `-C root` makes `root` the
/// working directory the command's own path output is relative to.
///
/// Empty where `root` is not inside a git repository — no ignore rule binds a
/// tree git does not see — on the same "nothing to report" posture
/// [`untracked`] takes rather than an error, so a caller with no repository at
/// all reads that as nothing ignored instead of failing.
pub fn ignored(root: &Path) -> Vec<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args([
            "ls-files",
            "--others",
            "--ignored",
            "--exclude-standard",
            "--directory",
            "-z",
        ])
        .output();
    match output {
        Ok(output) if output.status.success() => split_nul(&output.stdout),
        _ => Vec::new(),
    }
}

fn split_nul(bytes: &[u8]) -> Vec<String> {
    bytes
        .split(|byte| *byte == 0)
        .filter(|field| !field.is_empty())
        .map(|field| String::from_utf8_lossy(field).into_owned())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A repository this crate's own tests build, rather than one under
    /// `fixtures/`: what is under test is the git plumbing, and a fixture
    /// tree with no `.git` cannot exercise it. `tempfile` is not a dependency
    /// of this crate, so each case makes and tears down its own directory
    /// under `std::env::temp_dir()`, keyed by the test name and the process
    /// id for the reason `engine/crates/cli/tests/wiring.rs` states: cargo
    /// runs the cases of one target as threads of one process.
    struct Repo {
        at: PathBuf,
    }

    impl Repo {
        fn new(label: &str) -> Repo {
            let at = std::env::temp_dir().join(format!(
                "headwater-vcs-tests-{label}-{}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&at);
            fs::create_dir_all(&at).expect("the repo directory is there");
            let repo = Repo { at };
            repo.git(&["init", "-q"]);
            repo.git(&["config", "user.name", "fixtures"]);
            repo.git(&["config", "user.email", "fixtures@invalid"]);
            repo
        }

        fn git(&self, args: &[&str]) -> String {
            let output = Command::new("git")
                .arg("-C")
                .arg(&self.at)
                .args(args)
                .output()
                .expect("git runs");
            assert!(
                output.status.success(),
                "git {:?} failed: {}",
                args,
                String::from_utf8_lossy(&output.stderr)
            );
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        }

        fn write(&self, path: &str, text: &str) {
            let full = self.at.join(path);
            if let Some(parent) = full.parent() {
                fs::create_dir_all(parent).expect("the parent directory is there");
            }
            fs::write(full, text).expect("the file writes");
        }

        fn commit(&self, message: &str) -> String {
            self.git(&["add", "-A"]);
            self.git(&["commit", "-q", "-m", message]);
            self.git(&["rev-parse", "HEAD"])
        }

        fn out(&self, label: &str) -> PathBuf {
            let out = std::env::temp_dir().join(format!(
                "headwater-vcs-tests-{label}-out-{}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&out);
            out
        }
    }

    fn read(path: &Path) -> String {
        fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
    }

    /// An add and a modify, held against a hand-written manifest.
    ///
    /// This is the case Done-when item 2 of #929 asks for: a known change, and
    /// a wrong `added` line or a dropped `prior` line fails it.
    #[test]
    fn a_known_add_and_modify_produces_the_hand_written_manifest() {
        let repo = Repo::new("add-and-modify");
        repo.write("a.md", "before\n");
        let base = repo.commit("base");
        repo.write("a.md", "after\n");
        repo.write("b.md", "new\n");

        let out = repo.out("add-and-modify");
        let manifest_path = produce(&repo.at, &base, &out).expect("the manifest writes");
        let manifest = read(&manifest_path);
        let mut lines: Vec<&str> = manifest.lines().collect();
        lines.sort_unstable();
        let mut want = vec![FORMAT, "added\tb.md"];
        // The prior file's own name is an implementation detail (`priors`
        // counts up from 1), so this reads it back rather than hard-coding
        // `prior/1`, and confirms the bytes are the ones that stood at `base`.
        let prior_line = manifest
            .lines()
            .find(|line| line.starts_with("prior\t"))
            .expect("one prior line");
        let mut fields = prior_line.split('\t');
        assert_eq!(fields.next(), Some("prior"));
        assert_eq!(fields.next(), Some("a.md"));
        let prior_file = fields.next().expect("a third field");
        assert_eq!(fields.next(), None);
        assert_eq!(read(Path::new(prior_file)), "before\n");
        want.push(prior_line);
        want.sort_unstable();
        assert_eq!(lines, want, "{manifest}");

        let _ = fs::remove_dir_all(&out);
        let _ = fs::remove_dir_all(&repo.at);
    }

    /// A base revision this clone does not hold is refused, not skipped.
    #[test]
    fn a_base_revision_this_repository_does_not_hold_is_refused() {
        let repo = Repo::new("missing-base");
        repo.write("a.md", "x\n");
        repo.commit("base");
        let out = repo.out("missing-base");
        let error =
            produce(&repo.at, "0000000000000000000000000000000000000000", &out).unwrap_err();
        assert!(error.contains("does not hold"), "{error}");
        let _ = fs::remove_dir_all(&out);
        let _ = fs::remove_dir_all(&repo.at);
    }

    /// A path a manifest line cannot carry is refused before anything is
    /// written, the same way the shell script refused it.
    #[test]
    fn a_path_holding_a_tab_is_refused() {
        let repo = Repo::new("tab-path");
        repo.write("a.md", "x\n");
        let base = repo.commit("base");
        repo.write("a\tb.md", "y\n");
        let out = repo.out("tab-path");
        let error = produce(&repo.at, &base, &out).unwrap_err();
        assert!(error.contains("holds a tab"), "{error}");
        let _ = fs::remove_dir_all(&out);
        let _ = fs::remove_dir_all(&repo.at);
    }

    /// A deleted document is named as a `prior` line, on the terms
    /// `headwater_check::change` documents: a departure and a `prior` line for
    /// an edit are one grammar, and content is what tells them apart later.
    #[test]
    fn a_deleted_document_is_named_with_its_prior_version() {
        let repo = Repo::new("deleted");
        repo.write("gone.md", "was here\n");
        let base = repo.commit("base");
        fs::remove_file(repo.at.join("gone.md")).expect("the file removes");
        let out = repo.out("deleted");
        let manifest_path = produce(&repo.at, &base, &out).expect("the manifest writes");
        let manifest = read(&manifest_path);
        let prior_line = manifest
            .lines()
            .find(|line| line.starts_with("prior\tgone.md\t"))
            .unwrap_or_else(|| panic!("a prior line naming gone.md:\n{manifest}"));
        let file = prior_line.rsplit('\t').next().expect("a file field");
        assert_eq!(read(Path::new(file)), "was here\n");
        let _ = fs::remove_dir_all(&out);
        let _ = fs::remove_dir_all(&repo.at);
    }

    /// A rename is one document at a new path, and its prior version is read
    /// from the old one.
    #[test]
    fn a_rename_is_named_at_its_new_path_with_the_old_path_as_the_prior_source() {
        let repo = Repo::new("renamed");
        repo.write("old.md", "text that survives the move\n");
        let base = repo.commit("base");
        repo.git(&["mv", "old.md", "new.md"]);
        let out = repo.out("renamed");
        let manifest_path = produce(&repo.at, &base, &out).expect("the manifest writes");
        let manifest = read(&manifest_path);
        let prior_line = manifest
            .lines()
            .find(|line| line.starts_with("prior\tnew.md\t"))
            .unwrap_or_else(|| panic!("a prior line naming new.md:\n{manifest}"));
        assert!(
            !manifest.contains("old.md"),
            "the old path is not itself a named entry:\n{manifest}"
        );
        let file = prior_line.rsplit('\t').next().expect("a file field");
        assert_eq!(read(Path::new(file)), "text that survives the move\n");
        let _ = fs::remove_dir_all(&out);
        let _ = fs::remove_dir_all(&repo.at);
    }

    /// An untracked file — one `git diff` never reports — still reaches the
    /// manifest as added.
    #[test]
    fn an_untracked_file_is_named_as_added() {
        let repo = Repo::new("untracked");
        repo.write("a.md", "x\n");
        let base = repo.commit("base");
        repo.write("new-and-untracked.md", "y\n");
        let out = repo.out("untracked");
        let manifest_path = produce(&repo.at, &base, &out).expect("the manifest writes");
        let manifest = read(&manifest_path);
        assert!(
            manifest.contains("added\tnew-and-untracked.md\n"),
            "{manifest}"
        );
        let _ = fs::remove_dir_all(&out);
        let _ = fs::remove_dir_all(&repo.at);
    }

    /// A tree that moved nothing since `base` is a change that names nothing:
    /// the manifest holds the header alone.
    #[test]
    fn a_tree_that_moved_nothing_names_nothing() {
        let repo = Repo::new("no-change");
        repo.write("a.md", "x\n");
        let base = repo.commit("base");
        let out = repo.out("no-change");
        let manifest_path = produce(&repo.at, &base, &out).expect("the manifest writes");
        assert_eq!(read(&manifest_path), format!("{FORMAT}\n"));
        let _ = fs::remove_dir_all(&out);
        let _ = fs::remove_dir_all(&repo.at);
    }

    /// A whole ignored directory collapses to one entry ending in `/`, and an
    /// individually ignored file, not itself a whole directory, is named on
    /// its own.
    #[test]
    fn a_whole_ignored_directory_collapses_and_a_single_ignored_file_stands_alone() {
        let repo = Repo::new("ignored");
        repo.write(".gitignore", "build/\nnotes.local.md\n");
        repo.write("a.md", "x\n");
        repo.commit("base");
        repo.write("build/output.txt", "generated\n");
        repo.write("build/nested/deep.txt", "generated too\n");
        repo.write("notes.local.md", "scratch\n");

        let mut found = ignored(&repo.at);
        found.sort();
        assert_eq!(found, vec!["build/".to_string(), "notes.local.md".to_string()]);
        let _ = fs::remove_dir_all(&repo.at);
    }

    /// A tree with no `.git` at all — the shape this crate's own tests build
    /// for every other case above, before `Repo::new` runs `git init` — reads
    /// as nothing ignored rather than failing, the same posture [`untracked`]
    /// takes for the same reason.
    #[test]
    fn a_tree_with_no_git_repository_reports_nothing_ignored() {
        let at = std::env::temp_dir().join(format!(
            "headwater-vcs-tests-no-repository-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&at);
        fs::create_dir_all(&at).expect("the directory is there");
        fs::write(at.join("a.md"), "x\n").expect("the file writes");
        assert_eq!(ignored(&at), Vec::<String>::new());
        let _ = fs::remove_dir_all(&at);
    }
}
