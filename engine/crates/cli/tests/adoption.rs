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
//! Three of the four cases below are the three separable failures, and the
//! fourth is the refusal that replaces none of them. Each one runs the built
//! binary over a scratch repository **whose lock declares a task and whose
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
        copy(
            &repository.join(headwater_resolve::package::PACKAGES),
            &at.join(headwater_resolve::package::PACKAGES),
        );
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
        self.author_until(id, "2027-06-30");
    }

    /// The same block under an identifier and an expiry the case chooses.
    ///
    /// `until: 2027-06-30` is `PAYLOAD`'s own date, baked into the string, so
    /// this substitutes it the same way `author_as` substitutes the
    /// identifier. It is what the three-arm `adoption.task.expired` fixture
    /// below varies: one lock, one corpus, the date moved past and pulled
    /// forward across the two cases that are not the baseline.
    fn author_until(&self, id: &str, until: &str) {
        let text = self.text();
        assert!(
            !text.contains("\nadoption:\n"),
            "the fixture starts with no authored block"
        );
        let payload = PAYLOAD.replace("AD-9", id).replace("2027-06-30", until);
        self.write(&text.replacen("\nresolved:\n", &format!("{payload}\nresolved:\n"), 1));
    }

    /// The same block whose one pair names a `(path, rule)` the case chooses.
    ///
    /// Every helper above writes `PAYLOAD`'s own pair, which names
    /// `docs/spec/02-taxonomy-model.md` — a path the scratch corpus does not
    /// hold. Such a pair matches no finding, so it reads as *closed* and every
    /// case above runs at `0 open`. A case that needs an open pair therefore
    /// builds the corpus first and names the document the scaffolder actually
    /// wrote, which is why this helper takes the pair rather than baking one in.
    fn author_pair(&self, id: &str, until: &str, path: &str, rule: &str) {
        let text = self.text();
        assert!(
            !text.contains("\nadoption:\n"),
            "the fixture starts with no authored block"
        );
        let payload = PAYLOAD
            .replace("AD-9", id)
            .replace("2027-06-30", until)
            .replace("docs/spec/02-taxonomy-model.md", path)
            .replace("language.retired_term.used", rule);
        self.write(&text.replacen("\nresolved:\n", &format!("{payload}\nresolved:\n"), 1));
    }

    /// The one document the corpus holds, as the check layer names it.
    ///
    /// A path rather than a file name, because a pair of the `adoption` block is
    /// matched against the path a finding carries.
    fn document(&self) -> String {
        let at = std::fs::read_dir(self.at.join("docs/spec"))
            .expect("the spec shelf is there")
            .map(|entry| entry.expect("the entry reads").path())
            .next()
            .expect("the corpus was built first");
        let name = at.file_name().expect("the document has a name");
        format!("docs/spec/{}", name.to_string_lossy())
    }

    /// Every line of the adoption store, or an empty vector where none exists.
    fn store(&self) -> Vec<String> {
        match std::fs::read_to_string(self.at.join(".headwater/adoption.jsonl")) {
            Err(_) => vec![],
            Ok(text) => text
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(str::to_string)
                .collect(),
        }
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
    let ahead = root.text().replacen("  format: 3\n", "  format: 4\n", 1);
    assert!(ahead.contains("format: 4"), "the format line moved");
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

    let ran = root.run(&[
        "infer",
        "--owner",
        "a parent test",
        "--until",
        FAR,
        "--write",
    ]);
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
        ran.out
            .contains("carried the adoption block through, 1 task"),
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
        proposed
            .out
            .contains("the payload, which --write puts in the lock"),
        "the corpus of this fixture raises a finding to declare:\n{}",
        proposed.out
    );
    assert!(
        !proposed.out.contains("- id: AD-1\n"),
        "the proposal mints no identifier the lock already declares:\n{}",
        proposed.out
    );

    let ran = root.run(&[
        "infer",
        "--owner",
        "a parent test",
        "--until",
        FAR,
        "--write",
    ]);
    assert_eq!(ran.code, Some(0), "the write runs\n{}{}", ran.out, ran.err);
    let after = ids(&root.text());
    assert_eq!(
        after.len(),
        2,
        "the write added a task beside the declared one: {after:?}"
    );
    assert_eq!(after[0], "AD-1", "the declared task keeps its identifier");
    assert_ne!(
        after[1], "AD-1",
        "and the new one does not take it: {after:?}"
    );

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

    let first = root.run(&[
        "infer",
        "--owner",
        "a parent test",
        "--until",
        FAR,
        "--write",
    ]);
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

    let second = root.run(&[
        "infer",
        "--owner",
        "a parent test",
        "--until",
        FAR,
        "--write",
    ]);
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

    let third = root.run(&[
        "infer",
        "--owner",
        "a parent test",
        "--until",
        FAR,
        "--write",
    ]);
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

    let ran = root.run(&[
        "infer",
        "--owner",
        "a parent test",
        "--until",
        FAR,
        "--write",
    ]);
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

