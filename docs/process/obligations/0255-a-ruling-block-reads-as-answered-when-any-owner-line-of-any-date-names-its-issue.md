---
id: HW-OBL-0255
status: current
status_since: 2026-11-24
summary: "answered_issues in run-dir.sh reads every OWNER line of decisions.md. So a block owed after the last answer on its issue renders as answered."
last_verified: 2026-11-24
title: "A RULING block reads as answered when any OWNER line of any date names its issue"
waiting_on: build
provenance:
  warrant: asserted
---

# A RULING block reads as answered when any OWNER line of any date names its issue

## Context

`hw-run-policy` sends a finding about the build order, its tooling or the CI of this repository to this shelf. Run `20261122-0114` wrote intake line 90. The product owner ruled it RECORD into this record, which its end-of-run report calls P10. This record read the function again on `0cdd96298`.

## Obligation

`answered_issues` in `tools/run/run-dir.sh` marks an issue answered when any `OWNER` line of `decisions.md` names it, and it reads no date and no block order. On 2026-11-23 both RULING blocks owed on #1968 rendered `answered: yes`, from the merge license that the owner gave on 2026-11-22. A block needs the time at which it was owed before a comparison can work, so the remedy is more than one line.

## Discharge

This record discharges when `rulings` marks a block answered only by an `OWNER` line written after the block was owed. A fixture holds a block owed after the last answer.
