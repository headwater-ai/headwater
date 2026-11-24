---
id: HW-OBL-0248
status: current
status_since: 2026-11-24
summary: "The worktrees volume reached 95 percent in run 20261122-0114. No stage retires a merged agent worktree, and disk-free.sh enforces its floor on the pool alone."
last_verified: 2026-11-24
title: "Nothing retires a merged agent worktree, and the disk floor reads only the cargo pool"
waiting_on: build
provenance:
  warrant: asserted
---

# Nothing retires a merged agent worktree, and the disk floor reads only the cargo pool

## Context

`hw-run-policy` sends a finding about the build order, its tooling or the CI of this repository to this shelf. Run `20261122-0114` wrote intake line 15. The product owner ruled it RECORD into this record, which its end-of-run report calls P3. This record checked the script again on `0cdd96298`. The disk figures are as the parent reported them.

## Obligation

During the run, `/home/james/projects` reached 95%, with 3.0G free and 221 worktrees. A worktree that the harness made for a build or a verify stays after its pull request merges. No stage of a run removes it. `tools/run/disk-free.sh` measures the worktrees volume, but `--floor` fails only on the volume of the cargo pool, and `hw-iterate` reads only that floor. The parent removed 125 merged worktrees by hand and freed 28G.

## Discharge

This record discharges when a stage of a run retires merged agent worktrees, or when `disk-free.sh --floor` also fails on a full worktrees volume. A change that records why neither is done also discharges it.