// ---------------------------------------------------------------------------
// The seam read from the other side: what a person is told about the block the
// header invites them to edit, and what is said about a key nothing reads.
// ---------------------------------------------------------------------------

/// A hand edit inside the authored block is not a source that moved.
///
/// The lock's own header says the `adoption` block is "authored", that
/// "`headwater infer` writes it, a person edits it, and `headwater taxonomy
/// resolve` carries it through untouched". `--check` compares the whole file,
/// so the quoting of one scalar inside that block reads as a stale lock, and
/// the message a person gets names their sources — which did not move.
///
/// The edit here is one scalar, quoted the way a person writes one. Nothing
/// else in the file changes, and the resolved half is byte identical.
#[test]
fn a_hand_edited_adoption_block_does_not_read_as_a_source_that_moved() {
    let root = Root::new("hand-edited-block");
    root.author();
    // `author` writes the block by hand, so the file is not yet in the form the
    // renderer writes. One resolve puts it there, and the case starts from a
    // file that `--check` accepts.
    let resolved = root.run(&["taxonomy", "resolve"]);
    assert_eq!(
        resolved.code,
        Some(0),
        "the fixture resolves\n{}{}",
        resolved.out,
        resolved.err
    );
    let clean = root.run(&["taxonomy", "resolve", "--check"]);
    assert_eq!(
        clean.code,
        Some(0),
        "the case starts from a lock `--check` accepts\n{}{}",
        clean.out,
        clean.err
    );

    let canonical = root.text();
    let bare = "    - id: AD-9\n";
    assert!(
        canonical.contains(bare),
        "the renderer writes the identifier bare:\n{canonical}"
    );
    root.write(&canonical.replacen(bare, "    - id: \"AD-9\"\n", 1));

    let ran = root.run(&["taxonomy", "resolve", "--check"]);
    assert_eq!(
        ran.code,
        Some(1),
        "the file still differs from the one the sources produce\n{}{}",
        ran.out,
        ran.err
    );
    assert!(
        ran.err
            .contains("carries the taxonomy its sources resolve to"),
        "the diagnosis names the half that agrees:\n{}",
        ran.err
    );
    assert!(
        ran.err.contains("`adoption` block"),
        "and the half that does not:\n{}",
        ran.err
    );
    assert!(
        !ran.err.contains("is not what the sources resolve to"),
        "nothing about the sources changed:\n{}",
        ran.err
    );
}

/// The guard: a source that genuinely moved still gets the old message.
///
/// This is the harder half of the same split. The source edit is a comment, so
/// the resolved taxonomy is byte identical and only the recorded source digest
/// moves. A fix that decided "the resolution agrees" from the taxonomy alone
/// would call this a hand edit, which is worse than the defect it replaced.
#[test]
fn a_source_that_moved_still_names_itself_and_keeps_the_old_message() {
    let root = Root::new("source-moved");
    root.author();
    let resolved = root.run(&["taxonomy", "resolve"]);
    assert_eq!(
        resolved.code,
        Some(0),
        "the fixture resolves\n{}{}",
        resolved.out,
        resolved.err
    );

    let source = root
        .at
        .join(".headwater/packages/headwater-standard/taxonomy.yml");
    let mut text = std::fs::read_to_string(&source).expect("the source reads");
    text.push_str("\n# A comment this case appended, which moves the bytes and not the result.\n");
    std::fs::write(&source, text).expect("the source writes");

    let ran = root.run(&["taxonomy", "resolve", "--check"]);
    assert_eq!(
        ran.code,
        Some(1),
        "a moved source is still a stale lock\n{}{}",
        ran.out,
        ran.err
    );
    assert!(
        ran.err.contains("is not what the sources resolve to"),
        "the old message stands:\n{}",
        ran.err
    );
    assert!(
        ran.err
            .contains("taxonomy.yml has changed since the lock was written"),
        "and it names the file that moved:\n{}",
        ran.err
    );
}

