---
id: HW-OBL-0032
title: "A `remove` that orphans a name is refused, and the message names the wrong line"
status: discharged
status_since: 2026-11-28
waiting_on: build
last_verified: 2026-11-28
summary: "Referential integrity found every orphan a removal left and blamed the declaration that read it. The finding now names the `remove` and the reader both."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-taxonomy-model
---

# A `remove` that orphans a name is refused, and the message names the wrong line

## Context

[The taxonomy model](../spec/taxonomy-model.md#customization-by-composition) fails a `remove` when a declaration that survives still references the removed key. This obligation said that the second half of the rule was undecidable. The reason it gave was that a shelf declares `kind: decision` as a bare string, and the meta-schema types that position as a string.

The premise was right and the conclusion was wrong. A dependent set is undecidable from the *operation*, and the rule is decidable on the *result*. A removal that orphans a name leaves a declaration that reads a name which nothing declares, and referential integrity finds every one of them.

## Obligation

What was open was the blame. The finding named the declaration that reads the orphan and not the `remove` that made it. So an author read a message about a line that the overlay never touched.

## Discharge

`headwater taxonomy validate` runs the rule. Until this change, no part of the report carried the operation that caused the orphan.

**`Resolution::validate` discharges this record ([#1491](https://github.com/headwater-ai/headwater/issues/1491), slice 6, [PR #2141](https://github.com/headwater-ai/headwater/pull/2141)).** The rule still finds the orphan over the result, because only the result shows it. `Resolution::validate` holds the operations, so it adds the blame. For each referential integrity refusal, it finds the block that would declare the name. The sort of thing that the position reads gives the block: a `kind` read wants `kinds.<name>`, and a `facet` read wants `facets.<name>`. When a `remove` covers that address, the refusal stands at the `remove`, in the source that wrote it, at its line. The message names both ends, for example ``removing `kinds.decision` leaves `shelves.decisions.kind` reading `decision`, and no kind of that name is declared``. A refusal that no `remove` covers keeps its line. Two examples are a typo, and a facet that has the same name as the removed kind.

The rule keeps its name, `referential integrity`, so the count of rules does not change. The list of what no phase decides is now empty, and `taxonomy validate` prints `not decided anywhere: 0`.
