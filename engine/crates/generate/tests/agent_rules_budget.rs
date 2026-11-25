// SPDX-License-Identifier: Apache-2.0
//! What a `budget` on a projection may state, read over a lock (#2087,
//! HW-DR-0115).
//!
//! The meta-schema refuses a figure that is not an integer over the sources.
//! The lock is not re-validated against the meta-schema, so the reader refuses
//! here what the meta-schema would, and the one value it admits that no
//! declaration means: a zero budget. Each refusal stands beside the declaration
//! it would accept, so a reader that refused everything fails as surely as one
//! that refused nothing.

use headwater_generate::{Budget, Kind, Projections};

const TAXONOMY: &str = "\
kinds:
  decision: {}
shelves:
  decisions: {path: docs/decisions/**, kind: decision}
";

fn read(projections: &str) -> Result<Projections, Vec<String>> {
    let source = format!("{TAXONOMY}projections:\n{projections}");
    let root = headwater_yaml::load(&source)
        .expect("it loads")
        .value
        .as_map()
        .expect("a mapping")
        .clone();
    Projections::read(&root).map_err(|errors| {
        errors
            .into_iter()
            .map(|error| error.message)
            .collect::<Vec<_>>()
    })
}

fn rules(budget: &str) -> String {
    format!(
        "  - kind: agent_rules\n    for: [decisions]\n    output: \".github/instructions/{{name}}.instructions.md\"\n{budget}"
    )
}

fn refused(projections: &str, says: &str) {
    match read(projections) {
        Ok(_) => panic!("accepted, and it should be refused for `{says}`:\n{projections}"),
        Err(errors) => assert!(
            errors.iter().any(|error| error.contains(says)),
            "refused, and no refusal says `{says}`: {errors:?}"
        ),
    }
}

#[test]
fn an_agent_rules_budget_reads_both_figures_in_bytes() {
    let read = read(&rules("    budget: {per_file: 600, aggregate: 8192}\n")).expect("it reads");
    let declared = &read.declared[0];
    assert_eq!(declared.kind, Kind::AgentRules);
    assert_eq!(
        declared.budget,
        Some(Budget {
            per_file: 600,
            aggregate: 8192
        })
    );
}

#[test]
fn an_agent_rules_with_no_budget_reads_as_none_until_2088_requires_one() {
    let read = read(&rules("")).expect("it reads");
    assert_eq!(read.declared[0].budget, None);
}

#[test]
fn a_budget_on_every_other_declarable_kind_is_refused() {
    for kind in Kind::DECLARABLE {
        if kind == Kind::AgentRules {
            continue;
        }
        let entry = format!(
            "  - kind: {}\n    output: \"out/{}.md\"\n    budget: {{per_file: 600, aggregate: 8192}}\n",
            kind.name(),
            kind.name()
        );
        refused(
            &entry,
            &format!(
                "is a `{}` and states `budget`, and only an `agent_rules` takes `budget`",
                kind.name()
            ),
        );
    }
}

#[test]
fn a_figure_that_is_not_a_positive_whole_number_is_refused() {
    for (budget, says) in [
        (
            "    budget: {per_file: 0, aggregate: 8192}\n",
            "`projections.0.budget.per_file` is `0`",
        ),
        (
            "    budget: {per_file: 600, aggregate: 0}\n",
            "`projections.0.budget.aggregate` is `0`",
        ),
        (
            "    budget: {per_file: -1, aggregate: 8192}\n",
            "`projections.0.budget.per_file` is `-1`",
        ),
        (
            "    budget: {per_file: 600, aggregate: 8 KB}\n",
            "`projections.0.budget.aggregate` is `8 KB`",
        ),
        (
            "    budget: {per_file: 600}\n",
            "`projections.0.budget.aggregate` is absent",
        ),
        ("    budget: 600\n", "`projections.0.budget` is"),
    ] {
        refused(&rules(budget), says);
    }
}
