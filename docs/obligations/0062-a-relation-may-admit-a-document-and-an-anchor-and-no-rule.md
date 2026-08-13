---
id: OBL-repo-0062
title: "A relation may admit a document and an anchor, and no rule orders the two"
status: current
status_since: 2026-08-12
last_verified: 2026-08-13
summary: "`traces_to` takes a document or a code path at the target end, and nothing says which resolver wins a string that both claim."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - DR-repo-0004
---

# A relation may admit a document and an anchor, and no rule orders the two

## Context

The base package declares `traces_to` with both `governed_document` and [`code_path`](../spec/01-conceptual-model.md#external-anchor) at the target end. One target string can reach the identifier index and an anchor resolver, and nothing says which answer wins. [Q4](../spec/09-decisions.md#q4--relation-storage) rules that a target is an identifier and leaves the anchor case unordered.

## Obligation

The corpus owes a stated order, so that a target which both resolvers claim has one reading.

## Discharge

The engine reads the index first, then each anchor kind in declaration order. Where two anchor kinds claim one string, it reports a tie and binds neither. [Spec 2](../spec/02-taxonomy-model.md#kind-resolution) sets that precedent for two shelves of equal specificity.
