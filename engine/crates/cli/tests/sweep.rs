// SPDX-License-Identifier: Apache-2.0
//! The exit statuses of `headwater sweep`, which nothing else asserts.
//!
//! # What this target is evidence of
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#four-things-stop-a-sweep-from-gating-and-none-of-them-is-a-rule-that-somebody-keeps)
//! names four mechanisms that stop a sweep from gating, and one of them is the
//! exit status: `sweep report` exits 0 with findings, exits 0 with every
//! finding refused, and exits 0 when it refuses the whole file. That property
//! is stated in spec 12, restated in spec 6, restated in the doc comment on
//! `sweep_report`, and asserted nowhere.
//!
//! `engine/crates/sweep/tests/fixtures.rs` holds the refusals as a library. It
//! starts no process, so it cannot see an exit status at all. This target
//! starts the binary, which is the only place the property is observable.
//!
//! # The third non-zero exit
//!
//! Spec 12 says "the two non-zero exits are the caller's" and names an
//! unreadable path and a `--format` that names no target. There is a third: the
//! lock. Both halves load the corpus through it, so a repository that never ran
//! `headwater taxonomy resolve` gets a non-zero status out of either one.
//! [HW-IFACE-headwater-sweep](../../../../docs/interfaces/headwater-sweep.md)
//! states all three, and the two cases at the end of this file are why it may.

use std::path::{Path, PathBuf};
use std::process::Command;

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

/// A scratch corpus that holds this repository's taxonomy and none of its prose.
struct Root {
    at: PathBuf,
}

impl Root {
    /// `label` names the case rather than the target, because cargo runs the
    /// cases of one target as threads of one process.
    fn new(label: &str, resolving: bool) -> Root {
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-sweep-{}-{label}",
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
        if resolving {
            let resolved = root.run(&["taxonomy", "resolve"]);
            assert_eq!(
                resolved.code,
                Some(0),
                "the fixture resolves\n{}{}",
                resolved.out,
                resolved.err
            );
        }
        root
    }

    /// One document on a shelf, so that a plan has something to describe.
    fn decision(&self, seq: &str) {
        let path = self.at.join(format!("docs/decisions/{seq}-a-choice.md"));
        std::fs::create_dir_all(path.parent().expect("it has a parent"))
            .expect("the shelf directory is there");
        std::fs::write(
            &path,
            format!(
                "---\n\
                 id: HW-DR-{seq}\n\
                 status: draft\n\
                 status_since: 2026-01-01\n\
                 summary: \"A choice, so that the slice of a plan is not empty.\"\n\
                 last_verified: 2026-01-01\n\
                 ---\n\n\
                 # A choice\n\n\
                 ## Context\n\n\
                 One sentence.\n\n\
                 ## Decision\n\n\
                 One sentence.\n\n\
                 ## Consequences\n\n\
                 One sentence.\n"
            ),
        )
        .expect("the decision writes");
    }

