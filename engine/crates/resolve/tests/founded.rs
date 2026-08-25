// SPDX-License-Identifier: Apache-2.0
//! What this repository's own overlays make, and why no verb refuses one.
//!
//! # Why nothing refuses a founding
//!
//! A founding is a property of the application order and not of the resolved
//! taxonomy. Two overlays that write leaves nowhere near each other commute,
//! and [spec 2](../../../../docs/spec/02-taxonomy-model.md) guarantees that
//! every legal order of a set of overlays yields one taxonomy. One of those
//! orders can put the overlay that reaches into a kind before the overlay that
//! declares it, and only that order records a founding.
//!
//! So a verb that refused on this record would refuse one order of a pair that
//! `confluence.record` certifies as one resolution, and accept the other. That
//! is why `taxonomy validate` states the record and never gates on it, and why
//! `taxonomy resolve` reports it and still writes the lock.
//! `a_commuting_pair_records_a_founding_in_one_order_and_not_the_other` is that
//! reason as a case, so the next proposal to refuse fails a test that explains
//! itself.
//!
//! # What this repository's own overlays make
//!
//! [`headwater_resolve::Founding`] is a reading that `taxonomy diff` turns into
//! a broken `addressability` dimension, and a dimension that fires on a corpus
//! that is right is a dimension nobody reads. So the claim that the reading is
//! quiet here is a case rather than a sentence in a comment.
//!
//! It is also the one place the claim can be made. Every fixture overlay in
//! this engine is cut down to the declarations one case needs, so a fixture
//! that founds nothing proves nothing about a real overlay set. This one is the
//! base package, both bundles and the adopter overlay of this repository, in
//! the order the consumer declaration selects them.

#[test]
fn every_overlay_of_this_repository_reaches_a_declaration_that_is_there() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let repository = headwater_resolve::repository(&root).expect("this repository resolves");
    let resolution = &repository.resolution;

    let named: Vec<String> = resolution
        .founded
        .iter()
        .map(|founding| {
            format!(
                "{} in {}",
                founding.sentence(),
                resolution.sources[founding.source]
            )
        })
        .collect();
    assert!(
        named.is_empty(),
        "an overlay of this repository makes a declaration nothing under it declares:\n  {}",
        named.join("\n  ")
    );

    // The denominator. An empty list over an empty operation set would be the
    // same assertion made about nothing, and a consumer declaration that
    // selected no overlay would produce exactly that.
    assert!(
        resolution.operations.len() > 60,
        "the reading ran over {} operations",
        resolution.operations.len()
    );
}

/// The evidence that a gate on this record would gate on overlay order.
///
/// `two-adds-into-one-kind` is a bundle that declares `kinds.design_spec` and
/// an adopter overlay that writes `kinds.design_spec.identifier`. The two write
/// leaves that meet nowhere, so either order resolves to one taxonomy, and
/// `confluence.record` records both permutations agreeing. The bundle-first
/// order records no founding. The adopter-first order records one, because at
/// that position nothing under the operation declares the kind.
///
/// The resolved text is compared first. A pair that stopped commuting would
/// make the two counts differ for a second reason, and the assertion below
/// would then hold for a reason that has nothing to do with founding.
#[test]
fn a_commuting_pair_records_a_founding_in_one_order_and_not_the_other() {
    use headwater_resolve::{resolve, Role, Source};

    let case = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/cases/two-adds-into-one-kind");
    let read = |name: &str, role| {
        Source::read(&case.join(name), name, role).unwrap_or_else(|_| panic!("{name} loads"))
    };
    let base = read("base.yml", Role::Taxonomy);
    let bundle = read("overlay-1-bundle.yml", Role::Overlay);
    let adopter = read("overlay-2-adopter.yml", Role::Overlay);

    let forward = resolve(&[base.clone(), bundle.clone(), adopter.clone()]).expect("resolves");
    let reverse = resolve(&[base, adopter, bundle]).expect("resolves in the other order too");

    assert_eq!(
        forward.render(),
        reverse.render(),
        "the two orders resolve to one taxonomy, which is what makes the counts below a \
         property of the order alone"
    );

    assert!(
        forward.founded.is_empty(),
        "the bundle declares the kind before the adopter reaches into it, so nothing is made: {:?}",
        forward.founded
    );
    assert_eq!(
        reverse.founded.len(),
        1,
        "the adopter runs first, so its operation makes the kind it addresses: {:?}",
        reverse.founded
    );
    assert_eq!(reverse.founded[0].founds, "kinds.design_spec");
    assert_eq!(reverse.founded[0].at, "add.kinds.design_spec.identifier");
}
