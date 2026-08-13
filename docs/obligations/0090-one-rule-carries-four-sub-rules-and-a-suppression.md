---
id: OBL-repo-0090
title: "One rule carries four sub-rules, and a suppression cannot separate them"
status: current
status_since: 2026-08-13
last_verified: 2026-08-13
summary: "A directive names a rule, and `language.controlled.not_met` reports four different defects under one name."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - SPEC-HW-assurance-model
---

# One rule carries four sub-rules, and a suppression cannot separate them

## Context

A [directive](../spec/04-assurance-model.md#suppression) names a rule, and `language.controlled.not_met` reports the word limit, the contraction, the spelling variant and the semicolon.

## Obligation

So an author who suppresses a quoted semicolon on a block also suppresses the other three there. The report cannot then say that the other three were ever checked on it. The script this replaced named each rule separately and did not have the problem.

## Discharge

The grain that fits is the sub-rule, which either becomes four rule identifiers or becomes a second field on a directive. Four identifiers move the instance count and the coverage denominator, and a second field is a shape that no specification states.
