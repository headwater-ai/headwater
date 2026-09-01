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
//! that moved. The second case removes one step and asserts that the
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

mod common;
use common::pin;

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
            "    - {value: discharged, role: terminal-retained}",
        ],
        &[
            "    - {value: outline,    role: initial}",
            "    - {value: settled,    role: live}",
            "    - {value: provisional}",
            "    - {value: superseded, role: terminal-retained}",
            "    - {value: discharged, role: terminal-retained}",
        ],
    ),
    (
        &[
            "      draft: the document is being written or argued over, and nothing may rely on it",
            "      current: the document states what holds now, and a reader may rely on it",
            "      superseded: a later document replaced this one, and the succession edge names it",
            "      deprecated: the document is no longer to be relied on, and nothing replaced it",
            "      discharged: the document recorded something the corpus owed, the corpus paid it, and the record is kept as the receipt",
        ],
        &[
            "      outline: the document is being written or argued over, and nothing may rely on it",
            "      settled: the document states what holds now and nobody is arguing with it",
            "      provisional: the document states what holds now and the argument is not closed",
            "      superseded: a later document replaced this one, and the succession edge names it",
            "      discharged: the document recorded something the corpus owed, the corpus paid it, and the record is kept as the receipt",
        ],
    ),
    (
        &[
            "      initial: draft",
            "      transitions: {draft: [current, deprecated], current: [superseded, deprecated, discharged]}",
        ],
        &[
            "      initial: outline",
            "      transitions: {outline: [provisional, settled], provisional: [settled, superseded], settled: [superseded, discharged]}",
        ],
    ),
];

/// The candidate that renames a kind and moves no facet value.
///
/// The bundle of this package addresses `kinds.decision.facets`, and that
/// address survives the rename: an overlay entry over a kind the base no longer
/// declares adds the kind back rather than failing to address anything. So
/// `addressability` stays preserved, the candidate resolves, and
/// `classification` is the one dimension that reports a document. That was
/// measured rather than assumed, and the opposite was written down first.
const KIND: [(&[&str], &[&str]); 2] = [
    (&["  decision:"], &["  ruling:"]),
    (
        &["  decisions:      {path: docs/decisions/**,      homogeneous: true, kind: decision, layout: \"{seq:04d}-{slug}.md\"}"],
        &["  decisions:      {path: docs/decisions/**,      homogeneous: true, kind: ruling, layout: \"{seq:04d}-{slug}.md\"}"],
    ),
];

/// The candidate that renames a kind one of the package's *bundles* declares.
///
/// `decision_register` is declared by `docs/taxonomies/design-spec/bundle.yml`
/// and by no other source of this package, so neither end of the rename is
/// visible in the base taxonomy: the base declares no `decision_register` to
/// move and no `ruling_register` to move it to. The rename is performed here in
/// the bundle, so the artifact the publish writes really does carry the new name
/// and really does not carry the old one.
///
/// Two edits and not one. The declaration is what a `kind` step names, and
/// `shelves.spec_series.kinds` is the one other place in the bundle that names
/// the kind rather than the facet value of the same spelling.
const BUNDLE_KIND: [(&str, &str); 2] = [
    ("  kinds.decision_register:", "  kinds.ruling_register:"),
    (
        "    kinds: [design_spec, decision_register, obligation_register]",
        "    kinds: [design_spec, ruling_register, obligation_register]",
    ),
];

/// A payload whose one `kind` step moves the bundle-declared `decision_register`
/// to whatever target the case names.
fn register_payload(target: &str) -> String {
    format!(
        "# SPDX-License-Identifier: Apache-2.0\n\nmigration:\n  format: 1\n  from: \">=1 <2\"\n  \
         to: \">=2 <3\"\n\nsteps:\n  - subject: kind\n    from: decision_register\n    to: \
         [{target}]\n    because: >-\n      The register of settled decisions and the decisions \
         themselves were one\n      kind, and a reader of the shelf could not tell which one a \
         document was.\n"
    )
}

/// The candidate that renames a kind the adopter's overlay addresses.
///
/// The three `CANDIDATE` edits, and two more that rename the `specification`
/// kind and the shelf that carries it. Every value the corpus reads still
/// moves, so a run over this candidate writes a document and an overlay entry
/// in one set.
///
/// The new name is `norm` and not `standard`, which is what it was until the
/// publish began holding the payload against the package with every bundle it
/// ships. `docs/taxonomies/standards-spec/bundle.yml` declares
/// `add.kinds.standard`, and an `add` over a key the base declares is refused at
/// merge, so a base that renamed `specification` to `standard` would be a
/// package no consumer of that bundle could resolve. Spec 7 makes every subset
/// resolvable a property of the release, so the refusal is right and the
/// candidate was quietly wrong while nothing read a bundle here.
fn address() -> Vec<(&'static [&'static str], &'static [&'static str])> {
    let mut edits: Vec<(&[&str], &[&str])> = CANDIDATE.to_vec();
    edits.push((&["  specification:"], &["  norm:"]));
    edits.push((
        &["  specifications: {path: docs/specifications/**, homogeneous: true, kind: specification}"],
        &["  specifications: {path: docs/specifications/**, homogeneous: true, kind: norm}"],
    ));
    edits
}

