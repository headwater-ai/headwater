---
id: HW-OBL-0052
title: "A document has no identifier, and every relation target must be one"
status: current
status_since: 2026-08-12
last_verified: 2026-08-13
summary: "Spec 3 reserves identifiers for five artifacts and a document is none of them, so every adopter mints a scheme before declaring one edge."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0004
---

# A document has no identifier, and every relation target must be one

## Context

[Spec 3](../spec/03-authoring-and-lifecycle.md#identifiers) reserves identifiers for decisions, requirements, acceptance criteria, controls and obligations. A document is none of those, so neither the base package nor the design-spec entry gives one to a document. [Q4](../spec/09-decisions.md#q4--relation-storage) rules that a relation target is an identifier and never a path.

## Obligation

The typing pass minted four schemes in [this repository's overlay](../../.headwater/overlay.yml) before it could declare one edge, and every adopter meets that wall.

## Discharge

Either a document carries an identifier by default, or a target may be a path and the corpus pays for a rename. Nothing in the engine can choose between the two.
