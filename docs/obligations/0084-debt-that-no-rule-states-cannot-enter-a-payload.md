---
id: HW-OBL-0084
title: "Debt that no rule states cannot enter a payload"
status: current
status_since: 2026-08-13
waiting_on: ruling
last_verified: 2026-08-13
summary: "A corpus with no summaries is unsearchable and has nothing to declare, because a payload holds findings alone."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-ai-integration
    - HW-SPEC-distribution-and-federation
---

# Debt that no rule states cannot enter a payload

## Context

A payload holds findings. A [route](../spec/05-ai-integration.md#intent-time-routing) ranks on the facet in the `scent` role, so a document with no summary is reachable by its path and its anchors alone. Where a facet contract requires a summary, the missing one is a finding and a payload can hold it. Where no contract requires one, there is no finding, and the corpus is unsearchable with nothing to declare.

## Obligation

This is the clearest case of debt that is not a rule violation, and [first contact](../spec/07-distribution-and-federation.md#first-contact-adoption-is-a-migration-from-no-taxonomy) creates it. No document says whether the base package should require a summary of every concrete kind.

## Discharge

`headwater infer` reports the count as a separate line for that reason. A ruling that the base requires a summary turns the second case into the first, and nothing else reaches it.