/// A key at the top of the block that this engine does not read is named.
///
/// `adoption::read` already refuses a pair naming a rule the engine does not
/// carry, on the argument that "a directive against a rule that does not exist
/// is a directive its author believes is working". One level up, the same
/// belief was free: the block's siblings of `tasks` were ignored.
///
/// `from` is the case that matters, because it is the field
/// [#78](https://github.com/headwater-ai/headwater/issues/78) exists to add.
/// Nothing writes it and nothing reads it, so an adopter who writes it by hand
/// gets a green run and a false belief.
#[test]
fn a_block_level_key_this_engine_does_not_read_is_named() {
    let root = Root::new("unread-block-key");
    root.author();
    root.corpus();

    let text = root.text();
    root.write(&text.replacen(
        "\nadoption:\n  tasks:\n",
        "\nadoption:\n  from: 99.0.0\n  severity: quiet\n  tasks:\n",
        1,
    ));

    let ran = root.run(&["check"]);
    assert_eq!(
        ran.code,
        Some(0),
        "an unread key is reported and does not fail a run\n{}{}",
        ran.out,
        ran.err
    );
    assert!(
        ran.out.contains("`from`"),
        "the report names the key:\n{}",
        ran.out
    );
    assert!(
        ran.out.contains("`severity`"),
        "and the second one:\n{}",
        ran.out
    );
    assert!(
        ran.out.contains("AD-9"),
        "and the tasks beside it are still read:\n{}",
        ran.out
    );
}

/// A key inside a task is a refusal rather than a note.
///
/// The two levels want different answers. A key beside `tasks` holds nothing,
/// so the run names it and reads the tasks. A key inside a task may mean the
/// task is not the debt this engine read out of it, so the task holds nothing
/// and every finding it named is reported.
#[test]
fn a_task_level_key_this_engine_does_not_read_refuses_the_task() {
    let root = Root::new("unread-task-key");
    root.author();
    root.corpus();

    let text = root.text();
    let anchor = "      until: 2027-06-30\n";
    assert!(text.contains(anchor), "the task states its expiry:\n{text}");
    root.write(&text.replacen(anchor, &format!("{anchor}      to: 4.0.0\n"), 1));

    let ran = root.run(&["check"]);
    assert_eq!(
        ran.code,
        Some(0),
        "a refused task is reported and does not fail a run\n{}{}",
        ran.out,
        ran.err
    );
    assert!(
        ran.out.contains("AD-9 holds nothing"),
        "the task is refused by name:\n{}",
        ran.out
    );
    assert!(
        ran.out.contains("`to`"),
        "and the refusal names the key:\n{}",
        ran.out
    );
}

