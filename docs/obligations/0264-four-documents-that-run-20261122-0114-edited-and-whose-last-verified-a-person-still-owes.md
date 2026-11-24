---
id: HW-OBL-0264
status: current
status_since: 2026-11-24
summary: "Four documents that the run edited keep their old last_verified, because a re-read that moves the date is the act of a person."
last_verified: 2026-11-24
title: "Four documents that run 20261122-0114 edited and whose last_verified a person still owes"
waiting_on: ruling
provenance:
  warrant: asserted
relations:
  traces_to:
    - HW-DR-0095
---

# Four documents that run 20261122-0114 edited and whose last_verified a person still owes

## Context

Spec 13 holds a finding that has no reader outside this repository. Run `20261122-0114` wrote intake lines 67, 107 and 159. The product owner ruled each one RECORD into the group of stale text that its end-of-run report calls S2. That group held seventeen lines. A record that bundles gaps is split when it passes five, so this record holds the dates that a person owes. This record checked each date again on `0cdd96298`.

## Obligation

The product owner ruled that `last_verified` on an edited document moves only after a person re-reads it.

- `docs/tutorials/your-first-governed-corpus.md` keeps `last_verified: 2026-10-20` after #1987 edited its item on dates that differ.
- HW-DR-0095 was amended for #2022 and keeps `last_verified: 2026-09-27`.
- `docs/interfaces/headwater-generate.md` keeps 2026-10-02 and `docs/subsystems/projections-and-export.md` keeps 2026-10-01, after #2018 edited both.

## Discharge

Each item discharges alone, when a person re-reads the document and moves its `last_verified`. The record discharges when every item has.
