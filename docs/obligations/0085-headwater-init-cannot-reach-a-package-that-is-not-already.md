---
id: HW-OBL-0085
title: "`headwater init` cannot reach a package that is not already vendored"
status: current
status_since: 2026-08-13
last_verified: 2026-08-13
summary: "Nothing in this engine fetches anything, so the first verb a new adopter runs cannot complete on a fresh repository."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-distribution-and-federation
---

# `headwater init` cannot reach a package that is not already vendored

## Context

[Spec 7](../spec/07-distribution-and-federation.md#publishing) says that distribution is over the registry or repository an organization already uses. It also says that the engine needs only to fetch a version and check its digest. Nothing in this engine fetches anything. `headwater_resolve::package` reads `packages/` on disk and no other place.

## Obligation

So the one verb that a new adopter runs first cannot complete on a repository that does not already hold the package. The gap is that the fetch has no stated form: not a protocol, not a cache location, and not a digest check.

## Discharge

`init` reports the miss by name rather than writing a version it invented, which moves the failure to the verb that caused it. What closes it is a stated form for the fetch.
