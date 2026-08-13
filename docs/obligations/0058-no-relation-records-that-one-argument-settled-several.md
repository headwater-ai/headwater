---
id: OBL-repo-0058
title: "No relation records that one argument settled several decisions"
status: current
status_since: 2026-08-13
last_verified: 2026-08-13
summary: "The register states twelve times that one evaluation settled several decisions, and no declaration holds that fact."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - SPEC-HW-conceptual-model
---

# No relation records that one argument settled several decisions

## Context

The register states this about itself twelve times, in the form "one evaluation settles it with Q14 and Q7, because the three describe one boundary". That is a fact about the decision set and not a turn of phrase. When [#124](https://github.com/headwater-ai/headwater/issues/124) made each decision a document, the fact had no declaration to land in.

## Obligation

`traces_to` reaches the shared evaluation, so a reader can join two decisions through it. Nothing asserts the pairing, and no check reads it. The three relations the base runs between decisions each mean something else. `constrains` is a governance edge, `conflicts_with` asserts a disagreement, and `overrides` displaces an effect.

## Discharge

So the conversion declared 23 `traces_to` edges and no edge between two decisions. To mint one from the register prose is to reason about what the prose asserts, which [spec 1](../spec/01-conceptual-model.md) forswears. Either the decision-relation vocabulary gains a co-settlement relation, or the join through the evidence is the answer and spec 2 says so.
