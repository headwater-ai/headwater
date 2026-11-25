---
id: HW-OBL-0258
status: current
status_since: 2026-11-24
summary: "The pool targets on /mnt/build held 99G and stopped a verify at the disk floor. Only a prune of the whole pool exists, and disk-free.sh only refuses."
last_verified: 2026-11-24
title: "The cargo pool grows with nothing pruning it between builds"
waiting_on: build
provenance:
  warrant: asserted
---

# The cargo pool grows with nothing pruning it between builds

## Context

`hw-run-policy` sends a finding about the build order, its tooling or the CI of this repository to this shelf. Run `20261122-0114` wrote intake line 146. The product owner ruled it RECORD into this record, which its end-of-run report calls P14. This record checked both scripts again on `0cdd96298`. The disk figures are as the parent reported them.

## Obligation

`/mnt/build` fell to 28G free, under the floor of 40G, and stopped the verify of #1970. The three numbered pool targets under `/mnt/build/headwater/cargo-pool` held 99G, and 25G of that was `debug/incremental`. `tools/hw-cargo --prune` removes the whole pool, nothing else prunes a slot, and `tools/run/disk-free.sh --floor` only refuses. The parent removed `debug/incremental` by hand with no slot held, which left 54G free. A prune of one slot must hold that slot's lock, so the remedy is more than one line.

## Discharge

This record discharges when a step of a run prunes the incremental products of a slot under its lock, before the pool reaches the floor.
