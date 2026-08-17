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
//! `packages/headwater-standard` copied out of this repository, and nothing
//! else. That is the state `headwater init` sends a newcomer into: its refusal
//! says *"package headwater/standard is not under `packages/`, and nothing here
//! fetches one"*, and the copy that line invites carries a manifest whose
//! `contents.bundles` climbs out with `../..` into a library that the copy left
//! behind.
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
    /// This repository's package directory, copied, with no library beside it.
    fn copied(label: &str) -> Root {
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-publish-{}-{label}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).expect("the root is made");
        copy(
            &repository().join("packages/headwater-standard"),
            &at.join("packages/headwater-standard"),
        );
        assert!(
            !at.join("docs/taxonomies").exists(),
            "the library the manifest points at must not be in the copy"
        );
        Root(at)
    }

    fn publish(&self, out: &Path) -> (Option<i32>, String) {
        let output = Command::new(env!("CARGO_BIN_EXE_headwater"))
            .args(["taxonomy", "publish", "--package", "headwater/standard"])
            .arg("--out")
            .arg(out)
            .arg("--root")
            .arg(&self.0)
            .output()
            .expect("the binary runs");
        (
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).into_owned(),
        )
    }

    fn path(&self) -> &Path {
        &self.0
    }
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
    assert!(first.contains("headwater: nothing was published"), "{first}");
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

/// An empty `--out` that the caller made is emptied again, not removed.
///
/// The precondition takes an empty directory as well as an absent one, so the
/// undo has two states to return to and this is the second. A person who ran
/// `mkdir release` first gets their directory back, empty.
#[test]
fn an_output_directory_the_caller_made_is_left_empty_rather_than_removed() {
    let root = Root::copied("caller-made");
    let out = root.path().join("release");
    std::fs::create_dir_all(&out).expect("the caller makes it");

    let (code, message) = root.publish(&out);
    assert_eq!(code, Some(1), "{message}");
    assert!(out.is_dir(), "the caller's own directory was removed");
    assert_eq!(
        std::fs::read_dir(&out)
            .expect("it reads")
            .filter_map(Result::ok)
            .count(),
        0,
        "the directory the caller made is not empty again"
    );
}
