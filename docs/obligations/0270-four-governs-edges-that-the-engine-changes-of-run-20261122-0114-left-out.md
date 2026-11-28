---
id: HW-OBL-0270
status: current
status_since: 2026-11-24
summary: "Four documents rule on resolve, relation_view, lock_stale and the shared governed reach test, and declare no governs edge or sentence that names them."
last_verified: 2026-11-24
title: "Four governs edges that the engine changes of run 20261122-0114 left out"
waiting_on: build
provenance:
  warrant: asserted
relations:
  traces_to:
    - HW-DR-0095
---

# Four governs edges that the engine changes of run 20261122-0114 left out

## Context

[The register of open obligations](../spec/open-obligations.md) holds a finding that has no reader outside this repository. Run `20261122-0114` wrote intake lines 105, 131, 158 and 161. The product owner ruled each one RECORD into the group that its end-of-run report calls S6, edges and Invariants that the engine changes left out. That group held eight lines. A record that bundles gaps is split when it passes five, so this record holds the edges. This record read each front matter again on `0cdd96298`.

## Obligation

- HW-DR-0095 now rules on `Source::adopter` in `engine/crates/resolve/src/source.rs`, the `.adopter()` call in `package.rs` and `lib.rs` of the same crate. Its `governs` list names none of the three. Only the glob of `docs/subsystems/taxonomy-resolution.md` reaches them.
- `docs/subsystems/graph-build.md` governs `engine/crates/graph/src/scope.rs`, and does not say that `governed()` is now the one reach test that `taxonomy audit` and `relation.scope.ungoverned` share. `docs/subsystems/taxonomy-distribution-and-audit.md` does not say that the figure and the rule share it.
- `docs/interfaces/headwater-generate.md` states a relation view, and its `governs` names `engine/crates/generate/src/lib.rs` and not `relation_view.rs`.
- `docs/interfaces/headwater-check.md` states `taxonomy.lock.stale` at length, and its `governs` does not name `engine/crates/check/src/lock_stale.rs`. Only `docs/subsystems/checks-and-cache.md` governs that file.

## Discharge

Each item discharges alone, when the document declares the edge or states the missing sentence. A change that records why the edge belongs elsewhere also discharges it. The record discharges when every item has.
