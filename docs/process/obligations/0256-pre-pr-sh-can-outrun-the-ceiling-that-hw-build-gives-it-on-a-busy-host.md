---
id: HW-OBL-0256
status: current
status_since: 2026-11-24
summary: "hw-build gives tools/run/pre-pr.sh a timeout of 600000 ms. On a busy host one run passed that limit, and its second run took about 35 minutes."
last_verified: 2026-11-24
title: "pre-pr.sh can outrun the ceiling that hw-build gives it on a busy host"
waiting_on: build
provenance:
  warrant: asserted
---

# pre-pr.sh can outrun the ceiling that hw-build gives it on a busy host

## Context

`hw-run-policy` sends a finding about the build order, its tooling or the CI of this repository to this shelf. Run `20261122-0114` wrote intake line 119. The product owner ruled it RECORD into this record, which its end-of-run report calls P11. This record checked the agent definition again on `0cdd96298`. The times are as the hw-iterate of #1990 reported them.

## Obligation

`.claude/agents/hw-build.md` tells a builder to run `sh tools/run/pre-pr.sh` with a `timeout` of 600000. On a busy host, a run for #1990 passed that limit, and its second run took about 35 minutes. A longer run needs a background job and a wait on its exit, which is the shape that the run policy gives for a suite. So the remedy is more than one line.

## Discharge

This record discharges when `hw-build.md` gives `pre-pr.sh` a shape that a slow host does not cut off. A script that runs inside the limit on a busy host also discharges it.
