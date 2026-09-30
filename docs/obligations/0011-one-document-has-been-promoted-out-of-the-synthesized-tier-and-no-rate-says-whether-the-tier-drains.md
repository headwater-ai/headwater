---
id: HW-OBL-0011
title: "One document has been promoted out of the synthesized tier, and no rate says whether the tier drains"
status: current
status_since: 2026-08-10
waiting_on: measurement
last_verified: 2026-08-15
summary: "Q15 claims that asserted content moves to accepted rather than accumulates. Both halves of the instrument run, one change has promoted one document, and no rate over time has been read."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0015
---

# One document has been promoted out of the synthesized tier, and no rate says whether the tier drains

## Context

[Q15](../spec/09-decisions.md#q15--a-synthesized-content-tier) rules that a synthesized tier exists and that its content carries the `asserted` warrant. Asserted content should move to `accepted` rather than accumulate.

## Obligation

The promotion rate against the asserted count is the instrument.

## Discharge

Half of the instrument now runs. `taxonomy audit` reports the warrant of every classified document over the closed set, and it names the `asserted` count as the denominator a rate divides by. This record states no figure for that count. Run the verb and read the warrant section. A number copied into prose is a claim that no run re-derives. The population is not empty. Every document in it was written on or after 2026-08-14. The measurement of this record before that date was 0, which was true of the corpus it measured.

The numerator now runs, and it runs in another verb. A promotion is a lifecycle transition, from the `asserted` warrant to `accepted`. `warrant.promoted` declares the `needs_prior` input that [spec 12](../spec/12-check-layer.md#temporal-inputs-the-clock-and-the-prior-version) designs, and `headwater check --change` is where a caller supplies one. The count is change-scoped for that reason, so it cannot live in `taxonomy audit`, which reads one working tree.

What this record waits on is a rate rather than a first promotion. [#1425](https://github.com/headwater-ai/headwater/pull/1425), which merged as `40d08a1d`, moved [HW-DR-0078](../decisions/0078-a-recorded-terminal-demonstration-may-show-a-frozen-number-behind-a-recorded-on-date-marker.md) from `asserted` to `accepted`. It is the first promotion in this repository. On 2026-09-30, `headwater check --change` against `40d08a1d^` counted one promotion under `warrant.promoted`, and no other. Before that change, the count was zero on every run of the instrument, over the last 120 commits.

One promotion shows that the act happens. It does not show whether the `asserted` population drains or grows. So this record discharges on a reading over a window of changes. The reading is the promotions in the window, against the `asserted` count at each end of it. If promotions stay rare while the `asserted` population grows, the tier is a place documents go and do not leave. The honest response is to say so.
