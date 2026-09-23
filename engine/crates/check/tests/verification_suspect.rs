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

/// The lock string every run and every [`Cache::at`] in this file shares, so a
/// cache built for one call reads back for the next: [`Cache::key`] folds the
/// lock into every key it computes, and a cache built against a different
/// string could never hit regardless of what this file proves.
const LOCK: &str = "sha256:verification-suspect-fixture";

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
    run_over_with(root, observations, &mut Cache::disabled())
}

/// As [`run_over`], over a caller-supplied cache rather than always
/// [`Cache::disabled`]. [`warm_cache_sees_an_edited_snapshot_without_no_cache`]
/// is the one caller that needs a cache surviving across two calls.
fn run_over_with(root: &Path, observations: &Observations, cache: &mut Cache) -> Run {
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
            lock: LOCK,
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
        cache,
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

/// The cache-key regression: a warm `.headwater/cache` has to see an edit to
/// `.headwater/observations.yml` on its own, with no `--no-cache` and no
/// cache clear between runs. `Cache::disabled()` above never exercises the
/// cache at all, so none of the three tests above could have caught this: the
/// bug this proves fixed is a rule reading [`Observations`] straight from its
/// own struct field, with no input in its per-instance read set to show for
/// it, so `Cache::key` folded neither the file's absence nor its digest into
/// the key and served the first verdict forever.
///
/// Three runs, one persisted cache, on the same terms the manual differential
/// in this issue's veto used against the real corpus:
///
/// 1. Cold: no snapshot on disk, an empty cache. `declared`, a miss, and the
///    verdict is written to the cache file this call leaves behind.
/// 2. Warm, after a mismatched snapshot: the file now names the verification
///    with a digest that does not match the criterion's bytes. A *new*
///    [`Cache`] reads the file run 1 wrote. Before the fix, this instance's
///    key was unchanged (neither endpoint's digest moved), so it served run
///    1's cached `declared` verdict and reported nothing. After the fix, the
///    snapshot's own digest is part of the key, so this is a miss too, and
///    the run reports `suspect`.
/// 3. Warm, after the finding's own recommended fix: the snapshot is
///    corrected to the criterion's real current digest — exactly what
///    [`crate::verification::remediation`]'s text tells an author to do.
///    Another new [`Cache`] reads what run 2 wrote. This is a third distinct
///    key (a third distinct file content), so it is a third miss, and the
///    verdict returns to a pass.
///
/// A cache report of nonzero hits on runs 2 and 3, against strictly fewer
/// misses than the cold run 1, is the second half of the proof: the fix
/// reaches exactly the instances the snapshot's change implicates, and
/// re-runs nothing else.
///
/// `.headwater/observations.yml` is written to disk and read back with
/// [`Observations::at`] rather than built in memory with [`Observations::of`],
/// which is the one detail that makes this a cache test and not a repeat of
/// the three above. [`Observations::at`] records a digest of the bytes it
/// read ([`Observations::read_set_digest`]); [`Observations::of`] carries no
/// path and so no digest at all, which is the correct behavior for a snapshot
/// that never touched a disk, and it is exactly why a version of this test
/// built on it could never have shown this key moving.
#[test]
fn warm_cache_sees_an_edited_snapshot_without_no_cache() {
    let root = scratch_corpus("cache-differential");
    let criterion_path = root.join(CRITERION_PATH);
    let current_digest =
        headwater_hash::digest(&std::fs::read(&criterion_path).expect("the criterion reads"));
    let snapshot_path = root.join(".headwater").join("observations.yml");
    std::fs::create_dir_all(snapshot_path.parent().expect("a parent")).expect("the dir creates");

    // Run 1: cold cache, no snapshot at all. `declared`, and every instance in
    // this small fixture tree is a miss, because the cache started empty.
    let mut cache = Cache::at(&root, LOCK, &headwater_check::rules_digest());
    let run1 = run_over_with(&root, &Observations::at(&root), &mut cache);
    assert!(
        findings_of(&run1).is_empty(),
        "run 1 (no snapshot) is declared, not suspect: {:?}",
        findings_of(&run1)
    );
    let cold = cache.report();
    assert_eq!(
        cold.hits, 0,
        "run 1 is the first run over this cache file, so nothing in it is a \
         hit yet: {cold:?}"
    );
    cache.write(&root);

    // Run 2: a new `Cache` reads what run 1 wrote. The snapshot now names the
    // verification with a digest that does not match the criterion's bytes —
    // a mismatch a person could as easily have typed by hand as fabricated.
    std::fs::write(
        &snapshot_path,
        format!(
            "{VERIFICATION_ID}:\n  kind: verification\n  commit: {SNAPSHOT_COMMIT}\n  \
             criterion_digest: \"sha256:0000000000000000000000000000000000000000000000000000000000000000\"\n"
        ),
    )
    .expect("the snapshot writes");
    let mut cache = Cache::at(&root, LOCK, &headwater_check::rules_digest());
    let run2 = run_over_with(&root, &Observations::at(&root), &mut cache);
    let reported = findings_of(&run2);
    assert_eq!(
        reported.len(),
        1,
        "run 2 (warm cache, mismatched snapshot) has to report suspect on its \
         own, with no `--no-cache`: {reported:?}"
    );
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
    let warm = cache.report();
    assert!(
        warm.hits > 0,
        "run 2 has to be genuinely warm — most of this tree's instances read \
         nothing that moved between run 1 and run 2, so most of them are \
         hits — or this proves nothing about the cache at all: {warm:?}"
    );
    assert!(
        warm.misses < cold.misses,
        "run 2 reads a warm cache, so it misses on strictly fewer instances \
         than the cold run 1 did, and the verification rule's own instance is \
         still one of them: cold {cold:?}, warm {warm:?}"
    );
    cache.write(&root);

    // Run 3: a third new `Cache` reads what run 2 wrote. The snapshot is
    // corrected to the criterion's real digest today — the finding's own
    // remediation text, followed exactly.
    std::fs::write(
        &snapshot_path,
        format!(
            "{VERIFICATION_ID}:\n  kind: verification\n  commit: {SNAPSHOT_COMMIT}\n  \
             criterion_digest: \"{current_digest}\"\n"
        ),
    )
    .expect("the snapshot writes");
    let mut cache = Cache::at(&root, LOCK, &headwater_check::rules_digest());
    let run3 = run_over_with(&root, &Observations::at(&root), &mut cache);
    assert!(
        findings_of(&run3).is_empty(),
        "run 3 (warm cache, corrected snapshot) returns to a pass on its own: {:?}",
        findings_of(&run3)
    );
    let corrected_report = cache.report();
    assert!(
        corrected_report.hits > 0,
        "run 3 is warm too: {corrected_report:?}"
    );
    assert!(
        corrected_report.misses < cold.misses,
        "a third distinct snapshot is a third distinct key, so this is a miss \
         and not run 2's cached `suspect` verdict surviving past its own fix, \
         and it is still nowhere near a full re-run: cold {cold:?}, corrected \
         {corrected_report:?}"
    );

    let _ = std::fs::remove_dir_all(&root);
}
