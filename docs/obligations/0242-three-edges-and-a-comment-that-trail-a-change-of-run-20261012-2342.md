---
id: HW-OBL-0242
status: current
status_since: 2026-10-13
summary: "The dashboard and the export contract share no edge, a comment in the route ranking predates the name step, and nine governs edges onto ci.yml owe a re-read."
last_verified: 2026-10-13
title: "Three edges and a comment that trail a change of run 20261012-2342"
waiting_on: build
provenance:
  warrant: asserted
---

# Three edges and a comment that trail a change of run 20261012-2342

## Context

Run `20261012-2342` changed the version read of the coverage dashboard (#1435), the name step of `headwater route` (#1470) and a comment in `.github/workflows/ci.yml` (#1561). The maintainers and verifiers of those changes found documents and edges that now trail the code. None has a reader outside this repository, so the product owner ruled each one RECORD, the first at the fifth-merge pass and the other two at the final pass, both on 2026-10-13. This is one of two spec 13 records for that run. Each item was checked on `56febd34`. The product owner read the `relations` of `docs/interfaces/headwater-export.md` and the files that name `integrations/dashboard/dashboard.py`, the comment that opens "The budget never reaches this set" in `route_through` of `engine/crates/query/src/route.rs`, and the documents that name `.github/workflows/ci.yml`.

## Obligation

- `integrations/dashboard/dashboard.py` implements "Raising the version" of `docs/interfaces/headwater-export.md`, and no `traces_to` or `governs` edge joins them. So `headwater route`, given a description of the version read of the dashboard, does not offer the export contract: it has no edge to follow. One edge pays both halves.
- In `route_through`, the comment that opens "The budget never reaches this set" was written before the name step of HW-DR-0109. It now sits after the line that builds the named set, so "this set" no longer names the anchored set it was written for. The #1470 verifier found it.
- The #1561 change edited a comment in `.github/workflows/ci.yml`. By the count of the #1561 maintainer, that leaves nine `governs` edges onto the file suspect: HW-PD-0013 to HW-PD-0020 and the two-triggers-one-verdict evaluation. The mechanisms they describe did not move. Only a re-verification is owed, and it records a re-read by a person, so the builder did not do it.

## Discharge

Each item discharges alone. The first does when an edge joins the dashboard and the export contract. The second does when the comment names the set it describes. The third does when each of the nine edges is re-verified. The record discharges when every item has.
