---
id: HW-OBL-0042
title: "A declared `invalid_when` reaches no check, and two live contradictory decisions pass in silence"
status: current
status_since: 2026-08-13
waiting_on: build
last_verified: 2026-08-13
summary: "The base declares `conflicts_with` with `invalid_when`, two specifications promise the check, and no code performs it."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-taxonomy-model
    - HW-SPEC-adjacent-work
---

# A declared `invalid_when` reaches no check, and two live contradictory decisions pass in silence

## Context

The base declares `conflicts_with` with `invalid_when: {both: {status: current}}`. [Spec 2](../spec/02-taxonomy-model.md#the-decision-relation-vocabulary) lists that state first among the four checks that the decision-relation vocabulary brings. [Spec 11](../spec/11-adjacent-work.md) calls it a deterministic, blocking-eligible check that the design held all along. It also records that spec 4 gave the same job to the sampled sweep.

## Obligation

No rule in this engine reads it. The declaration reaches one place in the resolver, where it makes the `status` facet count as read for the relevance canon.

## Discharge

The [fixture corpus](../taxonomies/decision-record/fixtures/README.md) of the second entry plants two `current` decisions that declare the edge against each other, and the run is clean. This is a different class of gap from a form that nothing states. Two specifications promise the check and no code performs it.
