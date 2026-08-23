---
id: HW-OBL-0065
title: "A classified document with no check instance has a third cause"
status: current
status_since: 2026-08-12
waiting_on: build
last_verified: 2026-08-13
summary: "Spec 12 names two causes for a document with zero instances, and the one this corpus met is neither."
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

# A classified document with no check instance has a third cause

## Context

[Spec 12](../spec/12-check-layer.md#instances-and-why-coverage-needs-them) rules that a document with zero instances is a finding. It names two causes: a shelf pattern that is wrong, or a file in the wrong place.

The runner reported three such documents in this repository when it landed, and neither cause held for any of them. Each one sat on the shelf that its author intended, and the taxonomy declared no rule that read its kind. That is the ordinary state of a young taxonomy rather than a defect in the corpus.

## Obligation

Either spec 12 names the third cause, or the remediation prose says what a reader is to do about it.

## Discharge

The generated Shape and Graph checks took the count to zero here, because every classified document now carries a rule that reads its front matter. The finding is still worth making, because it is what an adopter meets on a first run.