/// The generated half is never blamed on the authored block, at the CLI.
///
/// The verifier of #310 built this: a comment added immediately above
/// `resolved:` sits between the payload and the generated taxonomy, and the
/// first cut of the split put those bytes inside the authored span. Exit code
/// and remedy were right and the diagnosis named a block nobody had touched,
/// which is the defect the branch exists to remove.
#[test]
fn a_comment_in_the_generated_half_is_not_blamed_on_the_authored_block() {
    let root = Root::new("comment-in-generated-half");
    root.author();
    let resolved = root.run(&["taxonomy", "resolve"]);
    assert_eq!(
        resolved.code,
        Some(0),
        "the fixture resolves\n{}{}",
        resolved.out,
        resolved.err
    );

    let canonical = root.text();
    root.write(&canonical.replacen("\nresolved:\n", "\n# somebody added a note\nresolved:\n", 1));

    let ran = root.run(&["taxonomy", "resolve", "--check"]);
    assert_eq!(
        ran.code,
        Some(1),
        "the file still differs from the one the sources produce\n{}{}",
        ran.out,
        ran.err
    );
    assert!(
        ran.err.contains("not inside the `adoption` block"),
        "the diagnosis places the difference outside the block:\n{}",
        ran.err
    );
    assert!(
        !ran.err.contains("where the two differ"),
        "and does not blame the block:\n{}",
        ran.err
    );
}

/// A lock that will not read says so, and names a remedy that can succeed.
///
/// `--check` used to print "is not what the sources resolve to" here, which is
/// false — the sources resolve to exactly this taxonomy — with no reason, no
/// moved source, and `headwater taxonomy resolve` as the remedy. That remedy
/// exits 1 and refuses on this input, because nothing can be seen of the
/// authored block. Naming a command that cannot succeed is its own wrong
/// diagnosis, which is the class of defect this branch exists to remove.
#[test]
fn a_lock_that_will_not_read_names_a_remedy_that_can_succeed() {
    let root = Root::new("unreadable-empty-block");
    root.author();
    let resolved = root.run(&["taxonomy", "resolve"]);
    assert_eq!(resolved.code, Some(0), "the fixture resolves");

    // The block a person is invited to edit, emptied. The key is there and its
    // value is null, so the lock is malformed rather than blockless.
    let canonical = root.text();
    let cut = canonical.find("\nadoption:\n").expect("the block is there") + 1;
    let end = canonical[cut..]
        .find("\n# The resolved taxonomy")
        .expect("the generated half follows it")
        + cut;
    root.write(&format!(
        "{}adoption:{}",
        &canonical[..cut],
        &canonical[end..]
    ));

    let ran = root.run(&["taxonomy", "resolve", "--check"]);
    assert_eq!(
        ran.code,
        Some(1),
        "a lock that will not read is not a lock\n{}{}",
        ran.out,
        ran.err
    );
    assert!(
        ran.err.contains("did not read"),
        "the run says what is wrong with the file:\n{}",
        ran.err
    );
    assert!(
        !ran.err.contains("is not what the sources resolve to"),
        "and says nothing about the sources, which it cannot see:\n{}",
        ran.err
    );

    // The remedy `--check` printed, run. It must agree with what was printed.
    let remedy = root.run(&["taxonomy", "resolve"]);
    let refuses = ran.err.contains("refuses this file");
    assert_eq!(
        remedy.code != Some(0),
        refuses,
        "the printed remedy and what the remedy does agree\n--check said:\n{}\nresolve said ({:?}):\n{}{}",
        ran.err,
        remedy.code,
        remedy.out,
        remedy.err
    );
}

/// The digest mismatch keeps the remedy that works, which is #213's case.
///
/// This is the other side of the split above. A lock whose digest no longer
/// matches also does not read, and there `taxonomy resolve` succeeds and
/// carries the authored block through. So the two states must not share one
/// remedy sentence.
#[test]
fn a_lock_whose_digest_does_not_match_is_told_to_resolve() {
    let root = Root::new("unreadable-digest");
    root.author();
    let resolved = root.run(&["taxonomy", "resolve"]);
    assert_eq!(resolved.code, Some(0), "the fixture resolves");
    root.write(
        &root
            .text()
            .replacen("digest: sha256:", "digest: sha256:0000", 1),
    );

    let ran = root.run(&["taxonomy", "resolve", "--check"]);
    assert_eq!(ran.code, Some(1), "{}{}", ran.out, ran.err);
    assert!(
        ran.err.contains("did not read"),
        "the run says what is wrong with the file:\n{}",
        ran.err
    );
    assert!(
        !ran.err.contains("refuses this file"),
        "and this is the state a resolve repairs:\n{}",
        ran.err
    );

    let remedy = root.run(&["taxonomy", "resolve"]);
    assert_eq!(
        remedy.code,
        Some(0),
        "the printed remedy succeeds\n{}{}",
        remedy.out,
        remedy.err
    );
    assert!(
        root.text().contains("AD-9"),
        "and the authored block survived it"
    );
}

