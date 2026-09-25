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

/// The `merge` attribute git gives each of `paths`, asked of git itself.
///
/// One run of `git check-attr -z --stdin merge`, with every path on standard
/// input, NUL-separated and relative to `root`. Git applies its own
/// precedence: every `.gitattributes` of the tree, the deeper file first,
/// then `$GIT_DIR/info/attributes`, and `core.attributesFile` from the
/// configuration of the clone. It expands its own patterns and macros, so a
/// caller reads the answer and never a pattern.
///
/// Each value is the one git prints: `unspecified`, `unset`, `set`, or the
/// name of a driver. The order of the result is the order of `paths`.
///
/// `None` where `root` is not inside a git work tree, the same posture
/// [`ignored`] takes: no attribute file binds a tree that git does not see,
/// and the caller then reads the declarations some other way.
///
/// `Some(Err(..))` where `root` is inside a work tree and git refused the
/// question, with what git printed. Git refuses the whole run for one path it
/// rejects, such as a path outside the work tree, so a caller that has to
/// ask about a path it did not find on disk filters such a path out first.
/// The refusal is kept apart from `None`, because a caller that read it as
/// "no repository" would fall back in silence inside one.
///
/// `Some(Err(..))` also where git cannot answer and `finds_a_repository`
/// finds one where git's own search would: git that does not run at all, git
/// that distrusts the owner of the repository ("dubious ownership"), and a
/// linked worktree whose git directory was pruned. The error carries what git
/// printed or why it did not run. Where git's search would find nothing, the
/// tree is one git does not see, and the answer is `None`: a `.git` file that
/// is not a gitfile, an empty `.git` directory, a repository beyond
/// `GIT_CEILING_DIRECTORIES` or beyond a filesystem boundary (#809).
pub fn merge_attributes(
    root: &Path,
    paths: &[String],
) -> Option<Result<Vec<(String, String)>, String>> {
    let inside = match Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
    {
        Ok(inside) => inside,
        Err(error) => {
            return finds_a_repository(root).then(|| Err(format!("git did not run: {error}")));
        }
    };
    if !inside.status.success() {
        return finds_a_repository(root).then(|| {
            Err(format!(
                "git did not answer whether this is a work tree: {}",
                String::from_utf8_lossy(&inside.stderr).trim()
            ))
        });
    }
    if String::from_utf8_lossy(&inside.stdout).trim() != "true" {
        return None;
    }
    let mut input = Vec::new();
    for path in paths {
        input.extend_from_slice(path.as_bytes());
        input.push(0);
    }
    Some(check_attr(root, input))
}

/// Whether git's own search upward from `root` would stop at a repository.
///
/// This is asked only after git failed to answer, so it cannot ask git. It
/// walks up from `root` the way git does and stops where git stops:
///
/// - A `.git` directory that holds `HEAD`, `objects` and `refs` is a
///   repository, even one git then refuses to open.
/// - A `.git` file that opens with `gitdir:` is a linked worktree or a
///   submodule, even where the directory it names was pruned.
/// - A `.git` file that is not a gitfile ends the search with no repository,
///   as git's "invalid gitfile format" does.
/// - A `.git` directory that is not a repository is passed over.
/// - The search does not go up into a directory that `GIT_CEILING_DIRECTORIES`
///   names, or across a filesystem boundary unless
///   `GIT_DISCOVERY_ACROSS_FILESYSTEM` is true.
///
/// A plain `.git` entry is not enough. Git says there is no repository in
/// each of the last three shapes, and a verb that refused there would refuse
/// a tree that git itself reads as no repository.
fn finds_a_repository(root: &Path) -> bool {
    let ceilings: Vec<PathBuf> = std::env::var_os("GIT_CEILING_DIRECTORIES")
        .map(|value| {
            std::env::split_paths(&value)
                .filter(|dir| !dir.as_os_str().is_empty())
                .map(|dir| dir.canonicalize().unwrap_or(dir))
                .collect()
        })
        .unwrap_or_default();
    let across =
        std::env::var("GIT_DISCOVERY_ACROSS_FILESYSTEM").is_ok_and(|value| git_bool(&value));
    search_upward(root, &ceilings, across, &device_of)
}

