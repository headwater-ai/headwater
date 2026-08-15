// SPDX-License-Identifier: Apache-2.0
//! The migration payload, from the manifest key that declares it to the report
//! that accounts it against a measured corpus.
//!
//! # The defect this target exists for
//!
//! A format with no caller is a schema that nobody has been forced to satisfy.
//! Before this target `contents.migrations` was a key that `taxonomy publish`
//! read and dropped: seven hand-written wrong payloads all published with exit
//! 0, digest and all, and a consumer would have fetched every one of them. So
//! the cases below are a caller. `fixtures/migration/1-to-2.yml` is a committed
//! payload that a run reads, and a change to the format that nothing else
//! notices fails here.
//!
//! # Why the accounting is the case that matters
//!
//! A step declares which of spec 2's six dimensions it remedies by the subject
//! it names, and a claim of that shape is worth nothing until something
//! measures it. So the first case publishes a candidate that moves three
//! lifecycle values, runs `taxonomy diff` over a corpus that carries all three,
//! and asserts that the documents the payload names are exactly the documents
//! whose validity moved. The second case removes one step and asserts that the
//! document it covered is now named as covered by nothing, which is what stops
//! the first case from passing against an accounting that says "all of them"
//! whatever it is handed.
//!
//! # The root each case runs over
//!
//! This repository's own package, overlay and consumer declaration, over the
//! three documents of `fixtures/migration/docs`. Copied rather than committed a
//! second time, for the reason `diff.rs` gives: a taxonomy under `fixtures/` is
//! a schema that no gate holds current, and it would go stale in silence.

use std::path::{Path, PathBuf};
use std::process::Command;

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/migration")
}

/// The candidate taxonomy every case publishes as 2.0.0.
///
/// One value splits in two, one is renamed, and one is removed. The three are
/// the three applications a step can have, and the committed payload carries one
/// step of each.
const CANDIDATE: [(&[&str], &[&str]); 3] = [
    (
        &[
            "    - {value: draft,      role: initial}",
            "    - {value: current,    role: live}",
            "    - {value: superseded, role: terminal-retained}",
            "    - {value: deprecated, role: terminal-retained}",
        ],
        &[
            "    - {value: outline,    role: initial}",
            "    - {value: settled,    role: live}",
            "    - {value: provisional}",
            "    - {value: superseded, role: terminal-retained}",
        ],
    ),
    (
        &[
            "      draft: the document is being written or argued over, and nothing may rely on it",
            "      current: the document states what holds now, and a reader may rely on it",
            "      superseded: a later document replaced this one, and the succession edge names it",
            "      deprecated: the document is no longer to be relied on, and nothing replaced it",
        ],
        &[
            "      outline: the document is being written or argued over, and nothing may rely on it",
            "      settled: the document states what holds now and nobody is arguing with it",
            "      provisional: the document states what holds now and the argument is not closed",
            "      superseded: a later document replaced this one, and the succession edge names it",
        ],
    ),
    (
        &[
            "      initial: draft",
            "      transitions: {draft: [current, deprecated], current: [superseded, deprecated]}",
        ],
        &[
            "      initial: outline",
            "      transitions: {outline: [provisional, settled], provisional: [settled, superseded], settled: [superseded]}",
        ],
    ),
];

/// A repository root that removes itself.
///
/// `label` names the case and not the target. Cargo runs the cases of one
/// target as threads of one process, so a directory keyed on the process
/// identifier alone is one that a second case removes while the first is
/// reading it.
struct Root {
    at: PathBuf,
}

