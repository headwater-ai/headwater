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
