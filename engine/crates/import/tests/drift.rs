// SPDX-License-Identifier: Apache-2.0
//! An edge that was verified against a revision the snapshot no longer pins.
//!
//! [Spec 7](../../../../docs/spec/07-distribution-and-federation.md#upstream-awareness)
//! states the rule this file holds: "A snapshot pin reports drift on each
//! affected edge, and not only on the pin. … Every edge into a changed item is
//! then a finding until a person re-verifies it." `relation.target.suspect` is
//! that finding, and these are its cases.
//!
//! # Why the cases are here and not in `headwater-check`
//!
//! `engine/crates/check/tests/cache.rs` builds `Resolvers::over(&corpus)`,
//! which supplies the source tree and nothing else. The check crate cannot
//! construct a snapshot resolver, because `headwater-import` depends on
//! `headwater-check` and not the other way round. The tree that has both is
//! this one: `chain` already builds a graph with the real snapshot resolver and
//! already runs `headwater_check::run` over it. What it did not have is a real
//! cache, and the case below is the reason to want one.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md) puts the bar at a failing
//! fixture rather than at a directory, and `engine/crates/scaffold/tests/`
//! cites the same bar from outside the check crate.

mod chain;

use chain::{declaration, import, publish, read, tree, Scratch, PAYLOAD};
use headwater_check::finding::Severity;
use headwater_check::paint::ColorMode;
use headwater_check::{Cache, Context, Date, Declared, Detail, Register, Run};
use headwater_import::Declaration;

/// The lock a scratch tree has, which is no lock at all. It keys the cache and
/// it never moves between two runs of one case, which is the point: every key
/// this file divides is divided by something else.
const LOCK: &str = "sha256:0000000000000000000000000000000000000000000000000000000000000000";

const TODAY: &str = "2026-08-14";

/// The document the import writes its edge into, which is where a finding about
/// that edge anchors.
const DECLARING: &str = "docs/spec/00-first.md";

/// One run of the check layer over the tree, with whichever cache the caller
/// holds.
fn run_over(scratch: &Scratch, declaration: &Declaration, cache: &mut Cache) -> Run {
    let read = read(scratch);
    let graph = read.graph(scratch.path(), std::slice::from_ref(declaration));
    let source = std::fs::read_to_string(scratch.path().join("taxonomy.yml")).expect("it is there");
    let root = headwater_yaml::load(&source)
        .expect("it reads")
        .value
        .as_map()
        .expect("it is a mapping")
        .clone();
    let register = Register::read(&root).expect("the register reads");
    headwater_check::run(
        &read.census,
        &graph,
        &Declared {
            lock: LOCK,
            taxonomy: &read.taxonomy,
            shape: &read.shape,
            relations: &read.relations,
            config: &read.config,
            register: &register,
            adoption: None,
            source: "taxonomy.yml",
        },
        &headwater_check::claim::Claims::empty(),
        &Context::at(Date::parse(TODAY).expect("the pinned date")),
        cache,
    )
}

/// Every finding of the rule this file is about.
fn suspect(run: &Run) -> Vec<&headwater_check::Finding> {
    run.findings
        .iter()
        .filter(|finding| finding.rule == headwater_check::suspect::RULE)
        .collect()
}

/// Advance the snapshot's item to a second revision, republish the record over
/// the new bytes, and answer with the digest a declaration now has to pin.
///
/// Nothing under the corpus base moves. The declaring document still says
/// `verified_revision: "7"`, and that document is the whole read set of the
/// instance that has to change its mind.
fn advance(scratch: &Scratch) -> String {
    scratch.write(
        "imports/ado/snapshot.yml",
        &PAYLOAD.replace("revision: \"7\"", "revision: \"8\""),
    );
    publish(scratch)
}

/// **The decisive case.** A revision that moved under a cached run is not
/// served from the entry written before it.
///
/// This is the mirror of
/// `engine/crates/check/tests/cache.rs::a_moved_anchor_target_is_not_served_from_the_entry_before_it`,
/// against the second thing an anchor's resolution carries. The read set of an
/// edge instance is a list of corpus paths. The snapshot is not one, so nothing
/// in that list moves when the item advances, and the identity does not move
/// either: `12345` is `12345` at both revisions. So the only thing in a key
/// that can divide these two runs is the revision the resolver answered with,
/// and until [#160](https://github.com/headwater-ai/headwater/issues/160)'s
/// condition was met at the graph there was no such thing.
#[test]
fn a_revision_that_moved_under_a_cached_run_is_not_served_from_the_entry_before_it() {
    let scratch = Scratch::new("drift-cached");
    let digest = tree(&scratch);
    let pinned = declaration(&digest);
    import(&scratch, &pinned);

    let mut cold = Cache::at(scratch.path(), LOCK);
    let before = run_over(&scratch, &pinned, &mut cold);
    cold.write(scratch.path());
    assert_eq!(
        suspect(&before).len(),
        0,
        "the edge was suspect on the first run, so this test proves nothing: {:#?}",
        suspect(&before)
    );

    // The one edit, and it is outside the corpus. No census row moves, no
    // document digest moves, and the corpus is byte-identical on both sides.
    let advanced = declaration(&advance(&scratch));

    let mut warm = Cache::at(scratch.path(), LOCK);
    let after = run_over(&scratch, &advanced, &mut warm);

    assert_eq!(
        suspect(&after).len(),
        1,
        "the cache served a verdict about an edge whose upstream item has moved: {:#?}",
        after.cache
    );
    assert_eq!(suspect(&after)[0].path, DECLARING);
    assert_eq!(
        after.render(Detail::EveryInstance, ColorMode::Plain),
        run_over(&scratch, &advanced, &mut Cache::disabled())
            .render(Detail::EveryInstance, ColorMode::Plain),
        "the cached run reported a verdict over the snapshot as it was"
    );

    // And exactly the instances about that one edge were evaluated again.
    //
    // There are two of them, and the number is derived rather than observed.
    // `EdgeUnit::Entry` is what makes an instance out of one entry of one
    // `relations:` block whatever its target became, and two rules declare it:
    // `relation.target.unresolved` and this one. Every other edge rule is
    // `EdgeUnit::Pair`, and a pair needs a target document, which an anchor is
    // not. Both keys name `Target::resolution`, so both divide when the
    // revision moves, and the second of the two is #161's guarantee still
    // holding over the field this change added.
    //
    // The other shape this defect admits is to refuse the key of an anchor
    // edge, which satisfies every assertion above and leaves every such
    // instance unkeyed and re-evaluated forever. Neither the byte-identity
    // differential nor the finding count can see that difference, so it is
    // asserted here: the revision divides a key, and it withholds none.
    assert!(after.cache.hits > 0, "{:?}", after.cache);
    assert_eq!(after.cache.misses, 2, "{:?}", after.cache);
    assert_eq!(
        after.cache.unkeyed, before.cache.unkeyed,
        "an instance lost its key rather than changing it: {:?}",
        after.cache
    );
}