impl Root {
    fn new(label: &str) -> Root {
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-migration-{}-{label}",
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
        copy(&fixtures().join("docs"), &at.join("docs"));
        for name in ["taxonomy.yml", "overlay.yml"] {
            let to = at.join(".headwater").join(name);
            std::fs::create_dir_all(to.parent().expect("it has a parent"))
                .expect("the declaration directory is there");
            std::fs::copy(repository.join(".headwater").join(name), to)
                .expect("the declaration copies");
        }

        let root = Root { at };
        let resolved = root.run(&["taxonomy", "resolve"]);
        assert_eq!(resolved.code, Some(0), "the fixture resolves: {resolved:?}");
        root
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

    fn publish(&self, name: &str) -> Ran {
        let out = self.at.join("released").join(name);
        self.run(&["taxonomy", "publish", "--out", out.to_str().expect("utf-8")])
    }

    fn released(&self, name: &str) -> PathBuf {
        self.at.join("released").join(name)
    }

    /// Rewrite the package source into the candidate 2.0.0, with `payload` in
    /// `migrations/` and `contents.migrations` declaring it.
    ///
    /// The lock is not re-resolved afterwards, which is the state a real
    /// consumer is in: the lock names the version this repository took, and the
    /// artifact names the version somebody is proposing.
    fn candidate(&self, payload: Option<&str>) {
        let taxonomy = self.at.join("packages/headwater-standard/taxonomy.yml");
        let mut text = std::fs::read_to_string(&taxonomy).expect("the taxonomy reads");
        for (from, to) in CANDIDATE {
            let (from, to) = (from.join("\n"), to.join("\n"));
            assert!(text.contains(&from), "the base still carries `{from}`");
            text = text.replacen(&from, &to, 1);
        }
        assert!(text.contains("version: 1.0.0"), "the base is at 1.0.0");
        std::fs::write(&taxonomy, text.replacen("version: 1.0.0", "version: 2.0.0", 1))
            .expect("the taxonomy writes");

        let manifest = self.at.join("packages/headwater-standard/package.yml");
        let mut text = std::fs::read_to_string(&manifest).expect("the manifest reads");
        text = text.replacen("version: 1.0.0", "version: 2.0.0", 1);
        if let Some(payload) = payload {
            text = text.replacen(
                "  bundles: ../../docs/taxonomies",
                "  bundles: ../../docs/taxonomies\n  migrations: migrations",
                1,
            );
            let directory = self.at.join("packages/headwater-standard/migrations");
            std::fs::create_dir_all(&directory).expect("the payload directory is there");
            std::fs::write(directory.join("1-to-2.yml"), payload).expect("the payload writes");
        }
        std::fs::write(&manifest, text).expect("the manifest writes");
    }

    fn diff(&self, name: &str) -> Ran {
        let at = self.released(name);
        self.run(&[
            "taxonomy",
            "diff",
            at.to_str().expect("utf-8"),
            "--now",
            "2026-08-01",
        ])
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.at);
    }
}

#[derive(Debug)]
struct Ran {
    code: Option<i32>,
    out: String,
    err: String,
}

