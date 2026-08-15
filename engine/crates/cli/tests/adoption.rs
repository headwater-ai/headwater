// SPDX-License-Identifier: Apache-2.0
//! What `taxonomy resolve` does to the one authored block of the lock.
//!
//! # The defect this target exists for
//!
//! `.headwater/taxonomy.lock` is generated except for its `adoption` block,
//! which an adopter writes and which moves a finding out of a strict run. The
//! lock's own header says so: "That block is authored, and `headwater taxonomy
//! resolve` carries it through rather than producing it."
//!
//! [#213](https://github.com/headwater-ai/headwater/issues/213) measured the
//! promise failing. A lock whose recorded digest no longer covers its body does
//! not read, the carry-through read turned that refusal into "there is no
//! block", and the next write replaced the file without it — at exit 0, with
//! nothing on either stream. The sequence a person meets is the one that hides
//! the loss: `headwater check` refuses and prints `taxonomy resolve` as the
//! remedy, the remedy runs, and the gate goes green **because** the block that
//! was holding a finding is gone. A released debt is silent by construction,
//! because the thing that would report it is the thing that was deleted.
//!
//! # Why these cases drive the binary
//!
//! The decision is a caller's: which of the states behind a lock that will not
//! read is a rewrite allowed to proceed over. A test that constructs the value
//! proves the classifier and not the decision, which is the defect restated —
//! the same reason `wiring.rs` exists. So each case here assembles a scratch
//! repository, runs the built binary over it, and reads the file afterwards.
//!
//! # The four states, and what each one asserts
//!
//! - **A digest that does not match.** The block is legible behind the refusal,
//!   and the digest never covered it, so the run carries it through, repairs the
//!   file and says on standard output that it did.
//! - **A lock that does not parse.** Nothing can be said about a block behind
//!   it, so the run refuses rather than replacing the file.
//! - **A lock a newer engine wrote.** The same refusal, for the reason the
//!   format number exists.
//! - **No lock at all.** A first resolve writes one. That is the case the
//!   fall-back exists for and it stays.

use std::path::{Path, PathBuf};
use std::process::Command;

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

/// The payload the cases write, in the shape `headwater infer` produces.
///
/// It names an owner and an expiry, which are the two values that no source
/// produces and that a rewrite therefore cannot reconstruct.
const PAYLOAD: &str = concat!(
    "\nadoption:\n",
    "  tasks:\n",
    "    - id: AD-9\n",
    "      statement: \"The debt this fixture declares\"\n",
    "      owner: \"a person\"\n",
    "      until: 2027-06-30\n",
    "      pairs:\n",
    "        - path: docs/spec/02-taxonomy-model.md\n",
    "          rule: language.retired_term.used\n",
);

/// A scratch repository that resolves, with a lock already written.
///
/// `label` names the case rather than the target. Cargo runs the cases of one
/// target as threads of one process, so a directory keyed on the process
/// identifier alone is one that a second case removes while the first reads it.
struct Root {
    at: PathBuf,
}

impl Root {
    fn new(label: &str) -> Root {
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-adoption-{}-{label}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).expect("the root is made");

        let repository = repository();
        copy(&repository.join("packages"), &at.join("packages"));
        copy(
            &repository.join("docs/taxonomies"),
            &at.join("docs/taxonomies"),
        );
        for name in ["taxonomy.yml", "overlay.yml"] {
            let to = at.join(".headwater").join(name);
            std::fs::create_dir_all(to.parent().expect("it has a parent"))
                .expect("the declaration directory is there");
            std::fs::copy(repository.join(".headwater").join(name), to)
                .expect("the declaration copies");
        }

        let root = Root { at };
        let resolved = root.run(&["taxonomy", "resolve"]);
        assert_eq!(
            resolved.code,
            Some(0),
            "the fixture resolves\n{}{}",
            resolved.out,
            resolved.err
        );
        root
    }

    fn lock(&self) -> PathBuf {
        self.at.join(".headwater/taxonomy.lock")
    }

    fn text(&self) -> String {
        std::fs::read_to_string(self.lock()).expect("the lock reads")
    }

    fn write(&self, text: &str) {
        std::fs::write(self.lock(), text).expect("the lock writes");
    }

    /// Put an authored block into the lock, where `resolve` wrote none.
    ///
    /// It goes in front of the `resolved` body, which is where `render` puts
    /// one, so the file the cases start from is the file a real resolve writes.
    fn author(&self) {
        let text = self.text();
        assert!(
            !text.contains("\nadoption:\n"),
            "the fixture starts with no authored block"
        );
        self.write(&text.replacen("\nresolved:\n", &format!("{PAYLOAD}\nresolved:\n"), 1));
    }

    fn run(&self, arguments: &[&str]) -> Ran {
        let output = Command::new(env!("CARGO_BIN_EXE_headwater"))
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
}

impl Drop for Root {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.at);
    }
}

