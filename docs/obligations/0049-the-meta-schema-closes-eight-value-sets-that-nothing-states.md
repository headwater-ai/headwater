---
id: OBL-repo-0049
title: "The meta-schema closes eight value sets that nothing states"
status: current
status_since: 2026-08-12
last_verified: 2026-08-13
summary: "Eight value sets are closed to the values this corpus writes, and each one is marked `guess`."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - DR-repo-0002
---

# The meta-schema closes eight value sets that nothing states

## Context

The role registry, the relation families and the `created_by` set are closed in [spec 2](../spec/02-taxonomy-model.md) itself. Eight more are not. Six are the role on a lifecycle-state value, the type of a facet, the severity of a facet or an expectation, `volatility`, `reciprocal` and `cardinality`. The other two arrived with an obligation, and they are its class and its severity.

## Obligation

`engine/crates/meta/meta-schema.yml` closes each one to the values that this corpus writes, and it marks each one with the word `guess`. Each guess is still a value set that an adopter meets before the specification states it.

## Discharge

[Q2](../spec/09-decisions.md#q2--schema-format) settles the direction, because a rule relaxes later at no cost. A rule that arrives later is a finding against every source that already used the form. What closes each guess is a statement in the specification of the set it names.
