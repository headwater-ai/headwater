---
id: HW-OBL-0041
title: "No declaration expresses a rule about content or state over time"
status: current
status_since: 2026-08-13
waiting_on: build
last_verified: 2026-08-13
summary: "The decision-record tradition holds two invariants over time, and every declaration that carries time is a participation expectation."
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
    - HW-SPEC-check-layer
---

# No declaration expresses a rule about content or state over time

## Context

The decision-record tradition holds two invariants, and the language expresses neither. An accepted record is amended by succession and never by edit, which reads a content change against a lifecycle state. A proposed record reaches a ruling, which reads dwell time in a state.

Every declaration that carries time is a [participation expectation](../spec/02-taxonomy-model.md#participation-expectations), and an expectation reports one thing: a missing edge inside a window. Both failures above have every edge that they need.

## Obligation

The language owes a declaration for a rule over content change and a rule over dwell time.

## Discharge

The entry declares each as an obligation with a `gap` disposition, and each names an engine input that already exists. [Spec 12](../spec/12-check-layer.md#temporal-inputs-the-clock-and-the-prior-version) supplies the prior version as `needs_prior`, and `taxonomy audit` already measures state dwell. So the gap is a declaration rather than a capability, and it is the check class that the second entry adds to this library.
