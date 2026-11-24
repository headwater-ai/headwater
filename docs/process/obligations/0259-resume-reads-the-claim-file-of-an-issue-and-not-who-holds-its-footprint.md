---
id: HW-OBL-0259
status: current
status_since: 2026-11-24
summary: "For an adjudicated handover, resume in run-dir.sh prints iterate when claims/issues/N exists, and does not read whether another issue holds the paths of its footprint."
last_verified: 2026-11-24
title: "resume reads the claim file of an issue and not who holds its footprint"
waiting_on: build
provenance:
  warrant: asserted
relations:
  traces_to:
    - HW-PD-0023
---

# resume reads the claim file of an issue and not who holds its footprint

## Context

`hw-run-policy` sends a finding about the build order, its tooling or the CI of this repository to this shelf. Run `20261122-0114` wrote intake line 147. The product owner ruled its second half RECORD into this record, which its end-of-run report calls P15. The first half was the ejection gap of `logged_issues`, and the run-lessons pull request of this run applied that remedy. This record read `resume` again on `0cdd96298`.

## Obligation

For a handover at stage `adjudicated`, `resume` in `tools/run/run-dir.sh` prints `iterate` when `claims/issues/<N>` exists, and `claim <artifacts>` otherwise. It does not read `claims/artifacts/`. On 2026-11-24 it printed `iterate` for #2016 while #1573 held the paths that the two issues shared.

## Discharge

This record discharges when `resume` waits for the holder of a path of the footprint that another issue holds. A fixture holds that case.
