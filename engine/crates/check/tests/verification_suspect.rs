// SPDX-License-Identifier: Apache-2.0
//! The decisive fixture for #937 / HW-DR-0073 ruling 3.
//!
//! A committed observation snapshot could name a verification and the commit
//! it ran at, and nothing compared it against anything: the freshness half of
//! ruling 3 ("a verification is `suspect` when the criterion it proves
//! changed after the snapshot commit") had no reader. This file proves the
//! new rule (`headwater_check::verification::RULE`) is that reader, and that
//! it is live rather than read-then-ignored the way `commit` already was for
//! a control ([HW-OBL-0199](../../../../docs/obligations/0199-an-observation-snapshot-records-a-commit-and-nothing-reads-it-back.md)).
//!
//! It reuses `fixtures/acceptance-criterion-proven/`, the pair #935 already
//! shipped for the `proven_by` participation expectation:
//! `criteria/proven.md` (`ACP-FIX-proven`) names `proven_by:
//! ACP-FIX-verification-one`, reaching `verifications/one.md`. That fixture
//! tree is copied into a scratch directory here rather than read in place,
//! because the second half of this test edits the criterion's bytes to prove
//! the comparison reacts to a real content change and not to a digest this
//! test fabricated by hand.
//!
//! Three runs, one scratch corpus:
//!
//! 1. No snapshot names the verification at all: `declared`, no finding.
//! 2. A snapshot names it with the criterion's digest as it stands right now:
//!    `observed at <commit>`, no finding.
//! 3. The same snapshot, after the criterion's prose changes underneath it
//!    with no new snapshot: `suspect`, and the rule reports it. This is the
//!    run that shows the freshness check live rather than silently accepted:
//!    before this change, nothing could tell runs 2 and 3 apart at all.

use headwater_census::census;
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::{
    Cache, Context, Date, Declared, Observation, Observations, Register, Run, Shape,
};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use std::path::{Path, PathBuf};

const RULE: &str = headwater_check::verification::RULE;

const VERIFICATION_ID: &str = "ACP-FIX-verification-one";
const CRITERION_PATH: &str = "acceptance-criterion-proven/criteria/proven.md";
const SNAPSHOT_COMMIT: &str = "788885a9";

/// The same pinned date `tests/acceptance_criterion_proven.rs` uses. No
/// window in this fixture matters to `verification::RULE`, so any date would
/// do; the shared constant is so a reader of both files reads one clock.
const PINNED: &str = "2026-08-12";

fn shipped_fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

/// Copy `fixtures/acceptance-criterion-proven/` into a scratch directory this
/// test owns, so editing the criterion's bytes below never touches the
/// checked-in fixture. Keyed by the caller's own name and the process id, on
/// `engine/crates/vcs/src/lib.rs`'s own precedent: cargo runs the cases of one
/// target as threads of one process, so the pid alone is not unique across the
/// three tests in this file and one's cleanup raced another's read the first
/// time this test ran.
fn scratch_corpus(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "hw-verification-suspect-test-{label}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    let source = shipped_fixtures_dir().join("acceptance-criterion-proven");
    copy_tree(&source, &root.join("acceptance-criterion-proven"));
    root
}

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("the scratch directory creates");
    for entry in std::fs::read_dir(from).expect("the fixture directory reads") {
        let entry = entry.expect("a directory entry reads");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("a file type reads").is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).expect("the fixture file copies");
        }
    }
}

fn run_over(root: &Path, observations: &Observations) -> Run {
    let corpus = Corpus::new(root, "acceptance-criterion-proven");
    let source = std::fs::read_to_string(
        shipped_fixtures_dir().join("acceptance-criterion-proven.taxonomy.yml"),
    )
    .expect("the fixture taxonomy");
    let root_value = headwater_yaml::load(&source)
        .expect("the fixture taxonomy loads")
        .value
        .as_map()
        .expect("a mapping")
        .clone();

    let taxonomy = Taxonomy::read(&root_value).expect("the taxonomy reads");
    let declarations = Declarations::read(&root_value).expect("the declarations read");
    let register = Register::read(&root_value).expect("the register reads");
    let shape = Shape::read(&root_value).expect("the shape reads");
    let taken = census::take(&corpus, &taxonomy);
    let config = Config::default();
    let graph = Graph::build(
        &taken,
        &declarations,
        &Resolvers::over(&corpus),
        &corpus,
        &config,
    );
    headwater_check::run(
        &taken,
        &graph,
        &Declared {
            lock: "sha256:verification-suspect-fixture",
            taxonomy: &taxonomy,
            shape: &shape,
            relations: &declarations,
            config: &config,
            register: &register,
            adoption: None,
            observations,
            source: "engine/crates/check/fixtures/acceptance-criterion-proven.taxonomy.yml",
        },
        &headwater_check::claim::Claims::empty(),
        &Context::at(Date::parse(PINNED).expect("the pinned date")),
        &mut Cache::disabled(),
    )
}

