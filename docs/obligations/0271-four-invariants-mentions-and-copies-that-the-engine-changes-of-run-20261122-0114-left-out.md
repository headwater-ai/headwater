---
id: HW-OBL-0271
status: current
status_since: 2026-11-24
summary: "Two subsystem specs name no test for new cases, and a spec names HW-DR-0095 with no edge. The lock fixtures copy a rule of the resolver fixtures."
last_verified: 2026-11-24
title: "Four Invariants, mentions and copies that the engine changes of run 20261122-0114 left out"
waiting_on: build
provenance:
  warrant: asserted
relations:
  traces_to:
    - HW-DR-0095
---

# Four Invariants, mentions and copies that the engine changes of run 20261122-0114 left out

## Context

Spec 13 holds a finding that has no reader outside this repository. Run `20261122-0114` wrote intake lines 92, 93, 106 and 111. The product owner ruled each one RECORD into the group that its end-of-run report calls S6, edges and Invariants that the engine changes left out. That group held eight lines. A record that bundles gaps is split when it passes five, so this record holds the Invariants, the mentions and the copies. This record read each document again on `0cdd96298`.

## Obligation

- The Invariants of `docs/subsystems/checks-and-cache.md` name no test for the neighbor dedup and the read dedup in `engine/crates/check/src/scope.rs`. One bullet could name `a_pair_that_both_ends_declare_is_one_neighbour_at_each_end` and `a_document_reached_by_two_relations_is_read_once`, in `engine/crates/check/tests/neighbourhood_dedup.rs`.
- The Invariants of `docs/subsystems/taxonomy-resolution.md` name neither `many_unknown_front_matter_keys_check_in_linear_time` nor `a_release_of_many_keys_is_refused_in_time_and_leaves_nothing`, in `engine/crates/cli/tests/publish.rs`. `parse-and-census.md` names them for slice 3. No sentence is false.
- `docs/subsystems/taxonomy-resolution.md` names HW-DR-0095 and declares no `traces_to` to it. `relation.mention.undeclared` reports it at Info, and it was on `main` before #2022.
- `engine/crates/lock/tests/fixtures.rs` reads the resolver fixture cases with its own copy of the file-name rule. A case that depends on how a source is marked must be marked in both files.

## Discharge

Each item discharges alone, when the document names the test or declares the edge, or when the copy reads one rule. The record discharges when every item has.