// ---------------------------------------------------------------------------
// `adoption.task.expired`: a lapsed task raises a finding rather than staying
// silent. #311 measured the block as a place debt could park forever, because
// nothing ever compared a task's own `until` against the clock. Three arms,
// one shared corpus, only the block's date moved: absent, lapsed, renewed.
// ---------------------------------------------------------------------------

/// The baseline: no adoption block at all, and `--strict` still exits 0.
///
/// This is the arm the other two are read against. `root.corpus()` raises
/// exactly one `language.controlled.not_met` finding, `warn` severity, which
/// does not fail a strict run on its own.
#[test]
fn check_strict_exits_zero_with_no_adoption_task_declared() {
    let root = Root::new("expired-baseline-absent");
    root.corpus();

    let checked = root.run(&["check", "--strict"]);
    assert_eq!(
        checked.code,
        Some(0),
        "a corpus with no declared task passes a strict run\n{}{}",
        checked.out,
        checked.err
    );
    assert!(
        !checked.out.contains("adoption.task.expired ("),
        "no task exists, so the rule fires zero times (the rule still names \
         itself in the catalog of what ran, which is not a finding):\n{}",
        checked.out
    );
}

/// The decisive arm: a task past its `until` fails `--strict`, and the
/// finding names the task and its owner.
///
/// Spec 7: "A migration state past its expiry is a finding against the
/// owner." `2020-01-01` stays in the past for the life of this suite, so the
/// case needs no `--now` flag the way none of this file's other cases do.
#[test]
fn check_strict_is_non_zero_when_a_declared_task_has_lapsed() {
    let root = Root::new("expired-lapsed");
    root.author_until("AD-9", "2020-01-01");
    root.corpus();

    let checked = root.run(&["check", "--strict"]);
    assert_eq!(
        checked.code,
        Some(1),
        "a lapsed task fails a strict run\n{}{}",
        checked.out,
        checked.err
    );
    assert!(
        checked.out.contains("adoption.task.expired"),
        "the rule fired:\n{}",
        checked.out
    );
    assert!(
        checked.out.contains("AD-9"),
        "the finding names the task:\n{}",
        checked.out
    );
    assert!(
        checked.out.contains("a person"),
        "and the owner PAYLOAD declares:\n{}",
        checked.out
    );
}

/// The other direction: the same task, renewed past today, moves the exit
/// code back off the lapsed arm's and onto the absent arm's.
///
/// `FAR` (`2035-01-01`) is the date the rest of this file already uses for an
/// expiry no case reaches, so a renewal here is the ordinary remedy
/// `adoption::expired`'s own message names: move `until` to a new date.
#[test]
fn check_strict_exits_zero_when_a_declared_task_is_renewed() {
    let root = Root::new("expired-renewed");
    root.author_until("AD-9", FAR);
    root.corpus();

    let checked = root.run(&["check", "--strict"]);
    assert_eq!(
        checked.code,
        Some(0),
        "a renewed task passes a strict run, same as the absent arm\n{}{}",
        checked.out,
        checked.err
    );
    assert!(
        !checked.out.contains("adoption.task.expired ("),
        "the rule does not fire on an open task (the rule still names itself \
         in the catalog of what ran, which is not a finding):\n{}",
        checked.out
    );
}

/// The date every case below injects, so a recorded reading is a function of
/// the tree rather than of the day the suite ran.
const AT: &str = "2026-08-28";

/// The rule the one scratch document raises, and the one it does not.
///
/// `corpus()` appends a sentence past the twenty-five-word limit, so the
/// controlled-language rule is the whole of what this corpus fails. The retired
/// term rule reads the same document and finds nothing, which is what makes the
/// two arms below differ in exactly one declared pair.
const RAISED: &str = "language.controlled.not_met";
const UNRAISED: &str = "language.retired_term.used";

