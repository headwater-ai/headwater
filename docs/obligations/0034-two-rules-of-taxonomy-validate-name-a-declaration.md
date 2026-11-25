---
id: HW-OBL-0034
title: "Two rules of `taxonomy validate` name a declaration or an artifact that does not exist"
status: current
status_since: 2026-08-12
waiting_on: build
last_verified: 2026-11-25
summary: "Context safety wants a size budget on every agent-facing projection, which an agent_rules can declare and nothing yet requires, and mapping integrity wants a taxonomy that nothing fetches."
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
    - HW-SPEC-ai-integration
---

# Two rules of `taxonomy validate` name a declaration or an artifact that does not exist

## Context

Context safety requires "an applicable size budget" on every agent-facing kind or projection. The `budget` member of an `agent_rules` projection carries one ([HW-DR-0115](../decisions/0115-an-agent-rules-file-is-a-budgeted-pointer-at-the-copilot-instructions-path.md)), and nothing refuses an agent-facing projection without it until [#2088](https://github.com/headwater-ai/headwater/issues/2088). No declaration marks a kind as agent-facing.

Mapping integrity requires that a mapping name "kinds and facet values that exist in both taxonomies". Nothing fetches or resolves the other taxonomy, and no section says which component would.

## Obligation

The corpus owes a declaration for a size budget, and a way to mark a kind agent-facing. It also owes the component that resolves the other taxonomy of a mapping.

## Discharge

The engine decides the staleness half of the first rule and states that it decides no more. It decides its side of the second and states the same. Both belong to the class of gap that the ten `gap:` markers record, at the level of a rule rather than a form.