fn findings_of(run: &Run) -> Vec<&headwater_check::finding::Finding> {
    run.findings.iter().filter(|f| f.rule == RULE).collect()
}

/// State 1: no snapshot names the verification. `declared`, and a pass.
#[test]
fn no_snapshot_is_declared_and_not_a_finding() {
    let root = scratch_corpus("declared");
    let run = run_over(&root, &Observations::empty());
    assert!(
        findings_of(&run).is_empty(),
        "a verification with no committed snapshot is `declared`, not suspect: {:?}",
        findings_of(&run)
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// State 2: a snapshot names the verification with the criterion's digest as
/// it stands today. `observed at <commit>`, and a pass.
#[test]
fn a_snapshot_matching_the_criterions_current_digest_is_observed_and_not_a_finding() {
    let root = scratch_corpus("observed");
    let digest = headwater_hash::digest(
        &std::fs::read(root.join(CRITERION_PATH)).expect("the criterion reads"),
    );
    let observations = Observations::of(vec![Observation::Verification {
        verification: VERIFICATION_ID.to_string(),
        commit: SNAPSHOT_COMMIT.to_string(),
        criterion_digest: digest,
    }]);
    let run = run_over(&root, &observations);
    assert!(
        findings_of(&run).is_empty(),
        "the snapshot's digest matches the criterion as it reads today: {:?}",
        findings_of(&run)
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// State 3, the decisive case: the same snapshot as above, and then the
/// criterion's prose changes with no new snapshot. `suspect`, and the rule
/// reports it — the run that shows the comparison is live.
#[test]
fn the_same_snapshot_after_the_criterion_changes_is_suspect() {
    let root = scratch_corpus("suspect");
    let criterion_path = root.join(CRITERION_PATH);
    let digest =
        headwater_hash::digest(&std::fs::read(&criterion_path).expect("the criterion reads"));
    let observations = Observations::of(vec![Observation::Verification {
        verification: VERIFICATION_ID.to_string(),
        commit: SNAPSHOT_COMMIT.to_string(),
        criterion_digest: digest,
    }]);

    // Sanity: this exact snapshot is `observed` before the edit below. A test
    // that skipped this and went straight to the edit could not tell "always
    // suspect" apart from "suspect because it changed".
    let before = run_over(&root, &observations);
    assert!(
        findings_of(&before).is_empty(),
        "before the edit, the snapshot still matches: {:?}",
        findings_of(&before)
    );

    // The criterion's prose changes underneath the snapshot. No new snapshot
    // is written, which is the whole scenario DOORS rule P.6 names.
    let original = std::fs::read_to_string(&criterion_path).expect("the criterion reads as text");
    let edited = original.replace(
        "summary: past the window and reaches a verification",
        "summary: past the window and reaches a verification, reworded after the snapshot",
    );
    assert_ne!(
        edited, original,
        "the replacement actually changed the text"
    );
    std::fs::write(&criterion_path, &edited).expect("the criterion writes");

    let after = run_over(&root, &observations);
    let reported = findings_of(&after);
    assert_eq!(
        reported.len(),
        1,
        "the changed criterion is reported exactly once: {reported:?}"
    );
    assert_eq!(reported[0].path, CRITERION_PATH);
    assert!(
        reported[0].message.contains(VERIFICATION_ID),
        "{}",
        reported[0].message
    );
    assert!(
        reported[0].message.contains("suspect"),
        "{}",
        reported[0].message
    );
    assert!(
        reported[0].message.contains(SNAPSHOT_COMMIT),
        "{}",
        reported[0].message
    );

    let _ = std::fs::remove_dir_all(&root);
}
