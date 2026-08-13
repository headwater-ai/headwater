---
id: OBL-repo-0010
title: "The corpus descriptor exists and no probe has run against it"
status: current
status_since: 2026-08-10
last_verified: 2026-08-13
summary: "Q14 claims that a descriptor lets a cold agent reach a governing document, and the probe categories that would show it do not exist."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - DR-repo-0014
---

# The corpus descriptor exists and no probe has run against it

## Context

[Q14](../spec/09-decisions.md#q14--discovery-surface) rules that the corpus descriptor is a generated projection at `.headwater/corpus.json`. A descriptor should let a cold agent reach a governing document that it otherwise misses.

## Obligation

The Discovery and Navigability probe categories are the instrument, and this corpus owes a reading from each.

## Discharge

**The artifact under test exists now, and the instrument still does not.** `headwater generate` writes `.headwater/corpus.json`, and `generate --check` holds it to regeneration. The descriptor over this corpus names one root, one exclusion, the taxonomy identity, the lock hash and five entry points. Five, because six shelves are declared and one holds no document. So a probe has something to run against, and no probe has run.

The descriptor carries each declared export profile as well, and this repository declares none, so that list is empty and means it.
