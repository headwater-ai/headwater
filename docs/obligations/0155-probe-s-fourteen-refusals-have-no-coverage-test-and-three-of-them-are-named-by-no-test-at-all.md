---
id: HW-OBL-0155
status: current
status_since: 2026-09-06
summary: "Three of the `probe` crate's fourteen refusals are named by no test, and no case holds the set to its own coverage."
last_verified: 2026-08-28
title: "probe's fourteen refusals have no coverage test, and three of them are named by no test at all"
waiting_on: build
---

# probe's fourteen refusals have no coverage test, and three of them are named by no test at all

## Context

The scaffolder carries a test named `every_refusal_branch_has_a_case`, and spec 12 states what it is for: a refusal with no fixture does not ship.

[#338](https://github.com/headwater-ai/headwater/issues/338) records that the test did not hold the property its name claims. Its `variant` function matched 26 arms and its `BRANCHES` constant listed 23. The three it omitted were the whole `--facet` refusal surface, which is why a discriminator value reached a document in silence with the suite green.

The `probe` crate raises fourteen refusals of its own and carries no test of that shape at all. Measured over `engine/crates/probe/tests/` and `engine/crates/cli/tests/`, by counting the cases that name each variant:

| refusal | cases that name it |
|---|---|
| `NoProbes` | 0 |
| `TierUndeclared` | 0 |
| `Unnameable` | 0 |
| `SelectionEmpty` | 1 |
| `CampaignNarrowed` | 1 |
| `ArmNotDeclared` | 1 |
| `Undeclared` | 3 |
| `OracleUnnamed` | 1 |
| `OracleNotUsed` | 1 |
| `OracleUnknown` | 1 |
| `AnswersUndeclared` | 1 |
| `AnswersNotUsed` | 1 |
| `ExpectationNamesNothing` | 1 |
| `OverBudget` | 2 |

Three refusals of a correctness layer therefore ship with no fixture, and nothing in the suite says so.

## Obligation

The corpus owes `engine/crates/probe/` a case that holds its refusals to their own coverage, and three fixtures for the three refusals no case reaches.

The count above is a measurement of one moment. A refusal added to the enum reaches the terminal, the plan render and the grade decision with nothing to report its absence. `stops_a_grade` is the only exhaustive match over the set, and it asks about a verdict rather than about a fixture.

## Discharge

A case over one list of the refusal names closes the first half. The scaffolder's `branches!` macro is the shape, and it makes the naming function and the coverage list one list rather than two that agree today.

Three fixtures close the second half. `NoProbes` needs a corpus that classifies no probe document. `TierUndeclared` needs a budget declaration with no envelope for a tier the caller names. `Unnameable` needs a probe document that carries no identifier.

The reader of this record is this repository alone, so it waits.
