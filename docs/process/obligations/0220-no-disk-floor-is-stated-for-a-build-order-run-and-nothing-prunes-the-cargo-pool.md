---
id: HW-OBL-0220
status: current
status_since: 2026-09-27
summary: "No stage prunes the pooled cargo targets, and nothing in tools/hw-cargo enforces the free-disk floor on the pool's volume. On 2026-09-27 the pool held about 160 GB and the root volume, which then held the pool, reached 95%."
last_verified: 2026-10-07
title: "No disk floor is stated for a build-order run, and nothing prunes the cargo pool"
waiting_on: build
---

# No disk floor is stated for a build-order run, and nothing prunes the cargo pool

## Context

Every cargo command of a build-order run goes through `tools/hw-cargo`. The script keeps one target directory for each slot under `~/.cache/headwater/cargo-pool/`. The skill `hw-run-policy` states that one slot target holds 10 to 13 GB, and that seven slots once took the root volume to 96%. The pool was on that volume then. It tells a verifier on its own slot to remove that slot target when it reports. It tells `hw-iterate` to run `sh tools/run/disk-free.sh` before it dispatches a verify. That script reads the free space on the volume that the pool resolves to, and on the volume that holds the worktrees. `hw-verify` states a floor of 40G on the pool's volume. Before #1276 the check was the duty of the parent. Before #1694 the check read `df -h /`, but the pool had moved to its own volume.

Run `20260926-1327` met this gap. The product owner ruled it Record, because the run tooling does not ship to an adopter.

## Obligation

On 2026-09-27, the root volume, which then held the pool, reached 95%, with 15 GB free. The five slot targets held about 160 GB. The parent removed the `debug` and `incremental` directories under each slot, with that slot's lock held, and got back about 33 GB. When this record was written later that day, the pool held 136 GB and the root volume stood at 90%.

`hw-run-policy` states no free-disk floor of its own. It cites the floor that `hw-verify` states, and that floor stops a verify but prunes nothing. It names no stage that prunes a slot target other than the verifier's own. `tools/hw-cargo --prune` removes the whole pool, and only a person runs it. No stage calls it, and nothing in `tools/hw-cargo` reads the free disk before a build. A full disk turns a CI run on the shared runner red with `ENOSPC`.

## Discharge

This record discharges when three conditions hold. First, `hw-run-policy` names a free-disk floor for a run. Second, it names the stage that prunes the pool. Third, `tools/hw-cargo` or a `tools/run/run-dir.sh` subcommand enforces the floor, and a case shows a build that stops or prunes below it.
