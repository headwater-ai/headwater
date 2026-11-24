---
id: HW-OBL-0253
status: current
status_since: 2026-11-24
summary: "run-dir.sh claim compares artifacts as exact strings. So it grants a directory claim and a file claim inside it, and accept prints no HELD line for the collision."
last_verified: 2026-11-24
title: "A claim on a directory does not hold a claim on a file under it, in either direction"
waiting_on: build
provenance:
  warrant: asserted
relations:
  traces_to:
    - HW-PD-0004
---

# A claim on a directory does not hold a claim on a file under it, in either direction

## Context

`hw-run-policy` sends a finding about the build order, its tooling or the CI of this repository to this shelf. Run `20261122-0114` wrote intake lines 49, 73, 101 and 102. The product owner ruled each one RECORD into this record, which its end-of-run report calls P8. This record read the claim loop again on `0cdd96298`.

## Obligation

`claim` in `tools/run/run-dir.sh` writes one file for each artifact under `claims/artifacts/`, through `create_only`, and compares nothing else. So a claim on a directory and a claim on a file inside it never collide. The run met this four times.

- The directory claims of #1973 on `engine/crates/check/src/` and `engine/crates/cli/tests/` did not hold the file claim of #1968 on `engine/crates/check/src/scope.rs`.
- The directory claim of #1977 on `engine/crates/check/src/` did not hold the file claim of slice 2 of #1968 on `engine/crates/check/src/cache.rs`. The adjudicator had named that collision.
- #2025 got `taxonomy-source/headwater-standard/taxonomy.yml` while #1639 held the directory `taxonomy-source/headwater-standard/`.
- #2017 got the directory `.headwater/packages/headwater-standard/` while #2025 held files inside it.

An atomic test of a prefix is more than one line, because the claim must stay create-only (HW-PD-0004).

## Discharge

This record discharges when `claim` reports a path as held by a claim on a directory that contains it, and the reverse. A fixture holds both directions.
