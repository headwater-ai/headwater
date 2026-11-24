// SPDX-License-Identifier: Apache-2.0
//! A relation that a `relation_view` chain names is a reference the resolve
//! reads, so a misspelled step is refused rather than a column that silently
//! stays empty (#2018, HW-DR-0114).

use headwater_resolve::rules::dangling;

fn taxonomy(source: &str) -> headwater_yaml::Mapping {
    headwater_yaml::load(source)
        .expect("it loads")
        .value
        .as_map()
        .expect("a mapping")
        .clone()
}

const DECLARED: &str = "kinds:\n  requirement: {}\n  acceptance_criterion: {}\nrelations:\n  verified_by: {from: [requirement], to: [acceptance_criterion]}\nshelves:\n  requirements: {path: docs/requirements/**, kind: requirement}\n";

#[test]
fn each_step_of_a_chain_is_read_against_the_declared_relations() {
    let clean = taxonomy(&format!(
        "{DECLARED}projections:\n  - kind: relation_view\n    for: [requirements]\n    relations: [verified_by]\n    output: docs/matrix.md\n"
    ));
    assert!(dangling(&clean).is_empty(), "{:?}", dangling(&clean));

    let wrong = taxonomy(&format!(
        "{DECLARED}projections:\n  - kind: relation_view\n    for: [requirements]\n    relations: [verified_by, verifies_by]\n    output: docs/matrix.md\n"
    ));
    let found: Vec<(String, String, String)> = dangling(&wrong)
        .into_iter()
        .map(|d| (d.at, d.what, d.name))
        .collect();
    assert_eq!(
        found,
        vec![(
            "projections.0.relations".to_string(),
            "relation".to_string(),
            "verifies_by".to_string()
        )]
    );
}
