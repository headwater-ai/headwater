---
id: HW-OBL-0229
status: current
status_since: 2026-10-01
summary: "The evaluation of the 2026-09-30 counterfactual campaign is warrant asserted, so the discharges edge onto HW-OBL-0223 and HW-OBL-0197 waits on a person promoting it."
last_verified: 2026-10-01
title: "The counterfactual campaign evaluation withholds its discharges edge until a person promotes it"
waiting_on: ruling
---

# The counterfactual campaign evaluation withholds its discharges edge until a person promotes it

## Context

#1511 changed the evidence direction of `discharges`, and HW-DR-0103 records the change. The build of #1511 in run `20261001-1107` found that `docs/evaluations/what-the-counterfactual-campaign-of-2026-09-30-measured-by-component.md` still declares no `discharges` edge onto HW-OBL-0223 or HW-OBL-0197, although its *What the documents alone change* rows measure what HW-OBL-0223 asks for. Its `provenance` declares `warrant: asserted` on `d309e914`. Spec 1 says that an asserted document does not discharge an evidence obligation. So the edge, if it is added now, is reported and does not discharge.

## Obligation

A person reads the evaluation and decides whether to promote its warrant. After a promotion, the evaluation declares `discharges` onto HW-OBL-0223 and onto HW-OBL-0197 where its figures answer that obligation. No agent promotes a warrant, so this record waits on a ruling and not on a build.

## Discharge

This record discharges when the evaluation declares the `discharges` edges with a warrant inside the closed set, or when a person records why the evaluation stays asserted.
