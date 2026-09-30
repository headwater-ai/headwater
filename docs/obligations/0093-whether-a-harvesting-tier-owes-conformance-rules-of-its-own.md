---
id: HW-OBL-0093
title: "Whether a harvesting tier owes conformance rules of its own"
status: current
status_since: 2026-08-11
waiting_on: ruling
last_verified: 2026-08-13
summary: "Q9 leaves open whether a harvesting tier has conformance rules, because `headwater conformance` evaluates one repository."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0009
---

# Whether a harvesting tier owes conformance rules of its own

## Context

[Q9](../spec/09-decisions.md#q9--multi-repository-corpora) rules that a repository holds one or more corpora and that the tier above harvests pinned exports.

## Obligation

The corpus owes a ruling on whether a harvesting tier owes conformance rules of its own.

## Discharge

`headwater conformance` evaluates one repository, so nothing today can even state a rule at the tier above it.

The tier has one check rule since [#1311](https://github.com/headwater-ai/headwater/issues/1311). `harvest.pin.unread` reports each pinned export that binds nothing, as an error on `.headwater/taxonomy.yml`. That rule is a `headwater check` rule and not a conformance rule, and one rule is not a set of rules. So this record stays open.