/// Whether git reads `value` of a boolean environment variable as true.
///
/// Git's own rule (`git_config_bool`): `true`, `yes` and `on` in any case are
/// true, and `false`, `no`, `off` and the empty value are false. Any other
/// value is an integer as `strtoimax` reads it in base 0, so `0x10` and `010`
/// are numbers, with an optional `k`, `m` or `g` unit. It is true where it is
/// not zero. Git refuses a value that is none of these, and this reads it as
/// false, so the search does not cross a boundary on it.
fn git_bool(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    match lower.as_str() {
        "true" | "yes" | "on" => return true,
        "false" | "no" | "off" | "" => return false,
        _ => {}
    }
    let number = lower.trim_start();
    let number = number.strip_suffix(['k', 'm', 'g']).unwrap_or(number);
    let digits = number.strip_prefix(['-', '+']).unwrap_or(number);
    let parsed = if let Some(hex) = digits.strip_prefix("0x") {
        u128::from_str_radix(hex, 16)
    } else if let Some(octal) = digits.strip_prefix('0').filter(|rest| !rest.is_empty()) {
        u128::from_str_radix(octal, 8)
    } else {
        digits.parse::<u128>()
    };
    parsed.is_ok_and(|number| number != 0)
}

/// [`finds_a_repository`] with the two environment settings passed in.
///
/// `device_of` names the filesystem a directory is on. It is passed in so
/// that a test can place a boundary where no test can mount one.
fn search_upward(
    root: &Path,
    ceilings: &[PathBuf],
    across: bool,
    device_of: &dyn Fn(&Path) -> Option<u64>,
) -> bool {
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let device = device_of(&root);
    for dir in root.ancestors() {
        if dir != root {
            if ceilings.iter().any(|ceiling| ceiling == dir) {
                return false;
            }
            if !across && device.is_some() && device_of(dir) != device {
                return false;
            }
        }
        let entry = dir.join(".git");
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        if meta.is_dir() {
            if entry.join("HEAD").is_file()
                && entry.join("objects").is_dir()
                && entry.join("refs").is_dir()
            {
                return true;
            }
        } else {
            return fs::read_to_string(&entry)
                .is_ok_and(|text| text.trim_start().starts_with("gitdir:"));
        }
    }
    false
}

/// The device a path is on, where the platform says.
#[cfg(unix)]
fn device_of(path: &Path) -> Option<u64> {
    use std::os::unix::fs::MetadataExt;
    fs::metadata(path).ok().map(|meta| meta.dev())
}

/// The device a path is on, where the platform says.
#[cfg(not(unix))]
fn device_of(_path: &Path) -> Option<u64> {
    None
}

