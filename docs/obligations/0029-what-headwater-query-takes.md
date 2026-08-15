---
id: HW-OBL-0029
title: "What `headwater query` takes"
status: current
status_since: 2026-08-13
last_verified: 2026-08-13
summary: "Spec 6 lists the verb `headwater query <expression>`, and no document states what an expression is."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-engine-architecture
    - HW-SPEC-ai-integration
---

# What `headwater query` takes

## Context

[Spec 6](../spec/06-engine-architecture.md#cli) lists the verb `headwater query <expression>`. No document states what an expression is: not a grammar, not an example, and not the set of things it selects over.

This is the sixth surface that has a required declaration and no stated form.

## Obligation

The corpus owes the grammar of an expression and the set of things it selects over.

## Discharge

The agent surface does not wait on it, because [spec 5](../spec/05-ai-integration.md#agent-surfaces) names six tools in the query class and `query` is not one of them. The engine implements no expression. `headwater query` reports this gap and names the two reads that exist, which is the posture the resolver takes over a `$package` reference.
