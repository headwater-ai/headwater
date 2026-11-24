---
id: HW-OBL-0246
status: current
status_since: 2026-11-24
summary: "The oracle suite leaves a copy of the tree in /tmp, and three recorder cases fail when TMPDIR is off /tmp. The shared runner shares one small /tmp."
last_verified: 2026-11-24
title: "No one TMPDIR holds both probe suites on a busy host, and the oracle suite leaves a copy of the tree in /tmp"
waiting_on: build
provenance:
  warrant: asserted
---

# No one TMPDIR holds both probe suites on a busy host, and the oracle suite leaves a copy of the tree in /tmp

## Context

`hw-run-policy` sends a finding about the build order, its tooling or the CI of this repository to this shelf. Run `20261122-0114` wrote intake lines 2, 6, 9, 129, 140, 142 and 163. The product owner ruled each one RECORD into this record, which its end-of-run report calls P1. Line 163 adds no new gap. This record checked each script item again on `0cdd96298`, by the file and the symbol it names. The quota failures and the CI runs are as the run reported them, and this record did not measure them again.

## Obligation

**The oracle suite leaves its scratch directory behind.** `tools/probe/oracle-fixtures.sh` makes one directory with `mktemp -d "${TMPDIR:-/tmp}/oracle-fixtures.XXXXXX"` and sets no `trap`. Each case copies the base tree into it. When its caller dies, a directory of 3 to 5 GB stays in `/tmp`. On 2026-11-22 such directories filled the user quota of the 16G `/tmp` tmpfs, and `rustc` and every Bash result of a builder failed with `EDQUOT`. On 2026-11-24 the suite failed 48 of 198 cases on the same quota while other builds ran, and all 198 held with `TMPDIR` on `/home/james/projects`.

**No one TMPDIR holds both probe suites.** With `TMPDIR` on `/mnt/build`, three confinement cases of `tools/probe/probe-record-fixtures.sh` fail, because they expect the workspace under `/tmp`. With `TMPDIR` on `/tmp`, the oracle suite fails on the quota when builds run in parallel. So a builder on a busy host has no `TMPDIR` on which `tools/run/pre-pr.sh` passes, and can only run it again when `/tmp` has room. `pre-pr.sh` reports the quota failure as a fixture failure and not as a full disk.

**The shared runner shares one small /tmp.** On 2026-11-24 the shared runner ran out of space three times. Two of them were a branch CI run on `merge_provocation.rs` and a merge group on the `cp` of the oracle suite under `/tmp`. Two `sccache` startup timeouts came on the same day. The host `/tmp` is a 16G tmpfs that two `Runner.Listener` processes and the build stages share, and `/` was at 94%. Each such failure costs one CI cycle and holds a slot for 20 to 40 minutes.

## Discharge

Each item discharges alone. The first discharges when `oracle-fixtures.sh` removes its directory on every exit, or a sweep removes stale ones. The second discharges when one `TMPDIR` lets both suites pass, or `pre-pr.sh` names a full disk as a full disk. The third discharges when the runner has a `/tmp` that the jobs cannot fill, or a change records why it stays. The record discharges when every item has.
