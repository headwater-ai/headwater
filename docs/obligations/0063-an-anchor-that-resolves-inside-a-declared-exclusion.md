---
id: HW-OBL-0063
title: "An anchor that resolves inside a declared exclusion has no stated outcome"
status: current
status_since: 2026-08-12
waiting_on: ruling
last_verified: 2026-08-13
summary: "Spec 1 fixes three outcomes for an anchor, and a hit inside a corpus exclusion is none of them."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0017
---

# An anchor that resolves inside a declared exclusion has no stated outcome

## Context

[Spec 1](../spec/01-conceptual-model.md#external-anchor) fixes three outcomes for an anchor: resolved, unresolved, and withheld. A corpus exclusion states that a path is not corpus content, which is neither a defect nor a withholding. [Q17](../spec/09-decisions.md#q17--governed-access-and-the-solution-layer) rules on the withheld case and not on this one.

## Obligation

Either spec 1 admits a fourth outcome, or an exclusion is a fact about the census alone and an anchor never sees it.

## Discharge

The engine reports the hit, because the file is there and write-time impact detection reads the file rather than the corpus. It names the exclusion beside the hit, so that no reader learns that a document governs something this corpus says it does not hold.
