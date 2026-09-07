// SPDX-License-Identifier: Apache-2.0
//! What a `taxonomy publish` that is killed part-way leaves at `--out`.
//!
//! # Why this is its own target, and out of process
//!
//! Every other case in this workspace that is about a killed publish plants the
//! state a kill lands in by hand — publish, then delete the record — because a
//! kill is not something a test can ask a function for. That is enough to hold
//! the *message* a later run gives, and it is no evidence at all about whether a
//! kill can still reach that state. The two questions are different, and only
//! the second one is
//! [#485](https://github.com/headwater-ai/headwater/issues/485).
//!
//! So this target spawns itself as a child process, sends the child `SIGKILL`
//! across a sweep of delays calibrated against an uninterrupted run, and asks
//! what is at `--out` afterwards. It is a real signal to a real process, which
//! is the only instrument that answers the question.
//!
//! `cargo test` fail-fasts across targets, so the sweep lives here rather than
//! in `publish.rs`: a failure of this property is then reported as itself rather
//! than hidden behind whichever target ran first.
//!
//! # What the sweep can and cannot prove
//!
//! It is a timing sweep, so a red is probabilistic and a green is not a proof.
//! Against the direct write it preceded, it reported 18 of 60 attempts leaving
//! files at `--out` with no record; against the staging rename, 0 of 60. The
//! window it was finding is the whole of the write phase. After it, the property
//! holds by construction rather than by timing: the artifact is assembled at a
//! sibling path and `--out` is reached with one `rename(2)`. The deterministic
//! cases below the sweep are what hold that construction — the suffix `--out`
//! reserves, the staging directory a successful run leaves behind, and the
//! residue an earlier killed run leaves in it.

use headwater_resolve::package;
use headwater_resolve::release;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// A tree that removes itself, named for the case that made it.
struct Scratch(PathBuf);

