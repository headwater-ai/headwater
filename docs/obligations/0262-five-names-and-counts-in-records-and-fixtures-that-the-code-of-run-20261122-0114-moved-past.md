---
id: HW-OBL-0262
status: current
status_since: 2026-11-24
summary: "Three records cite a removed file, a moved function and old crate counts. A test keeps a line ceiling above main.rs, and a fixture comment names the overlay."
last_verified: 2026-11-24
title: "Five names and counts in records and fixtures that the code of run 20261122-0114 moved past"
waiting_on: build
provenance:
  warrant: asserted
relations:
  traces_to:
    - HW-OBL-0060
    - HW-OBL-0131
    - HW-OBL-0132
    - HW-OBL-0148
    - HW-DR-0023
---

# Five names and counts in records and fixtures that the code of run 20261122-0114 moved past

## Context

[The register of open obligations](../spec/open-obligations.md) holds a finding that has no reader outside this repository. Run `20261122-0114` wrote intake lines 33, 74, 75, 110 and 155. The product owner ruled each one RECORD into the group of stale text that its end-of-run report calls S2. That group held seventeen lines. A record that bundles gaps is split when it passes five, so this record holds the names and counts that the code moved past. This record checked each one again on `0cdd96298`.

## Obligation

- HW-OBL-0060 cites `engine/crates/census/src/pattern.rs`, which is absent from the tree. The pattern language lives in `engine/crates/meta/src/pattern.rs`.
- HW-OBL-0131 and HW-OBL-0132 name `Loaded::runs()`, which PR #1991 moved to `Session::runs` in `engine/crates/api/src/lib.rs`. The frame of HW-OBL-0132, a place where `main.rs` chooses which library value to trust, also moved into `headwater-api`.
- HW-DR-0023 says twenty-two entries for twenty-two crates, and HW-OBL-0148 says 23 crates. `engine/Cargo.toml` lists 28 members.
- `MAIN_RS_LINES` in `engine/crates/cli/tests/verbs.rs` stays 8762, while `main.rs` holds fewer lines. `docs/subsystems/command-surface.md` says that each step lowers it. #2019 and #1968 have both merged, so the next change to `main.rs` lowers it.
- The comment of `engine/crates/check/fixtures/comment-scan-target.taxonomy.yml` says "the verification identifier this repository's overlay declares". Since #2017, the requirement-verification entry declares the scheme, and the overlay writes only its namespace.

## Discharge

Each item discharges alone, when the file it names states the current name or count, or states none. The record discharges when every item has.
