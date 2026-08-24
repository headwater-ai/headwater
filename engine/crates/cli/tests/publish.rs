// SPDX-License-Identifier: Apache-2.0
//! `headwater taxonomy publish`, from the state an adopter is actually in.
//!
//! # The defect this target exists for
//!
//! The library guarantee is held one crate down, in
//! `headwater-resolve/tests/publish.rs`: a publish that cannot read what its
//! manifest declares leaves `--out` exactly as it found it. The message a person
//! reads is a different component. `main.rs` prints `headwater: nothing was
//! published` for every error the library returns, with no knowledge of whether
//! a byte was written, so that line was a claim about the disk that no code
//! established. [#271] is the issue where it was false: three files and an empty
//! `bundles/` under a directory the run said it had not written to, and a second
//! run refused by the verb's own precondition catching the first run's
//! leftovers.
//!
//! A correctness root can be completely right and still produce a broken result,
//! and only running the next component reveals it. So this case runs the built
//! binary and reads standard error and the disk, rather than calling the library
//! that the case below it already covers.
//!
//! # The root it runs over
//!
//! This repository's own maintained source of `headwater/standard` —
//! `taxonomy-source/headwater-standard/` since #336 moved it out of
//! `packages/` — copied to where `--package` looks a package up, and nothing
//! else. That is the state `headwater init` sends a newcomer into: its refusal
//! says *"package headwater/standard is not under `packages/`, and nothing here
//! fetches one"*, and the copy that line invites carries a manifest whose
//! `contents.bundles` climbs out with `../..` into a library that the copy left
//! behind. `packages/headwater-standard/` itself is no longer this fixture,
//! because #336 also made it a vendored artifact whose manifest carries the
//! library inside it rather than climbing out to reach one.
//!
//! [#271]: https://github.com/headwater-ai/headwater/issues/271

use std::path::{Path, PathBuf};
use std::process::Command;

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

/// A scratch tree that removes itself, named for the case that made it.
struct Root(PathBuf);

