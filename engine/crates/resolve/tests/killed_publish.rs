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

/// The marker file a publish writes into its staging directory as its first act.
const MARKER: &str = ".headwater-publish-staging";

/// A file an earlier killed run left in the staging directory is not carried
/// into the artifact.
///
/// The deterministic half of the sweep's last assertion. `vendor` holds the same
/// property over `packages/~staging` and found it the same way: a staging path
/// that is written into rather than made is a staging path that publishes
/// whatever an earlier run left in it.
///
/// The marker is planted with the residue, because that is what a killed run
/// leaves: a publish writes the marker before it writes an artifact byte, so
/// every state a kill can produce carries one.
#[test]
fn a_file_left_in_the_staging_directory_is_not_published() {
    let scratch = Scratch::new("staging-residue");
    let root = wide_publisher(&scratch);
    let out = scratch.path().join("artifact");
    let staging = scratch.path().join("artifact~staging");
    std::fs::create_dir_all(staging.join("assembly/stale"))
        .expect("the residue an earlier killed run left is planted");
    std::fs::write(
        staging.join("assembly/stale/leftover.txt"),
        "not this run's\n",
    )
    .expect("the residue is written");
    std::fs::write(staging.join(MARKER), "an earlier run claimed this\n")
        .expect("the marker a killed run leaves is planted");

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

/// A directory at the staging path that no publish wrote is refused, and it
/// survives.
///
/// This is the case the removal exists to get right. An unconditional
/// `remove_dir_all` at a path derived from every `--out` anybody passes is the
/// same undecidable delete that #485's own flag was refused for, with the flag
/// that made it deliberate taken away. The marker is what makes the removal
/// decidable: a publish removes a directory it created, and nothing else.
#[test]
fn a_directory_at_the_staging_path_that_no_publish_made_is_refused_and_survives() {
    let scratch = Scratch::new("staging-not-ours");
    let root = wide_publisher(&scratch);
    let out = scratch.path().join("artifact");
    let staging = scratch.path().join("artifact~staging");
    std::fs::create_dir_all(&staging).expect("the caller's own directory is made");
    std::fs::write(staging.join("notes.txt"), "the caller's own file")
        .expect("their file is written");
    std::fs::write(staging.join("main.rs"), "fn main() {}").expect("their second file is written");

    let refused = package::publish(&root, "acme/fixture", &out).expect_err("it does not publish");
    let message = headwater_resolve::render_errors(&refused);
    assert!(
        message.contains("artifact~staging"),
        "the refusal does not name the path that stopped the run: {message}"
    );
    assert!(
        message.contains(MARKER),
        "the refusal does not say what it looked for: {message}"
    );
    assert_eq!(
        std::fs::read_to_string(staging.join("notes.txt")).expect("their file reads"),
        "the caller's own file"
    );
    assert!(
        staging.join("main.rs").exists(),
        "the refused run deleted the caller's directory anyway"
    );
    assert!(!out.exists(), "the refused run wrote an artifact anyway");
}

/// A regular file at the staging path is refused, and the refusal names that
/// path rather than `--out`.
///
/// Before the marker, this reached `create_dir_all` and reported `<out>: cannot
/// create it: File exists` — a message naming a path that exists and is not the
/// one that stopped the run.
#[test]
fn a_file_at_the_staging_path_is_refused_by_the_path_that_stopped_the_run() {
    let scratch = Scratch::new("staging-is-a-file");
    let root = wide_publisher(&scratch);
    let out = scratch.path().join("artifact");
    let staging = scratch.path().join("artifact~staging");
    std::fs::write(&staging, "somebody's file\n").expect("the file at the staging path is planted");

    let refused = package::publish(&root, "acme/fixture", &out).expect_err("it does not publish");
    let message = headwater_resolve::render_errors(&refused);
    assert!(
        message.contains("artifact~staging"),
        "the refusal does not name the path that stopped the run: {message}"
    );
    assert!(
        !message.contains("cannot create it: File exists"),
        "the refusal still names --out for a file at the staging path: {message}"
    );
    assert_eq!(
        std::fs::read_to_string(&staging).expect("their file reads"),
        "somebody's file\n"
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

// ---------------------------------------------------------------------------
// How the artifact reached `--out`, and whether the run says so. #664.
// ---------------------------------------------------------------------------

/// Write a line past `libtest`'s output capture, onto the real standard error.
///
/// `eprintln!` goes through `std::io::_eprint`, which `libtest` redirects into a
/// per-case buffer that a passing case throws away. A reason nobody reads is the
/// silent skip this run has been cataloguing, so the two lines below that say a
/// case did not run write to the file descriptor instead, where a person and a
/// continuous-integration log both see them whatever `--nocapture` says.
fn say(line: &str) {
    use std::io::Write;
    let _ = writeln!(std::io::stderr().lock(), "{line}");
}

/// Whether the kernel says `at` is a mount point.
///
/// Read out of `/proc/self/mountinfo` rather than probed with a rename, because
/// the probe worth running is the one the publish itself runs and a second one
/// beside it would move the directory under test. Field 4 of a `mountinfo` line
/// is the mount point; the paths here are made by this file and carry no space,
/// so the octal escaping that format uses never applies.
fn is_a_mount_point(at: &Path) -> bool {
    let Ok(table) = std::fs::read_to_string("/proc/self/mountinfo") else {
        return false;
    };
    let wanted = at.to_string_lossy().to_string();
    table
        .lines()
        .filter_map(|line| line.split_whitespace().nth(4))
        .any(|point| point == wanted)
}

/// Publish into `out` and assert everything the weaker path must say.
///
/// One body, two routes to a mount point, so the assertions cannot drift apart
/// between the case that runs here and the case that runs inside a namespace.
fn a_direct_delivery_is_reported(root: &Path, out: &Path) {
    let done = package::publish_delivered(root, "acme/fixture", out).expect("it publishes");

    // The artifact still arrives. The fallback is a weaker guarantee and not a
    // failure, and a case that let the publish fail would be measuring the
    // wrong thing.
    assert_eq!(
        at_out(out),
        AtOut::Artifact,
        "the artifact did not reach the mount point at all"
    );

    match &done.delivery {
        package::Delivery::Direct(package::Direct::Mount { error }) => assert!(
            !error.is_empty(),
            "the fallback quotes no kernel error, so the report cannot name one"
        ),
        other => panic!("a rename onto a mount point was reported as {other:?}"),
    }
    assert_eq!(done.delivery.wire(), "direct");
    let shortfall = done
        .delivery
        .shortfall()
        .expect("a direct delivery tells the publisher what it did not get");
    assert!(
        shortfall.contains("mount point"),
        "the sentence a publisher reads does not name the reason: {shortfall}"
    );

    let document = release::document(&done.release, out, &done.delivery).render_pretty();
    assert!(
        document.contains("\"delivery\": \"direct\""),
        "the JSON document does not carry the fallback: {document}"
    );
}

/// The value a publish onto an ordinary path reports, and the shape of the
/// document that carries it.
///
/// **This case is not evidence on its own, and it must not be read as any.** A
/// `document` with `"delivery": "renamed"` written into it as a constant passes
/// it, and so does an engine that can never produce `direct` at all. It is the
/// always-running half of a pair whose other half is
/// [`a_publish_into_a_mount_point_says_it_was_not_atomic`], and only that half
/// can fail for the reason the pair exists.
#[test]
fn a_publish_onto_an_ordinary_path_is_delivered_by_a_rename() {
    let scratch = Scratch::new("delivery-renamed");
    let root = wide_publisher(&scratch);
    let out = scratch.path().join("artifact");

    let done = package::publish_delivered(&root, "acme/fixture", &out).expect("it publishes");
    assert_eq!(done.delivery, package::Delivery::Renamed);
    assert_eq!(done.delivery.wire(), "renamed");
    assert!(
        done.delivery.shortfall().is_none(),
        "an atomic publish printed a shortfall, so every publish prints one and nobody reads it"
    );

    let document = release::document(&done.release, &out, &done.delivery).render_pretty();
    assert!(
        document.contains("\"delivery\": \"renamed\""),
        "the JSON document does not say how the artifact arrived: {document}"
    );
    assert!(
        document.contains("\"version\": \"1.1\""),
        "the document shape did not move with the member added to it: {document}"
    );
}

/// The name of the child case, run inside whatever made the mount point.
const MOUNT_CHILD: &str = "publishes_into_a_mount_point";

/// A publish whose `--out` is a real mount point reports that it was not atomic.
///
/// # Why this is the decisive case
///
/// `rename(2)` onto a mount point is refused with `EBUSY`, so the staging
/// directory cannot be moved into place and the artifact is written file by
/// file instead. Before [#664] the run exited 0 with the whole artifact and said
/// nothing about it on standard output, on standard error or in the JSON
/// document — a continuous-integration publisher writing into a mounted volume
/// was in the one configuration without the atomicity guarantee, and the only
/// way to find out was to be killed part-way and read the *next* run's refusal.
///
/// # What it does not reach, and how it says so
///
/// It needs a real mount point, which needs either a pre-made one or a user
/// namespace. Two routes are tried and each is named in the log:
///
/// 1. `HEADWATER_MOUNT_POINT` naming an empty directory somebody already mounted
///    — the route for a runner with `sudo` and no user namespaces.
/// 2. `bwrap --dev-bind / / --tmpfs <out>`, which needs `bubblewrap` installed
///    and unprivileged user namespaces permitted. `bwrap` is not setuid, so this
///    is not a privilege; Ubuntu's `kernel.apparmor_restrict_unprivileged_userns`
///    refuses a bare `unshare -Umr` and permits `bwrap` through its own profile.
///
/// **Where neither route works this case does not skip quietly.** It writes the
/// reason onto the real standard error, past `libtest`'s capture, so the log of
/// a green run says in as many words that the end-to-end wiring between the
/// classifier and the document went unmeasured on that host. Set
/// `HEADWATER_MOUNT_REQUIRED=1` to turn that into a failure on a host that is
/// supposed to be able to do it.
///
/// [#664]: https://github.com/headwater-ai/headwater/issues/664
#[test]
fn a_publish_into_a_mount_point_says_it_was_not_atomic() {
    let scratch = Scratch::new("delivery-direct");
    let root = wide_publisher(&scratch);
    let mut refused = Vec::new();

    // Route 1: a mount point somebody else made, used in this process.
    match std::env::var("HEADWATER_MOUNT_POINT") {
        Ok(named) => {
            let out = PathBuf::from(named);
            match is_a_mount_point(&out) {
                true => {
                    a_direct_delivery_is_reported(&root, &out);
                    say("headwater-resolve: the mount-point case ran, at HEADWATER_MOUNT_POINT");
                    return;
                }
                false => refused.push(format!(
                    "HEADWATER_MOUNT_POINT names `{}`, and the kernel does not call it a mount \
                     point",
                    out.display()
                )),
            }
        }
        Err(_) => refused.push(
            "HEADWATER_MOUNT_POINT is unset, so no mount point was handed to this run".to_string(),
        ),
    }

    // Route 2: make one, in a user namespace, and run the child case inside it.
    let out = scratch.path().join("artifact");
    let result = scratch.path().join("delivery.result");
    std::fs::create_dir_all(&out).expect("the mount point is made");
    let exe = std::env::current_exe().expect("this test binary has a path");
    let ran = std::process::Command::new("bwrap")
        .args(["--dev-bind", "/", "/", "--tmpfs"])
        .arg(&out)
        .arg("--")
        .arg(&exe)
        .args(["--exact", MOUNT_CHILD, "--ignored"])
        .env("HEADWATER_MOUNT_ROOT", &root)
        .env("HEADWATER_MOUNT_OUT", &out)
        .env("HEADWATER_MOUNT_RESULT", &result)
        .output();

    match ran {
        Err(why) => refused.push(format!("`bwrap` did not start: {why}")),
        Ok(done) => match std::fs::read_to_string(&result).unwrap_or_default().trim() {
            // The child confirmed the mount before it asserted anything, so its
            // status is a verdict about the publish and never about the sandbox.
            "mounted" => {
                assert!(
                    done.status.success(),
                    "the publish into a mount point did not report itself:\n{}\n{}",
                    String::from_utf8_lossy(&done.stdout),
                    String::from_utf8_lossy(&done.stderr)
                );
                say("headwater-resolve: the mount-point case ran, inside `bwrap --tmpfs`");
                return;
            }
            "not-a-mount" => refused
                .push("`bwrap` ran and its `--tmpfs` did not become a mount point".to_string()),
            _ => refused.push(format!(
                "`bwrap` reached no verdict ({}): {}",
                done.status,
                String::from_utf8_lossy(&done.stderr).trim()
            )),
        },
    }

    let reason = refused.join("; ");
    say(&format!(
        "headwater-resolve: THE MOUNT-POINT CASE DID NOT RUN on this host, so nothing here \
         measured that a publish into a mount point reports `delivery: direct`. Only the \
         classifier and the ordinary path were covered, and neither can fail for that reason. \
         Why: {reason}"
    ));
    assert!(
        std::env::var("HEADWATER_MOUNT_REQUIRED").is_err(),
        "HEADWATER_MOUNT_REQUIRED is set, so this host is supposed to reach a mount point: {reason}"
    );
}

/// The half of the mount-point case that runs inside the namespace.
///
/// It is `#[ignore]`d, so an ordinary run never reaches it and the parent names
/// it. It states in the result file whether the mount point is real **before**
/// it asserts anything, so the parent can tell a sandbox that did not work from
/// a publish that did not report itself. Those two are the same exit status
/// otherwise, and reading one for the other is how a check that cannot run comes
/// to read as a check that passes.
#[test]
#[ignore = "the child half of the mount-point case; the parent runs it inside a namespace"]
fn publishes_into_a_mount_point() {
    let root = PathBuf::from(std::env::var("HEADWATER_MOUNT_ROOT").expect("the parent sets it"));
    let out = PathBuf::from(std::env::var("HEADWATER_MOUNT_OUT").expect("the parent sets it"));
    let result =
        PathBuf::from(std::env::var("HEADWATER_MOUNT_RESULT").expect("the parent sets it"));

    if !is_a_mount_point(&out) {
        std::fs::write(&result, "not-a-mount").expect("the verdict is written");
        return;
    }
    std::fs::write(&result, "mounted").expect("the verdict is written");
    a_direct_delivery_is_reported(&root, &out);
}