/// The passing case. An edge the import has just written records the revision
/// the snapshot pins, so it is not suspect and the rule says nothing.
#[test]
fn an_edge_whose_verified_revision_matches_the_snapshot_is_not_suspect() {
    let scratch = Scratch::new("drift-matches");
    let digest = tree(&scratch);
    let pinned = declaration(&digest);
    import(&scratch, &pinned);

    let run = run_over(&scratch, &pinned, &mut Cache::disabled());
    assert!(suspect(&run).is_empty(), "{:#?}", suspect(&run));

    // And the instance exists, rather than the rule reporting nothing because
    // it reached nothing. A rule whose instances are only its findings reports
    // a count that reads as its own denominator.
    let instances = run
        .instances
        .iter()
        .filter(|instance| instance.rule == headwater_check::suspect::RULE)
        .count();
    assert_eq!(instances, 1, "{:#?}", run.instances);
}

/// The failing case, uncached. Kept apart from the decisive one so that "the
/// rule works" and "the cache cannot hide it" are two claims with two failures.
#[test]
fn an_edge_whose_verified_revision_moved_is_reported_at_the_document_that_declares_it() {
    let scratch = Scratch::new("drift-moved");
    let digest = tree(&scratch);
    import(&scratch, &declaration(&digest));
    let advanced = declaration(&advance(&scratch));

    let run = run_over(&scratch, &advanced, &mut Cache::disabled());
    let reported = suspect(&run);
    assert_eq!(reported.len(), 1, "{:#?}", run.findings);
    let finding = reported[0];

    // The report is at the origin: the document whose author can act, at the
    // list entry that carries the edge rather than at the top of the file.
    assert_eq!(finding.path, DECLARING);
    assert!(finding.line > 0, "{finding:#?}");
    assert_eq!(finding.rule, "relation.target.suspect");

    // Advisory, because the remedy is a person re-reading an upstream item.
    assert_eq!(finding.severity, Severity::Warn);
    assert!(finding.patch.is_none(), "{finding:#?}");

    // Both revisions, so the reader knows what moved without opening the
    // snapshot, and the identity of the item that moved.
    assert!(finding.message.contains('7'), "{}", finding.message);
    assert!(finding.message.contains('8'), "{}", finding.message);
    assert!(finding.message.contains("12345"), "{}", finding.message);
    assert!(
        finding.message.contains("audited_by"),
        "{}",
        finding.message
    );
    assert!(
        finding.remediation.contains("verified_revision"),
        "{}",
        finding.remediation
    );
}

/// A document target and an edge with no recorded revision are not this rule's
/// business, and the second one is the deliberate silence.
///
/// A person who types an entry of an importable relation by hand records no
/// revision. There is nothing to compare, so there is nothing to report here.
/// That such an edge is *itself* worth a finding is a different rule with a
/// different remedy, and it is open.
#[test]
fn an_edge_of_an_importable_relation_with_no_recorded_revision_is_not_suspect() {
    let scratch = Scratch::new("drift-unrecorded");
    let digest = tree(&scratch);
    let pinned = declaration(&digest);

    // The same target the snapshot pins, written by hand with no attribute
    // beside it, which is the one form the importer never writes.
    scratch.write(
        "docs/spec/01-second.md",
        "---\nid: SPEC-FIX-two\ndoc_type: design_spec\nrelations:\n  audited_by:\n    - \"12345\"\n---\n\n# The second part\n",
    );

    let run = run_over(&scratch, &pinned, &mut Cache::disabled());
    assert!(suspect(&run).is_empty(), "{:#?}", suspect(&run));

    // It is an instance all the same, so the denominator counts the edge that
    // was examined and found to carry nothing to compare.
    let instances = run
        .instances
        .iter()
        .filter(|instance| instance.rule == headwater_check::suspect::RULE)
        .count();
    assert_eq!(instances, 1, "{:#?}", run.instances);
}