/// The overlay entry every case in this file's last section is about.
///
/// This repository's own overlay addresses exactly one kind that the base
/// declares. It is `kinds.decision.language`, which
/// [Q27](../../../../docs/spec/09-decisions.md#q27--whether-a-decision-record-is-governed-prose)
/// added, and `specification` is the base kind it does not reach. This section
/// needs an entry over a base kind that the candidate renames, and the overlay's
/// one entry is over the wrong member: [`Root::without`] explains that renaming
/// `decision` takes three document rules off three documents and makes two
/// dimensions report the same set. The copied overlay therefore gains this one,
/// and it is an ordinary entry — a base kind with no identifier scheme, given
/// the scheme the same overlay declares two blocks above.
const ENTRY: &str = "\n  kinds.specification.identifier: {scheme: spec_id}\n";

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
        // `packages/headwater-standard/` is a vendored artifact since #366
        // (a real `release.yml`), and `taxonomy publish` now refuses to
        // publish a directory in that state — every case here calls it by
        // name, with no `--from`. The maintained source is
        // `taxonomy-source/headwater-standard/`, copied here to the path the
        // by-name lookup expects.
        copy(
            &repository.join("taxonomy-source/headwater-standard"),
            &at.join("packages/headwater-standard"),
        );
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

        pin(&at, "1.0.0");

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
        self.candidate_of(&CANDIDATE, payload);
    }

    fn candidate_of(&self, edits: &[(&[&str], &[&str])], payload: Option<&str>) {
        let taxonomy = self.at.join("packages/headwater-standard/taxonomy.yml");
        let mut text = std::fs::read_to_string(&taxonomy).expect("the taxonomy reads");
        for (from, to) in edits {
            let (from, to) = (from.join("\n"), to.join("\n"));
            assert!(text.contains(&from), "the base still carries `{from}`");
            text = text.replacen(&from, &to, 1);
        }
        assert!(text.contains("version: 1.0.0"), "the base is at 1.0.0");
        std::fs::write(
            &taxonomy,
            text.replacen("version: 1.0.0", "version: 2.0.0", 1),
        )
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

    /// Rewrite one bundle of the package in place.
    ///
    /// [`Root::candidate_of`] rewrites the base taxonomy, and the cases about
    /// bundle-declared content need the other file. The package declares
    /// `contents.bundles: ../../docs/taxonomies`, so the bundles of this scratch
    /// root are the copied `docs/taxonomies` tree.
    fn in_bundle(&self, bundle: &str, edits: &[(&str, &str)]) {
        let at = self
            .at
            .join("docs/taxonomies")
            .join(bundle)
            .join("bundle.yml");
        let mut text = std::fs::read_to_string(&at).expect("the bundle reads");
        for (from, to) in edits {
            assert!(text.contains(from), "the bundle still carries `{from}`");
            text = text.replacen(from, to, 1);
        }
        std::fs::write(&at, text).expect("the bundle writes");
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

    fn migrate(&self, name: &str, flags: &[&str]) -> Ran {
        let at = self.released(name);
        let mut arguments = vec![
            "taxonomy",
            "migrate",
            at.to_str().expect("utf-8"),
            "--now",
            "2026-08-01",
        ];
        arguments.extend_from_slice(flags);
        self.run(&arguments)
    }

    fn read(&self, path: &str) -> String {
        std::fs::read_to_string(self.at.join(path)).expect("the document reads")
    }

    /// Add one `add` entry to the copied overlay, under the `add:` block it
    /// already declares.
    fn addresses(&self, entry: &str) {
        let at = self.at.join(".headwater/overlay.yml");
        let text = std::fs::read_to_string(&at).expect("the overlay reads");
        assert!(text.starts_with("# SPDX"), "the overlay is the copied one");
        let (before, after) = text
            .split_once("\nadd:\n")
            .expect("it declares an `add` block");
        std::fs::write(&at, format!("{before}\nadd:\n{entry}{after}")).expect("the overlay writes");
        let resolved = self.run(&["taxonomy", "resolve"]);
        assert_eq!(
            resolved.code,
            Some(0),
            "the overlay with the entry resolves against the base: {resolved:?}"
        );
    }

    /// Drop one line from the copied overlay, and re-resolve.
    ///
    /// The one case that needs this is the kind rename. This repository's
    /// overlay binds a language regime to `decision`, and the base is what
    /// declares that kind, so a rename of it takes three document rules off
    /// three documents. `instance_validity` then reports the same documents
    /// that `classification` does, and a case meant to reach one half reaches
    /// both. The assertion fails loudly if the line ever goes away.
    fn without(&self, line: &str) {
        let at = self.at.join(".headwater/overlay.yml");
        let text = std::fs::read_to_string(&at).expect("the overlay reads");
        assert!(text.contains(line), "the overlay still carries `{line}`");
        std::fs::write(&at, text.replacen(line, "", 1)).expect("the overlay writes");
        let resolved = self.run(&["taxonomy", "resolve"]);
        assert_eq!(
            resolved.code,
            Some(0),
            "the overlay without the line resolves: {resolved:?}"
        );
    }

    /// Make one document of the corpus unwritable.
    ///
    /// A read-only file rather than a missing one, because a missing file is
    /// also a corpus a census never walked, and the case worth testing is the
    /// document a run named and then could not write.
    fn read_only(&self, path: &str) {
        let at = self.at.join(path);
        let mut permissions = std::fs::metadata(&at)
            .expect("the document is there")
            .permissions();
        permissions.set_readonly(true);
        std::fs::set_permissions(&at, permissions).expect("the document locks");
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

/// The committed payload that carries a step over an overlay address.
fn addresses() -> String {
    std::fs::read_to_string(fixtures().join("addresses-1-to-2.yml"))
        .expect("the committed payload reads")
}

/// The one case the whole issue is about.
///
/// A step's remedy is fixed by its subject, and that is a claim until something
/// measures it. Here the candidate moves three lifecycle values, the corpus
/// carries two of them, `instance_validity` reports three documents, and the
/// payload names those three and no others.
///
/// The `BROKEN, 3 failed / 3 skipped` assertion is deliberately the loose
/// half. A payload that named nothing would still meet it, and the
/// accounting line below is where this case fails.
///
/// Six instances and three documents. Three are `facet.value.not_permitted`
/// going from passed to failed, which is what "stopped validating" means. The
/// other three are `lifecycle.state.not_admitted` going from passed to
/// *skipped*, because the value the document carries is outside the
/// candidate's vocabulary and that rule defers to the one above it. A skip is
/// a rule that declined to decide rather than a document that broke, and the
/// two are counted apart in the printed word for that reason.
/// [#221](https://github.com/headwater-ai/headwater/issues/221) held the
/// conflation this pins the fix for.
#[test]
fn the_payload_names_every_document_whose_validity_moved() {
    let root = Root::new("names-every-document");
    assert_eq!(root.publish("1.0.0").code, Some(0));
    root.candidate(Some(&payload()));
    let published = root.publish("2.0.0");
    assert_eq!(published.code, Some(0), "{published:?}");

    let ran = root.diff("2.0.0");
    assert_eq!(ran.code, Some(0), "{ran:?}");
    assert_eq!(
        ran.dimension("instance_validity"),
        "BROKEN, 3 failed / 3 skipped"
    );
    assert!(
        ran.out.contains(
            "3 of the 3 documents a dimension reports as moved lie under a step of this payload"
        ),
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
        assert!(
            ran.out.contains(line),
            "the report states `{line}`: {ran:?}"
        );
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
        "BROKEN, 3 failed / 3 skipped",
        "the corpus and the candidate are the ones the case above measured: {ran:?}"
    );
    assert!(
        ran.out.contains(
            "1 of the 3 documents a dimension reports as moved lie under no step of this payload"
        ),
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
        ran.err
            .contains("a choice with no `task` beside it is a question nobody was asked"),
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
        ran.err.contains(
            "still declares that facet_value, so the step renames something that did \
                      not move"
        ),
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

/// The other subject a step can name, over the dimension it is a remedy for.
///
/// This candidate renames a kind and moves no facet value, so
/// `instance_validity` is preserved and `classification` is the only dimension
/// that reports a document. The accounting therefore comes from the
/// classification half alone, which no other case reaches.
///
/// The isolation is built rather than assumed. [Q27](../../../../docs/spec/09-decisions.md#q27--whether-a-decision-record-is-governed-prose)
/// binds the house language regime to `decision`, and that kind comes from the
/// base, so the copied overlay is what takes three rules off three documents
/// when the rename lands. [`Root::without`] removes the line first.
#[test]
fn a_kind_step_is_accounted_against_classification() {
    let root = Root::new("kind-step");
    root.without("  kinds.decision.language:            ste_house\n");
    assert_eq!(root.publish("1.0.0").code, Some(0));
    let payload =
        std::fs::read_to_string(fixtures().join("kinds-1-to-2.yml")).expect("the payload reads");
    root.candidate_of(&KIND, Some(&payload));
    let published = root.publish("2.0.0");
    assert_eq!(published.code, Some(0), "{published:?}");

    let ran = root.diff("2.0.0");
    assert_eq!(ran.code, Some(0), "{ran:?}");
    assert_eq!(ran.dimension("classification"), "BROKEN, 3 of them");
    assert_eq!(
        ran.dimension("instance_validity"),
        "preserved",
        "the other half contributes nothing here, or this case proves neither: {ran:?}"
    );
    for line in [
        "kind decision",
        "mechanical, and it becomes `ruling`",
        "remedies classification",
        "3 documents of this corpus carry the old value",
        "3 of the 3 documents a dimension reports as moved lie under a step of this payload",
    ] {
        assert!(
            ran.out.contains(line),
            "the report states `{line}`: {ran:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// Which taxonomy each half of a payload step is held against.
// ---------------------------------------------------------------------------

/// A rename a bundle declares both ends of publishes.
///
/// The publisher moved `decision_register` in `design-spec/bundle.yml`, so after
/// the edit no source of this package declares the old name and one source
/// declares the new one. The payload that states the move is correct and
/// complete, and it is the payload a publisher of an optional shelf writes.
///
/// Held against the base taxonomy alone the target end is invisible — the base
/// declares three kinds and neither of these is one of them — so before
/// [#194](https://github.com/headwater-ai/headwater/issues/194) this refused
/// with `the taxonomy this publishes declares no kind` and a publisher of a
/// bundle had no correct payload to write. The target is now held against the
/// base with every bundle the package ships, which is the most any consumer
/// selects.
#[test]
fn a_rename_of_a_kind_a_bundle_declares_publishes() {
    let root = Root::new("bundle-kind-renamed");
    root.in_bundle("design-spec", &BUNDLE_KIND);
    let payload = format!(
        "{}\n  - subject: overlay_address\n    from: kinds.decision_register\n    to: \
         [kinds.ruling_register]\n    because: >-\n      An overlay entry under the old address \
         adds the old kind back rather than\n      failing to address anything.\n",
        register_payload("ruling_register")
    );
    root.candidate_of(&[], Some(&payload));

    let ran = root.publish("2.0.0");
    assert_eq!(ran.code, Some(0), "{ran:?}");
}

/// The widening of the target half is a widening and not a deletion.
///
/// The same candidate, and a target that no source of the package declares —
/// not the base and not any of the five bundles. The publish refuses, and the
/// message says which set was asked. Without this case
/// [`a_rename_of_a_kind_a_bundle_declares_publishes`] is satisfied by removing
/// the target check outright.
#[test]
fn a_target_no_bundle_declares_stops_the_publish() {
    let root = Root::new("bundle-kind-no-target");
    root.in_bundle("design-spec", &BUNDLE_KIND);
    root.candidate_of(&[], Some(&register_payload("no_source_declares_this")));

    let ran = root.publish("2.0.0");
    assert_eq!(ran.code, Some(1), "{ran:?}");
    assert!(ran.err.contains("nothing was published"), "{ran:?}");
    assert!(
        ran.err.contains("nor any bundle it ships"),
        "the message names the set the target was held against: {ran:?}"
    );
    assert!(
        !root.released("2.0.0").exists(),
        "a refused publish writes no artifact"
    );
}

/// The two halves are held against two taxonomies, and this is the case that
/// says the asymmetry is deliberate.
///
/// The bundle is left declaring `decision_register`, and the payload says it
/// became `specification`. Nothing moved for the consumers who selected that
/// bundle, so the step is wrong for them — and the publish accepts it, because
/// the source half is held against the base alone.
///
/// The base alone is what a consumer who selects no bundle resolves, and a
/// bundle is add-only ([spec 2](../../../../docs/spec/02-taxonomy-model.md#customization-by-composition)),
/// so a value the base declares is declared under every selection. That is what
/// "renames something that did not move" means, and it is the only reading of it
/// a publisher can certify. A value only a bundle declares moved for the
/// consumers who selected that bundle and for no other, and a payload step
/// states no condition. Widening this half would refuse the migration spec 7
/// most expects instead: content moving out of the base and into a bundle, told
/// to base-only consumers as a re-statement.
///
/// `headwater taxonomy diff` is the end that holds both the consumer's taxonomy
/// and the artifact's and can decide it, and
/// [#388](https://github.com/headwater-ai/headwater/issues/388) is that work.
/// A later change that makes the two halves symmetric fails here.
#[test]
fn a_source_only_a_bundle_declares_is_not_the_publishers_to_refuse() {
    let root = Root::new("bundle-source-stands");
    root.candidate_of(&[], Some(&register_payload("specification")));

    let ran = root.publish("2.0.0");
    assert_eq!(ran.code, Some(0), "{ran:?}");
    assert!(
        std::fs::read_to_string(
            root.released("2.0.0")
                .join("bundles/design-spec/bundle.yml")
        )
        .expect("the published bundle reads")
        .contains("kinds.decision_register:"),
        "the artifact still declares the kind the step says moved"
    );
}

// ---------------------------------------------------------------------------
// `taxonomy migrate --apply`: the half that writes.
// ---------------------------------------------------------------------------

/// The committed payload with its closed choice made mechanical.
///
/// The committed one has exactly one mechanical step and it reaches exactly one
/// document, so no case built on it can tell a run that writes one file from a
/// run that writes a set. `current` reaches two documents, and turning its
/// choice into a single target turns it into a step the engine applies.
fn two_document_payload() -> String {
    let full = payload();
    let choice = [
        "    to: [settled, provisional]",
        "    task: >-",
        "      Say whether the argument this document makes is closed. A document that a",
        "      reader may rely on and still argue with is `provisional`. One that nobody",
        "      is arguing with is `settled`.",
    ]
    .join("\n");
    assert!(full.contains(&choice), "the committed payload carries it");
    full.replacen(&choice, "    to: [settled]", 1)
}

/// The case the whole issue is about, and the one it would be worst to get
/// wrong in the other direction.
///
/// A mechanical step writes the documents it covers. A judgment step writes
/// nothing, and that is asserted against the *bytes* of the two documents the
/// closed choice names rather than against a line of the report.
#[test]
fn the_mechanical_step_writes_and_a_judgment_step_writes_nothing() {
    let root = Root::new("apply-writes-the-mechanical-half");
    assert_eq!(root.publish("1.0.0").code, Some(0));
    root.candidate(Some(&payload()));
    assert_eq!(root.publish("2.0.0").code, Some(0));

    let live = root.read("docs/decisions/0001-the-live-document.md");
    let second = root.read("docs/decisions/0002-the-second-live-document.md");
    let written = root.read("docs/decisions/0003-the-document-still-being-written.md");
    assert!(written.contains("status: draft"), "the fixture is at draft");

    let ran = root.migrate("2.0.0", &["--apply"]);
    assert_eq!(ran.code, Some(0), "{ran:?}");
    assert!(ran.out.contains("wrote 1 value in 2 files"), "{ran:?}");

    assert!(
        root.read("docs/decisions/0003-the-document-still-being-written.md")
            .contains("status: outline"),
        "the mechanical step wrote its one target"
    );
    assert_eq!(
        root.read("docs/decisions/0001-the-live-document.md"),
        live,
        "a closed choice is settled by an author, and `--apply` wrote this document"
    );
    assert_eq!(
        root.read("docs/decisions/0002-the-second-live-document.md"),
        second,
        "a closed choice is settled by an author, and `--apply` wrote this document"
    );
    assert!(
        ran.out.contains("2 tasks for an author"),
        "the judgment half is emitted as a task list: {ran:?}"
    );
    assert!(
        ran.out
            .contains("Say whether the argument this document makes is closed"),
        "the task carries its own text: {ran:?}"
    );
}

/// A run without `--apply` reports the same thing and writes nothing.
#[test]
fn a_run_without_apply_writes_no_document() {
    let root = Root::new("apply-is-required");
    assert_eq!(root.publish("1.0.0").code, Some(0));
    root.candidate(Some(&payload()));
    assert_eq!(root.publish("2.0.0").code, Some(0));

    let before = root.read("docs/decisions/0003-the-document-still-being-written.md");
    let ran = root.migrate("2.0.0", &[]);
    assert_eq!(ran.code, Some(0), "{ran:?}");
    assert!(
        ran.out
            .contains("1 value in 2 files would be written. Nothing was: pass `--apply`"),
        "{ran:?}"
    );
    assert_eq!(
        root.read("docs/decisions/0003-the-document-still-being-written.md"),
        before
    );
}

/// The bar the issue set, tested where it can fail.
///
/// Two documents lie under one mechanical step and one of them cannot be
/// written. The run refuses and names it, and the other document is byte for
/// byte what it was. A writer that wrote as it went would have written that
/// document before it reached the one it could not, and the tree would then be
/// half migrated with nothing recording which half.
#[test]
fn a_document_that_cannot_be_written_leaves_every_other_document_as_it_was() {
    let root = Root::new("half-written");
    assert_eq!(root.publish("1.0.0").code, Some(0));
    root.candidate(Some(&two_document_payload()));
    assert_eq!(root.publish("2.0.0").code, Some(0));

    // Three documents lie under the two mechanical steps, which is what makes
    // the assertion below about a set rather than about one file. The
    // unwritable one is the middle of the three in path order, so one document
    // sits before it and one after: a writer that wrote as it went would have
    // written the first and not the third.
    let planned = root.migrate("2.0.0", &[]);
    assert_eq!(planned.code, Some(0), "{planned:?}");
    assert!(
        planned.out.contains("3 values in 4 files would be written"),
        "{planned:?}"
    );

    let first = "docs/decisions/0001-the-live-document.md";
    let second = "docs/decisions/0002-the-second-live-document.md";
    let third = "docs/decisions/0003-the-document-still-being-written.md";
    let before = root.read(first);
    let after = root.read(third);
    root.read_only(second);

    let ran = root.migrate("2.0.0", &["--apply"]);
    assert_eq!(ran.code, Some(1), "{ran:?}");
    assert!(
        ran.err.contains(second),
        "the refusal names the file it could not write: {ran:?}"
    );
    assert!(
        ran.err
            .contains("a file of this migration cannot be written, so none was"),
        "{ran:?}"
    );
    assert_eq!(
        root.read(first),
        before,
        "the document before the unwritable one is exactly what it was"
    );
    assert_eq!(root.read(third), after, "and so is the document after it");
    assert!(
        root.read(second).contains("status: current"),
        "the unwritable document did not move either"
    );
}

/// A kind a homogeneous shelf carries has no byte to rewrite, and the run says
/// which shelf carries it rather than reporting nothing.
#[test]
fn a_kind_the_shelf_carries_is_named_and_no_document_is_written() {
    let root = Root::new("kind-by-placement");
    assert_eq!(root.publish("1.0.0").code, Some(0));
    let payload =
        std::fs::read_to_string(fixtures().join("kinds-1-to-2.yml")).expect("the payload reads");
    root.candidate_of(&KIND, Some(&payload));
    assert_eq!(root.publish("2.0.0").code, Some(0));

    let before = root.read("docs/decisions/0001-the-live-document.md");
    let ran = root.migrate("2.0.0", &["--apply"]);
    assert_eq!(ran.code, Some(0), "{ran:?}");
    assert!(
        ran.out.contains(
            "takes its kind from `decisions`, which is homogeneous, so the document declares \
             the kind nowhere. The remedy is to move the file"
        ),
        "{ran:?}"
    );
    assert!(
        ran.out.contains("wrote 0 values in 1 file"),
        "no document moved, and the lock still records the migration: {ran:?}"
    );
    assert_eq!(
        root.read("docs/decisions/0001-the-live-document.md"),
        before
    );
}

/// HW-DR-0046: `--apply` writes `adoption.from` as `{version, digest}` and
/// `adoption.to` as the target version, read off the pin `.headwater/
/// taxonomy.yml` still carries at the moment this run reads it.
#[test]
fn apply_records_the_migration_state_in_the_lock() {
    let root = Root::new("records-the-migration-state");
    let digest = root
        .read(".headwater/taxonomy.yml")
        .lines()
        .find_map(|line| line.trim().strip_prefix("digest: "))
        .expect("the fixture's consumer declaration pins a digest")
        .to_string();
    assert_eq!(root.publish("1.0.0").code, Some(0));
    root.candidate(Some(&payload()));
    assert_eq!(root.publish("2.0.0").code, Some(0));

    let ran = root.migrate("2.0.0", &["--apply"]);
    assert_eq!(ran.code, Some(0), "{ran:?}");
    assert!(
        ran.out
            .contains("the lock records this migration on `--apply`"),
        "the run states it: {ran:?}"
    );

    let lock =
        std::fs::read_to_string(root.at.join(".headwater/taxonomy.lock")).expect("the lock reads");
    assert!(
        lock.contains(&format!(
            "adoption:\n  from:\n    version: 1.0.0\n    digest: \"{digest}\"\n  to: 2.0.0\n"
        )),
        "the block states what this run measured: {lock}"
    );
}

/// A run with no digest pinned writes every other file and leaves the lock
/// exactly as it was, rather than writing a `from` this run cannot verify.
#[test]
fn apply_with_no_pinned_digest_writes_no_migration_state() {
    let root = Root::new("no-digest-no-lock-write");
    let consumer = root.read(".headwater/taxonomy.yml");
    assert!(
        consumer.contains("digest: "),
        "the fixture pins one to start with"
    );
    let unpinned = consumer
        .lines()
        .filter(|line| !line.trim_start().starts_with("digest: "))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    std::fs::write(root.at.join(".headwater/taxonomy.yml"), unpinned)
        .expect("the consumer declaration writes");

    assert_eq!(root.publish("1.0.0").code, Some(0));
    root.candidate(Some(&payload()));
    assert_eq!(root.publish("2.0.0").code, Some(0));

    let before =
        std::fs::read_to_string(root.at.join(".headwater/taxonomy.lock")).expect("the lock reads");
    let ran = root.migrate("2.0.0", &["--apply"]);
    assert_eq!(ran.code, Some(0), "{ran:?}");
    assert!(
        ran.out
            .contains("pins no digest, so this run cannot write a verifiable `adoption.from`"),
        "the run states why: {ran:?}"
    );
    assert!(
        ran.out.contains("wrote 1 value in 1 file"),
        "every other file is still written: {ran:?}"
    );
    assert_eq!(
        std::fs::read_to_string(root.at.join(".headwater/taxonomy.lock")).expect("the lock reads"),
        before,
        "and the lock is byte for byte what it was"
    );
}

/// An artifact with no payload for this transition is a refusal rather than a
/// run that migrated nothing and said it was done.
#[test]
fn an_artifact_with_no_payload_for_the_transition_is_refused() {
    let root = Root::new("no-payload");
    assert_eq!(root.publish("1.0.0").code, Some(0));
    root.candidate(None);
    assert_eq!(root.publish("2.0.0").code, Some(0));

    let ran = root.migrate("2.0.0", &["--apply"]);
    assert_eq!(ran.code, Some(1), "{ran:?}");
    assert!(
        ran.err
            .contains("the artifact ships no migration payload for 1.0.0 to 2.0.0"),
        "{ran:?}"
    );
    assert!(
        !ran.err.contains("headwater --help"),
        "the artifact is this repository's package and it ships no payload for the transition. \
         No command line reads a payload that is not there:\n{}",
        ran.err
    );
}

// ---------------------------------------------------------------------------
// The overlay address: the subject that names no document, and the state that
// nothing reported before it.
// ---------------------------------------------------------------------------

/// The case this whole issue is about, and the one that was silent.
///
/// A candidate renames a kind. This repository's overlay writes into that kind
/// at an address the new base no longer declares — and the `add` puts the kind
/// back. Every phase downstream reads a `kinds.specification` that no taxonomy
/// declares: the candidate resolves, `classification` reports nothing, and
/// before this measurement `addressability` said `preserved`.
///
/// The resolution assertion is deliberately the loose half. A candidate that
/// failed to resolve would satisfy no reading of this case at all, and the
/// dimension below it is where this case fails.
#[test]
fn an_add_over_a_path_the_new_base_dropped_is_named_with_the_entry_that_makes_it() {
    let root = Root::new("resurrection-is-named");
    root.addresses(ENTRY);
    assert_eq!(root.publish("1.0.0").code, Some(0));
    root.candidate_of(&address(), None);
    assert_eq!(root.publish("2.0.0").code, Some(0));

    let ran = root.diff("2.0.0");
    assert_eq!(
        ran.code,
        Some(0),
        "the candidate resolves, which is what makes the state quiet: {ran:?}"
    );
    assert_eq!(
        ran.dimension("addressability"),
        "BROKEN, 1 of them",
        "{ran:?}"
    );
    for line in [
        "add.kinds.specification.identifier in .headwater/overlay.yml",
        "makes `kinds.specification` rather than reaching into it",
        "an address into `kinds.specification`, which the taxonomy under it declared",
    ] {
        assert!(
            ran.out.contains(line),
            "the report states `{line}`: {ran:?}"
        );
    }
    assert!(
        ran.out.contains("this change requires a major version"),
        "spec 2: any dimension broken forces a major: {ran:?}"
    );
}

/// The direction that keeps the case above from passing against any reading.
///
/// The same overlay entry, the same corpus, and a candidate that renames
/// nothing the entry addresses. The address still reaches the declaration it
/// was written against, and the dimension says so.
#[test]
fn an_overlay_address_the_new_base_still_declares_is_preserved() {
    let root = Root::new("address-still-reaches");
    root.addresses(ENTRY);
    assert_eq!(root.publish("1.0.0").code, Some(0));
    root.candidate(None);
    assert_eq!(root.publish("2.0.0").code, Some(0));

    let ran = root.diff("2.0.0");
    assert_eq!(ran.code, Some(0), "{ran:?}");
    assert_eq!(
        ran.dimension("addressability"),
        "preserved",
        "the candidate moves three facet values and no address: {ran:?}"
    );
    assert_eq!(
        ran.dimension("instance_validity"),
        "BROKEN, 3 failed / 3 skipped",
        "and the run did measure something, or this case proves nothing: {ran:?}"
    );
}

/// A step over an address, accounted the way a step over a value is.
///
/// The reach line is the assertion. It counts entries of the overlay and not
/// documents of the corpus, and the noun is what says the subject reached a
/// file that no census walks.
#[test]
fn a_step_over_an_address_reports_the_overlay_entries_it_reaches() {
    let root = Root::new("address-step-reports");
    root.addresses(ENTRY);
    assert_eq!(root.publish("1.0.0").code, Some(0));
    root.candidate_of(&address(), Some(&addresses()));
    let published = root.publish("2.0.0");
    assert_eq!(published.code, Some(0), "{published:?}");

    let ran = root.diff("2.0.0");
    assert_eq!(ran.code, Some(0), "{ran:?}");
    for line in [
        "overlay_address kinds.specification",
        "mechanical, and it becomes `kinds.norm`",
        "remedies addressability",
        "1 entry of this repository's overlay is addressed at or under it",
    ] {
        assert!(
            ran.out.contains(line),
            "the report states `{line}`: {ran:?}"
        );
    }
}

/// A publisher that ships a step over an address it did not move.
///
/// The half of the payload check the publisher can make. `kinds.decision` is
/// still declared in the taxonomy being published, so a step that re-addresses
/// overlays away from it renames something that did not move, and the digest is
/// never taken.
#[test]
fn a_step_over_an_address_the_new_taxonomy_still_declares_stops_the_publish() {
    let root = Root::new("address-source-stands");
    root.addresses(ENTRY);
    assert_eq!(root.publish("1.0.0").code, Some(0));
    let payload = addresses().replacen(
        "    from: kinds.specification\n    to: [kinds.norm]",
        "    from: kinds.decision\n    to: [kinds.norm]",
        1,
    );
    root.candidate_of(&address(), Some(&payload));

    let published = root.publish("2.0.0");
    assert_eq!(published.code, Some(1), "{published:?}");
    assert!(
        published
            .err
            .contains("the taxonomy this publishes still declares that overlay_address"),
        "{published:?}"
    );
}

/// A `from` that is not an address at all.
///
/// The step is refused when the payload is read, before anything decides
/// whether it is mechanical, so no writer ever meets an address it cannot
/// parse.
#[test]
fn a_step_over_something_that_is_not_an_address_is_refused_when_the_payload_is_read() {
    let root = Root::new("address-unparseable");
    root.addresses(ENTRY);
    assert_eq!(root.publish("1.0.0").code, Some(0));
    let payload =
        addresses().replacen("from: kinds.specification", "from: kinds..specification", 1);
    root.candidate_of(&address(), Some(&payload));

    let published = root.publish("2.0.0");
    assert_eq!(published.code, Some(1), "{published:?}");
    assert!(
        published
            .err
            .contains("is not an address into a taxonomy, and an `overlay_address` step moves one"),
        "{published:?}"
    );
}

/// `--apply` over both subjects, and the write set that holds them together.
///
/// One step writes a document of the corpus and one writes the overlay beside
/// it. The assertion is on the bytes of both files, because a run that wrote
/// the document and reported the overlay would print the same two lines.
#[test]
fn apply_rewrites_the_overlay_address_beside_the_document() {
    let root = Root::new("address-apply");
    root.addresses(ENTRY);
    assert_eq!(root.publish("1.0.0").code, Some(0));
    root.candidate_of(&address(), Some(&addresses()));
    assert_eq!(root.publish("2.0.0").code, Some(0));

    assert!(
        root.read(".headwater/overlay.yml")
            .contains("kinds.specification.identifier: {scheme: spec_id}"),
        "the entry is the one the fixture added"
    );

    let ran = root.migrate("2.0.0", &["--apply"]);
    assert_eq!(ran.code, Some(0), "{ran:?}");
    assert!(ran.out.contains("wrote 2 values in 3 files"), "{ran:?}");
    assert!(
        ran.out.contains(
            ".headwater/overlay.yml  add.kinds.specification.identifier  becomes \
                      `kinds.norm.identifier`"
        ),
        "{ran:?}"
    );

    let overlay = root.read(".headwater/overlay.yml");
    assert!(
        overlay.contains("kinds.norm.identifier: {scheme: spec_id}"),
        "the address moved and the value beside it did not: {overlay}"
    );
    assert!(
        !overlay.contains("kinds.specification"),
        "no old address survives: {overlay}"
    );
    assert!(
        overlay.contains("kinds.decision_register.identifier:"),
        "an address the step does not reach is untouched: {overlay}"
    );
    assert!(
        root.read("docs/decisions/0003-the-document-still-being-written.md")
            .contains("status: outline"),
        "the document half of the same write set landed"
    );
}

/// The property #186 built, over a set that holds an overlay and a document.
///
/// The overlay is read-only. `headwater_scaffold::tree::Reserved` opens every
/// target before a byte is written, so the run refuses there — and the
/// assertion is on the *document*, which a writer that took the corpus first
/// would already have written.
#[test]
fn an_unwritable_overlay_leaves_the_document_beside_it_untouched() {
    let root = Root::new("address-apply-refuses");
    root.addresses(ENTRY);
    assert_eq!(root.publish("1.0.0").code, Some(0));
    root.candidate_of(&address(), Some(&addresses()));
    assert_eq!(root.publish("2.0.0").code, Some(0));

    let document = root.read("docs/decisions/0003-the-document-still-being-written.md");
    let overlay = root.read(".headwater/overlay.yml");
    root.read_only(".headwater/overlay.yml");

    let ran = root.migrate("2.0.0", &["--apply"]);
    assert_eq!(ran.code, Some(1), "{ran:?}");
    assert!(
        ran.err
            .contains("a file of this migration cannot be written, so none was"),
        "{ran:?}"
    );
    assert!(ran.err.contains(".headwater/overlay.yml"), "{ran:?}");
    assert_eq!(
        root.read("docs/decisions/0003-the-document-still-being-written.md"),
        document,
        "the document is in the same write set as the overlay, so neither moved"
    );
    assert_eq!(root.read(".headwater/overlay.yml"), overlay);
}
