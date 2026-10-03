---
id: HW-OBL-0236
status: current
status_since: 2026-10-03
summary: "Run 20261002-1233 raised six questions that no record answers. One asks whether the governs edge of a superseded decision ages. Five ask what acceptance or a moved rule owes."
last_verified: 2026-10-03
title: "Six questions from run 20261002-1233 about when an edge ages and what a moved rule owes"
waiting_on: ruling
provenance:
  warrant: asserted
---

# Six questions from run 20261002-1233 about when an edge ages and what a moved rule owes

## Context

Run `20261002-1233` moved rules out of spec 6, stamped edges by the rule of #1520 and recorded the shadow-mode reading of #927. Its builders and maintainer found six cases that no rule settles. No build closes them until a person rules, so the product owner ruled each one RECORD with `waiting_on: ruling`. Intake lines 25 and 91 were ruled during the run. Lines 87, 107, 113 and 117 were ruled at the top of run `20261003-1026`. This is one of three spec 13 records for that run. Each item was checked on `a46bd95e` by the document and the facet or sentence it names.

## Obligation

**When an edge ages.**

- HW-DR-0083 is at `status: superseded`, and it declares `governs` onto `taxonomy-source/headwater-standard/taxonomy.yml` with a `verified_revision`. Each release of the package edits that file, so each release makes the edge suspect, as release 4.16.0 did. The question is whether the `governs` edge of a superseded document ages at all.

**Which acceptance clears a warning.**

- `docs/evaluations/what-the-shadow-mode-routing-log-showed-at-its-bound.md` and HW-DR-0105 claim `evidence_basis: evidenced` and trace to HW-DR-0064. HW-DR-0064 is at `warrant: asserted`, so `headwater check` warns `warrant.evidence.unsupported` on both. The warning clears when a person accepts HW-DR-0064. The owner ruled on 2026-10-03 that HW-DR-0105 stays a draft until #1670 grades.
- HW-OBL-0028 keeps `traces_to: HW-SPEC-engine-architecture`. Its rule now lives in `docs/subsystems/checks-and-cache.md`, and a move of the edge raised `warrant.evidence.unsupported`, because that page is at `warrant: asserted`. The edge moves once a person accepts the page.

**What a moved rule owes.**

- Slice 4b of #1572 moved rules from spec 6 into `docs/interfaces/headwater-taxonomy.md`, `docs/subsystems/projections-and-export.md` and `docs/spec/12-check-layer.md`. No relation records the move on either side, and no rule says that a move of prose owes a `traces_to` edge.
- HW-DR-0106 states its rule for "a link". Slice 4c also retargeted two `traces_to` edges under it: HW-DR-0035 to HW-IFACE-headwater-taxonomy, and HW-OBL-0110 to HW-SPEC-distribution-and-federation. The question is whether one sentence extends HW-DR-0106 to a declared relation whose only credit moved.
- HW-DR-0092 says that the comments that name an identifier "make no claim that the checker can hold". `engine/crates/cli/tests/spec_six_overview.rs` now holds a claim over the comments that credit spec 6. The question is whether HW-DR-0092 still describes that population of comments.

## Discharge

Each item discharges alone, when a person rules on it and a change applies the ruling, or when a decision record says why the case stays open. The record discharges when every item has. An item that gains a reader outside this repository leaves this record for an issue, and the record says where it went.
