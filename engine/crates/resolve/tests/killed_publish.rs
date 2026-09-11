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
        "publisher/.headwater/packages/acme-fixture/package.yml",
        "package: acme/fixture\nversion: 1.0.0\ncontents:\n  taxonomy: taxonomy.yml\n",
    );
    scratch.write("publisher/.headwater/packages/acme-fixture/taxonomy.yml", TAXONOMY);
    let body = "x".repeat(512);
    for index in 0..FILLER {
        scratch.write(
            &format!("publisher/.headwater/packages/acme-fixture/filler/{index:04}.txt"),
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
/// asked for a flag that deletes the leftovers on the predicate *files at
/// `--out` with no record*, which is byte-for-byte what a directory holding
/// somebody's unrelated work looks like. So the window is closed on this path
/// instead, and this is what says it is closed. The flag that shipped fires on
/// a narrower predicate and covers the one path this case cannot: see
/// `package::clear_killed`.
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
/// property over `.headwater/packages/~staging` and found it the same way: a staging path
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
/// same undecidable delete that #485 first asked for, with even the flag that
/// would have made it deliberate taken away. The marker is what makes the
/// removal decidable: a publish removes a directory it created, and nothing
/// else, which is the rule `--clear-killed` keeps as well.
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

// ---------------------------------------------------------------------------
// What a publish killed inside the *direct* write leaves, and whether the next
// run can say whose it is. #677.
// ---------------------------------------------------------------------------

/// The file a publish writes beside [`MARKER`] once it has given up on the
/// rename and is writing the artifact straight into `--out`.
const DIRECT: &str = ".headwater-publish-direct";

/// How many kills each mount-point sweep sends.
///
/// Smaller than [`ATTEMPTS`], because these run inside a namespace and there are
/// two of them. The direct write is the second of the two writes a fallback
/// publish makes, so half of a calibrated run is inside it and a sweep across
/// that half lands there many times over. The cases below fail rather than pass
/// when a sweep lands there zero times, so this number being too small is a red
/// and never a quiet green.
const MOUNT_ATTEMPTS: usize = 30;

/// The staging path a publish derives from `out`.
fn staging_beside(out: &Path) -> PathBuf {
    let mut name = out
        .file_name()
        .expect("the output path has a name")
        .to_os_string();
    name.push("~staging");
    out.with_file_name(name)
}

/// Empty a directory without removing it.
///
/// `--out` is a mount point in every case below, so `remove_dir_all` on it is
/// `EBUSY` and the next attempt of a sweep would publish into a directory the
/// last one filled.
fn empty_out(out: &Path) {
    let Ok(entries) = std::fs::read_dir(out) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        match entry.file_type().map(|kind| kind.is_dir()) {
            Ok(true) => {
                let _ = std::fs::remove_dir_all(&path);
            }
            _ => {
                let _ = std::fs::remove_file(&path);
            }
        }
    }
}

/// How long an uninterrupted publish into `out` takes, having asserted that it
/// is a publish at all.
///
/// A sweep calibrated against a number written down on another machine measures
/// that machine. This is the same calibration the atomic sweep makes, run here
/// against the fallback path, where the run does the artifact write twice — once
/// into the staging directory and once into `--out` — so it is a different
/// number and cannot be borrowed from the case above.
fn calibrate(root: &Path, out: &Path) -> Duration {
    let started = Instant::now();
    let status = spawn_child(root, out)
        .wait()
        .expect("the child is waited on");
    let whole = started.elapsed();
    assert!(status.success(), "the uninterrupted child did not publish");
    assert_eq!(
        at_out(out),
        AtOut::Artifact,
        "the calibration run did not leave an artifact, so the sweep would measure nothing"
    );
    empty_out(out);
    let _ = std::fs::remove_dir_all(staging_beside(out));
    whole
}

/// A publish killed while it writes straight into a mount point leaves, at that
/// instant, a staging directory beside `--out` saying the files are its own.
///
/// # Why this is the case the change exists for
///
/// [`a_killed_publish_never_leaves_files_at_out`] measured the atomic path and
/// found the window closed. The fallback path is the one configuration where a
/// kill can still leave files at `--out`, and until this change the run deleted
/// its own marker on the line **before** that write began — so a kill inside it
/// left files at `--out` and nothing anywhere on disk saying whose. That is
/// byte-for-byte what a directory holding somebody's unrelated work looks like,
/// which is the undecidable state
/// [#485](https://github.com/headwater-ai/headwater/issues/485)'s first
/// predicate could not tell apart from residue. The pair of files this case
/// asserts is what `--clear-killed` fires on, so this case is also what says
/// that flag has something decidable to read.
///
/// # What it would print if the property were absent
///
/// The names of the attempts whose kill left files at the mount point with no
/// staging directory beside them. Against the ordering this change replaced it
/// reports every attempt that landed. It is a timing sweep, so a green is not a
/// proof — which is why it also **fails when no kill lands in the direct write
/// at all**, rather than passing on having measured nothing.
#[test]
#[ignore = "the calibrated write is faster than this host's kill+wait can land inside, \
            deterministically, on ubuntu-latest's tmpfs at HEADWATER_MOUNT_POINT \
            (measured 2026-09-08: 0 of 30 attempts landed, twice); needs a slower real \
            mount point or a different timing strategy, not a CI environment fix"]
fn a_publish_killed_writing_into_a_mount_point_leaves_a_directory_saying_whose_the_files_are() {
    inside_a_mount_point(
        "killed-direct",
        KILLED_DIRECT_CHILD,
        killed_inside_the_direct_write,
    );
}

const KILLED_DIRECT_CHILD: &str = "sweeps_a_mount_point_with_kills";

/// The half of the case above that runs where the mount point is.
#[test]
#[ignore = "the child half of the killed-direct-write case; the parent runs it inside a namespace"]
fn sweeps_a_mount_point_with_kills() {
    in_the_namespace(killed_inside_the_direct_write);
}

fn killed_inside_the_direct_write(root: &Path, out: &Path) {
    let staging = staging_beside(out);
    let whole = calibrate(root, out);

    let mut landed = 0usize;
    let mut unattributed: Vec<usize> = Vec::new();
    let mut refusal = None;
    for attempt in 0..MOUNT_ATTEMPTS {
        // Across the second half of an uninterrupted run and a little past its
        // end. The direct write is the second of the run's two artifact writes,
        // so that span is the one it is in.
        let share = 0.5 + 0.55 * (attempt as f64) / (MOUNT_ATTEMPTS as f64);
        let mut child = spawn_child(root, out);
        std::thread::sleep(Duration::from_secs_f64(whole.as_secs_f64() * share));
        let _ = child.kill();
        let _ = child.wait();

        if let AtOut::Leftovers(_) = at_out(out) {
            landed += 1;
            match staging.join(MARKER).is_file() && staging.join(DIRECT).is_file() {
                false => unattributed.push(attempt),
                // The kill produced the state; the next run is what has to read
                // it. Taking the refusal here rather than in a second case is
                // what makes the verdict a fact about a real killed process.
                true if refusal.is_none() => {
                    refusal = Some(headwater_resolve::render_errors(
                        &package::publish(root, "acme/fixture", out)
                            .expect_err("a publish into a directory holding a killed run's files"),
                    ));
                }
                true => {}
            }
        }
        empty_out(out);
        let _ = std::fs::remove_dir_all(&staging);
    }

    assert!(
        unattributed.is_empty(),
        "a publish killed while writing straight into the mount point left files there and no \
         staging directory saying whose they were, in {} of {MOUNT_ATTEMPTS} attempts: {unattributed:?}",
        unattributed.len()
    );
    assert!(
        landed > 0,
        "no kill of {MOUNT_ATTEMPTS} landed inside the direct write, so this case measured \
         nothing at all about the state it exists for"
    );

    let refusal = refusal.expect("a landing was recorded, so a refusal was read");
    assert!(
        refusal.contains(DIRECT),
        "the next run does not name the file that says the residue is a publish's: {refusal}"
    );
    assert!(
        refusal.contains("killed"),
        "the next run does not say the files are a killed publish's: {refusal}"
    );
}

/// A direct write that fails for a reason other than a kill leaves no staging
/// directory behind.
///
/// # Why this is here
///
/// The change moves the clear of the staging directory from *before* the direct
/// write to *after* it. The reason it was before was that a failed direct write
/// would otherwise leave two directories rather than one, and that reason is
/// still a real one. So the clear runs on the failure path as well, and this is
/// what says it does: without it the case reports a staging directory that
/// outlived a run that failed for an ordinary reason, which is the residue the
/// change would have traded for the one it closed.
#[test]
fn a_direct_write_that_fails_leaves_no_staging_directory() {
    inside_a_mount_point("direct-fails", DIRECT_FAILS_CHILD, a_failed_direct_write);
}

const DIRECT_FAILS_CHILD: &str = "fails_a_direct_write_into_a_mount_point";

/// The half of the case above that runs where the mount point is.
#[test]
#[ignore = "the child half of the failed-direct-write case; the parent runs it inside a namespace"]
fn fails_a_direct_write_into_a_mount_point() {
    in_the_namespace(a_failed_direct_write);
}

fn a_failed_direct_write(root: &Path, out: &Path) {
    use std::os::unix::fs::PermissionsExt;

    let staging = staging_beside(out);
    let _ = std::fs::remove_dir_all(&staging);
    empty_out(out);

    // A mount point this process may read and may not write into. The rename is
    // still refused for the mount, so the run still takes the fallback; the
    // write it falls back to is the thing that then fails.
    let restore = std::fs::metadata(out)
        .expect("the mount point reads")
        .permissions();
    std::fs::set_permissions(out, std::fs::Permissions::from_mode(0o500))
        .expect("the mount point is made unwritable");

    let refused = package::publish(root, "acme/fixture", out);

    std::fs::set_permissions(out, restore).expect("the mount point is made writable again");

    let refused = refused.expect_err("a publish into a directory it cannot write into");
    let message = headwater_resolve::render_errors(&refused);
    assert!(
        !staging.exists(),
        "a direct write that failed for an ordinary reason left `{}` behind, so the ordering \
         traded one residue for another. The run said: {message}",
        staging.display()
    );
    assert_eq!(
        at_out(out),
        AtOut::Empty,
        "the failed run put something at the mount point after all: {message}"
    );
}

/// After a kill, the next run can say whose the files at `--out` are — and the
/// two delivery paths agree about that rather than each being right separately.
///
/// # Why the agreement is the assertion
///
/// The two paths leave different things behind. A kill on the atomic path
/// leaves `--out` as it found it, so the next run publishes into it. A kill on
/// the fallback path leaves files at `--out`, so the next run refuses. Those
/// are different outcomes and asserting them separately says nothing about the
/// property, which is one predicate over both: **is the next run ever left
/// unable to say whose the files are?** That is the state
/// [#485](https://github.com/headwater-ai/headwater/issues/485)'s first
/// predicate could not decide, and what `--clear-killed` needs decided before
/// it removes anything. Before this change the fallback path answered it
/// differently from
/// the atomic path on the same host, in the same run, over the same package.
///
/// # What it would print if the property were absent
///
/// Both verdicts, side by side, with the fallback path's message quoted. The
/// atomic path was already `CanTell::Yes` and stays so, so the inequality names
/// the path that regressed.
#[test]
#[ignore = "the same tmpfs-speed problem as \
            a_publish_killed_writing_into_a_mount_point_leaves_a_directory_saying_whose_the_files_are: \
            the fallback-route half of this comparison calibrates against \
            HEADWATER_MOUNT_POINT and cannot land a kill inside its own write window on \
            ubuntu-latest's tmpfs (measured 2026-09-08: 0 of 30, naming the mount path); the \
            atomic-route half runs on ordinary disk and is not implicated"]
fn both_delivery_paths_agree_that_the_next_run_can_say_whose_the_files_are() {
    inside_a_mount_point("kill-agreement", AGREEMENT_CHILD, both_paths_agree);
}

const AGREEMENT_CHILD: &str = "compares_both_delivery_paths_after_a_kill";

/// The half of the case above that runs where the mount point is.
#[test]
#[ignore = "the child half of the agreement case; the parent runs it inside a namespace"]
fn compares_both_delivery_paths_after_a_kill() {
    in_the_namespace(both_paths_agree);
}

/// Whether the run after a killed publish can say whose the files at `--out`
/// are.
#[derive(Debug, PartialEq)]
enum CanTell {
    /// It can. Either nothing was in its way, or its refusal names the killed
    /// publish that left what is.
    Yes,
    /// It cannot. Files at `--out`, no record, and a refusal that tells the
    /// reader to go and look by hand.
    No(String),
}

/// Which delivery path a half of the comparison is about.
#[derive(Clone, Copy)]
enum Route {
    /// `--out` is an ordinary path, so the artifact is moved into place by one
    /// `rename(2)`.
    Atomic,
    /// `--out` is a mount point, so the artifact is written into it file by
    /// file.
    Fallback,
}

/// Whether a kill landed where the half it is in is about.
///
/// The two are different states by construction, and that is the whole reason
/// the outcomes cannot be compared directly and the predicate over them can. A
/// landed kill on the fallback path leaves files at `--out`. A landed kill on
/// the atomic path leaves `--out` as it found it, so what says the run had
/// started is the staging directory it claimed and did not live to clear.
fn a_kill_landed(route: Route, out: &Path, staging: &Path) -> bool {
    match route {
        Route::Fallback => matches!(at_out(out), AtOut::Leftovers(_)),
        Route::Atomic => staging.exists() && at_out(out) != AtOut::Artifact,
    }
}

/// Kill a publish into `out` until the kill lands, then ask the next run.
fn what_the_next_run_can_tell(root: &Path, out: &Path, route: Route) -> CanTell {
    let staging = staging_beside(out);
    let whole = calibrate(root, out);

    let mut landed = false;
    for attempt in 0..MOUNT_ATTEMPTS {
        let share = 0.5 + 0.55 * (attempt as f64) / (MOUNT_ATTEMPTS as f64);
        let mut child = spawn_child(root, out);
        std::thread::sleep(Duration::from_secs_f64(whole.as_secs_f64() * share));
        let _ = child.kill();
        let _ = child.wait();

        landed = a_kill_landed(route, out, &staging);
        if landed {
            break;
        }
        empty_out(out);
        let _ = std::fs::remove_dir_all(&staging);
    }
    assert!(
        landed,
        "no kill of {MOUNT_ATTEMPTS} into `{}` reached the state this half is about, so the \
         comparison would be between two things that did not happen",
        out.display()
    );

    match package::publish(root, "acme/fixture", out) {
        Ok(_) => CanTell::Yes,
        Err(errors) => {
            let message = headwater_resolve::render_errors(&errors);
            match message.contains(DIRECT) && message.contains("killed") {
                true => CanTell::Yes,
                false => CanTell::No(message),
            }
        }
    }
}

fn both_paths_agree(root: &Path, out: &Path) {
    // The atomic path, on the bind-mounted real filesystem beside the package.
    let ordinary = root
        .parent()
        .expect("the publisher root has a parent")
        .join("agreement-artifact");
    let by_rename = what_the_next_run_can_tell(root, &ordinary, Route::Atomic);

    // The fallback path, at the mount point.
    let by_direct = what_the_next_run_can_tell(root, out, Route::Fallback);

    assert_eq!(
        by_rename, by_direct,
        "the two delivery paths do not agree about whether the run after a kill can say whose \
         the files at `--out` are"
    );
    assert_eq!(
        by_rename,
        CanTell::Yes,
        "both paths agree, and they agree on the wrong answer"
    );
}

// ---------------------------------------------------------------------------
// Reaching a real mount point, for the three cases above.
// ---------------------------------------------------------------------------

/// Read the paths the parent handed the namespace, confirm the mount point
/// before asserting anything, and run `body` there.
///
/// The confirmation is written **before** the body, so the parent can tell a
/// sandbox that did not work from a property that does not hold. Those two are
/// the same exit status otherwise, and reading one for the other is how a check
/// that cannot run comes to read as a check that passes. This is the same
/// contract [`publishes_into_a_mount_point`] states, factored out so the four
/// children cannot drift apart from it.
fn in_the_namespace(body: fn(&Path, &Path)) {
    let root = PathBuf::from(std::env::var("HEADWATER_MOUNT_ROOT").expect("the parent sets it"));
    let out = PathBuf::from(std::env::var("HEADWATER_MOUNT_OUT").expect("the parent sets it"));
    let result =
        PathBuf::from(std::env::var("HEADWATER_MOUNT_RESULT").expect("the parent sets it"));

    if !is_a_mount_point(&out) {
        std::fs::write(&result, "not-a-mount").expect("the verdict is written");
        return;
    }
    std::fs::write(&result, "mounted").expect("the verdict is written");
    body(&root, &out);
}

/// Run `body` somewhere `--out` is a real mount point, or say on the real
/// standard error that this host could not.
///
/// Two routes, the same two [`a_publish_into_a_mount_point_says_it_was_not_atomic`]
/// documents: a mount point somebody already made and named in
/// `HEADWATER_MOUNT_POINT`, used in this process; or one made here with `bwrap
/// --dev-bind / / --tmpfs`, with `child` run inside it. Where neither works the
/// case says so past `libtest`'s capture rather than passing quietly, and
/// `HEADWATER_MOUNT_REQUIRED=1` turns that into a failure on a host that is
/// supposed to manage it.
fn inside_a_mount_point(case: &str, child: &str, body: fn(&Path, &Path)) {
    let scratch = Scratch::new(case);
    let root = wide_publisher(&scratch);
    let mut refused = Vec::new();

    match std::env::var("HEADWATER_MOUNT_POINT") {
        Ok(named) => {
            let out = PathBuf::from(named);
            match is_a_mount_point(&out) {
                true => {
                    body(&root, &out);
                    say(&format!(
                        "headwater-resolve: `{case}` ran, at HEADWATER_MOUNT_POINT"
                    ));
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

    let out = scratch.path().join("artifact");
    let result = scratch.path().join("result");
    std::fs::create_dir_all(&out).expect("the mount point is made");
    let exe = std::env::current_exe().expect("this test binary has a path");
    let ran = std::process::Command::new("bwrap")
        .args(["--dev-bind", "/", "/", "--tmpfs"])
        .arg(&out)
        .arg("--")
        .arg(&exe)
        .args(["--exact", child, "--ignored"])
        .env("HEADWATER_MOUNT_ROOT", &root)
        .env("HEADWATER_MOUNT_OUT", &out)
        .env("HEADWATER_MOUNT_RESULT", &result)
        .output();

    match ran {
        Err(why) => refused.push(format!("`bwrap` did not start: {why}")),
        Ok(done) => match std::fs::read_to_string(&result).unwrap_or_default().trim() {
            "mounted" => {
                assert!(
                    done.status.success(),
                    "`{case}` did not hold inside the namespace:\n{}\n{}",
                    String::from_utf8_lossy(&done.stdout),
                    String::from_utf8_lossy(&done.stderr)
                );
                say(&format!(
                    "headwater-resolve: `{case}` ran, inside `bwrap --tmpfs`"
                ));
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
        "headwater-resolve: `{case}` DID NOT RUN on this host, so nothing here measured what a \
         publish killed inside a direct write into a mount point leaves behind. Why: {reason}"
    ));
    assert!(
        std::env::var("HEADWATER_MOUNT_REQUIRED").is_err(),
        "HEADWATER_MOUNT_REQUIRED is set, so this host is supposed to reach a mount point: {reason}"
    );
}

/// An empty directory at the staging path does not block a publish forever.
///
/// # The kill this is about
///
/// `claim_staging` makes the directory and writes the marker with two syscalls.
/// A kill between them leaves an empty unmarked directory, and every later
/// publish into the same `--out` was then refused by the arm that refuses a
/// directory no publish wrote — over a directory a publish had in fact just
/// made. The kill sweep above reproduced it on about one run in three before
/// this was fixed, and the refusal it produced named `~staging` and told the
/// reader to publish somewhere else.
///
/// The removal stays decidable: it is `remove_dir` and not `remove_dir_all`, so
/// a directory holding anything at all is still refused, which the case below
/// this one is what says.
#[test]
fn an_empty_directory_at_the_staging_path_does_not_block_a_publish() {
    let scratch = Scratch::new("staging-empty");
    let root = wide_publisher(&scratch);
    let out = scratch.path().join("artifact");
    let staging = staging_beside(&out);
    std::fs::create_dir_all(&staging).expect("the killed run's directory is planted");

    package::publish(&root, "acme/fixture", &out)
        .expect("a publish is not blocked by an empty directory a killed publish left");
    assert_eq!(at_out(&out), AtOut::Artifact);
    assert!(
        !staging.exists(),
        "the staging directory outlived the run that took it over"
    );
}

// ---------------------------------------------------------------------------
// `--clear-killed` against the only `--out` that can ever hold this residue:
// a real mount point. #485.
// ---------------------------------------------------------------------------

/// **`clear_killed` must empty `--out` and never remove it.**
///
/// # Why the ordinary case cannot find this
///
/// [`DIRECT`] is written on exactly one branch of `deliver` — the one a failed
/// rename reaches, which is a mount point or a filesystem boundary. So every
/// `--out` that can carry the residue this flag clears is a path
/// `remove_dir_all` answers with `EBUSY`. Every case that plants the residue at
/// an ordinary directory therefore exercises a removal the flag can never
/// perform in production, and passes.
///
/// [`empty_out`] above this line already knew it, and says so in its own doc
/// comment: `--out` is a mount point in every case in this file, so
/// `remove_dir_all` on it is `EBUSY`. The first cut of `clear_killed` called
/// the primitive that this file rules out, one directory over. The contents
/// went, the mount survived, the run exited 1, and every retry repeated
/// identically — a recovery that cannot recover, in the one configuration it
/// exists for.
///
/// # What it would print if the property were absent
///
/// The refusal `clear_killed` returns, which quotes the `EBUSY` verbatim.
#[test]
fn the_flag_empties_a_mount_point_it_cannot_remove_and_publishes_into_it() {
    inside_a_mount_point(
        "clear-killed-mount",
        CLEAR_KILLED_CHILD,
        clears_a_mount_point,
    );
}

const CLEAR_KILLED_CHILD: &str = "clears_a_killed_run_inside_a_mount_point";

/// The half of the case above that runs where the mount point is.
#[test]
#[ignore = "the child half of the clear-killed case; the parent runs it inside a namespace"]
fn clears_a_killed_run_inside_a_mount_point() {
    in_the_namespace(clears_a_mount_point);
}

fn clears_a_mount_point(root: &Path, out: &Path) {
    let staging = staging_beside(out);

    // The residue a killed direct write leaves, planted at a path that really
    // cannot be renamed onto and really cannot be removed.
    let _ = std::fs::remove_dir_all(&staging);
    std::fs::create_dir_all(&staging).expect("the staging directory is made");
    std::fs::write(
        staging.join(MARKER),
        "an earlier run claimed this directory\n",
    )
    .expect("the marker is written");
    std::fs::write(
        staging.join(DIRECT),
        format!("an earlier run\nThe output path is: {}\n", out.display()),
    )
    .expect("the note is written");
    empty_out(out);
    std::fs::write(out.join("taxonomy.yml"), "half of an artifact\n")
        .expect("the residue is written");

    assert!(
        std::fs::remove_dir_all(out).is_err(),
        "this case measures nothing unless `--out` is a path `remove_dir_all` refuses"
    );

    let cleared = package::clear_killed(root, out).unwrap_or_else(|why| {
        panic!(
            "the clear failed: {}",
            headwater_resolve::render_errors(&why)
        )
    });
    assert!(
        matches!(cleared, package::Cleared::KilledDirectWrite { .. }),
        "the flag did not recognize the residue it was handed"
    );
    assert!(
        out.is_dir(),
        "the flag removed the mount point rather than emptying it"
    );
    assert_eq!(
        std::fs::read_dir(out)
            .expect("the mount point reads")
            .count(),
        0,
        "the killed run's files outlived the clear"
    );
    assert!(
        !staging.exists(),
        "the killed run's own directory outlived the clear"
    );

    package::publish(root, "acme/fixture", out)
        .expect("the same run publishes into the mount point");
    assert!(
        release::at(out).is_ok(),
        "the publish after the clear left no record"
    );
}
