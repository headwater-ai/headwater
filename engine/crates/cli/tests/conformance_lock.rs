// SPDX-License-Identifier: Apache-2.0
//! `headwater conformance`'s `lock.current`, held against the verb that decides
//! the same question.
//!
//! # The defect this target exists for
//!
//! [#648](https://github.com/headwater-ai/headwater/issues/648). `lock.current`
//! read the source digests the lock records and re-hashed the files on disk
//! against them, which is what `taxonomy resolve --check` prints *after* it has
//! failed. It is not what that verb decides with. So everything the lock states
//! that is not a source file was invisible to the rule: the header, the
//! `sources` list, and the `founded:` block, which sits outside the digest
//! `headwater_lock::read` verifies. A lock with its `founded:` block deleted by
//! hand was reported `lock.current met` and refused by `headwater taxonomy
//! resolve --check` one command later — a red build the adopter had just been
//! told they did not have.
//!
//! # Why the assertion is correspondence and not a verdict
//!
//! **No case below writes down what either verb should say.** A fixture that
//! asserted `lock.current` is a gap on the stripped lock would be a second copy
//! of the answer the engine holds, and a second copy with nothing checking it
//! against the first is the shape of the defect itself. What each case asserts
//! is that the two verbs *agree*: a root `taxonomy resolve --check` accepts is a
//! root `lock.current` reports met over, and a root it refuses is a root
//! `lock.current` reports a gap over.
//!
//! That assertion is decisive rather than tautological, and
//! [`the_table_discriminates`] is why: it fails unless the four roots are not
//! all in one state. Two verbs that both answered "fine" to everything would
//! satisfy correspondence and fail that.
//!
//! # One case is deliberately outside the table
//!
//! A lock whose digest does not verify. `headwater conformance` refuses the
//! whole run there — `headwater_lock::read` fails before any rule is evaluated
//! — and prints no `lock.current` line at all, so there is no verdict for
//! `taxonomy resolve --check` to correspond to. That is also why the fix for
//! #648 is in the rule and not in the digest: bringing `founded:` inside the
//! digest turns the quiet wrong answer below into a loud one with no report.

mod common;
use common::{Root, DECLARES, FOUNDS};

/// What the two verbs said about one root.
struct Pair {
    /// `taxonomy resolve --check` accepted the committed lock.
    resolves: bool,
    /// The word `lock.current` carries in the report: `met`, `gap`, or
    /// `not decided`.
    current: String,
    /// The whole report, so a failure names what the rule actually said.
    report: String,
}

impl Pair {
    /// Run both verbs over one root, in the order an adopter meets them.
    fn over(root: &Root) -> Pair {
        let checked = root.run(&["taxonomy", "resolve", "--check"]);
        assert!(
            matches!(checked.code, Some(0 | 1)),
            "`taxonomy resolve --check` reaches a verdict: {checked:?}"
        );
        let ran = root.run(&["conformance"]);
        let current = ran
            .out
            .lines()
            .find_map(|line| line.strip_prefix("  lock.current "))
            .unwrap_or_else(|| panic!("the report names `lock.current`: {ran:?}"))
            .trim()
            .to_string();
        Pair {
            resolves: checked.code == Some(0),
            current,
            report: ran.out,
        }
    }

    /// The one assertion this target makes.
    fn corresponds(&self, label: &str) -> &Pair {
        match self.resolves {
            true => assert_eq!(
                self.current, "met",
                "over `{label}`, `headwater taxonomy resolve --check` accepts the committed lock \
                 and `lock.current` does not report it met. An adopter reading this report is \
                 told about a lock this same engine holds a different opinion of one command \
                 later:\n{}",
                self.report
            ),
            false => assert_eq!(
                self.current, "gap",
                "over `{label}`, `headwater taxonomy resolve --check` refuses the committed lock \
                 and `lock.current` does not report a gap. That is issue #648: the rule reports \
                 met over a lock the resolver refuses, so the adopter meets the refusal in their \
                 commit gate instead:\n{}",
                self.report
            ),
        }
        self
    }
}

/// The lock exactly as `taxonomy resolve` wrote it.
///
/// The quiet direction, and the one a change to this rule is most likely to
/// break: a rule that reported a gap over every root would satisfy the three
/// cases below it and fail this one.
#[test]
fn a_lock_the_resolver_just_wrote_is_current_to_both_verbs() {
    let root = Root::founding("conformance-lock-clean");
    Pair::over(&root).corresponds("the lock as `taxonomy resolve` wrote it");
}

