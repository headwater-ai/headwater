---
id: HW-OBL-0060
title: "\"The most specific shelf wins\" names no order"
status: current
status_since: 2026-08-12
waiting_on: build
last_verified: 2026-08-13
summary: "Spec 2 rules that the most specific shelf pattern wins, and it never states what makes one pattern more specific."
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
---

# "The most specific shelf wins" names no order

## Context

[Spec 2](../spec/02-taxonomy-model.md#kind-resolution) rules that the most specific shelf pattern wins, and that a tie is a schema-validation error rather than a runtime coin-flip. It never states what makes one pattern more specific than another. It never states which glob constructs a shelf path may hold either.

## Obligation

The engine decided both, and a decision of that class belongs to the meta-schema.

## Discharge

Until the meta-schema takes it, `engine/crates/census/src/pattern.rs` states the order: first the count of leading segments that hold no wildcard, then the count of literal characters.
