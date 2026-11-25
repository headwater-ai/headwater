---
id: HW-OBL-0260
status: current
status_since: 2026-11-24
summary: "HW-DR-0112, a subsystem spec and two campaign evaluations claim evidenced on records at warrant asserted. So warrant.evidence.unsupported warns on main."
last_verified: 2026-11-24
title: "Three documents of run 20261122-0114 claim evidence that rests on records nobody accepted"
waiting_on: ruling
provenance:
  warrant: asserted
relations:
  traces_to:
    - HW-DR-0112
    - HW-PD-0026
---

# Three documents of run 20261122-0114 claim evidence that rests on records nobody accepted

## Context

Spec 13 holds a finding that has no reader outside this repository. Run `20261122-0114` wrote intake lines 8, 38 and 51. The product owner ruled each one RECORD into the group that its end-of-run report calls S1. This record checked each front matter again on `0cdd96298`.

## Obligation

`warrant.evidence.unsupported` warns when a document claims `evidence_basis: evidenced` and rests that claim on a document at `warrant: asserted`. Three findings of the run are that warning, and each was on `main` before the change that met it.

- HW-DR-0112 claims `evidenced` and traces to HW-PD-0026, whose warrant is `asserted`.
- `docs/subsystems/taxonomy-distribution-and-audit.md` claims `evidenced` at `warrant: asserted` and rests on an acceptance criterion.
- `docs/evaluations/what-the-additive-campaign-measured-for-the-eleven-actions-headwater-supports.md` carries 17 such warnings, and the sibling evaluation of the tuning round carries the same 17. Each traces to HW-DR-0112 at `warrant: asserted`.

The remedy of each is a person who reads the record it rests on and sets its warrant. No agent sets a warrant to `accepted`.

## Discharge

Each item discharges alone, when a person accepts the record that the claim rests on, or the document lowers its `evidence_basis`. The record discharges when every item has.
