---
id: HW-OBL-0254
status: current
status_since: 2026-11-24
summary: "Since #1998 hw-iterate dispatches headwater-maintainer on UPKEEP not run. The orchestration spec and the decision on the loop below the parent do not name that child."
last_verified: 2026-11-24
title: "The orchestration spec and HW-PD-0022 do not name the maintainer that hw-iterate now dispatches"
waiting_on: build
provenance:
  warrant: asserted
relations:
  traces_to:
    - HW-PD-0022
---

# The orchestration spec and HW-PD-0022 do not name the maintainer that hw-iterate now dispatches

## Context

`hw-run-policy` sends a finding about the build order, its tooling or the CI of this repository to this shelf. Run `20261122-0114` wrote intake lines 79 and 80, from the maintainer pass over #1998. The product owner ruled both RECORD into this record, which its end-of-run report calls P9. This record checked both documents again on `0cdd96298`.

## Obligation

- The diagram and the loop-stage prose of `docs/process/specs/orchestration-architecture.md` list only build and verify as children of hw-iterate. They do not name the `UPKEEP:` line of the build report.
- The summary of HW-PD-0022, and the section that names the children of hw-iterate, say that it dispatches the builder and each verifier. No process decision rules on the third child.

## Discharge

This record discharges when both documents name `headwater-maintainer` as the child that hw-iterate dispatches on `UPKEEP: not run`. A process decision that rules otherwise, with agent definitions that follow it, also discharges it.
