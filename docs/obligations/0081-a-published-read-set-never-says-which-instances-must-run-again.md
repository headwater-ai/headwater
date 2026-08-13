---
id: OBL-repo-0081
title: "A published read set never says which instances must run again"
status: current
status_since: 2026-08-13
last_verified: 2026-08-13
summary: "A union of inputs answers whether a verdict survives a merge, and the artifact that answers the finer question is one row per instance."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - DR-repo-0021
---

# A published read set never says which instances must run again

## Context

[Spec 12](../spec/12-check-layer.md#the-read-set-and-what-a-merge-does-to-a-verdict) asks a run to report the union of its in-scope inputs, and the union is what `headwater check` publishes. A gate that holds the union against a later tree learns one thing: whether anything the run read has moved.

The same section also says that the engine derives the invalidated instances from a merge, and the union cannot support that. One changed document voids the instances that read it, and a union does not say which instances those are.

## Obligation

The artifact that answers the second question is one row per instance rather than one per document. Over this repository that is 424 rows against 36.

## Discharge

Either spec 12 states that the published artifact answers the coarse question alone, or the artifact carries a row per instance. The cost question of [Q21](../spec/09-decisions.md#q21--terminological-succession-and-validity-under-merge) is then asked again at that size.
