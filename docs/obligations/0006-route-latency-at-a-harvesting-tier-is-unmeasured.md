---
id: OBL-repo-0006
title: "Route latency at a harvesting tier is unmeasured"
status: current
status_since: 2026-08-10
last_verified: 2026-08-13
summary: "Q9 claims that harvest keeps a solution-tier route query inside 100 ms, and no such tier exists."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - DR-repo-0009
---

# Route latency at a harvesting tier is unmeasured

## Context

[Q9](../spec/09-decisions.md#q9--multi-repository-corpora) rules that the tier above harvests pinned exports and that no merged graph exists. Harvest should keep a solution-tier route query inside the same 100 ms budget.

## Obligation

Route latency at that tier is the instrument, and this corpus owes a reading of it.

## Discharge

The route query exists now, and its latency at one corpus is measured. A release build over 36 documents answers the verb in 20 to 30 ms. Inside one session, where the walk is paid once, a call is about 3.5 ms. So the cost is the walk rather than the route, and the tier the instrument names does not exist. The claim stays open with a baseline on the record.
