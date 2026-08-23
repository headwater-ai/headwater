---
id: HW-OBL-0039
title: "A participation expectation names one target kind"
status: current
status_since: 2026-08-12
waiting_on: build
last_verified: 2026-08-13
summary: "The declaration carries one `to_kind`, so a corpus that expects a citation from any of several register kinds cannot state it."
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
---

# A participation expectation names one target kind

## Context

The [declaration](../spec/02-taxonomy-model.md#participation-expectations) carries one `to_kind`. A corpus that expects a citation from any one of several register kinds cannot state that.

## Obligation

An abstract kind over those kinds expresses it, at the cost of a kind that exists only as a target.

## Discharge

The [fixture corpus](../taxonomies/decision-record/fixtures/README.md) of the second library entry measures the cost from the outside. It holds an evaluation and no `decision_register`, so the `evidence-cited` expectation reports a finding that no document in that corpus can ever satisfy.
