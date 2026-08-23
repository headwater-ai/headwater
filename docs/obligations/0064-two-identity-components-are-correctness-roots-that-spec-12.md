---
id: HW-OBL-0064
title: "Two identity components are correctness roots that spec 12 does not cover"
status: current
status_since: 2026-08-12
waiting_on: build
last_verified: 2026-08-13
summary: "The anchor resolver has no normalization fixture set asked of it, and the identifier index is absent from the list of roots."
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

# Two identity components are correctness roots that spec 12 does not cover

## Context

Almost every entry on [the list of correctness roots](../spec/12-check-layer.md#the-correctness-roots) names the instrument that tests it. The census walker ships a fixture tree, the importer ships a fixture set, and the grader ships a corpus of transcripts.

The external-anchor resolver entry states the failure and stops, so nothing asks a resolver for a normalization fixture set. The identifier index is absent from the list, and every edge target resolves through it. An index that binds one identifier to the wrong document gives a correct check result over a wrong graph. No check finds that, which is the same silent pass the importer entry describes.

## Obligation

One sentence in spec 12 makes each fixture set an obligation rather than a habit.

## Discharge

`engine/crates/graph/fixtures/` holds both fixture sets today. They exist because somebody wrote them, and nothing in the specification asks for them.
