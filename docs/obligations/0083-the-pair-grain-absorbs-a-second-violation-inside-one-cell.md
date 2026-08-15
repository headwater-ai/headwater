---
id: HW-OBL-0083
title: "The pair grain absorbs a second violation inside one cell"
status: current
status_since: 2026-08-13
last_verified: 2026-08-13
summary: "A `(document, rule)` cell holds every finding of that rule on that document, so a second violation is silent."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-distribution-and-federation
---

# The pair grain absorbs a second violation inside one cell

## Context

[Spec 7](../spec/07-distribution-and-federation.md#between-majors-the-corpus-is-legitimately-between-valid-states) refuses the document grain in one sentence: "the pair grain keeps yesterday's regression loud while the declared debt stays patient". Writing the payload found the limit of that claim. It holds across documents and across rules, and it does not hold inside one cell.

## Obligation

A pair is a `(document, rule)` cell, so it holds every finding of that rule on that document. A second violation of a declared rule in a declared document is therefore silent.

## Discharge

The grain that would report it is `(document, rule, line)`, and a line number moves whenever a paragraph above it moves. A payload at that grain expires on the next edit rather than on its date. So the cell is the coarsest grain that cannot blanket a rule or a document, and the absorption is its price. Spec 7 states the property and not the price.
