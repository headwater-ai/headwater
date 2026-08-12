// SPDX-License-Identifier: Apache-2.0
//! The lock's own fixtures.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#the-correctness-roots)
//! names "the overlay resolver and the lock" one correctness-root entry, and it
//! says of the lock that "a committed and diffable lock decreases the risk but
//! does not test the resolver". The converse is what this file is for: the
//! resolver's fixtures do not test the lock either. So the lock carries its own,
//! and they answer the two questions that are the lock's alone.
//!
//! - **`identity.record`** — does the identity that the lock uses agree with the
//!   identity that spec 2 requires? Spec 2 says any legal order of an overlay
//!   set gives "the same resolved taxonomy", and the lock hashes text. So every
//!   permutation of every case is resolved and hashed, and one digest per case is
//!   the claim. This is `confluence.record` restated in the coordinate that
//!   `check` actually reads.
//! - **`corpus.lock`** — this repository's own lock, at the grain the other
//!   corpus records use: the digest, the sources with theirs, and nothing that
//!   moves when somebody writes a sentence.
//!
//!     HEADWATER_BLESS=1 cargo test -p headwater-lock --test fixtures
//!
//! Read the diff before committing it. A blessed fixture is the change.

use headwater_resolve::{resolve, Role, Source};
use std::path::{Path, PathBuf};

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root")
}

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn compare(path: &Path, actual: &str) {
    if std::env::var_os("HEADWATER_BLESS").is_some() {
        std::fs::create_dir_all(path.parent().expect("a directory")).expect("cannot create it");
        std::fs::write(path, actual).expect("cannot write the expectation");
        return;
    }
    let expected = std::fs::read_to_string(path).unwrap_or_else(|e| {
        panic!(
            "{}: {e}. Run with HEADWATER_BLESS=1 to record it.",
            path.display()
        )
    });
    assert_eq!(expected, actual, "\n{} is out of date", path.display());
}

/// The resolver's own cases, which are the overlay sets that exist.
fn cases() -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = std::fs::read_dir(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../resolve/fixtures/cases"),
    )
    .expect("the resolver's case directory")
    .map(|entry| entry.expect("a directory entry").path())
    .filter(|path| path.is_dir())
    .collect();
    found.sort();
    found
}

fn sources(case: &Path) -> Vec<Source> {
    let base = case.join("base.yml");
    let mut out =
        vec![Source::read(&base, "base.yml", Role::Taxonomy).expect("the base package loads")];
    let mut overlays: Vec<PathBuf> = std::fs::read_dir(case)
        .expect("a case directory")
        .map(|entry| entry.expect("a directory entry").path())
        .filter(|path| {
            path.file_name()
                .is_some_and(|found| found.to_string_lossy().starts_with("overlay-"))
        })
        .collect();
    overlays.sort();
    for overlay in overlays {
        let shown = overlay.file_name().unwrap().to_string_lossy().to_string();
        out.push(Source::read(&overlay, &shown, Role::Overlay).expect("an overlay loads"));
    }
    out
}

/// One digest per case, over every legal order of its overlays.
///
/// This is the decision of [#51](https://github.com/headwater-ai/headwater/issues/51)
/// under test rather than in a comment. If the canonical writer stopped
/// normalizing key order or scalar style, two orders would produce two digests
/// and this record would move, which is what a reviewer needs to see.
#[test]
fn every_order_of_an_overlay_set_produces_one_digest() {
    let mut out = String::new();
    for case in cases() {
        let name = case.file_name().expect("a name").to_string_lossy().to_string();
        let sources = sources(&case);
        let (base, overlays) = sources.split_first().expect("a base");

        let mut digests: Vec<String> = Vec::new();
        let orders = permutations(overlays.len());
        for order in &orders {
            let mut set = vec![base.clone()];
            set.extend(order.iter().map(|index| overlays[*index].clone()));
            match resolve(&set) {
                Ok(resolution) => digests.push(headwater_lock::digest(&resolution.render())),
                Err(_) => digests.push(String::from("refused")),
            }
        }
        digests.dedup();
        assert_eq!(
            digests.len(),
            1,
            "{name} hashes to {} different locks under {} orders",
            digests.len(),
            orders.len()
        );
        out.push_str(&format!(
            "{name}: {} order{}, {}\n",
            orders.len(),
            if orders.len() == 1 { "" } else { "s" },
            digests[0]
        ));
    }
    compare(&fixtures_dir().join("identity.record"), &out);
}

/// A digest is over the canonical text and over nothing else.
///
/// The failure this guards is a lock whose number depends on how the file laid
/// the taxonomy out. Then a change to the header would read as a schema change,
/// and `resolve --check` would be red for a reason no reviewer could find.
#[test]
fn the_digest_is_over_the_canonical_text_and_not_the_file() {
    let root = repository_root();
    let repository = headwater_resolve::repository(&root).expect("this repository resolves");
    let sources =
        headwater_resolve::package::sources(&root, &repository.consumer).expect("its sources");
    let text = headwater_lock::write(
        &repository.consumer.package,
        &repository.consumer.version,
        &sources,
        &repository.resolution,
    )
    .expect("it validates");

    let lock = headwater_lock::read(&text).expect("the lock reads");
    assert_eq!(
        lock.digest,
        headwater_lock::digest(&repository.resolution.render())
    );
    assert_eq!(lock.canonical(), repository.resolution.render());
}

/// This repository's own lock, and the one the working tree holds.
///
/// The committed lock is compared against what the sources resolve to, which is
/// `taxonomy resolve --check` as a test. CI runs the verb as well, because the
/// verb is what an author runs and a test is not.
#[test]
fn the_committed_lock_is_what_the_sources_resolve_to() {
    let root = repository_root();
    let repository = headwater_resolve::repository(&root).expect("this repository resolves");
    let sources =
        headwater_resolve::package::sources(&root, &repository.consumer).expect("its sources");
    let text = headwater_lock::write(
        &repository.consumer.package,
        &repository.consumer.version,
        &sources,
        &repository.resolution,
    )
    .expect("it validates");

    let committed = std::fs::read_to_string(root.join(headwater_lock::LOCK))
        .expect("the lock is committed. Run `headwater taxonomy resolve`");
    assert_eq!(
        committed, text,
        "the committed lock is not what the sources resolve to. \
         Run `headwater taxonomy resolve`"
    );

    let lock = headwater_lock::read(&committed).expect("the committed lock reads");
    assert!(lock.moved(&root).is_empty(), "a source moved under the lock");

    let mut out = String::new();
    out.push_str(&format!("{} {}\n", lock.package, lock.version));
    out.push_str(&format!("{}\n\nsources\n", lock.digest));
    for source in &lock.sources {
        out.push_str(&format!("  {} {}\n", source.digest, source.path));
    }
    out.push_str(&format!(
        "\n{} lines of canonical taxonomy\n",
        lock.canonical().lines().count()
    ));
    compare(&fixtures_dir().join("corpus.lock"), &out);
}

/// Every permutation of `count` items, as index lists. Heap's algorithm.
fn permutations(count: usize) -> Vec<Vec<usize>> {
    let mut current: Vec<usize> = (0..count).collect();
    let mut out = vec![current.clone()];
    let mut counters = vec![0; count];
    let mut index = 0;
    while index < count {
        if counters[index] < index {
            let swap = if index % 2 == 0 { 0 } else { counters[index] };
            current.swap(swap, index);
            out.push(current.clone());
            counters[index] += 1;
            index = 0;
        } else {
            counters[index] = 0;
            index += 1;
        }
    }
    out
}