impl Scratch {
    fn new(case: &str) -> Self {
        let at =
            std::env::temp_dir().join(format!("headwater-killed-{}-{case}", std::process::id()));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).expect("the scratch tree is made");
        Scratch(at)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn write(&self, relative: &str, text: &str) {
        let path = self.0.join(relative);
        std::fs::create_dir_all(path.parent().expect("it has a parent"))
            .expect("the parent is made");
        std::fs::write(path, text).expect("the file is written");
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

const TAXONOMY: &str = "\
taxonomy: acme/fixture
version: 1.0.0
purposes:
  rationale: {intent: explain why a choice was made and what it forecloses}
";

/// How many files of filler the publisher carries beside its two real ones.
///
/// A three-file package publishes in well under a millisecond, and a sweep
/// against one measures the scheduler rather than the verb. The filler widens
/// the write phase — the walk that reads it, the writes that place it and the
/// digest over every member — until a kill can land inside it often enough for a
/// red to mean something. It is carried into the artifact as ordinary files,
/// which is what a package's own tree is.
const FILLER: usize = 240;

/// How many kills the sweep sends.
const ATTEMPTS: usize = 60;

/// A publisher whose package carries [`FILLER`] files, so that the phase this
/// case is about takes measurable time.
fn wide_publisher(scratch: &Scratch) -> PathBuf {
    scratch.write(
        "publisher/packages/acme-fixture/package.yml",
        "package: acme/fixture\nversion: 1.0.0\ncontents:\n  taxonomy: taxonomy.yml\n",
    );
    scratch.write("publisher/packages/acme-fixture/taxonomy.yml", TAXONOMY);
    let body = "x".repeat(512);
    for index in 0..FILLER {
        scratch.write(
            &format!("publisher/packages/acme-fixture/filler/{index:04}.txt"),
            &body,
        );
    }
    scratch.path().join("publisher")
}

/// What a directory at `--out` holds, in the terms this case is about.
#[derive(Debug, PartialEq)]
enum AtOut {
    /// Not there. The state a kill before the first write leaves.
    Absent,
    /// There and holding nothing. The state a kill leaves when the caller made
    /// the directory, and the state a run publishes into at exit 0.
    Empty,
    /// A publish that finished before the kill reached it.
    Artifact,
    /// Files with no record that reads back. This is the state the issue is
    /// about and the one the sweep refuses.
    Leftovers(usize),
}

fn at_out(out: &Path) -> AtOut {
    let Ok(entries) = std::fs::read_dir(out) else {
        return AtOut::Absent;
    };
    let count = entries.count();
    if count == 0 {
        return AtOut::Empty;
    }
    match release::at(out).is_ok() {
        true => AtOut::Artifact,
        false => AtOut::Leftovers(count),
    }
}

/// The child half of the sweep, run by [`a_killed_publish_never_leaves_files_at_out`]
/// as its own process so that a signal has something to arrive at.
///
/// It is `#[ignore]`d, so an ordinary run of this target never reaches it and
/// the parent names it explicitly. It publishes and says nothing: the parent
/// reads the disk rather than the child, because a killed child reports nothing
/// by definition.
#[test]
#[ignore = "the child half of the kill sweep; the parent spawns it by name"]
fn publishes_until_it_is_killed() {
    let root = PathBuf::from(std::env::var("HEADWATER_KILL_ROOT").expect("the parent sets it"));
    let out = PathBuf::from(std::env::var("HEADWATER_KILL_OUT").expect("the parent sets it"));
    let _ = package::publish(&root, "acme/fixture", &out);
}

fn spawn_child(root: &Path, out: &Path) -> std::process::Child {
    std::process::Command::new(std::env::current_exe().expect("this test binary has a path"))
        .args(["--exact", "publishes_until_it_is_killed", "--ignored"])
        .env("HEADWATER_KILL_ROOT", root)
        .env("HEADWATER_KILL_OUT", out)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("the child starts")
}

/// A publish killed at any point leaves `--out` in a state the next publish
/// takes, and never in a state a publisher has to clear by hand.
///
/// This is [#485](https://github.com/headwater-ai/headwater/issues/485)'s
/// question, asked of a process rather than of a planted directory. The issue
/// asked for a flag that deletes the leftovers; the measurement that refused the
/// flag is that *files at `--out` with no record* is byte-for-byte what a
/// directory holding somebody's unrelated work looks like. So the window is
/// closed instead, and this is what says it is closed.
///
/// The sweep alternates the state `--out` starts in, because a publish observes
/// two of them and the undo has a branch for each. `<out>~staging` is
/// deliberately **not** cleared between attempts: residue accumulating there is
/// the same class of defect one directory over, and the final publish is what
/// says the residue neither blocks the retry nor reaches the artifact.
#[test]
fn a_killed_publish_never_leaves_files_at_out() {
    let scratch = Scratch::new("kill-sweep");
    let root = wide_publisher(&scratch);
    let out = scratch.path().join("artifact");

    // Calibrate against an uninterrupted child, so the sweep is over this
    // machine's timings rather than over a number written down on another one.
    let started = Instant::now();
    let status = spawn_child(&root, &out)
        .wait()
        .expect("the child is waited on");
    let whole = started.elapsed();
    assert!(status.success(), "the uninterrupted child did not publish");
    assert_eq!(
        at_out(&out),
        AtOut::Artifact,
        "the calibration run did not leave an artifact, so the sweep would measure nothing"
    );
    std::fs::remove_dir_all(&out).expect("the calibration artifact is cleared");

    let mut seen: Vec<(usize, AtOut)> = Vec::new();
    for attempt in 0..ATTEMPTS {
        if attempt % 2 == 1 {
            std::fs::create_dir_all(&out).expect("the empty starting state is made");
        }
        // Across the second half of an uninterrupted run and a little past its
        // end. Every write a publish makes is in that span, and the tail catches
        // the record write itself.
        let share = 0.5 + 0.55 * (attempt as f64) / (ATTEMPTS as f64);
        let mut child = spawn_child(&root, &out);
        std::thread::sleep(Duration::from_secs_f64(whole.as_secs_f64() * share));
        let _ = child.kill();
        let _ = child.wait();

        let state = at_out(&out);
        if let AtOut::Leftovers(count) = state {
            seen.push((attempt, AtOut::Leftovers(count)));
        }
        let _ = std::fs::remove_dir_all(&out);
    }

    assert!(
        seen.is_empty(),
        "a killed publish left files at --out with no record, in {} of {ATTEMPTS} attempts: {seen:?}",
        seen.len()
    );

    // The retry needs nothing removed, including the staging residue the sweep
    // has been leaving beside `--out` for the whole of it.
    let record = package::publish(&root, "acme/fixture", &out)
        .expect("a publish after the sweep is refused nothing");
    assert_eq!(
        release::at(&out).expect("the artifact reads back").digest,
        record.digest,
        "the artifact the retry wrote is not the one it recorded"
    );
}

/// A file an earlier killed run left in the staging directory is not carried
/// into the artifact.
///
/// The deterministic half of the sweep's last assertion. `vendor` holds the same
/// property over `packages/~staging` and found it the same way: a staging path
/// that is written into rather than made is a staging path that publishes
/// whatever an earlier run left in it.
#[test]
fn a_file_left_in_the_staging_directory_is_not_published() {
    let scratch = Scratch::new("staging-residue");
    let root = wide_publisher(&scratch);
    let out = scratch.path().join("artifact");
    let staging = scratch.path().join("artifact~staging");
    std::fs::create_dir_all(staging.join("stale"))
        .expect("the residue an earlier killed run left is planted");
    std::fs::write(staging.join("stale/leftover.txt"), "not this run's\n")
        .expect("the residue is written");

    package::publish(&root, "acme/fixture", &out).expect("the residue does not refuse the publish");
    assert!(
        !out.join("stale").exists(),
        "the artifact carries a file no publish staged"
    );
    assert!(
        !staging.exists(),
        "the staging directory is still there after a run that finished"
    );
}

/// An `--out` whose name ends in the suffix a publish reserves is refused.
///
/// A publish clears that path before it writes, so an `--out` there would be an
/// `--out` a second publish sweeps. `vendor` gets this for free from the package
/// grammar, which admits no `~`; `--out` is a free path and has to be told.
#[test]
fn an_output_path_that_ends_in_the_reserved_suffix_is_refused() {
    let scratch = Scratch::new("reserved-suffix");
    let root = wide_publisher(&scratch);
    let out = scratch.path().join("artifact~staging");

    let refused = package::publish(&root, "acme/fixture", &out).expect_err("it does not publish");
    let message = headwater_resolve::render_errors(&refused);
    assert!(
        message.contains("~staging"),
        "the refusal does not name the suffix it is about: {message}"
    );
    assert!(!out.exists(), "the refused run made the path anyway");
}

/// A publish that succeeds leaves nothing beside `--out`.
#[test]
fn a_publish_that_succeeds_leaves_no_staging_directory() {
    let scratch = Scratch::new("no-residue");
    let root = wide_publisher(&scratch);
    let out = scratch.path().join("artifact");

    package::publish(&root, "acme/fixture", &out).expect("it publishes");
    assert!(
        !scratch.path().join("artifact~staging").exists(),
        "the staging directory outlived the run that made it"
    );
    assert_eq!(at_out(&out), AtOut::Artifact);
}
