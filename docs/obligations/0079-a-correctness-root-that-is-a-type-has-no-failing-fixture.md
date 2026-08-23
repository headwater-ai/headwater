---
id: HW-OBL-0079
title: "A correctness root that is a type has no failing fixture"
status: current
status_since: 2026-08-12
waiting_on: build
last_verified: 2026-08-13
summary: "Scope enforcement is a type now, and a build that passes is not a fixture that fails."
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

# A correctness root that is a type has no failing fixture

## Context

[Spec 12](../spec/12-check-layer.md#the-correctness-roots) asks each correctness root for conformance fixtures of its own. The rule behind that ask is that [a check without a failing fixture does not ship](../spec/12-check-layer.md#testing-a-check-without-a-failing-fixture-does-not-ship).

Scope enforcement is a type now, and a document-scoped check that reaches for a sibling is a program the compiler refuses. A build that passes is not a fixture that fails.

## Obligation

Nothing states what stands in for a failing fixture when the enforcement is a type.

## Discharge

The instrument that fits is a test which asserts that a program does not compile, and no part of this specification asks for one.
