// SPDX-License-Identifier: Apache-2.0
//! The two `headwater` verbs that keep a check cache key it on the lock they
//! read (HW-OBL-0208, item 3).
//!
//! The check cache keys every verdict on the taxonomy lock digest and on the
//! identity of the compiled rule set, and `crates/check/tests/cache.rs` holds
//! that each component moves a key. That suite builds its own `Cache`, so it
//! cannot see what the binary passes. A call site that passed a frozen lock
//! digest would leave every other test of this crate green, and a warm check
//! would serve a verdict the taxonomy no longer gives.
//!
//! The case here holds the lock half through the binary: a taxonomy change
//! that moves one verdict, with no document changed, must reach a warm run.
//! It costs three runs of `check` and two of `taxonomy resolve` over the small
//! fixture root, under a second in a `dev-release` build.
//! The rule-set half cannot be held this way, because a test cannot change
//! the rule set of the binary it runs. `Cache::compiled` reads that digest
//! itself, and `wiring.rs` holds that the CLI calls no other constructor.

mod common;
use common::Root;

/// The term the case retires. The fixture decision uses it in its prose, and
/// no retired term of the copied overlay matches it.
const TERM: &str = "promotion";

/// What a report says of a sentence that writes `TERM` once it is retired.
const RETIRED: &str = "retires `promotion`";

/// Add `TERM` to the house list of retired terms and resolve the lock again.
fn retire(root: &Root) {
    let overlay = root.at.join(".headwater/overlay.yml");
    let text = std::fs::read_to_string(&overlay).expect("the overlay reads");
    let from = "    retired_terms:\n";
    assert!(text.contains(from), "the overlay lists retired terms");
    let to = format!("{from}      - term: {TERM}\n        reason: a test retires it\n");
    std::fs::write(&overlay, text.replacen(from, &to, 1)).expect("the overlay writes");
    let resolved = root.run(&["taxonomy", "resolve"]);
    assert_eq!(resolved.code, Some(0), "the lock resolves again: {resolved:?}");
}

/// A warm `check` gives the verdict a cold one gives after the lock moves.
///
/// The first run fills the cache. The taxonomy then retires a term the
/// fixture decision uses, and no document changes, so the lock digest is the
/// only component of that decision's keys that moves. A warm run whose cache
/// was keyed on a frozen lock digest serves the first verdict again.
///
/// `check --fix` builds a cache at a second call site, in `fix`, and no case
/// here holds it, because no run can see it fail. The report run after the
/// fix writes the cache file again, so the next fix reads keys that the
/// report run made. A fix with a frozen digest misses every key and decides
/// each verdict again, which costs time and moves no verdict. `wiring.rs`
/// holds that call site by its source.
#[test]
fn a_warm_check_gives_the_cold_verdict_after_the_lock_moves() {
    let root = Root::new("cache-key-lock");
    let lock = root.at.join(".headwater/taxonomy.lock");
    let first = root.run(&["check"]);
    assert!(
        !first.out.contains(RETIRED),
        "the term is not retired before the change:\n{}",
        first.out
    );
    let before = std::fs::read(&lock).expect("the lock reads");

    retire(&root);
    assert_ne!(
        before,
        std::fs::read(&lock).expect("the lock reads"),
        "retiring a term moves the lock"
    );

    let warm = root.run(&["check"]);
    let cold = root.run(&["check", "--no-cache"]);
    assert_ne!(
        first.out, cold.out,
        "retiring a term the fixture uses changes the report"
    );
    assert!(
        cold.out.contains(RETIRED),
        "the cold run reports the retired term:\n{}",
        cold.out
    );
    assert_eq!(
        warm.out, cold.out,
        "a warm `check` served a verdict keyed on the lock before it moved"
    );
    assert_eq!(warm.code, cold.code, "the warm and cold runs exit alike");
}
