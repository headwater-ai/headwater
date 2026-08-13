---
id: OBL-repo-0072
title: "A cache of check results does not make a run proportional to the change"
status: current
status_since: 2026-08-12
last_verified: 2026-08-13
summary: "Spec 6 promises a run proportional to the change, and a warm run still walks the corpus and builds the whole graph."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - SPEC-HW-engine-architecture
---

# A cache of check results does not make a run proportional to the change

## Context

[Spec 6](../spec/06-engine-architecture.md#pipeline) puts the cache after the graph build. It says the cache is "content-addressed per file plus taxonomy hash, so incremental runs are proportional to the change, not the corpus". The cache that landed keys a check instance rather than a file, and it holds a verdict rather than a parse.

## Obligation

So a warm run serves every verdict it can and it still walks the corpus, parses every document and builds the whole graph. Nobody has measured which of the two halves a run spends its time in.

## Discharge

Either a phase before the checks caches a parse per file, or spec 6 moves the cache in its diagram and says why.
