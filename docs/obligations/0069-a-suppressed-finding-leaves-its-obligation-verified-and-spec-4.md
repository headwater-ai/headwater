---
id: HW-OBL-0069
title: "A suppressed finding leaves its obligation verified, and spec 4 does not say so"
status: current
status_since: 2026-08-13
waiting_on: build
last_verified: 2026-08-13
summary: "The engine rules that an obligation with accepted deviations against it stays verified, and spec 4 states no such rule."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-assurance-model
---

# A suppressed finding leaves its obligation verified, and spec 4 does not say so

## Context

A [suppression](../spec/04-assurance-model.md#suppression) runs after the runner stamps each finding with the obligation that its control discharges, so the inventory holds findings that name obligations. Spec 4 does not say what an obligation with two accepted deviations against it is.

## Obligation

What is open is whether spec 4 admits the engine's reading. A second question is whether the register needs a rate rather than a count.

## Discharge

The engine rules that it stays verified. A disposition is about whether a control discharges the invariant, and not about whether the corpus conforms today. A suppression is one author's judgment about one finding, and it removes no control. So the register prints the escaped count beside the obligation rather than moving the disposition.