impl Ran {
    fn dimension(&self, name: &str) -> String {
        self.out
            .lines()
            .find_map(|line| line.trim_start().strip_prefix(name))
            .map(|rest| rest.trim().to_string())
            .unwrap_or_else(|| panic!("the report names `{name}`: {self:?}"))
    }
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

/// The committed payload, which is the artifact that pays the cost of the
/// format.
fn payload() -> String {
    std::fs::read_to_string(fixtures().join("1-to-2.yml")).expect("the committed payload reads")
}

/// The one case the whole issue is about.
///
/// A step's remedy is fixed by its subject, and that is a claim until something
/// measures it. Here the candidate moves three lifecycle values, the corpus
/// carries two of them, `instance_validity` reports three documents, and the
/// payload names those three and no others.
///
/// The `BROKEN, 3` assertion is deliberately the loose half. A payload that
/// named nothing would still meet it, and the accounting line below is where
/// this case fails.
#[test]
fn the_payload_names_every_document_whose_validity_moved() {
    let root = Root::new("names-every-document");
    assert_eq!(root.publish("1.0.0").code, Some(0));
    root.candidate(Some(&payload()));
    let published = root.publish("2.0.0");
    assert_eq!(published.code, Some(0), "{published:?}");

    let ran = root.diff("2.0.0");
    assert_eq!(ran.code, Some(0), "{ran:?}");
    assert_eq!(ran.dimension("instance_validity"), "BROKEN, 3 of them");
    assert!(
        ran.out
            .contains("3 of the 3 documents whose validity moved lie under a step of this payload"),
        "{ran:?}"
    );

    // Every application arm, and the subject count each one reaches in this
    // corpus. Three counts rather than one, because a reading that answered the
    // same number for every step would satisfy a single assertion.
    for line in [
        "a closed choice the author settles, among `settled`, `provisional`",
        "2 documents of this corpus carry the old value",
        "mechanical, and it becomes `outline`",
        "1 document of this corpus carries the old value",
        "a re-statement, and nothing replaces the old value",
        "no document of this corpus carries the old value, so this step is a no-op here",
        "3 steps, 1 mechanical and 2 judgment-bearing",
        "remedies instance_validity, consequence",
    ] {
        assert!(ran.out.contains(line), "the report states `{line}`: {ran:?}");
    }
}

/// The direction that stops the case above from being a green light for any
/// accounting at all.
///
/// One step is removed from the payload. The measurement is identical — the
/// same candidate, the same three documents — and the one document that step
/// covered is now covered by nothing, by name.
#[test]
fn a_document_no_step_names_is_named() {
    let root = Root::new("names-what-is-uncovered");
    assert_eq!(root.publish("1.0.0").code, Some(0));

    let full = payload();
    let step = [
        "  - subject: facet_value",
        "    facet: status",
        "    from: draft",
        "    to: [outline]",
    ]
    .join("\n");
    assert!(full.contains(&step), "the committed payload carries it");
    let (before, rest) = full.split_once(&step).expect("the step is there");
    let after = rest
        .split_once("  # A re-statement")
        .expect("the step after it is there")
        .1;
    root.candidate(Some(&format!("{before}  # A re-statement{after}")));

    let published = root.publish("2.0.0");
    assert_eq!(published.code, Some(0), "{published:?}");
    let ran = root.diff("2.0.0");
    assert_eq!(ran.code, Some(0), "{ran:?}");
    assert_eq!(
        ran.dimension("instance_validity"),
        "BROKEN, 3 of them",
        "the corpus and the candidate are the ones the case above measured: {ran:?}"
    );
    assert!(
        ran.out
            .contains("1 of the 3 documents whose validity moved lie under no step of this payload"),
        "{ran:?}"
    );
    assert!(
        ran.out
            .contains("docs/decisions/0003-the-document-still-being-written.md"),
        "the uncovered document is named: {ran:?}"
    );
}

/// A closed choice with nothing that settles it stops the publish.
///
/// Spec 2 emits the judgment half "as a task list with the affected documents
/// attached". A step that leaves a choice and names no task is a question that
/// list would never carry.
#[test]
fn a_closed_choice_with_no_task_stops_the_publish() {
    let root = Root::new("choice-with-no-task");
    let full = payload();
    let task = [
        "    task: >-",
        "      Say whether the argument this document makes is closed. A document that a",
        "      reader may rely on and still argue with is `provisional`. One that nobody",
        "      is arguing with is `settled`.",
        "",
    ]
    .join("\n");
    assert!(full.contains(&task), "the committed payload carries it");
    root.candidate(Some(&full.replacen(&task, "", 1)));

    let ran = root.publish("2.0.0");
    assert_eq!(ran.code, Some(1), "{ran:?}");
    assert!(ran.err.contains("nothing was published"), "{ran:?}");
    assert!(
        ran.err.contains("a choice with no `task` beside it is a question nobody was asked"),
        "{ran:?}"
    );
    assert!(
        !root.released("2.0.0").exists(),
        "a refused publish writes no artifact"
    );
}

/// A step whose source is still declared in the taxonomy being published
/// renames something that did not move.
///
/// This is the half a publisher can check and a consumer cannot. It runs
/// against the candidate that keeps `draft`, so the step is the only route to
/// the refusal.
#[test]
fn a_source_the_taxonomy_still_declares_stops_the_publish() {
    let root = Root::new("source-still-stands");
    let full = payload();
    let step = ["from: draft", "    to: [outline]"].join("\n");
    assert!(full.contains(&step), "the committed payload carries it");
    root.candidate(Some(&full.replacen(
        &step,
        &["from: superseded", "    to: [outline]"].join("\n"),
        1,
    )));

    let ran = root.publish("2.0.0");
    assert_eq!(ran.code, Some(1), "{ran:?}");
    assert!(
        ran.err
            .contains("still declares that facet_value, so the step renames something that did \
                      not move"),
        "{ran:?}"
    );
}

/// The digest covers the payload, so the migration a consumer applies is the
/// one the publisher took the digest over.
#[test]
fn the_release_record_covers_the_payload() {
    let root = Root::new("record-covers-it");
    root.candidate(Some(&payload()));
    assert_eq!(root.publish("2.0.0").code, Some(0));

    let at = root.released("2.0.0");
    assert!(at.join("migrations/1-to-2.yml").is_file(), "it is carried");
    let record = std::fs::read_to_string(at.join("release.yml")).expect("the record reads");
    assert!(
        record.contains("path: migrations/1-to-2.yml"),
        "the record names it: {record}"
    );

    // The check that makes the record more than a listing: an edited payload is
    // an artifact that is not what its own record says it is, and `diff` refuses
    // to measure against one.
    std::fs::write(at.join("migrations/1-to-2.yml"), "migration: {}\n").expect("the edit writes");
    let ran = root.diff("2.0.0");
    assert_eq!(ran.code, Some(1), "{ran:?}");
    assert!(
        ran.err
            .contains("the artifact is not what its own release record says it is"),
        "{ran:?}"
    );
}

/// A major upgrade that ships no payload says so, with the number of documents
/// it left the consumer.
#[test]
fn a_major_upgrade_with_no_payload_is_reported() {
    let root = Root::new("no-payload-at-all");
    assert_eq!(root.publish("1.0.0").code, Some(0));
    root.candidate(None);
    assert_eq!(root.publish("2.0.0").code, Some(0));

    let ran = root.diff("2.0.0");
    assert_eq!(ran.code, Some(0), "{ran:?}");
    assert!(
        ran.out.contains(
            "the artifact ships no migration payload for 1.0.0 to 2.0.0, and 3 documents stopped \
             validating"
        ),
        "{ran:?}"
    );
}
