---
id: OBL-repo-0015
title: "Imported edge staleness is unmeasured against scaffolded edges"
status: current
status_since: 2026-08-10
last_verified: 2026-08-13
summary: "Q19 claims that imported edges do not decay faster than scaffolded ones, and nothing has been imported."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - DR-repo-0019
---

# Imported edge staleness is unmeasured against scaffolded edges

## Context

[Q19](../spec/09-decisions.md#q19--inbound-integration-an-external-system-of-record) rules how an external system of record feeds a corpus. Imported edges should not decay faster than scaffolded ones.

## Obligation

Staleness by `created_by` in `taxonomy audit` is the instrument.

## Discharge

No importer has run against this corpus, so every edge here carries an author rather than an importer, and the comparison has one side.

**An importer now exists and neither half of the instrument does.** `headwater import` ships and its refusals carry a fixture set, which is what Q19's other claim rests on. Two things stand between that and this record. `taxonomy audit` is not a verb of this engine, so nothing reports staleness by creator at all. And this repository declares no relation with `created_by: import`, so a run of the verb here would have nothing to write. Both are what the first adopter supplies, and the second is a taxonomy choice rather than an engine gap.