/// Arm 1 of the decisive pair: an open, undischarged payload reads as open.
///
/// The corpus is built **before** the block is authored, because the pair has
/// to name the document the scaffolder actually wrote. `PAYLOAD` names
/// `docs/spec/02-taxonomy-model.md`, which this corpus does not hold, so every
/// other case in this file runs at `0 open` and none of them could have caught
/// a reading that reported zero whatever the corpus did.
#[test]
fn a_recorded_reading_states_an_open_payload_as_open() {
    let root = Root::new("decay-open");
    root.corpus();
    let document = root.document();
    root.author_pair("AD-9", "2027-06-30", &document, RAISED);

    let ran = root.run(&["taxonomy", "audit", "--now", AT, "--record"]);
    assert_eq!(
        ran.code,
        Some(0),
        "the audit gates nothing, so it exits 0 whatever it read\n{}{}",
        ran.out,
        ran.err
    );

    let lines = root.store();
    assert_eq!(lines.len(), 1, "one invocation is one reading: {lines:?}");
    let line = &lines[0];
    assert!(
        line.contains("\"id\":\"AD-9\""),
        "the reading names the task:\n{line}"
    );
    assert!(
        line.contains("\"open\":1"),
        "the declared pair still raises its finding:\n{line}"
    );
    assert!(
        line.contains("\"closed\":0"),
        "and nothing about it is discharged:\n{line}"
    );
    assert!(
        line.contains("\"held\":1"),
        "the task is holding the finding out of the report:\n{line}"
    );
    assert!(
        line.contains("\"until\":\"2027-06-30\""),
        "with the expiry a later reader needs to say whether it reached zero in time:\n{line}"
    );
    assert!(
        ran.out.contains("1 pair open"),
        "and the section states the same number:\n{}",
        ran.out
    );
}

/// Arm 2 of the decisive pair: a discharged payload reads as zero.
///
/// One declared pair away from the arm above, and every other input is the
/// same. A store that could not tell these two apart would record a number that
/// no corpus moves, which is the whole of what
/// [HW-OBL-0008](../../../../docs/obligations/0008-an-adoption-payload-has-a-first-reading-and-no-elapsed-time.md)
/// asks the series to measure.
#[test]
fn a_recorded_reading_states_a_discharged_payload_as_zero() {
    let root = Root::new("decay-zero");
    root.corpus();
    let document = root.document();
    root.author_pair("AD-9", "2027-06-30", &document, UNRAISED);

    let ran = root.run(&["taxonomy", "audit", "--now", AT, "--record"]);
    assert_eq!(
        ran.code,
        Some(0),
        "the audit gates nothing\n{}{}",
        ran.out,
        ran.err
    );

    let lines = root.store();
    assert_eq!(lines.len(), 1, "one invocation is one reading: {lines:?}");
    let line = &lines[0];
    assert!(
        line.contains("\"open\":0"),
        "the declared pair raises nothing, so the payload stands at zero:\n{line}"
    );
    assert!(
        line.contains("\"closed\":1"),
        "and the pair that stopped failing is what shrank it:\n{line}"
    );
    assert!(
        line.contains("\"held\":0"),
        "a task at zero holds nothing out of the report:\n{line}"
    );
    assert!(
        ran.out.contains("0 pairs open"),
        "and the section states the same number:\n{}",
        ran.out
    );
    assert!(
        ran.out.contains("until 2027-06-30"),
        "with the expiry beside it, which is what `before the expiry` is read against:\n{}",
        ran.out
    );
}

