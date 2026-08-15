---
id: HW-OBL-0067
title: "A rule that reads the taxonomy has no grain in spec 12's list"
status: current
status_since: 2026-08-13
last_verified: 2026-08-13
summary: "Two register rules are about the taxonomy and about no document, and spec 12 draws every scope over the corpus."
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

# A rule that reads the taxonomy has no grain in spec 12's list

## Context

[Spec 12](../spec/12-check-layer.md#scope--the-declaration-everything-else-rests-on) draws every scope over the corpus, and it names five of them. The register makes two findings that no document is about. An obligation with no disposition and a control whose mechanism the engine does not implement are both defects of the taxonomy. Neither one has a document to point at, and neither creates a check instance, so neither accounts anything against the census.

## Obligation

Either spec 12 names the fifth grain, or these two rules move out of the check layer and into `taxonomy validate`.

## Discharge

The engine states the grain as `Taxonomy` and reports it beside the other four. A corpus-grained verdict moves when a document moves, and a taxonomy-grained one moves when the lock moves. To fold the second into the first would put a document in a read set that no document was read for.
