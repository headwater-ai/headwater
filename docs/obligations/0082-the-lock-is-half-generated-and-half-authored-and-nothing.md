---
id: HW-OBL-0082
title: "The lock is half generated and half authored, and nothing states the rule for the seam"
status: current
status_since: 2026-08-13
waiting_on: ruling
last_verified: 2026-08-31
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

Three questions stand here, and the second one now has a measured answer beside it. Whether a payload survives a rewrite is a rule that belongs in spec 7 rather than in one function. `resolve --check` does pass while the authored half is stale, and the lock of this repository is the instance it passes over. `resolve` reads the package sources and never the corpus, so no run of it holds the input that would answer otherwise. What no document states is whether it **may**, and that is a ruling rather than a measurement. And a payload that no digest covers is a part of a reviewed artifact that no later reader can verify.

## Discharge

The engine answers locally. `taxonomy resolve` reads the committed lock and carries the block through, the header comment names the exception, and the digest still covers the resolution alone.

The third question narrowed to one field rather than the whole block. [Issue #78](https://github.com/headwater-ai/headwater/issues/78) needed `adoption.from` to be writable at all. [HW-DR-0046](../decisions/0046-migrating-from-version-carries-a-semver-and-a-release-digest-kept-as-separate-fields.md) rules that it is a semver and a release digest. The two are kept as plain fields, and never as a bare assertion or a hash of the two. That answers what `from` may hold. It does not answer the first two questions here. Whether a payload surviving a rewrite is a rule spec 7 should state stays the owner's. So does whether `resolve --check` may pass while the authored half is stale.
