---
id: HW-DR-0109
status: current
status_since: 2026-10-13
summary: "A route offers a document whose declared name a task repeats as a run of two or more words. It comes before the purposes and outside the budget. Before this rule, 45 of 133 decision records missed the default five for their own title."
last_verified: 2026-10-13
title: "109 — A task that repeats the declared name of a document names that document"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  constrains:
    - HW-DR-0070
  governs:
    - engine/crates/query/src/route.rs
    - engine/crates/query/src/json.rs
---

# 109 — A task that repeats the declared name of a document names that document

## Context

[Issue #1470](https://github.com/headwater-ai/headwater/issues/1470) found that `headwater route` did not offer a decision record for a query of its own title. The title is the facet in the `name` role, so a person who types it has named the record. The route still made the record win the purpose gate of [spec 5](../spec/05-ai-integration.md#intent-time-routing), and then the turns of [HW-DR-0070](0070-the-matched-purposes-take-turns-at-a-route-budget-and-each-pointer-states-what-reached-it.md).

The measurement routed the title of each record on `docs/decisions/` and `docs/process/decisions/`, with the leading number removed. On `9c9c6749`, 45 of 133 records were outside the default budget of five for their own title. For 35 of the 45, the route offered the record at no budget at all. A title such as `Relation storage` holds no word of a `rationale` question, so no decision was a candidate. HW-DR-0101 matched six purposes, and `rationale` was last, so the record stood 6th at a budget of 100,000. The adjudication measured 46 of 132 on `3aa0024c`, before this record was added.

The cause was the gate, and not the ranking. The title was already on the facet surface, at the lowest weight. A change to the weights or the purpose scores would move every route. It would also leave nothing for a title that matches no purpose.

## Decision

**A task that repeats the declared name of a document names that document.** The step runs after the anchors and before the purposes, because it is an identity and not a guess. The author declared the name, and the task repeats it, so the engine inferred nothing.

The rule is stated, not tuned:

- The words of a text are the words of two characters or more, lowercased, in the order written. This is the split that the route uses for terms, with each repeat kept.
- One leading word made only of digits comes off the name, because the decision shelves number their titles.
- A name of fewer than two words names nothing. One word is a term, and the purpose gate decides a term.
- A name names its document when the task holds its words as a run: in the same order and next to each other.

The rule asks for a run and not a set. [HW-DR-0071](0071-a-route-offers-a-document-and-never-a-heading-inside-it.md) measured that function words reach wrong documents at every threshold. A set rule lets the function words of a name match wherever a task scatters them. With a set rule, the fixture task `why is throttling applied at the edge` names the decision called `Throttling at the edge`. With a run rule, it does not.

A named pointer goes after the anchored pointers and before the ranked ones. The budget never removes it, but it takes a place that a ranked pointer would otherwise take, as an anchored pointer does. A document that is both anchored and named is offered once, as anchored. The pointer states `repeats its name` with the words, and in JSON its `evidence.by` is `name`. This adds a third value to a set that the contract closed at two, so the route document moves to version `1.1`.

This record constrains HW-DR-0070 and does not replace it. The turns still order every ranked pointer.

## Consequences

On this branch, 132 of 133 records are in the default five for their own title, 128 of them first. The one exception is HW-DR-0010, whose title less its number is `Naming`. The test `every_decision_record_is_offered_for_its_own_title_inside_the_default_budget` in `engine/crates/query/tests/reads.rs` lists it by name with the reason, and it prints one line for each record.

The step also names documents in tasks that are not titles. Each such task holds a declared name as a run. The sentence `headwater export now withholds every write…` names `docs/interfaces/headwater-export.md`, whose name is `headwater export`. Three sentence queries of #1470 still miss a target, each for the purpose gate: HW-DR-0102, spec 6 and `headwater-generate.md`. `three_sentence_queries_miss_by_the_purpose_gate_and_not_by_the_name_step` holds those cases.

A name made only of function words, such as `What it is`, is named by any task that holds that run. A name that is a run inside another name is named whenever the other is. The step reads every document once for each route, which adds one pass to the cost that [HW-OBL-0006](../obligations/0006-route-latency-at-a-harvesting-tier-is-unmeasured.md) records.