/// **The case #648 was filed for.** This fails on `e477f7e`, the commit before
/// the fix: `taxonomy resolve --check` exits 1 and `lock.current` reports met.
///
/// The `founded:` block is generated from the resolution and is not a source
/// file, so nothing about it reaches a source digest. It is also outside the
/// digest the lock declares over its own taxonomy — deliberately, because a
/// founding record is not part of a resolution — so `headwater_lock::read`
/// accepts the edited file and every rule then runs over it.
#[test]
fn a_lock_whose_founding_record_was_deleted_by_hand_is_a_gap_to_both_verbs() {
    let root = Root::founding("conformance-lock-stripped");
    let path = root.at.join(".headwater/taxonomy.lock");
    let text = std::fs::read_to_string(&path).expect("the lock reads");
    assert!(
        text.contains("\nfounded:\n"),
        "this root founds a kind, so its lock records one: {text}"
    );
    let (before, rest) = text.split_once("\nfounded:\n").expect("the block is there");
    let (_, after) = rest
        .split_once("\n\n")
        .expect("a blank line ends the block");
    std::fs::write(&path, format!("{before}\n{after}")).expect("the lock writes");

    Pair::over(&root).corresponds("the `founded:` block deleted, nothing re-resolved");
}

/// A source file that moved. The case the reading before the fix did catch,
/// held so the change is not a swap of one blind spot for another.
///
/// This one also asserts the sentence, because a gap here has a remediation the
/// other gaps do not: which file moved is the line an author acts on, and it is
/// the one thing the previous reading produced that the new one has to keep
/// producing.
#[test]
fn a_bundle_edited_under_the_lock_is_a_gap_that_names_the_bundle() {
    let root = Root::founding("conformance-lock-edited");
    let bundle = root.at.join("docs/taxonomies/zz-a/bundle.yml");
    let text = std::fs::read_to_string(&bundle).expect("the bundle reads");
    std::fs::write(&bundle, format!("{text}# edited under the lock\n")).expect("the bundle writes");

    let pair = Pair::over(&root);
    pair.corresponds("a bundle edited, nothing re-resolved");
    assert!(
        pair.report.contains("docs/taxonomies/zz-a/bundle.yml"),
        "the gap names the source that moved, because that is the file an author edits back \
         or resolves over:\n{}",
        pair.report
    );
}

/// The two bundles swap which of them founds the kind, and the root is
/// re-resolved.
///
/// **This is the case a founding comparison would redden, and the reason the
/// fix is not one.** The pair commutes: whichever bundle the declaration lists
/// first creates `kinds.zz_thing`, so listing `zz-a` first and giving it the
/// block that *declares* the kind leaves the leaf in `zz-b` reaching into a key
/// that already exists. Nothing is founded, `founded:` is honestly empty, and
/// the lock is current. A rule that compared a fresh `founding_records()`
/// against a lock written before the swap would call this current lock stale.
#[test]
fn a_re_resolved_lock_with_no_founding_left_is_current_to_both_verbs() {
    let root = Root::founding("conformance-lock-swapped");
    common::write_bundle(&root.at, "zz-a", DECLARES);
    common::write_bundle(&root.at, "zz-b", FOUNDS);
    let resolved = root.run(&["taxonomy", "resolve"]);
    assert_eq!(
        resolved.code,
        Some(0),
        "the swapped root resolves: {resolved:?}"
    );

    let text =
        std::fs::read_to_string(root.at.join(".headwater/taxonomy.lock")).expect("the lock reads");
    assert!(
        !text.contains("\nfounded:\n"),
        "the swap leaves nothing founded, which is what makes this case worth running: {text}"
    );

    Pair::over(&root).corresponds("the two `add:` blocks swapped, and re-resolved");
}

/// The guard that stops correspondence from being a tautology.
///
/// Correspondence is satisfied by two verbs that agree on everything, including
/// two that both say "fine" to every root there is. This case fails unless the
/// four perturbations above put the resolver in both of its states, so a change
/// that flattened one verb is caught here rather than passing four times over.
///
/// It re-runs `taxonomy resolve --check` alone and never `conformance`, because
/// a claim about the population of the table must not be decided by the rule
/// the table is about.
#[test]
fn the_table_discriminates() {
    let clean = Root::founding("conformance-lock-discriminates-clean");
    let stale = Root::founding("conformance-lock-discriminates-stale");
    let bundle = stale.at.join("docs/taxonomies/zz-a/bundle.yml");
    let text = std::fs::read_to_string(&bundle).expect("the bundle reads");
    std::fs::write(&bundle, format!("{text}# edited under the lock\n")).expect("the bundle writes");

    let accepted = clean.run(&["taxonomy", "resolve", "--check"]);
    let refused = stale.run(&["taxonomy", "resolve", "--check"]);
    assert_eq!(
        accepted.code,
        Some(0),
        "a root nothing perturbed has a current lock: {accepted:?}"
    );
    assert_eq!(
        refused.code,
        Some(1),
        "a root whose source moved under its lock does not: {refused:?}"
    );
}
