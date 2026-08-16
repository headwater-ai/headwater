---
id: HW-OBL-0010
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
    - HW-DR-0014
---

# The corpus descriptor exists and no probe has run against it

## Context

[Q14](../spec/09-decisions.md#q14--discovery-surface) rules that the corpus descriptor is a generated projection at `.headwater/corpus.json`. A descriptor should let a cold agent reach a governing document that it otherwise misses.

## Obligation

The Discovery and Navigability probe categories are the instrument, and this corpus owes a reading from each.

## Discharge

**The artifact under test exists, the instrument is declared, and no run has happened.** [HW-PROBE-a-cold-agent-reaches-the-governing-document-through-the-corpus-descriptor](../probes/a-cold-agent-reaches-the-governing-document-through-the-corpus-descriptor.md) states the task and the expectation, and `headwater probe plan` selects it. What is missing is a recorder: a transcript is observed from outside the session that produced it, and no such process is in this repository. So the wait moved from an instrument nobody had declared to a run nobody has taken.

`headwater generate` writes `.headwater/corpus.json`, and `generate --check` holds it to regeneration. The descriptor over this corpus names one root, one exclusion, the taxonomy identity and the lock hash. It names one entry point for each shelf that holds a document. `.headwater/corpus.json` is where that set is counted, and a shelf with no document on it puts nothing there. So a probe has something to run against, and no probe has run.

The descriptor carries each declared export profile as well, and this repository declares none, so that list is empty and means it.
