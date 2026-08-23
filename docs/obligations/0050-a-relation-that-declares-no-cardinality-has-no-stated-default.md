---
id: HW-OBL-0050
title: "A relation that declares no cardinality has no stated default"
status: current
status_since: 2026-08-13
waiting_on: ruling
last_verified: 2026-08-13
summary: "Five relations declare no cardinality, and the whole content of the generated check is the default that nothing states."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-conceptual-model
    - HW-SPEC-taxonomy-model
    - HW-SPEC-check-layer
---

# A relation that declares no cardinality has no stated default

## Context

[Spec 12](../spec/12-check-layer.md#the-five-origins-of-a-check) lists cardinality among the Shape checks that a taxonomy generates. [Spec 1](../spec/01-conceptual-model.md#relation) says a cardinality states how many are legal and if one is required, and it names no default. Three relations in the resolved taxonomy declare `cardinality: many` and five declare nothing, so silence means exactly one, at most one, or any number.

## Obligation

`engine/crates/meta/meta-schema.yml` guesses in a comment that the unwritten default is one, and this corpus refutes that guess. One review prompt names three targets under `applied_in`, which declares no cardinality, and the document is correct. The other reading carries no rule either. Every declared cardinality in this taxonomy is `many`, so a check that read only what a relation declares would generate instances that can never fail. The whole content of the rule is the default.

A second axis is open beside it. A document writes an inverse relation name such as `cited_by`, and nothing says whether a cardinality declared on `cites_evidence` binds its inverse. Twelve documents name more than one target under `cited_by` today.

## Discharge

So [spec 2](../spec/02-taxonomy-model.md) states the default, and it states whether a declaration binds the inverse name. The meta-schema then closes the value set on that statement rather than on the two values this corpus writes.

The rule that waits on it is a count per relation name per document, reported at the entry that passes the limit. It is document-scoped for the reason a participation expectation is: the finding is about a set of edges rather than about any one of them.
