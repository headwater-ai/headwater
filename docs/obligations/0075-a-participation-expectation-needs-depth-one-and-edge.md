---
id: HW-OBL-0075
title: "A participation expectation needs depth one, and `Edge` does not reach it"
status: current
status_since: 2026-08-13
waiting_on: build
last_verified: 2026-08-13
summary: "The check layer calls the neighbourhood scope speculative and offers a test that misses the check that arrived."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-check-layer
---

# A participation expectation needs depth one, and `Edge` does not reach it

## Context

[The check layer](../spec/check-layer.md#what-this-leaves-open) calls the `Neighbourhood(depth)` scope speculative, and says to cut it if no real check needs a depth above one. That test misses the case that arrived. An edge-scoped instance exists for each edge, and this rule is about an edge that nobody declared.

## Obligation

Either [the check layer](../spec/check-layer.md) keeps the scope and restates its test, or it names the grain that a check about an absent edge receives.

## Discharge

The engine reads the document and the kinds one relation away from it, which is depth one exactly. That reading runs today, and no sentence of the specification carries it.
