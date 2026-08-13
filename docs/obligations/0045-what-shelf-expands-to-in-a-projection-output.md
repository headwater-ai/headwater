---
id: OBL-repo-0045
title: "What `{shelf}` expands to in a projection output"
status: current
status_since: 2026-08-13
last_verified: 2026-08-13
summary: "The base writes `output: \"{shelf}/README.md\"`, and no document states what the placeholder holds."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - SPEC-HW-taxonomy-model
    - SPEC-HW-engine-architecture
---

# What `{shelf}` expands to in a projection output

## Context

The base package writes `output: "{shelf}/README.md"` and no document states what the placeholder holds. It is not the shelf's name. A name is not a path, and `decisions/README.md` puts the index outside the corpus root that holds the shelf.

## Obligation

The engine reads it as the shelf's directory. It takes that directory as the literal prefix of the shelf's `path` glob. The reading is the engine's, and [the specification](../spec/06-engine-architecture.md#projections) does not state it.

## Discharge

It sits beside the open question about which glob constructs a shelf path admits. Both answers come from one string, and only one of them is written down. [Spec 2](../spec/02-taxonomy-model.md#the-thirteen-declarations) is where the statement belongs.