/// The two streams held apart. A verb that reports on one and decides on the
/// other is read wrongly by a case that merges them.
#[derive(Debug)]
struct Ran {
    code: Option<i32>,
    out: String,
    err: String,
}

fn copy(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("the directory is there");
    for entry in std::fs::read_dir(from).expect("the fixture directory reads") {
        let entry = entry.expect("the entry reads");
        let target = to.join(entry.file_name());
        match entry.file_type().expect("the file type reads").is_dir() {
            true => copy(&entry.path(), &target),
            false => {
                std::fs::copy(entry.path(), &target).expect("the fixture copies");
            }
        }
    }
}

/// #213, as the person meets it: a red gate, the printed remedy, and the block.
///
/// The digest is corrupted by four characters, which is exactly what a hand
/// edit to the body does and what a mechanical rewrite across the tree did in
/// #207. `check` refuses and names `taxonomy resolve`. Running it must leave the
/// authored block on disk, because the digest covers the resolution and has
/// never covered that block, so a mismatch is no evidence about it.
#[test]
fn a_lock_whose_digest_does_not_match_keeps_its_authored_block() {
    let root = Root::new("digest-does-not-match");
    root.author();
    let before = root.text();

    // Loose on purpose. The decisive assertions are the two after the resolve.
    let corrupted = before.replacen("digest: sha256:", "digest: sha256:0000", 1);
    assert_ne!(corrupted, before, "the digest line is there to corrupt");
    root.write(&corrupted);

    let resolved = root.run(&["taxonomy", "resolve"]);
    assert_eq!(
        resolved.code,
        Some(0),
        "the printed remedy runs\n{}{}",
        resolved.out,
        resolved.err
    );

    // The decision: the block an adopter wrote survived the rewrite, with the
    // owner and the expiry that no source can reconstruct.
    let after = root.text();
    assert!(
        after.contains("id: AD-9")
            && after.contains("owner: \"a person\"")
            && after.contains("until: 2027-06-30"),
        "the authored block survived a resolve over a lock that did not read:\n{after}"
    );

    // And it said so. The caller reached this verb because another one printed
    // it as a remedy, so a run that reports nothing is read as a run that did
    // nothing but succeed.
    assert!(
        resolved.out.contains("adoption"),
        "standard output says what became of the authored block:\n{}",
        resolved.out
    );
}

/// A lock nothing can parse is not a lock a rewrite may replace.
///
/// This is the state the classifier cannot see through: the file may hold an
/// authored block and this engine cannot say. A rewrite there discards data it
/// cannot name, so the run stops and the file stays as it is.
#[test]
fn a_lock_that_does_not_parse_stops_a_resolve() {
    let root = Root::new("does-not-parse");
    root.author();
    let broken = format!("{}\n\t- this is not a lock\n", root.text());
    root.write(&broken);

    let resolved = root.run(&["taxonomy", "resolve"]);
    assert_ne!(
        resolved.code,
        Some(0),
        "a lock this engine cannot read stops a rewrite\n{}{}",
        resolved.out,
        resolved.err
    );
    assert_eq!(root.text(), broken, "the file the run refused is untouched");
    assert!(
        resolved.err.contains("adoption"),
        "the refusal names what a rewrite would have discarded:\n{}",
        resolved.err
    );
}

/// A first resolve, with no lock at all, still writes one.
///
/// That is what the carry-through read's fall-back exists for, and removing the
/// fall-back for the states above must not remove it here.
#[test]
fn a_first_resolve_with_no_lock_writes_one() {
    let root = Root::new("no-lock-at-all");
    std::fs::remove_file(root.lock()).expect("the lock is removed");

    let resolved = root.run(&["taxonomy", "resolve"]);
    assert_eq!(
        resolved.code,
        Some(0),
        "a first resolve writes a lock\n{}{}",
        resolved.out,
        resolved.err
    );
    assert!(root.lock().exists(), "the lock is there");
    assert!(
        resolved.out.contains("adoption"),
        "standard output says there was no block to carry:\n{}",
        resolved.out
    );
}

/// A lock a newer engine wrote stops a rewrite as well, and for the same reason.
///
/// The format number is the promise that an engine which cannot honor a payload
/// refuses the lock rather than reporting a louder verdict than the adopter
/// agreed to. An older engine that rewrote the file would honor it by deleting
/// it, which is the loudest verdict of all.
#[test]
fn a_lock_a_newer_engine_wrote_stops_a_resolve() {
    let root = Root::new("newer-format");
    root.author();
    let ahead = root.text().replacen("  format: 2\n", "  format: 3\n", 1);
    assert!(ahead.contains("format: 3"), "the format line moved");
    root.write(&ahead);

    let resolved = root.run(&["taxonomy", "resolve"]);
    assert_ne!(
        resolved.code,
        Some(0),
        "an older engine does not rewrite a newer lock\n{}{}",
        resolved.out,
        resolved.err
    );
    assert_eq!(root.text(), ahead, "the file the run refused is untouched");
}
