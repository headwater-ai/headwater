---
id: HW-OBL-0011
title: "Nothing has been promoted out of the synthesized tier"
status: current
status_since: 2026-08-10
waiting_on: measurement
last_verified: 2026-08-15
summary: "Q15 claims that asserted content moves to accepted rather than accumulates, both halves of the instrument now run, and no change of this repository has moved a warrant yet."
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

# Nothing has been promoted out of the synthesized tier

## Context

[Q15](../spec/09-decisions.md#q15--a-synthesized-content-tier) rules that a synthesized tier exists and that its content carries the `asserted` warrant. Asserted content should move to `accepted` rather than accumulate.

## Obligation

The promotion rate against the asserted count is the instrument.

## Discharge

Half of the instrument now runs. `taxonomy audit` reports the warrant of every classified document over the closed set, and it names the `asserted` count as the denominator a rate divides by. This record states no figure for that count. Run the verb and read the warrant section. A number copied into prose is a claim that no run re-derives. What changed is that the population is no longer empty. Every document in it was written on or after 2026-08-14. The measurement of this record before that date was 0, which was true of the corpus it measured.

The numerator now runs, and it runs in another verb. A promotion is a lifecycle transition, from the `asserted` warrant to `accepted`. `warrant.promoted` declares the `needs_prior` input that [spec 12](../spec/12-check-layer.md#temporal-inputs-the-clock-and-the-prior-version) designs, and `headwater check --change` is where a caller supplies one. The count is change-scoped for that reason, so it cannot live in `taxonomy audit`, which reads one working tree.

What this record waits on is a corpus rather than a build. No change of this repository has moved a warrant from `asserted` to `accepted`. So the count has been zero on every run of the instrument. The run set is now the last 120 commits, rather than the changes a person thought to look at. If it stays at zero while the `asserted` population grows, the tier is a place documents go and do not leave. The honest response is to say so.
