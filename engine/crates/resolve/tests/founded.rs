// SPDX-License-Identifier: Apache-2.0
//! What this repository's own overlays make, which has to stay nothing.
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
        resolution.operations.len() > 40,
        "the reading ran over {} operations",
        resolution.operations.len()
    );
}
