// SPDX-License-Identifier: Apache-2.0
//! The four readings this engine holds, each driven against a tree written for
//! it.
//!
//! # Why `pin.current` gets six arms and the others get two
//!
//! `pin.current` is the reading with branches. The pin is **two numbers** — a
//! version and a digest — and a rule that read the version alone would pass a
//! repository whose pinned digest names an artifact nobody publishes any more.
//! Each of the six states of those two numbers is asserted here, because the one
//! this repository happens to be in exercises exactly one of them.
//!
//! The other three are a comparison each, and the arm that matters for each is
//! the failing one. A reading tested only where it says "met" is a reading whose
//! green answer is the only one anybody measured.

use headwater_census::census::{Census, Outcome, Row, Unreadable, Untyped};
use headwater_conformance::{
    corpus_classified, lock_current, pin_current, projections_current, Verdict,
};
use headwater_resolve::package::Consumer;
use std::path::{Path, PathBuf};

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "headwater-conformance-{name}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("a scratch directory");
    dir
}

fn consumer(version: &str, digest: Option<&str>) -> Consumer {
    Consumer {
        package: "acme/taxonomy".to_string(),
        version: version.to_string(),
        bundles: Vec::new(),
        digest: digest.map(str::to_string),
        overlay: None,
        corpus_root: "docs".to_string(),
        exclusions: Vec::new(),
    }
}

/// A package directory under `packages/`, with a manifest and one content file.
fn package(root: &Path, version: &str) -> PathBuf {
    let dir = root.join("packages").join("acme");
    std::fs::create_dir_all(&dir).expect("the package directory");
    std::fs::write(
        dir.join("package.yml"),
        format!("package: acme/taxonomy\nversion: {version}\n"),
    )
    .expect("the manifest");
    std::fs::write(dir.join("taxonomy.yml"), "taxonomy: acme/taxonomy\n").expect("a source");
    dir
}

/// Write the release record `taxonomy publish` would write, and return its
/// digest. The record is computed rather than typed, so the test cannot pin a
/// number the reader would reject for a reason other than the one under test.
fn publish(dir: &Path) -> String {
    let members = headwater_resolve::release::members(dir).expect("the members");
    let release = headwater_resolve::release::Release {
        package: "acme/taxonomy".to_string(),
        version: "1.0.0".to_string(),
        requires_engine: None,
        digest: headwater_resolve::release::digest_of(&members),
        members,
    };
    std::fs::write(
        dir.join(headwater_resolve::release::RECORD),
        headwater_resolve::release::render(&release),
    )
    .expect("the record");
    release.digest
}

