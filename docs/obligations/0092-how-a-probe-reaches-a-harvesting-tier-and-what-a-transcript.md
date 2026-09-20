---
id: HW-OBL-0092
title: "How a probe reaches a harvesting tier, and what a transcript costs to keep"
status: discharged
status_since: 2026-09-20
waiting_on: build
last_verified: 2026-09-20
summary: "Q8 leaves the route from a probe to a harvesting tier, the retention policy for transcripts, and who pays for a campaign."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0008
    - HW-DR-0011
---

# How a probe reaches a harvesting tier, and what a transcript costs to keep

## Context

[Q8](../spec/09-decisions.md#q8--probe-cost-and-cadence) rules that cadence follows the purpose of the run, and it leaves three things open.

## Obligation

How a probe reaches a harvesting tier, which has no tier to try it on. The retention policy for transcripts, whose size nobody has measured. Whether any adopter ever pays for a campaign.

## Discharge

The third is a business-model question, and [Q11](../spec/09-decisions.md#q11--license-and-distribution-posture) places it rather than answers it. The first two wait on a tier and on a measurement, and neither exists.

## Discharge, 2026-09-20

**All three threads are now answered, and this record closes.** [#165](https://github.com/headwater-ai/headwater/issues/165) shipped the route from a probe to a tier: two tiers exist in `.headwater/probe.yml`, and `--tier` selects one. [#515](https://github.com/headwater-ai/headwater/issues/515) settled the retention policy for a probe transcript. [Spec 15](../spec/15-the-recorder-contract.md#how-long-a-result-stays-citable-and-why-that-period-is-not-a-number-this-schema-produces) states it: the transcript and the probes never expire, and nothing in that part sets the window in which a reader can still build the named grader. The third was already answered when this record was drafted, by the reference to Q11 two paragraphs above.

Neither #165 nor #515 was [#87](https://github.com/headwater-ai/headwater/issues/87)'s to close. #87 asked a narrower question this record never named: whether a budget prices only a probe run, or every mechanism that reaches a model. [HW-DR-0076](../decisions/0076-a-probe-budget-prices-a-run-with-a-pinned-model-and-a-committed-transcript-and-a-sweep-has-neither.md) answers it. It rules that the coherence sweep sits outside the budget mechanism this record's route thread already described.
