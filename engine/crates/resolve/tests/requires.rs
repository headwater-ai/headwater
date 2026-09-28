// SPDX-License-Identifier: Apache-2.0
//! `requires` as the resolver reads it for Q67 (HW-DR-0095), over the sources
//! a selection builds rather than over the fixture corpus.
//!
//! `fixtures/cases/` holds what a dependent write does. This file holds where
//! the name a bundle is known by comes from: the selection, and never the
//! source itself.

use headwater_resolve::order;
use headwater_resolve::{package, Role, Source};
use std::path::Path;

fn overlay(text: &str) -> Source {
    Source::from_text("overlay.yml", Role::Overlay, text).expect("the overlay loads")
}

const NAMES_A_MISSING_BUNDLE: &str =
    "bundle: dependent\nrequires: [provider]\nadd:\n  kinds.x: {}\n";

/// A source that no selection chose as a bundle has no `requires` the
/// resolver reads, whatever its own `bundle:` key says. An adopter overlay is
/// that source.
#[test]
fn a_source_not_selected_as_a_bundle_requires_nothing() {
    let ordered = order::order(&[overlay(NAMES_A_MISSING_BUNDLE)]).expect("it orders");
    assert_eq!(ordered.missing, vec![Vec::<String>::new()]);
}

#[test]
fn a_source_selected_as_a_bundle_names_what_it_requires() {
    let ordered = order::order(&[overlay(NAMES_A_MISSING_BUNDLE).selected_as("dependent")])
        .expect("it orders");
    assert_eq!(ordered.missing, vec![vec!["provider".to_string()]]);
}

/// A cycle of three is named whole, in the order `requires` walks it, and a
/// bundle that only waits on the cycle is not named.
#[test]
fn a_cycle_of_three_names_each_bundle_in_it_and_no_other() {
    let bundle = |name: &str, requires: &str| {
        overlay(&format!(
            "requires: [{requires}]\nadd:\n  kinds.{name}: {{}}\n"
        ))
        .selected_as(name)
    };
    let refused = order::order(&[
        bundle("outside", "a"),
        bundle("a", "b"),
        bundle("b", "c"),
        bundle("c", "a"),
    ])
    .expect_err("a cycle has no order");
    assert_eq!(refused.len(), 1);
    let text = refused[0].to_string();
    for name in ["`a`", "`b`", "`c`"] {
        assert!(text.contains(name), "{text}");
    }
    assert!(!text.contains("`outside`"), "{text}");
}

/// The widest selection a package publishes names each bundle by its
/// directory, so it applies each one after the bundles it requires. The
/// directory order puts `decision-record` and `design-spec` before
/// `evidence-and-obligation`, which both of them require.
#[test]
fn the_shipped_set_applies_each_dependency_before_its_dependents() {
    let root = std::fs::canonicalize(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.."))
        .expect("the repository root resolves");
    let directory = root.join("taxonomy-source/headwater-standard");
    let resolution = package::maximal(&root, &directory).expect("the shipped bundle set resolves");
    let at = |bundle: &str| {
        resolution
            .sources
            .iter()
            .position(|name| name.contains(&format!("/{bundle}/bundle.yml")))
            .unwrap_or_else(|| panic!("{bundle} is shipped: {:?}", resolution.sources))
    };
    assert!(at("evidence-and-obligation") < at("decision-record"));
    assert!(at("evidence-and-obligation") < at("design-spec"));
}
