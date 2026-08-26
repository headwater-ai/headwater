---
id: HW-OBL-0150
status: draft
status_since: 2026-08-26
last_verified: 2026-08-26
title: "A `current` decision declares no required edge to the document it settled, so 22 of 23 reach no specification at all"
summary: "22 of 23 current decisions declare no edge to the specification they settled, and one declares no relation at all."
provenance:
  warrant: accepted
  agency: agent
  drafted_by: claude-sonnet-5
  activity: draft
  accepted_by: j.baxter
  evidence_basis: evidenced
waiting_on: adopter
---

# A `current` decision declares no required edge to the document it settled, so 22 of 23 reach no specification at all

## Context

A count against `main` at the current tip found 33 decision documents, of which 23 carry `status: current`. The taxonomy assigns `decision` the purpose `rationale`, and assigns `specification` and `design_spec` the purpose `behavior`. The base package requires every corpus to carry both purposes. The design depends on a settled decision handing its ruling to the specification layer. `headwater explain` and `related` then walk from one document to the other. Those verbs read only declared relations in front matter, not prose citations. A decision that names its consequence only in Markdown text stays invisible to the graph.

## Obligation

Of the 23 `current` decisions, only `HW-DR-0022` declares a `traces_to` edge reaching a specification. That edge runs one way, and spec 7 declares no reciprocal edge back to the decision. The other 22 declare no relation reaching an `HW-SPEC-*` or `HW-REG-*` target at all. `HW-DR-0010`, the decision that named this project, declares no `relations` key at all. This repository's own `CLAUDE.md` restates that decision's ruling at its top, and the graph still carries zero edges to or from it. Across the 33 decision documents, `traces_to` is used 21 times, nearly always pointing at the evaluation that supports the decision. `governs` is used 9 times, and it points directly at engine source files, skipping the specification layer. Neither direction answers what document now states the rule a decision settled. The taxonomy already has a mechanism for a forced two-way edge, `reciprocal: required` with a named `inverse`. It applies that mechanism to `supersedes` and `discharges`, but not to `traces_to` or `governs`. The corpus owes a ruling on whether a `current` decision must declare at least one edge to the document or code that carries its ruling. It also owes a ruling on whether that edge must be reciprocal.

## Discharge

This closes on a ruling, recorded as a decision, on whether `current` decisions carry the obligation and whether the reach must be two-way. If the ruling requires the edge, discharge also needs the decision-record bundle or the base package to declare its shape. That shape could add `reciprocal: required` to `traces_to`, or declare a new relation scoped from `decision` to `specification` and `design_spec`. It needs `HW-DR-0010` resolved, either with an edge added or with naming stated as the honest exception the case against argues for. It needs `HW-DR-0022` and spec 7 to become a real two-way test case. That means a declared edge on the spec side, not only the Markdown citation that stands there today. Discharge is met when `headwater check --strict` reports zero `current` decisions with no outbound non-evidence edge, or names the obligation that tracks the remaining count.