/// Two lock digests are two measurements, and the report never averages them.
///
/// A denominator made of declarations moves when the taxonomy moves, so a
/// series that summed across the two would report a schema change as a payload
/// that grew or shrank. `headwater capture` refuses the same average for the
/// same reason.
///
/// **The edit is a guidance sentence and deliberately not a comment.** The
/// digest a reading carries is the taxonomy digest, which is over the canonical
/// text and not over the source file, so the comment edit that
/// `a_source_that_moved_still_names_itself_and_keeps_the_old_message` makes
/// leaves it byte identical — that case exists to prove exactly that. A second
/// digest therefore needs a declaration to move, and guidance is the one this
/// corpus reads nothing from: no rule evaluates it, so the two readings differ
/// in their taxonomy and in nothing else.
#[test]
fn two_lock_digests_are_two_measurements_and_the_report_says_so() {
    let root = Root::new("decay-two-locks");
    root.corpus();
    let document = root.document();
    root.author_pair("AD-9", "2027-06-30", &document, RAISED);

    let first = root.run(&["taxonomy", "audit", "--now", AT, "--record"]);
    assert_eq!(
        first.code,
        Some(0),
        "the first reading is taken\n{}{}",
        first.out,
        first.err
    );

    let source = root
        .at
        .join(".headwater/packages/headwater-standard/taxonomy.yml");
    let text = std::fs::read_to_string(&source).expect("the source reads");
    let moved = text.replace(
        "draft: the document is being written or argued over, and nothing may rely on it",
        "draft: the document is being written, and nothing may rely on it",
    );
    assert_ne!(text, moved, "the guidance this case edits is still there");
    std::fs::write(&source, moved).expect("the source writes");
    let resolved = root.run(&["taxonomy", "resolve"]);
    assert_eq!(
        resolved.code,
        Some(0),
        "the moved source resolves, and the authored block is carried through\n{}{}",
        resolved.out,
        resolved.err
    );
    assert!(
        root.text().contains("AD-9"),
        "the payload survived the resolve, or the second reading is over a different corpus"
    );

    let second = root.run(&["taxonomy", "audit", "--now", AT, "--record"]);
    assert_eq!(
        second.code,
        Some(0),
        "the second reading is taken\n{}{}",
        second.out,
        second.err
    );

    let lines = root.store();
    assert_eq!(
        lines.len(),
        2,
        "one date and two digests are two readings: {lines:?}"
    );
    let digest = |line: &str| {
        line.split("\"lock\":\"")
            .nth(1)
            .and_then(|rest| rest.split('"').next())
            .expect("a reading names its lock")
            .to_string()
    };
    assert_ne!(
        digest(&lines[0]),
        digest(&lines[1]),
        "the two readings were taken under two taxonomies: {lines:?}"
    );
    assert!(
        second.out.contains("2 taxonomies"),
        "the section counts the denominators:\n{}",
        second.out
    );
    assert!(
        second.out.contains("not a trend"),
        "and refuses to trend across them:\n{}",
        second.out
    );
}

/// Two recorded audits of one tree at one date still write the same bytes.
///
/// That sentence is in the committed `--now` help text, so a verb that appended
/// on every run would falsify a promise a caller already reads. The refusal is
/// keyed on `(lock, date)`, which is also what makes the store idempotent under
/// a `merge=union` resolution that kept two copies of one reading.
#[test]
fn a_second_recorded_audit_of_one_tree_at_one_date_adds_nothing() {
    let root = Root::new("decay-idempotent");
    root.corpus();
    let document = root.document();
    root.author_pair("AD-9", "2027-06-30", &document, RAISED);

    let first = root.run(&["taxonomy", "audit", "--now", AT, "--record"]);
    assert_eq!(first.code, Some(0), "{}{}", first.out, first.err);
    let after_one = root.store();

    let second = root.run(&["taxonomy", "audit", "--now", AT, "--record"]);
    assert_eq!(second.code, Some(0), "{}{}", second.out, second.err);
    assert_eq!(
        root.store(),
        after_one,
        "the store holds one reading of one tree at one date"
    );
    assert!(
        second.err.contains("already holds"),
        "and the run says on standard error that it added nothing:\n{}",
        second.err
    );
}

