---
id: HW-OBL-0247
status: current
status_since: 2026-11-24
summary: "After any cargo command that passes, tools/hw-cargo copies the CLI binary of its pooled slot into the tree. A test of one crate can leave the CLI of another tree."
last_verified: 2026-11-24
title: "The cargo wrapper copies a CLI binary into the tree after a command that did not build the CLI"
waiting_on: build
provenance:
  warrant: asserted
---

# The cargo wrapper copies a CLI binary into the tree after a command that did not build the CLI

## Context

`hw-run-policy` sends a finding about the build order, its tooling or the CI of this repository to this shelf. Run `20261122-0114` wrote intake lines 32 and 34. The product owner ruled both RECORD into this record, which its end-of-run report calls P2. This record checked the script again on `0cdd96298`. The two incidents are as the builders of #1969 and #1971 reported them.

## Obligation

At its end, `tools/hw-cargo` copies `$CARGO_TARGET_DIR/<profile>/headwater` to `engine/target/<profile>/headwater` whenever the command exits 0 and that file exists. It does not ask whether the command built `headwater-cli`. The slot's target directory is shared by every tree that takes the slot, so the binary there can come from another tree.

- In the tree of #1969, `sh tools/hw-cargo test --profile dev-release -p headwater-meta` replaced the tree's binary with one that held an older `pattern.rs`. An end-to-end timing read 8.0 s for a fix that read 0.83 s on a rebuilt binary.
- In the tree of #1971, `test -p headwater-fetch` on slot 1 left a binary without the fix of #1971, and a hand run of it hung.

The run policy records this trap for verifiers only, under the rule that gives each verifier its own slot.

## Discharge

This record discharges when `tools/hw-cargo` copies the binary only after a command that built `headwater-cli` in the calling tree. A change that records why the copy stays also discharges it.
