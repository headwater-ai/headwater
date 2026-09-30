---
id: HW-OBL-0096
title: "Whether anything is ever promoted out of the asserted tier"
status: current
status_since: 2026-08-10
waiting_on: measurement
last_verified: 2026-08-13
summary: "Q15 leaves open whether asserted content is promoted as fast as it arrives, and one promotion in this repository cannot answer it."
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

# Whether anything is ever promoted out of the asserted tier

## Context

[Q15](../spec/09-decisions.md#q15--a-synthesized-content-tier) rules that a synthesized content tier exists. Whether anything is ever promoted out of it is open.

## Obligation

The corpus owes an observation of promotions against the asserted count.

## Discharge

This entry and [the unmeasured claim under Q15](0011-one-document-has-been-promoted-out-of-the-synthesized-tier-and-no-rate-says-whether-the-tier-drains.md) are one debt recorded in two of the register's lists. That record carries the instrument, which is the promotion rate against the asserted count. The denominator now has a value. `taxonomy audit` reports it under the warrant reading on every run, and this record states no copy of the figure. The numerator runs too. A promotion is a lifecycle transition, and `warrant.promoted` declares the `needs_prior` input that spec 12 designs for one. What neither half answers yet is the claim. One change has moved a warrant, [#1425](https://github.com/headwater-ai/headwater/pull/1425), and one promotion is not a rate.
