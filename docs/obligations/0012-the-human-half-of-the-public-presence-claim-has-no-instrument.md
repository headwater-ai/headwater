---
id: HW-OBL-0012
title: "The human half of the public presence claim has no instrument at all"
status: current
status_since: 2026-08-11
waiting_on: ruling
last_verified: 2026-09-30
summary: "Q16 claims that a generated site answers \"is this for me?\", and only the machine half has an instrument."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0016
    - HW-RUN-campaign-of-2026-09-30-campaign-tier-present-arm-discovery
    - HW-RUN-campaign-of-2026-09-30-campaign-tier-absent-arm-discovery
---

# The human half of the public presence claim has no instrument at all

## Context

[Q16](../spec/09-decisions.md#q16--public-presence) settles the public presence of the project. A generated site should let a reader answer "is this for me?" without reading the specification.

## Obligation

The Discovery probe category is the instrument for the machine half.

## Discharge

The human half has no instrument at all, which is worth a statement rather than a silence. No probe category grades what a person understood from a page, and this project has proposed none.

**The machine half has a reading, and it does not separate the arms.** The batch of 2026-09-30 graded the two discovery probes in both arms of the `campaign` tier, 30 sessions of each probe in each arm ([#1384](https://github.com/headwater-ai/headwater/issues/1384)). The present arm satisfied 9 of 60 (15.0%, 8.1% to 26.1%). The absent arm satisfied 5 of 60 (8.3%, 3.6% to 18.1%). The difference is +6.7 points, in a 95% Newcombe interval of -5.3 to +18.7 points ([the result](../probe-results/campaign-of-2026-09-30-campaign-tier-present-arm-discovery.md)). The interval contains zero, so this batch does not show that the governance layer helps a cold session find a document.

**The reading has limits.** 5 of the 60 present-arm sessions and 3 of the 60 absent-arm sessions named a path outside their workspace. The corpus was not frozen between the two arms' sessions. `docs/spec/12-check-layer.md` changed after the recording, so each verdict is graded over a moved read set. [The evaluation of that batch](../evaluations/what-the-counterfactual-campaign-of-2026-09-30-measured-by-component.md) states every limit and the cost. This record stays open by the owner's ruling of 2026-09-30, until [#1472](https://github.com/headwater-ai/headwater/issues/1472) runs a discovery selection sized for power. The human half still has no instrument.