/// Without the flag the verb writes nothing at all.
///
/// The audit gates nothing and exits 0, so a run of it inside a gate must leave
/// the tree as it found it. That is the reason the write is opt-in rather than
/// a precaution about a store.
#[test]
fn an_audit_without_the_flag_writes_no_reading() {
    let root = Root::new("decay-no-flag");
    root.corpus();
    let document = root.document();
    root.author_pair("AD-9", "2027-06-30", &document, RAISED);

    let ran = root.run(&["taxonomy", "audit", "--now", AT]);
    assert_eq!(ran.code, Some(0), "{}{}", ran.out, ran.err);
    assert_eq!(root.store(), Vec::<String>::new(), "nothing was written");
    assert!(
        ran.out.contains("no reading recorded"),
        "and the section says the series is empty rather than saying nothing:\n{}",
        ran.out
    );
    assert!(
        ran.out.contains("this run reads"),
        "while still stating what this run read:\n{}",
        ran.out
    );
}

/// A corpus that declares no payload takes a reading all the same.
///
/// `tasks: []` is a real state and it is not the absent store under another
/// name. A series that skipped it would make "nobody has recorded anything" and
/// "the payload is gone" one file.
#[test]
fn a_corpus_with_no_declared_payload_records_a_reading_of_none() {
    let root = Root::new("decay-no-payload");
    root.corpus();

    let ran = root.run(&["taxonomy", "audit", "--now", AT, "--record"]);
    assert_eq!(ran.code, Some(0), "{}{}", ran.out, ran.err);

    let lines = root.store();
    assert_eq!(lines.len(), 1, "a run over no payload is still a reading");
    assert!(
        lines[0].contains("\"tasks\":[]"),
        "and it states that the corpus declared none:\n{}",
        lines[0]
    );
    assert!(
        ran.out.contains("declares no adoption payload"),
        "the section says so in words:\n{}",
        ran.out
    );
}

/// A task this engine cannot read is counted, and the report names the refusal
/// as the cause rather than an absent payload.
///
/// **Two defects at once, and the second is why this arm exists end to end.**
/// The report used to say "this run reads no task, because the lock declares no
/// adoption payload" directly above a line saying that a task was refused. Both
/// states arrive with an empty task list, and the first sentence is the wrong
/// diagnosis for an adopter whose lock holds a task the engine could not parse.
/// `headwater check` gets it right on the same tree.
///
/// And nothing held the `refused` member itself. Forcing `Reading::of` to
/// report `refused: 0` and deleting the render branch outright left the whole
/// suite green, because every series a case built stated zero and the one unit
/// case round-tripped a hand-built value through the writer. Spec 7's words for
/// this member are that "a series that dropped it would report a payload
/// shrinking when it went dark", which is exactly what that pair of mutations
/// produced. This case runs a real refusal through `Reading::of` and out to
/// both artifacts, so neither mutation survives it.
#[test]
fn a_refused_task_is_counted_and_the_report_names_the_refusal_as_the_cause() {
    let root = Root::new("decay-refused");
    root.corpus();
    root.author_until("AD-9", "soon");

    // The premise: this is a refusal and not a lock that will not parse.
    let checked = root.run(&["check", "--now", AT]);
    assert_eq!(
        checked.code,
        Some(0),
        "the lock still reads, and one task in it does not\n{}{}",
        checked.out,
        checked.err
    );
    assert!(
        checked.out.contains("AD-9 holds nothing"),
        "the check layer refuses the task rather than the block:\n{}",
        checked.out
    );

    let ran = root.run(&["taxonomy", "audit", "--now", AT, "--record"]);
    assert_eq!(ran.code, Some(0), "{}{}", ran.out, ran.err);

    let lines = root.store();
    assert_eq!(lines.len(), 1, "one invocation is one reading: {lines:?}");
    let line = &lines[0];
    assert!(
        line.contains("\"refused\":1"),
        "the reading counts the task nobody is measuring:\n{line}"
    );
    assert!(
        line.contains("\"tasks\":[]"),
        "and it holds no task entry, because the task did not read:\n{line}"
    );

    assert!(
        ran.out.contains("the cause is a refusal rather than an"),
        "the section names the cause it actually read:\n{}",
        ran.out
    );
    assert!(
        !ran.out
            .contains("because the lock declares no adoption payload"),
        "and never the cause it did not: this lock declares one\n{}",
        ran.out
    );
    assert!(
        ran.out.contains("it could not read 1 task"),
        "with the count beside it:\n{}",
        ran.out
    );
}
