---
id: HW-OBL-0076
title: "A generated check reads its severity from the taxonomy"
status: current
status_since: 2026-08-13
last_verified: 2026-08-13
summary: "An adopter moves a generated finding from `warn` to `error` by editing data, and spec 12 reads as though only a control is data."
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

# A generated check reads its severity from the taxonomy

## Context

[Spec 12](../spec/12-check-layer.md#severity-is-the-checks-posture-is-the-controls) gives a check its severity and gives a control the decision to block, which makes a promotion a configuration change. A participation expectation declares `severity: warn` in the taxonomy, and the generated check reports what the declaration says.

## Obligation

So an adopter moves a finding from `warn` to `error` by editing data, and no code changes. What is open is whether spec 12 admits it, because its own sentence reads as though only a control is data.

## Discharge

For a generated rule the declaration is the check, so the engine reads this as consistent. A sentence in spec 12 is what settles whether that reading is the intended one.
