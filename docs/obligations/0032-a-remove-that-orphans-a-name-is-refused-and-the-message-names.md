---
id: HW-OBL-0032
title: "A `remove` that orphans a name is refused, and the message names the wrong line"
status: current
status_since: 2026-08-12
waiting_on: build
last_verified: 2026-08-13
summary: "Referential integrity finds every orphan a removal leaves, and the finding blames the declaration that reads it rather than the `remove`."
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

[Spec 2](../spec/02-taxonomy-model.md#customization-by-composition) fails a `remove` when a declaration that survives still references the removed key. This obligation said that the second half of the rule was undecidable. The reason it gave was that a shelf declares `kind: decision` as a bare string, and the meta-schema types that position as a string.

The premise was right and the conclusion was wrong. A dependent set is undecidable from the *operation*, and the rule is decidable on the *result*. A removal that orphans a name leaves a declaration that reads a name which nothing declares, and referential integrity finds every one of them.

## Obligation

What is open is the blame. The finding names the declaration that reads the orphan and not the `remove` that made it. So an author reads a message about a line that the overlay never touched.

## Discharge

`headwater taxonomy validate` runs the rule. What closes the rest is a finding that carries the operation that caused the orphan, and no part of the report holds one today.
