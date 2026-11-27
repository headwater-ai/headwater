---
id: HW-OBL-0079
title: "A correctness root that is a type has no failing fixture"
status: discharged
status_since: 2026-11-20
waiting_on: build
last_verified: 2026-11-20
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

Discharged on 2026-11-20 by [#1961](https://github.com/headwater-ai/headwater/pull/1961), slice C of [#1494](https://github.com/headwater-ai/headwater/issues/1494). The module comment of `engine/crates/check/src/scope.rs` now holds two `compile_fail` doctests. In the first, a document-scoped check calls `view.edges()`, which only a corpus view has. In the second, code outside the crate mints a `Scope` with a wider grain. Each one has a compiling twin that differs only in the forbidden line. The twin is necessary because stable rustdoc does not check the error code of a `compile_fail` block. The CI suite runs doctests. [Spec 12](../spec/12-check-layer.md#the-correctness-roots) now states this fixture in its bullet for scope enforcement.

The text below is the condition as it was recorded. The instrument that fits is a test which asserts that a program does not compile, and no part of this specification asks for one.