fn gap(verdict: &Verdict) -> String {
    match verdict {
        Verdict::Gap(detail) => detail.clone(),
        other => panic!("expected a gap and got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// pin.current — the two numbers
// ---------------------------------------------------------------------------

/// **Met needs both numbers.** This is the only arm in which the rule passes,
/// and it needs a published artifact and a pin that names it.
#[test]
fn the_pin_is_current_when_the_version_and_the_digest_both_name_what_is_installed() {
    let root = scratch("pin-met");
    let dir = package(&root, "1.0.0");
    let digest = publish(&dir);
    assert_eq!(
        pin_current(&root, &consumer("1.0.0", Some(&digest))),
        Verdict::Met
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// **The arm a version-only rule would get wrong.** The version agrees and the
/// digest does not, so a reading of the version alone would call this met. That
/// repository takes an artifact that nobody publishes any more.
#[test]
fn a_matching_version_with_a_stale_digest_is_a_gap() {
    let root = scratch("pin-stale-digest");
    let dir = package(&root, "1.0.0");
    let published = publish(&dir);
    let stale = "sha256:0000000000000000000000000000000000000000000000000000000000000000";
    assert_ne!(published, stale);
    let detail = gap(&pin_current(&root, &consumer("1.0.0", Some(stale))));
    assert!(detail.contains(stale), "the message names the pin");
    assert!(detail.contains(&published), "and the artifact on disk");
    let _ = std::fs::remove_dir_all(&root);
}

/// A published artifact with nothing pinning it. `vendor` refuses to run with no
/// pin rather than recording what it found, and this is the same fact reported
/// as a gap.
#[test]
fn a_published_artifact_that_nothing_pins_is_a_gap() {
    let root = scratch("pin-unpinned");
    let dir = package(&root, "1.0.0");
    let digest = publish(&dir);
    let detail = gap(&pin_current(&root, &consumer("1.0.0", None)));
    assert!(detail.contains("nothing pins a digest"));
    assert!(detail.contains(&digest));
    let _ = std::fs::remove_dir_all(&root);
}

/// No release record and no pin. **This is the arm this repository is in**: it
/// consumes the package it publishes, from source, so no published artifact
/// stands behind the directory.
#[test]
fn a_package_with_no_release_record_and_no_pin_is_a_gap() {
    let root = scratch("pin-unpublished");
    package(&root, "1.0.0");
    let detail = gap(&pin_current(&root, &consumer("1.0.0", None)));
    assert!(detail.contains("carries no release record"));
    let _ = std::fs::remove_dir_all(&root);
}

/// A pin with nothing to check it against. A consumer who copied a digest out of
/// a directory that carries no record has pinned nothing.
#[test]
fn a_pin_with_no_release_record_to_check_it_against_is_a_gap() {
    let root = scratch("pin-no-record");
    package(&root, "1.0.0");
    let detail = gap(&pin_current(
        &root,
        &consumer("1.0.0", Some("sha256:whatever")),
    ));
    assert!(detail.contains("no release record to check it against"));
    let _ = std::fs::remove_dir_all(&root);
}

/// The version half on its own. A pin that names a version the installed package
/// does not declare is refused before any digest is read, because the message
/// about a version is the one an author acts on.
#[test]
fn a_version_the_installed_package_does_not_declare_is_a_gap() {
    let root = scratch("pin-version");
    let dir = package(&root, "2.0.0");
    let digest = publish(&dir);
    let detail = gap(&pin_current(&root, &consumer("1.0.0", Some(&digest))));
    assert!(detail.contains("1.0.0"));
    assert!(detail.contains("2.0.0"));
    let _ = std::fs::remove_dir_all(&root);
}

/// No package at all. A repository that pins a package nothing installed is not
/// pin-current, and the message says which name it looked for.
#[test]
fn a_package_that_is_not_installed_at_all_is_a_gap() {
    let root = scratch("pin-absent");
    std::fs::create_dir_all(root.join("packages")).expect("an empty packages directory");
    let detail = gap(&pin_current(&root, &consumer("1.0.0", None)));
    assert!(detail.contains("acme/taxonomy"));
    let _ = std::fs::remove_dir_all(&root);
}

// ---------------------------------------------------------------------------
// lock.current
// ---------------------------------------------------------------------------

/// The failing arm, and it is the one that matters. The lock records the digest
/// of every source it was written from, so a source whose bytes moved is a lock
/// that no run can reproduce.
#[test]
fn a_source_that_moved_since_the_lock_was_written_is_a_gap() {
    let root = scratch("lock-moved");
    let lock = headwater_lock::at(&repository_root()).expect("this repository's lock reads");
    // The lock names sources relative to a root, and this scratch root holds
    // none of them, so every source reads as moved.
    let detail = gap(&lock_current(&root, &lock));
    assert!(detail.contains("moved since the lock was written"));
    assert!(detail.contains(".headwater/overlay.yml"));
    let _ = std::fs::remove_dir_all(&root);

    // The met arm, over the tree the lock was actually written from.
    assert_eq!(
        lock_current(&repository_root(), &lock),
        Verdict::Met,
        "this repository's committed lock is current, and `taxonomy resolve --check` agrees"
    );
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root")
}

// ---------------------------------------------------------------------------
// corpus.classified
// ---------------------------------------------------------------------------

fn row(path: &str, outcome: Outcome) -> Row {
    Row {
        path: path.to_string(),
        outcome,
        document: None,
        digest: None,
    }
}

/// **The population is the two outcomes that leave a file with no kind and no
/// stated reason.** The four that state one are asserted in the same test,
/// because a reading that counted an excluded file would report a gap on every
/// repository that declares an exclusion.
#[test]
fn the_reading_counts_a_file_with_no_kind_and_no_stated_reason_and_no_other() {
    let accounted = Census {
        rows: vec![
            row(
                "docs/a.md",
                Outcome::Typed {
                    kind: "decision".to_string(),
                    derivation: Box::new(headwater_census::Resolution {
                        outcome: headwater_census::resolve::Outcome::Typed("decision".to_string()),
                        steps: Vec::new(),
                        span: None,
                    }),
                },
            ),
            row(
                "docs/b.md",
                Outcome::Generated {
                    projection: Some("shelf_index".to_string()),
                    kind: None,
                    derivation: None,
                },
            ),
            row(
                "docs/c.md",
                Outcome::Excluded {
                    pattern: "docs/c.md".to_string(),
                    reason: "it states one".to_string(),
                },
            ),
            row("docs/LICENSE", Outcome::NotADocument),
        ],
    };
    assert_eq!(corpus_classified(&accounted), Verdict::Met);

    let untyped = Census {
        rows: vec![row(
            "docs/loose.md",
            Outcome::Untyped(Untyped::NoFrontMatter),
        )],
    };
    let detail = gap(&corpus_classified(&untyped));
    assert!(detail.contains("docs/loose.md"));
    assert!(detail.contains("1 file under the corpus root carries"));

    let unreadable = Census {
        rows: vec![row(
            "docs/bytes.md",
            Outcome::Unreadable(Unreadable::NotText),
        )],
    };
    assert!(gap(&corpus_classified(&unreadable)).contains("docs/bytes.md"));
}

// ---------------------------------------------------------------------------
// projections.current
// ---------------------------------------------------------------------------

/// The failing arm. A declared projection whose committed file is not what the
/// plan produces is a derived file that somebody edited unseen, and a reader
/// then trusts a file that is wrong.
#[test]
fn a_projection_that_is_not_what_the_plan_produces_is_a_gap() {
    let root = scratch("projection-drift");
    std::fs::write(root.join("index.md"), "what somebody typed\n").expect("the committed file");
    let plan = headwater_generate::Plan {
        outputs: vec![headwater_generate::Output {
            path: "index.md".to_string(),
            kind: headwater_generate::Kind::ShelfIndex,
            bytes: "what the plan produces\n".to_string(),
        }],
        ..Default::default()
    };
    let detail = gap(&projections_current(&root, &plan));
    assert!(detail.contains("index.md"));

    // The met arm, over the same plan and the bytes it produces.
    std::fs::write(root.join("index.md"), "what the plan produces\n")
        .expect("the regenerated file");
    assert_eq!(projections_current(&root, &plan), Verdict::Met);
    let _ = std::fs::remove_dir_all(&root);
}