    /// A file at the shape `sweep plan` asks an agent to write back.
    fn returned(&self, name: &str, body: &str) -> PathBuf {
        let path = self.at.join(name);
        std::fs::write(&path, body).expect("the return file writes");
        path
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

/// The two streams held apart, because the report is on one and the refusal
/// account of a caller error is on the other.
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

/// The briefing exits 0, and it exits 0 over a slice that holds nothing.
///
/// An empty slice is the reading a caller is most likely to mistake for an
/// error, and it is the one a sampler must not report as one.
#[test]
fn the_briefing_exits_zero_over_a_slice_that_holds_nothing() {
    let root = Root::new("plan", true);
    root.decision("0001");

    let whole = root.run(&["sweep", "plan"]);
    assert_eq!(whole.code, Some(0), "{whole:?}");
    assert!(
        whole.out.contains("1 of the 1 classified documents"),
        "the plan states its own extent:\n{}",
        whole.out
    );

    let empty = root.run(&["sweep", "plan", "--under", "docs/nowhere"]);
    assert_eq!(
        empty.code,
        Some(0),
        "a slice that holds no document is a fact rather than an error\n{empty:?}"
    );

    // Deterministic, which is the property that lets a plan be compared at all.
    let again = root.run(&["sweep", "plan"]);
    assert_eq!(
        again.out, whole.out,
        "two runs over one tree write one plan"
    );
}

/// A model wrote nothing this engine could use, and the status stays 0.
///
/// Three refusals, each decided by a different reading, and every one of them
/// is a statement about what came back rather than about the corpus. That is
/// the whole of the exit-status guarantee spec 12 leans on.
#[test]
fn every_refusal_of_a_returned_file_exits_zero() {
    let root = Root::new("refusals", true);
    root.decision("0001");

    let lock = std::fs::read_to_string(root.at.join(".headwater/taxonomy.lock"))
        .expect("the resolve wrote a lock");
    let digest = lock
        .lines()
        .find_map(|line| line.trim().strip_prefix("digest: "))
        .expect("the lock states its digest")
        .to_string();

    // The whole file refuses itself, because it is not YAML.
    let broken = root.returned(
        "broken.yml",
        "findings: [\n  - class: undeclared_conflict\n",
    );
    let ran = root.run(&["sweep", "report", broken.to_str().expect("a path")]);
    assert_eq!(ran.code, Some(0), "{ran:?}");
    assert!(
        ran.out.contains("did not parse as YAML"),
        "the refusal reached the reader on standard output:\n{}",
        ran.out
    );

    // The whole file refuses itself, because it was planned against another
    // taxonomy. A different reading, and the same status.
    let moved = root.returned(
        "moved.yml",
        "taxonomy: sha256:not-the-lock\nslice: docs\nfindings: []\n",
    );
    let ran = root.run(&["sweep", "report", moved.to_str().expect("a path")]);
    assert_eq!(ran.code, Some(0), "{ran:?}");

    // One finding, refused, because the path it names is not a classified
    // document. The file itself is well formed and current.
    let stray = root.returned(
        "stray.yml",
        &format!(
            "taxonomy: {digest}\n\
             slice: docs\n\
             findings:\n  \
             - class: undefined_concept\n    \
             documents:\n      \
             - docs/nowhere/absent.md\n    \
             evidence:\n      \
             - path: docs/nowhere/absent.md\n        \
             quote: a passage no document holds\n    \
             message: a term nothing defines\n"
        ),
    );
    let ran = root.run(&["sweep", "report", stray.to_str().expect("a path")]);
    assert_eq!(ran.code, Some(0), "{ran:?}");
    assert!(
        ran.out.contains("is not a classified document"),
        "the finding was refused and the run still succeeded:\n{}",
        ran.out
    );

    // And `--strict` is a flag of another verb that this one accepts and
    // ignores. Spec 12 says there is no `--strict`, which is true of what one
    // would do and false of what the parser admits.
    let strict = root.run(&[
        "sweep",
        "report",
        stray.to_str().expect("a path"),
        "--strict",
    ]);
    assert_eq!(strict.code, Some(0), "{strict:?}");
    assert_eq!(
        strict.out, ran.out,
        "`--strict` moves neither the status nor a byte of the report"
    );
}

/// The two non-zero exits spec 12 names, and both are the caller's.
#[test]
fn a_path_that_does_not_read_and_a_format_that_names_nothing_exit_one() {
    let root = Root::new("caller", true);

    let absent = root.run(&["sweep", "report", "/nonexistent/return.yml"]);
    assert_eq!(absent.code, Some(1), "{absent:?}");
    assert!(
        absent.err.contains("the sweep file at"),
        "the message names the file:\n{}",
        absent.err
    );

    let file = root.returned("empty.yml", "findings: []\n");
    let target = root.run(&[
        "sweep",
        "report",
        file.to_str().expect("a path"),
        "--format",
        "sarif",
    ]);
    assert_eq!(target.code, Some(1), "{target:?}");
    assert!(
        target.err.contains("names no target"),
        "the message names the two targets that exist:\n{}",
        target.err
    );

    // The grammar refusals, which are decided before either half is entered.
    for words in [
        vec!["sweep"],
        vec!["sweep", "report"],
        vec!["sweep", "audit"],
    ] {
        let ran = root.run(&words);
        assert_eq!(ran.code, Some(1), "{words:?} is refused\n{ran:?}");
    }
}

/// The third non-zero exit, which spec 12 does not name.
///
/// Both halves load the corpus through the lock, so a repository that never
/// resolved gets 1 out of either one. It is a caller's error on the same
/// reading as the two above, so the property spec 12 defends survives and its
/// count of two does not.
///
/// The ordering claim does not survive either. Spec 12 says both non-zero exits
/// are "decided before any file is parsed". This one is decided after the
/// return file has been read, which the second half of this case shows: the
/// path is a real file and the status is still 1.
#[test]
fn a_corpus_that_never_resolved_exits_one_out_of_either_half() {
    let root = Root::new("no-lock", false);

    let planned = root.run(&["sweep", "plan"]);
    assert_eq!(
        planned.code,
        Some(1),
        "a plan cannot be taken without a lock\n{planned:?}"
    );
    assert!(
        planned.err.contains("taxonomy.lock"),
        "the message names the file and the verb that writes it:\n{}",
        planned.err
    );

    let file = root.returned("empty.yml", "findings: []\n");
    let reported = root.run(&["sweep", "report", file.to_str().expect("a path")]);
    assert_eq!(
        reported.code,
        Some(1),
        "and neither can a report, over a file that read perfectly well\n{reported:?}"
    );
    assert!(
        reported.err.contains("taxonomy.lock"),
        "the file was not the problem, and the message says which one was:\n{}",
        reported.err
    );
}
