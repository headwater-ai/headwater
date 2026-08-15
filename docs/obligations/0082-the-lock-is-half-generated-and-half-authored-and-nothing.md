---
id: HW-OBL-0082
title: "The lock is half generated and half authored, and nothing states the rule for the seam"
status: current
status_since: 2026-08-13
last_verified: 2026-08-13
summary: "The adoption payload is authored and every other line of the lock is derived, and no document says which is which."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0012
---

# The lock is half generated and half authored, and nothing states the rule for the seam

## Context

[Spec 7](../spec/07-distribution-and-federation.md#between-majors-the-corpus-is-legitimately-between-valid-states) and [Q12](../spec/09-decisions.md#q12--migration-path-for-an-existing-corpus) both put the adoption payload in the lock, and Q12 gives the reason: the lock is committed and reviewed. Every other line of that file is a function of the sources. An owner is not one, and a task that closes is a person who deletes lines. So one artifact has a generated part and an authored part, and no document says which is which.

## Obligation

Three questions stay open. Whether a payload survives a rewrite is a rule that belongs in spec 7 rather than in one function. No document says whether `resolve --check` may pass while the authored half is stale. And a payload that no digest covers is a part of a reviewed artifact that no later reader can verify.

## Discharge

The engine answers locally. `taxonomy resolve` reads the committed lock and carries the block through, the header comment names the exception, and the digest still covers the resolution alone.
