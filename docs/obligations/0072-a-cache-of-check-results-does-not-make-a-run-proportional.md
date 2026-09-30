---
id: HW-OBL-0072
title: "A cache of check results does not make a run proportional to the change"
status: current
status_since: 2026-08-12
waiting_on: ruling
last_verified: 2026-10-01
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
    - HW-SPEC-engine-architecture
    - HW-OBL-0080
---

# A cache of check results does not make a run proportional to the change

## Context

[Spec 6](../spec/06-engine-architecture.md#pipeline) puts the cache after the graph build. It says the cache is "content-addressed per file plus taxonomy hash, so incremental runs are proportional to the change, not the corpus". The cache that landed keys a check instance rather than a file, and it holds a verdict rather than a parse.

## Obligation

So a warm run serves every verdict it can and it still walks the corpus, parses every document and builds the whole graph. [HW-OBL-0080](0080-changed-only-is-the-content-addressed-cache-under-another-name.md) measures which of the two halves a run spends its time in. In 2026-08, at 158 documents, Phase A cost about 40 ms of a 57 ms warm run. The cache then served nearly all of check evaluation. On 2026-10-01, at 500 typed documents, Phase A cost 90 ms of a 177 ms warm run. A run with no cache cost 523 ms. So a warm run spends about 87 ms after Phase A that the cache does not remove. `sh tools/measure/check-phases.sh` states what that time is, stage by stage, and HW-OBL-0080 records the reading. The largest stage is the edge rules, which read the digest of every governed file for the cache key of each edge.

The cache therefore makes the second half proportional to the change and leaves the first half proportional to the corpus. What spec 6 promises is the whole run.

## Discharge

Either a phase before the checks caches a parse per file, or spec 6 moves the cache in its diagram and says why.