impl Root {
    /// An empty tree, for a case whose root is this repository itself.
    fn scratch(label: &str) -> Root {
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-publish-{}-{label}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).expect("the scratch tree is made");
        Root(at)
    }

    /// This repository's package and the bundle library it points at, copied,
    /// with one regular file named `bundles` added at the package root.
    ///
    /// This root publishes right up to the write phase and fails inside it: the
    /// staged set holds a file at `bundles` and files under `bundles/`, so `put`
    /// writes part of the artifact and then cannot make a directory where it has
    /// just written a file. It is the only provocation in this file that reaches
    /// the undo, and no mode and no race is in it. See the fixture of the same
    /// shape in `headwater-resolve/tests/publish.rs` for why the suite needed
    /// one.
    fn colliding(label: &str) -> Root {
        let root = Root::scratch(label);
        copy(
            &repository().join("taxonomy-source/headwater-standard"),
            &root.0.join("packages/headwater-standard"),
        );
        copy(
            &repository().join("docs/taxonomies"),
            &root.0.join("docs/taxonomies"),
        );
        std::fs::write(
            root.0.join("packages/headwater-standard/bundles"),
            "a regular file where the artifact needs a directory\n",
        )
        .expect("the colliding file is written");
        root
    }

    /// This repository's maintained source, copied to where `--package` looks
    /// a package up, with no library beside it.
    fn copied(label: &str) -> Root {
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-publish-{}-{label}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).expect("the root is made");
        copy(
            &repository().join("taxonomy-source/headwater-standard"),
            &at.join("packages/headwater-standard"),
        );
        assert!(
            !at.join("docs/taxonomies").exists(),
            "the library the manifest points at must not be in the copy"
        );
        Root(at)
    }

    fn publish(&self, out: &Path) -> (Option<i32>, String) {
        publish_from(&self.0, out)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

/// `taxonomy publish` over a root the caller names, as a person types it.
fn publish_from(root: &Path, out: &Path) -> (Option<i32>, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_headwater"))
        .args(["taxonomy", "publish", "--package", "headwater/standard"])
        .arg("--out")
        .arg(out)
        .arg("--root")
        .arg(root)
        .output()
        .expect("the binary runs");
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

impl Drop for Root {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn copy(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("the destination is made");
    for entry in std::fs::read_dir(from).expect("the source reads") {
        let entry = entry.expect("the entry reads").path();
        let name = entry.file_name().expect("it has a name");
        match entry.is_dir() {
            true => copy(&entry, &to.join(name)),
            false => {
                std::fs::copy(&entry, to.join(name)).expect("the file copies");
            }
        }
    }
}

/// `nothing was published` and the disk agree, and the second run is the proof.
///
/// Three assertions, and the second is the one no wording change satisfies. The
/// third is what an adopter met: before this, the second run said *the output
/// directory holds files already*, so the person had to delete a directory the
/// tool had told them it never wrote to.
#[test]
fn a_publish_that_says_nothing_was_published_wrote_nothing() {
    let root = Root::copied("says-nothing");
    let out = root.path().join("release");

    let (code, first) = root.publish(&out);
    assert_eq!(code, Some(1), "{first}");
    assert!(
        first.contains("headwater: nothing was published"),
        "{first}"
    );
    assert!(
        first.contains("package.yml"),
        "the refusal does not name the manifest that declares the path: {first}"
    );
    assert!(
        first.contains("../../docs/taxonomies"),
        "the refusal does not name the declared value: {first}"
    );
    assert!(
        !out.exists(),
        "the run that published nothing left {:?}",
        std::fs::read_dir(&out).map(|entries| entries
            .filter_map(Result::ok)
            .map(|entry| entry.file_name())
            .collect::<Vec<_>>())
    );

    let (code, again) = root.publish(&out);
    assert_eq!(code, Some(1), "{again}");
    assert_eq!(
        again, first,
        "the second run met a different refusal, so the first left something behind"
    );
}

/// A mode, set on a path.
#[cfg(unix)]
fn mode(at: &Path, bits: u32) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(at, std::fs::Permissions::from_mode(bits)).expect("the mode is set");
}

/// Whether a mode can stop this process. See the note of the same name in
/// `headwater-resolve/tests/publish.rs`, and `engine/README.md` on why the
/// container half of the toolchain runs with `--user`.
#[cfg(unix)]
fn modes_hold(root: &Root) -> bool {
    let at = root.path().join("mode-probe");
    std::fs::create_dir_all(&at).expect("the probe is made");
    std::fs::write(at.join("held"), "x").expect("the probe holds a file");
    mode(&at, 0o300);
    let held = std::fs::read_dir(&at).is_err();
    mode(&at, 0o700);
    std::fs::remove_dir_all(&at).expect("the probe goes");
    held
}

/// `nothing was published` over a directory that could not be read, with a whole
/// artifact written into it.
///
/// This is the same claim as the case above and the state that reached it is the
/// narrower one. The root here is this repository, so the publish would
/// otherwise succeed and the write phase has every file of a real artifact to
/// put somewhere. `--out` is a directory holding a person's file at mode `0300`:
/// no `r`, so the precondition could not see the file, and `w` and `x`, so every
/// write below it lands. Read as absence, that published the artifact beside
/// their file, failed reading the directory back, printed `nothing was
/// published`, and ran an undo whose one instruction is to remove `--out`.
///
/// A verb that says it wrote nothing and wrote 39 files is the defect the issue
/// is named for. This is that defect through a doorway the `--out` precondition
/// does not cover, so the assertion is the disk rather than the wording.
#[cfg(unix)]
#[test]
fn a_publish_into_a_directory_it_cannot_read_writes_nothing_into_it() {
    let holder = Root::scratch("unreadable-out");
    if !modes_hold(&holder) {
        eprintln!("skipped: this process is root, and root reads through mode 0300");
        return;
    }
    let out = holder.path().join("release");
    std::fs::create_dir_all(&out).expect("the caller's directory is made");
    std::fs::write(out.join("theirs.txt"), "the caller's own file").expect("their file is written");
    mode(&out, 0o300);

    let (code, message) = publish_from(&repository(), &out);
    mode(&out, 0o700);

    let left: Vec<String> = std::fs::read_dir(&out)
        .expect("it reads once the mode is back")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        left,
        vec!["theirs.txt".to_string()],
        "the run published into a directory it could not read: {message}"
    );
    assert_eq!(code, Some(1), "{message}");
    assert!(
        message.contains("the output directory cannot be read"),
        "the refusal does not say why the directory was refused: {message}"
    );
}

/// An empty `--out` that the caller made is emptied again, not removed.
///
/// The precondition takes an empty directory as well as an absent one, so the
/// undo has two states to return to and this is the second. A person who ran
/// `mkdir release` first gets their directory back, empty.
///
/// **The root here is the colliding one and it used to be the copied one.** With
/// the copied root this case failed in phase 1, so `--out` was never written to
/// and the assertion was that an untouched empty directory is empty: it held
/// with the undo deleted. The root below reaches the write phase, so what is
/// removed between the failure and these lines is a part of an artifact that was
/// really on disk.
#[test]
fn an_output_directory_the_caller_made_is_left_empty_rather_than_removed() {
    let root = Root::colliding("caller-made");
    let out = root.path().join("release");
    std::fs::create_dir_all(&out).expect("the caller makes it");

    let (code, message) = root.publish(&out);
    assert_eq!(code, Some(1), "{message}");
    assert!(
        message.contains("headwater: nothing was published"),
        "{message}"
    );
    assert!(out.is_dir(), "the caller's own directory was removed");
    assert_eq!(
        std::fs::read_dir(&out)
            .expect("it reads")
            .filter_map(Result::ok)
            .count(),
        0,
        "the directory the caller made is not empty again: {message}"
    );
}

/// A write that fails leaves neither `--out` nor the directories the run made to
/// reach it, and the line a person reads is true of the disk.
///
/// `--out` is three levels below a directory that is not there either. `put`
/// reaches it with `create_dir_all`, so before this the verb printed `nothing
/// was published` and left three directories it had made standing.
#[test]
fn a_failed_write_leaves_neither_the_output_directory_nor_the_path_to_it() {
    let root = Root::colliding("nested-out");
    let nested = root.path().join("nested");
    let out = nested.join("a/b/c");

    let (code, message) = root.publish(&out);
    assert_eq!(code, Some(1), "{message}");
    assert!(
        message.contains("headwater: nothing was published"),
        "{message}"
    );
    assert!(
        !out.exists(),
        "the output directory is still there: {message}"
    );
    assert!(
        !nested.exists(),
        "the run that published nothing left the directories it made: {message}"
    );
}

/// #336's `--from <dir>`, exercised as a person would type it: two flags that
/// name the same thing are refused together, and `--from` alone publishes a
/// directory `--package` would never find under `packages/`.
///
/// The directory this hands `--from` is `taxonomy-source/headwater-standard/`
/// itself, in this repository's own tree rather than a copy of it, precisely
/// because the point of the flag is that it reads a manifest `find` would never
/// walk to (it does not sit under `packages/` at all). A copy would test the
/// read and hide the one thing worth proving: this path bypasses the lookup by
/// name entirely.
#[test]
fn from_and_package_together_are_refused_and_from_alone_publishes_the_relocated_source() {
    let source = repository().join("taxonomy-source/headwater-standard");
    let out_root = Root::scratch("from-flag");
    let out = out_root.path().join("release");

    let both = Command::new(env!("CARGO_BIN_EXE_headwater"))
        .args(["taxonomy", "publish", "--package", "headwater/standard"])
        .arg("--from")
        .arg(&source)
        .arg("--out")
        .arg(&out)
        .arg("--root")
        .arg(repository())
        .output()
        .expect("the binary runs");
    assert_eq!(both.status.code(), Some(1));
    let both_err = String::from_utf8_lossy(&both.stderr);
    assert!(
        both_err.contains("--package") && both_err.contains("--from"),
        "the refusal does not name both flags: {both_err}"
    );
    assert!(!out.exists(), "the refused run wrote into --out anyway");

    let from_alone = Command::new(env!("CARGO_BIN_EXE_headwater"))
        .args(["taxonomy", "publish"])
        .arg("--from")
        .arg(&source)
        .arg("--out")
        .arg(&out)
        .arg("--root")
        .arg(repository())
        .output()
        .expect("the binary runs");
    let stdout = String::from_utf8_lossy(&from_alone.stdout).into_owned();
    assert_eq!(
        from_alone.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&from_alone.stderr)
    );
    assert!(
        stdout.contains("published headwater/standard"),
        "the run did not report a publish: {stdout}"
    );
    assert!(
        out.join("release.yml").is_file(),
        "publishing `--from` a directory outside `packages/` wrote no release record"
    );
}
