---
id: HW-OBL-0073
title: "The prior version still owes the key edit that the clock has now made"
status: current
status_since: 2026-08-13
last_verified: 2026-08-13
summary: "The cache key covers the injected clock, and nothing forces whoever adds the prior version to write the matching branch."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-check-layer
---

# The prior version still owes the key edit that the clock has now made

## Context

A [key](../spec/12-check-layer.md#determinism-concretely) covers the in-scope input hashes, the taxonomy lock hash, the check version and the injected values. The fourth component had no instance until a windowed participation expectation read `ctx.now`, and it has one now. The key writes the clock where the scope declares it, so one declaration decides what a view carries and what the key covers.

## Obligation

The prior version is the other injected value spec 12 names, and nothing forces whoever adds it to write the matching branch of the key.

## Discharge

The differential of `--no-cache` cannot catch that omission, because both sides of it hold one value of each injected input. So no test in this engine reaches the defect, and a sentence in spec 12 is what would.