/// One run of `git check-attr -z --stdin merge` over `input`, parsed.
fn check_attr(root: &Path, input: Vec<u8>) -> Result<Vec<(String, String)>, String> {
    let mut child = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["check-attr", "-z", "--stdin", "merge"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|error| format!("git did not run: {error}"))?;
    // Written from a second thread, so that a large input cannot block on a
    // full output pipe that nothing reads yet. A write that fails because git
    // exited early is not the error to report: git's own message is.
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| "git took no standard input".to_string())?;
    let writer = std::thread::spawn(move || {
        use std::io::Write;
        stdin.write_all(&input)
    });
    let output = child
        .wait_with_output()
        .map_err(|error| format!("git did not finish: {error}"))?;
    let written = writer.join();
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    match written {
        Ok(Ok(())) => {}
        _ => return Err("the paths did not reach git".to_string()),
    }
    // `-z` prints each answer as three fields: path, attribute, value.
    let fields: Vec<String> = output
        .stdout
        .split(|byte| *byte == 0)
        .map(|field| String::from_utf8_lossy(field).into_owned())
        .collect();
    Ok(fields
        .as_chunks::<3>()
        .0
        .iter()
        .map(|[path, _, value]| (path.clone(), value.clone()))
        .collect())
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
        assert_eq!(
            found,
            vec!["build/".to_string(), "notes.local.md".to_string()]
        );
        let _ = fs::remove_dir_all(&repo.at);
    }

    /// Git's answer for each path, in the order asked, with a nested file
    /// winning over the root and a driver name passed through as git prints it.
    #[test]
    fn merge_attributes_are_git_answers_in_the_order_asked() {
        let repo = Repo::new("merge-attributes");
        repo.write(
            ".gitattributes",
            "a.md merge=headwater-regenerate\nsub/b.md merge=headwater-regenerate\n",
        );
        repo.write("sub/.gitattributes", "b.md -merge\nc.md merge=ours\n");
        let paths: Vec<String> = ["sub/c.md", "a.md", "sub/b.md", "none.md"]
            .iter()
            .map(|path| path.to_string())
            .collect();
        let answers = merge_attributes(&repo.at, &paths)
            .expect("a repository")
            .expect("git answers");
        assert_eq!(
            answers,
            vec![
                ("sub/c.md".to_string(), "ours".to_string()),
                ("a.md".to_string(), "headwater-regenerate".to_string()),
                ("sub/b.md".to_string(), "unset".to_string()),
                ("none.md".to_string(), "unspecified".to_string()),
            ]
        );
        let _ = fs::remove_dir_all(&repo.at);
    }

    /// Inside a repository, a path git rejects is a refusal that names
    /// itself, and never reads as "no repository".
    #[test]
    fn merge_attributes_names_a_refusal_inside_a_repository() {
        let repo = Repo::new("merge-attributes-refused");
        repo.write(".gitattributes", "a.md merge=union\n");
        let paths = vec!["a.md".to_string(), "../outside.md".to_string()];
        let refusal = merge_attributes(&repo.at, &paths)
            .expect("a repository")
            .expect_err("git refuses a path outside the work tree");
        assert!(refusal.contains("outside"), "{refusal}");
        let _ = fs::remove_dir_all(&repo.at);
    }

    /// A tree git does not see has no git answer, and the caller reads it
    /// some other way.
    #[test]
    fn merge_attributes_outside_a_repository_is_none() {
        let at = std::env::temp_dir().join(format!(
            "headwater-vcs-tests-attributes-no-repository-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&at);
        fs::create_dir_all(&at).expect("the directory is there");
        fs::write(at.join(".gitattributes"), "a.md merge=union\n").expect("the file writes");
        assert_eq!(merge_attributes(&at, &["a.md".to_string()]), None);
        let _ = fs::remove_dir_all(&at);
    }

    /// Where git cannot answer, the search upward stops where git's own does.
    ///
    /// A gitfile and a `.git` directory shaped as a repository are found from
    /// a directory below them. An empty `.git` directory is passed over, a
    /// `.git` file that is not a gitfile ends the search with nothing, and a
    /// ceiling directory is not entered.
    #[test]
    fn the_search_upward_finds_a_repository_where_git_would() {
        let at = std::env::temp_dir().join(format!(
            "headwater-vcs-tests-git-entry-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&at);
        for dir in [
            "worktree/sub",
            "repo/sub",
            "empty/sub",
            "plain/sub",
            "ceiling/sub",
        ] {
            fs::create_dir_all(at.join(dir)).expect("the directories are there");
        }
        let found = |dir: &str| search_upward(&at.join(dir), &[], false, &device_of);

        assert!(!found("worktree/sub"));
        fs::write(at.join("worktree/.git"), "gitdir: /elsewhere\n").expect("the file writes");
        assert!(
            found("worktree/sub"),
            "a gitfile names a repository, even a pruned one"
        );

        for part in ["repo/.git/objects", "repo/.git/refs"] {
            fs::create_dir_all(at.join(part)).expect("the directory is there");
        }
        fs::write(at.join("repo/.git/HEAD"), "ref: refs/heads/main\n").expect("the file writes");
        assert!(
            found("repo/sub"),
            "a `.git` directory shaped as a repository is one"
        );

        fs::create_dir_all(at.join("empty/.git")).expect("the directory is there");
        assert!(
            !found("empty/sub"),
            "an empty `.git` directory is not a repository"
        );

        fs::write(at.join("plain/.git"), "hello\n").expect("the file writes");
        assert!(
            !found("plain/sub"),
            "a `.git` file that is not a gitfile ends the search"
        );

        fs::write(at.join("ceiling/.git"), "gitdir: /elsewhere\n").expect("the file writes");
        let ceiling = at
            .join("ceiling")
            .canonicalize()
            .expect("the directory is there");
        assert!(
            !search_upward(
                &at.join("ceiling/sub"),
                std::slice::from_ref(&ceiling),
                false,
                &device_of
            ),
            "the search does not go up into a ceiling directory"
        );
        assert!(
            search_upward(&ceiling, std::slice::from_ref(&ceiling), false, &device_of),
            "a ceiling directory that is the root itself is still read"
        );
        let _ = fs::remove_dir_all(&at);
    }

    /// A `.git` directory with `objects`, `refs` and a `HEAD` that is not a
    /// reference counts as a repository, on purpose.
    ///
    /// Git reads that directory as no repository. The verb refuses it anyway,
    /// because a corrupt repository that is refused is safer than one that is
    /// read in silence as a tree with no attributes but the root file (ruled
    /// on #1115, 2026-09-25). Test the contents of `HEAD`, and this case fails.
    #[test]
    fn a_repository_shaped_git_directory_with_a_corrupt_head_is_refused_on_purpose() {
        let at = std::env::temp_dir().join(format!(
            "headwater-vcs-tests-corrupt-head-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&at);
        for part in ["sub", ".git/objects", ".git/refs"] {
            fs::create_dir_all(at.join(part)).expect("the directory is there");
        }
        fs::write(at.join(".git/HEAD"), "not a reference\n").expect("the file writes");
        assert!(search_upward(&at.join("sub"), &[], false, &device_of));
        let _ = fs::remove_dir_all(&at);
    }

    /// The search stops at a filesystem boundary, and crosses it only where
    /// `GIT_DISCOVERY_ACROSS_FILESYSTEM` is true.
    ///
    /// No test can mount a filesystem, so the device lookup is a stand-in that
    /// puts `inner` on one device and everything above it on another. Drop the
    /// boundary check, and the first assertion fails.
    #[test]
    fn the_search_upward_stops_at_a_filesystem_boundary() {
        let at = std::env::temp_dir().join(format!(
            "headwater-vcs-tests-boundary-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&at);
        fs::create_dir_all(at.join("inner/sub")).expect("the directories are there");
        fs::write(at.join(".git"), "gitdir: /elsewhere\n").expect("the file writes");
        let inner = at
            .join("inner")
            .canonicalize()
            .expect("the directory is there");
        let device = |dir: &Path| Some(if dir.starts_with(&inner) { 2 } else { 1 });
        assert!(
            !search_upward(&inner.join("sub"), &[], false, &device),
            "the repository above the boundary is not reached"
        );
        assert!(
            search_upward(&inner.join("sub"), &[], true, &device),
            "across the boundary, the repository above it is found"
        );
        let _ = fs::remove_dir_all(&at);
    }

    /// `GIT_DISCOVERY_ACROSS_FILESYSTEM` is read as git reads a boolean.
    ///
    /// Git takes `true`, `yes` and `on` in any case, `false`, `no`, `off` and
    /// the empty value as false, and otherwise an integer, with an optional
    /// `k`, `m` or `g` unit, that is true where it is not zero.
    #[test]
    fn a_boolean_is_read_as_git_reads_one() {
        for value in [
            "1", "2", "-1", "true", "TRUE", "Yes", "on", "0x10", "010", "1k", " 3",
        ] {
            assert!(git_bool(value), "git reads {value:?} as true");
        }
        for value in ["0", "00", "0x0", "0k", "false", "No", "OFF", ""] {
            assert!(!git_bool(value), "git reads {value:?} as false");
        }
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
