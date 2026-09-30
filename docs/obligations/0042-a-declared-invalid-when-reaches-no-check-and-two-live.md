---
id: HW-OBL-0042
title: "A declared `invalid_when` reaches no check, and two live contradictory decisions pass in silence"
status: discharged
status_since: 2026-10-01
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
    - HW-EVAL-adjacent-work
---

# A declared `invalid_when` reaches no check, and two live contradictory decisions pass in silence

## Context

The base declares `conflicts_with` with `invalid_when: {both: {status: current}}`. [Spec 2](../spec/02-taxonomy-model.md#the-decision-relation-vocabulary) lists that state first among the four checks that the decision-relation vocabulary brings. [HW-EVAL-adjacent-work](../evaluations/adjacent-work.md) calls it a deterministic, blocking-eligible check that the design held all along. It also records that spec 4 gave the same job to the sampled sweep.

## Obligation

No rule in this engine reads it. The declaration reaches one place in the resolver, where it makes the `status` facet count as read for the relevance canon.

## Discharge

The [fixture corpus](../taxonomies/decision-record/fixtures/README.md) of the second entry plants two `current` decisions that declare the edge against each other, and the run is clean. This is a different class of gap from a form that nothing states. Two specifications promise the check and no code performs it.

**`relation.pair.invalid` discharges this record ([#1491](https://github.com/headwater-ai/headwater/issues/1491)).** The rule is edge-scoped, and `headwater_check::invalid_pair` implements it. It reads `invalid_when.both` from the declaration of each relation, and it reads no relation name and no state facet. It reports a warning where every facet that the condition names holds its value at both ends. The finding carries no patch, because [Q18](../spec/09-decisions.md#q18--recording-adjudicated-disagreements) gives the remedy to a person who writes a decision that `overrides` or `supersedes` one side. A symmetric relation gives one finding for each entry that an author wrote, on the file that wrote it. So two decisions that each declare the conflict give two findings, and a one-sided entry gives one. `engine/crates/check/tests/conflicting_pair.rs` holds each case. The [fixture corpus](../taxonomies/decision-record/fixtures/README.md) now reports its planted pair.

The new rule reaches no obligation yet, so `headwater check` names it with the rules that cite none. A control that binds it belongs to the package release of [#1492](https://github.com/headwater-ai/headwater/issues/1492).
