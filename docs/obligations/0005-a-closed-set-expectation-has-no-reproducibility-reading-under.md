---
id: HW-OBL-0005
title: "A closed-set expectation has no reproducibility reading under grading"
status: current
status_since: 2026-08-11
waiting_on: build
last_verified: 2026-08-13
summary: "Q8 claims that a closed-set expectation makes a probe verdict reproducible, and no probe has run."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0008
---

# A closed-set expectation has no reproducibility reading under grading

## Context

[Q8](../spec/09-decisions.md#q8--probe-cost-and-cadence) rules that a probe is a document with a declared expectation. A closed-set expectation should make a probe verdict reproducible under grading.

## Obligation

`generate --check` over a result document is the instrument, and this corpus owes a reading of it.

## Discharge

`headwater generate --check` exists and holds three artifacts of this repository. No probe kind is declared here, so no result document exists for it to hold, and the instrument has nothing to read.
