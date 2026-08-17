// SPDX-License-Identifier: Apache-2.0
//! What `taxonomy resolve` and `infer --write` do to the one authored block of
//! the lock.
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
//!
//! # The same class, one verb along: `headwater infer --write`
//!
//! [#249](https://github.com/headwater-ai/headwater/issues/249) measured the
//! same loss on the verb `CLAUDE.md` names as *the* escape hatch for debt.
//! `infer --write` built a payload from a counter that starts at `AD-1` and
//! reads nothing, and handed it to the lock writer as the whole `adoption`
//! block. A corpus that already declared a task lost it — its statement, its
//! owner and its expiry — at exit 0, with nothing on either stream.
//!
//! The three cases below are the three separable failures, and each one runs the
//! built binary over a scratch repository **whose lock declares a task and whose
//! corpus raises a finding**. Both halves are needed: over an empty `adoption`
//! block no case can tell a merge from a replacement, and over a corpus that
//! raises nothing the verb writes no payload at all.

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
        self.author_as("AD-9");
    }

    /// The same block under an identifier the case chooses.
    ///
    /// `AD-1` is the identifier a counter that reads nothing mints first, so a
    /// case that wants the collision asks for it by name rather than relying on
    /// which identifier the fixture happens to carry.
    fn author_as(&self, id: &str) {
        let text = self.text();
        assert!(
            !text.contains("\nadoption:\n"),
            "the fixture starts with no authored block"
        );
        let payload = PAYLOAD.replace("AD-9", id);
        self.write(&text.replacen("\nresolved:\n", &format!("{payload}\nresolved:\n"), 1));
    }

    /// A corpus of one document that raises exactly one finding.
    ///
    /// The document is scaffolded rather than written out here, so a taxonomy
    /// that adds a required facet fails this fixture rather than typing the
    /// document as nothing and raising no finding — which is the shape in which
    /// a case passes for the opposite of its reason.
    fn corpus(&self) {
        let made = self.run(&["new", "design_spec", "--title", "A scratch part"]);
        assert_eq!(
            made.code,
            Some(0),
            "the corpus document scaffolds\n{}{}",
            made.out,
            made.err
        );
        let at = std::fs::read_dir(self.at.join("docs/spec"))
            .expect("the spec shelf is there")
            .map(|entry| entry.expect("the entry reads").path())
            .next()
            .expect("the scaffolder wrote a document");
        let mut text = std::fs::read_to_string(&at).expect("the document reads");
        text.push_str(
            "\n## A scratch section\n\nThis document is scratch and it does not meet the \
             controlled language, because the sentence is deliberately long enough to pass the \
             twenty five word limit that the regime sets for a sentence of running prose.\n",
        );
        std::fs::write(&at, text).expect("the document writes");
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

/// The task identifiers the `adoption` block declares, in file order.
///
/// Scoped to that block on purpose. The resolved taxonomy below it states `id`
/// keys of its own, and a reading that swept the whole file would count a
/// control identifier as an adoption task.
fn ids(text: &str) -> Vec<String> {
    let block = match text.split_once("\nadoption:\n") {
        Some((_, rest)) => rest.split("\nresolved:\n").next().unwrap_or_default(),
        None => return Vec::new(),
    };
    block
        .lines()
        .filter_map(|line| line.trim().strip_prefix("- id: "))
        .map(|id| id.trim().to_string())
        .collect()
}

/// An expiry no run of this suite reaches, so no case here reads a clock.
const FAR: &str = "2035-01-01";

/// #249's first failure: a task the lock declared is gone after the write.
///
/// This is the data loss. The fixture's task names an owner and an expiry that
/// no source and no run can reconstruct, which is the whole reason the block is
/// the authored part of a generated file.
#[test]
fn an_infer_write_keeps_every_task_the_lock_already_declared() {
    let root = Root::new("infer-keeps-the-task");
    root.author();
    root.corpus();

    let ran = root.run(&["infer", "--owner", "a parent test", "--until", FAR, "--write"]);
    assert_eq!(
        ran.code,
        Some(0),
        "the documented remedy runs\n{}{}",
        ran.out,
        ran.err
    );

    let after = root.text();
    assert!(
        after.contains("id: AD-9")
            && after.contains("owner: \"a person\"")
            && after.contains("until: 2027-06-30"),
        "the task the lock declared survived `infer --write`:\n{after}"
    );
    // And the payload landed. A run that refused to write anything would pass
    // the assertion above for the opposite reason.
    assert!(
        after.contains("owner: \"a parent test\""),
        "the new payload is in the lock as well:\n{after}"
    );

    // #249's third failure: the verb reported what it wrote and never what it
    // did to the block that was there. `taxonomy resolve` is the precedent.
    assert!(
        ran.out.contains("carried the adoption block through, 1 task"),
        "standard output says what became of the authored block:\n{}",
        ran.out
    );
}

/// #249's second failure: the identifier is minted from a counter that reads
/// nothing, so it collides with whatever the lock already holds.
///
/// The proposal is the arm that matters as much as the write, because #249's
/// fourth clause is that the two cannot disagree: the identifier printed by a
/// run without `--write` is the identifier the write uses.
#[test]
fn the_identifier_a_run_mints_is_one_no_declared_task_holds() {
    let root = Root::new("identifier-not-taken");
    root.author_as("AD-1");
    root.corpus();

    let proposed = root.run(&["infer", "--until", FAR]);
    assert_eq!(
        proposed.code,
        Some(0),
        "the proposal runs\n{}{}",
        proposed.out,
        proposed.err
    );
    // Vacuity guard: a taxonomy under which this document raises nothing would
    // pass every assertion below by proposing nothing at all.
    assert!(
        proposed.out.contains("the payload, which --write puts in the lock"),
        "the corpus of this fixture raises a finding to declare:\n{}",
        proposed.out
    );
    assert!(
        !proposed.out.contains("- id: AD-1\n"),
        "the proposal mints no identifier the lock already declares:\n{}",
        proposed.out
    );

    let ran = root.run(&["infer", "--owner", "a parent test", "--until", FAR, "--write"]);
    assert_eq!(
        ran.code,
        Some(0),
        "the write runs\n{}{}",
        ran.out,
        ran.err
    );
    let after = ids(&root.text());
    assert_eq!(
        after.len(),
        2,
        "the write added a task beside the declared one: {after:?}"
    );
    assert_eq!(after[0], "AD-1", "the declared task keeps its identifier");
    assert_ne!(after[1], "AD-1", "and the new one does not take it: {after:?}");

    // The two runs agree. The proposal printed the identifier the write used.
    assert!(
        proposed.out.contains(&format!("- id: {}\n", after[1])),
        "the proposal printed {}, which is what the write used:\n{}",
        after[1],
        proposed.out
    );
}

/// A second and a third `--write` add nothing and remove nothing.
///
/// This is the case that separates a fix from a half-fix. A verb that merges
/// but re-proposes the debt it already declared grows the block without bound,
/// and a verb that refuses the second run after clobbering on the first passes
/// every single-run assertion above.
#[test]
fn a_second_and_a_third_infer_write_change_nothing() {
    let root = Root::new("infer-run-twice");
    root.author();
    root.corpus();

    let first = root.run(&["infer", "--owner", "a parent test", "--until", FAR, "--write"]);
    assert_eq!(
        first.code,
        Some(0),
        "the first write runs\n{}{}",
        first.out,
        first.err
    );
    let after_first = ids(&root.text());
    assert_eq!(
        after_first.len(),
        2,
        "the first write added one task beside the declared one: {after_first:?}"
    );

    let second = root.run(&["infer", "--owner", "a parent test", "--until", FAR, "--write"]);
    assert_eq!(
        second.code,
        Some(0),
        "the second write runs\n{}{}",
        second.out,
        second.err
    );
    assert_eq!(
        ids(&root.text()),
        after_first,
        "the second run declares nothing new and removes nothing\n{}",
        second.out
    );

    let third = root.run(&["infer", "--owner", "a parent test", "--until", FAR, "--write"]);
    assert_eq!(
        third.code,
        Some(0),
        "the third write runs\n{}{}",
        third.out,
        third.err
    );
    assert_eq!(
        ids(&root.text()),
        after_first,
        "and so does the third\n{}",
        third.out
    );

    let after = root.text();
    assert!(
        after.contains("id: AD-9") && after.contains("until: 2027-06-30"),
        "the declared task survived all three runs:\n{after}"
    );
}

/// A block this run cannot add to stops the write rather than replacing it.
///
/// The state is a block with no `tasks` sequence, which `headwater check`
/// already refuses to read as debt. A run that resolved the refusal by writing
/// its own block over it would be the defect of #249 wearing an error message,
/// so the file is left exactly as it was.
#[test]
fn an_adoption_block_with_no_tasks_stops_an_infer_write() {
    let root = Root::new("no-tasks-sequence");
    root.author();
    root.corpus();
    let broken = root.text().replacen(
        PAYLOAD,
        "\nadoption:\n  note: this block declares no tasks\n",
        1,
    );
    assert!(
        broken.contains("\nadoption:\n") && !broken.contains("tasks:"),
        "the block is there and its tasks sequence is not:\n{broken}"
    );
    root.write(&broken);

    let ran = root.run(&["infer", "--owner", "a parent test", "--until", FAR, "--write"]);
    assert_ne!(
        ran.code,
        Some(0),
        "a block this run cannot add to stops the write\n{}{}",
        ran.out,
        ran.err
    );
    assert_eq!(root.text(), broken, "the file the run refused is untouched");
    assert!(
        ran.err.contains("adoption"),
        "the refusal names what a write would have discarded:\n{}",
        ran.err
    );
}
