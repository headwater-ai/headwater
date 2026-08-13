---
id: OBL-repo-0034
title: "Two rules of `taxonomy validate` name a declaration or an artifact that does not exist"
status: current
status_since: 2026-08-12
last_verified: 2026-08-13
summary: "Context safety wants a size budget that no declaration carries, and mapping integrity wants a taxonomy that nothing fetches."
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
    - SPEC-HW-ai-integration
---

# Two rules of `taxonomy validate` name a declaration or an artifact that does not exist

## Context

Context safety requires "an applicable size budget" on every agent-facing kind or projection. No declaration in the language carries a budget, and none marks a kind as agent-facing.

Mapping integrity requires that a mapping name "kinds and facet values that exist in both taxonomies". Nothing fetches or resolves the other taxonomy, and no section says which component would.

## Obligation

The corpus owes a declaration for a size budget, a way to mark a kind agent-facing, and a component that resolves the other taxonomy of a mapping.

## Discharge

The engine decides the staleness half of the first rule and states that it decides no more. It decides its side of the second and states the same. Both belong to the class of gap that the ten `gap:` markers record, at the level of a rule rather than a form.
