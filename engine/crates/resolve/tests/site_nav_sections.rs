// SPDX-License-Identifier: Apache-2.0
//! A shelf that a `site_nav` section names is a reference the resolve reads,
//! so a misspelled one is refused rather than silently left out of the
//! navigation (#1681).

use headwater_resolve::rules::dangling;

fn taxonomy(source: &str) -> headwater_yaml::Mapping {
    headwater_yaml::load(source)
        .expect("it loads")
        .value
        .as_map()
        .expect("a mapping")
        .clone()
}

const SHELVES: &str = "shelves:\n  guides: {path: docs/guides/**}\n  decisions: {path: docs/decisions/**}\n";

#[test]
fn a_section_shelf_and_a_group_shelf_are_each_read_against_the_declared_shelves() {
    let clean = taxonomy(&format!(
        "{SHELVES}projections:\n  - kind: site_nav\n    output: nav.yml\n    sections:\n      - title: A\n        for: [guides]\n      - title: B\n        groups:\n          - title: G\n            for: [decisions]\n"
    ));
    assert!(dangling(&clean).is_empty(), "{:?}", dangling(&clean));

    let wrong = taxonomy(&format!(
        "{SHELVES}projections:\n  - kind: site_nav\n    output: nav.yml\n    sections:\n      - title: A\n        for: [guidse]\n      - title: B\n        groups:\n          - title: G\n            for: [decisons]\n"
    ));
    let found: Vec<(String, String)> = dangling(&wrong)
        .into_iter()
        .map(|d| (d.at, d.name))
        .collect();
    assert_eq!(
        found,
        vec![
            ("projections.0.sections.0.for".to_string(), "guidse".to_string()),
            (
                "projections.0.sections.1.groups.0.for".to_string(),
                "decisons".to_string()
            ),
        ]
    );
}
