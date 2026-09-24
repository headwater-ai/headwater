---
id: HW-OBL-0085
title: "`headwater init` cannot reach a package that is not already vendored"
status: discharged
status_since: 2026-09-24
waiting_on: build
last_verified: 2026-09-24
summary: "The first verb a new adopter runs could not complete on a fresh repository, because this engine fetched no package until `taxonomy vendor` took a location."
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

[Spec 7](../spec/07-distribution-and-federation.md#publishing) says that distribution is over the registry or repository an organization already uses. It also says that the engine needs only to fetch a version and check its digest. Nothing in this engine fetched anything when this record was written. `headwater_resolve::package` reads `.headwater/packages/` on disk and no other place.

## Obligation

So the one verb that a new adopter runs first cannot complete on a repository that does not already hold the package. The gap is that the fetch has no stated form: not a protocol, not a cache location, and not a digest check.

## Discharge

`init` reports the miss by name rather than writing a version it invented, which moves the failure to the verb that caused it. What closes it is a stated form for the fetch.

This obligation is discharged. [#959](https://github.com/headwater-ai/headwater/issues/959) gave the fetch a stated form. `headwater taxonomy vendor <dir-or-location>` takes an `https://` location, fetches the artifact zip, and checks its digest before it installs the package. [The interface contract](../interfaces/headwater-taxonomy.md) states that argument.

`init` now names the same argument in the line it prints and in the comment it writes. Neither message says that nothing fetches a package. The test `the_vendor_route_init_names_takes_the_location_a_release_publishes` in `engine/crates/cli/tests/init.rs` holds both messages to the row of the contract. It then serves a published artifact zip on a loopback location, runs the route that the messages name, and reaches a resolved version.

One gap stays open, and it is not this record. No released binary carries the fetch yet. The newest release is v0.1.2, and it reads a location as a directory. So a reader who installs that release still meets the old remedy, until a release carries #959. [#1063](https://github.com/headwater-ai/headwater/issues/1063) holds the pages that move with that release.
