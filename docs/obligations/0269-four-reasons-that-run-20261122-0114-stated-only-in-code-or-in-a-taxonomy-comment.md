---
id: HW-OBL-0269
status: current
status_since: 2026-11-24
summary: "Four reasons live only in module docs and taxonomy comments, from the initial-state warning to the hash-and-compare choice of lock_stale."
last_verified: 2026-11-24
title: "Four reasons that run 20261122-0114 stated only in code or in a taxonomy comment"
waiting_on: build
provenance:
  warrant: asserted
relations:
  traces_to:
    - HW-DR-0086
---

# Four reasons that run 20261122-0114 stated only in code or in a taxonomy comment

## Context

Spec 13 holds a finding that has no reader outside this repository. Run `20261122-0114` wrote intake lines 57, 76, 78 and 162. The product owner ruled each one RECORD into the group that its end-of-run report calls S5, a reason with no decision record. That group held eight lines. A record that bundles gaps is split when it passes five, so this record holds the reasons that live in code. This record searched `docs/decisions/` again on `0cdd96298`.

## Obligation

- A replacing document left at its initial state is reported against itself, advisory and with no patch. HW-DR-0086 says that "a source at its initial state retires nothing yet". The reason lives only in the module doc of `engine/crates/check/src/state_set_after_initial.rs`.
- #1999 (PR #2043) wrote `headwater_api::Session`, `LoadError` and the engine range into `docs/subsystems/command-surface.md`. Three things remain unstated. The page does not say why `LoadError` has six variants for nine messages, or why `main.rs` keeps a once-per-process guard on the engine line. No other subsystem page names headwater-api as a consumer of its library.
- No decision record states why `relation.mention.undeclared` is advisory, at Info, and offers a relation rather than requires one. The `promotion.permanently_advisory` reason of CT-REL-10 in `taxonomy-source/headwater-standard/taxonomy.yml` states why it never blocks.
- #1975 chose to hash every lock source and compare the selection, rather than to run the full resolve on each check. Only the module doc of `lock_stale.rs`, the issue and the adjudication state the choice.

## Discharge

Each item discharges alone, when a decision record or a subsystem spec states the reason. A change that records why the code is the right home also discharges it. The record discharges when every item has.
