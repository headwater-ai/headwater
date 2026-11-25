---
id: HW-OBL-0251
status: current
status_since: 2026-11-24
summary: "After a parent restart, the completion of a child that a resumed hw-iterate dispatched reached the parent. The hw-iterate waited for a report that never came."
last_verified: 2026-11-24
title: "A child that a resumed hw-iterate dispatches notifies the parent and not the hw-iterate"
waiting_on: build
provenance:
  warrant: asserted
relations:
  traces_to:
    - HW-PD-0021
    - HW-PD-0022
---

# A child that a resumed hw-iterate dispatches notifies the parent and not the hw-iterate

## Context

`hw-run-policy` sends a finding about the build order, its tooling or the CI of this repository to this shelf. Run `20261122-0114` wrote intake lines 41 and 59. The product owner ruled both RECORD into this record, which its end-of-run report calls P6. The routing is as the parent reported it in sessions of #1969, #1971 and #1973. The half about the report on disk was a one-line remedy, and the run-lessons pull request of this run applied it to `.claude/agents/hw-iterate.md`.

## Obligation

The section of `.claude/agents/hw-iterate.md` that says to wait by ending the turn assumes that the completion of a child wakes the hw-iterate that dispatched it. In run `20261122-0114`, after a parent restart, an hw-iterate resumed from a handover dispatched `hw-build` and `hw-verify`. The completion of each child reached the parent and not the hw-iterate. The hw-iterate held the rework and waited, and the parent relayed each result by hand. This is how the harness routes a notification, and no sentence of an agent definition changes it.

## Discharge

This record discharges when a resumed hw-iterate receives the completion of each child it dispatches. It also discharges when the agent definitions name another path to the result, or when a change records why the parent relays it.
