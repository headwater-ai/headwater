---
id: HW-OBL-0078
title: "Two positions that the meta-schema marks a gap are a closed set in the engine"
status: current
status_since: 2026-08-13
last_verified: 2026-08-13
summary: "A voice regime names categories and a language regime names a controlled language, and the language states neither set."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-authoring-and-lifecycle
    - HW-SPEC-taxonomy-model
---

# Two positions that the meta-schema marks a gap are a closed set in the engine

## Context

[`voice_regime.forbid`](../spec/03-authoring-and-lifecycle.md#voice) names categories and states no set of them. `language_regime.controlled` names a controlled language and states no set either. A check needs a pattern set for each category and a rule set for each profile, and [the language](../spec/02-taxonomy-model.md#the-thirteen-declarations) holds neither.

## Obligation

What is open is whether the language should carry the sets, because an adopter who adds a category today waits for a build.

## Discharge

So the engine carries a closed set of both, and an instance that meets a name outside it skips with that name in the reason. The skip reports the gap once per document rather than once per release.
