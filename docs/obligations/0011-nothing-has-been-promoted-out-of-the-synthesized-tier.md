---
id: OBL-repo-0011
title: "Nothing has been promoted out of the synthesized tier"
status: current
status_since: 2026-08-10
last_verified: 2026-08-13
summary: "Q15 claims that asserted content moves to accepted rather than accumulates, the denominator of the rate now runs, and the numerator waits on a check that declares `needs_prior`."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - DR-repo-0015
---

# Nothing has been promoted out of the synthesized tier

## Context

[Q15](../spec/09-decisions.md#q15--a-synthesized-content-tier) rules that a synthesized tier exists and that its content carries the `asserted` warrant. Asserted content should move to `accepted` rather than accumulate.

## Obligation

The promotion rate against the asserted count is the instrument.

## Discharge

Half of the instrument now runs. `taxonomy audit` reports the warrant of every classified document over the closed set, and it names the `asserted` count as the denominator a rate divides by. This record states no figure for that count. Run the verb and read the warrant section. A number copied into prose is a claim that no run re-derives. What changed is that the population is no longer empty. Every document in it was written on or after 2026-08-14. The measurement of this record before that date was 0, which was true of the corpus it measured.

The numerator is unbuilt, and it is not zero. A promotion is a lifecycle transition, from the `asserted` warrant to `accepted`. [Spec 12](../spec/12-check-layer.md#temporal-inputs-the-clock-and-the-prior-version) already designs the input one needs. A check that declares `needs_prior` receives the previously committed version of each changed document. No check declares it and nothing implements it. The count is change-scoped for that reason, so it cannot live in `taxonomy audit`, which reads one working tree. This record therefore waits on a build rather than on a population. If the count never falls in a corpus that has one, the tier is a dumping ground and the honest response is to say so.
