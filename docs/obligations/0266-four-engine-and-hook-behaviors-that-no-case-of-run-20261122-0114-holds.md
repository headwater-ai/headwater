---
id: HW-OBL-0266
status: current
status_since: 2026-11-24
summary: "Nothing holds the command-surface Invariants against its tests, and report_duplicates reads the first claim only. Two hook and ratchet cases have no fixture."
last_verified: 2026-11-24
title: "Four engine and hook behaviors that no case of run 20261122-0114 holds"
waiting_on: build
provenance:
  warrant: asserted
---

# Four engine and hook behaviors that no case of run 20261122-0114 holds

## Context

[The register of open obligations](../spec/open-obligations.md) holds a finding that has no reader outside this repository. Run `20261122-0114` wrote intake lines 77, 136 and 144. The product owner ruled each one RECORD into the group that its end-of-run report calls S3, behavior that no case holds. That group held eight lines. A record that bundles gaps is split when it passes five, so this record holds the engine and the hook. Line 144 holds two gaps. This record read each function again on `0cdd96298`.

## Obligation

- Nothing holds the Invariants list of `docs/subsystems/command-surface.md` against the test names it cites. A Rust case that searches for each named `fn` would catch a renamed test. The adjudication of #1999 ruled it scope growth.
- `Index::report_duplicates` in `engine/crates/graph/src/index.rs` compares each claim with the first claim of a value only. When a former claim comes first in path order, two live claims of one value are reported with `former: true`. No reader takes `Defect::Duplicate.former` as the kind of the pair today.
- `.githooks/pre-commit` passes `--tracked-only` to `.githooks/change-manifest`. An engine that refuses the flag falls to the branch with no described change, and no fixture holds that case.
- `ScopeRatchet::ungoverned` in `engine/crates/check/src/scope_ratchet.rs` computes `governed` with `any` over the admitting patterns. A mutant that reads only the first pattern survives, because the fixture lists the governing pattern first.

## Discharge

Each item discharges alone, when a case fails on the defect it names, or when a change records why the behavior needs no case. The record discharges when every item has.
