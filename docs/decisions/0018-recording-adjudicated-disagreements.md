---
id: HW-DR-0018
title: Q18 — Recording adjudicated disagreements
status: current
status_since: 2026-08-11
last_verified: 2026-08-11
summary: The edge does not become a node. An adjudication is a decision document that carries `overrides`.
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-EVAL-warrant-and-adjudication
---

# Q18 — Recording adjudicated disagreements

## Context

One [evaluation](../evaluations/warrant-and-adjudication.md) settles this with [Q15](0015-a-synthesized-content-tier.md) and [Q19](0019-inbound-integration-an-external-system-of-record.md), because all three are about who vouched for what.

[Q4](0004-relation-storage.md) handed this decision one question to decide, and the answer is no. The edge does not become a node. An adjudication that carries what the open-question entry asks for — a named adjudicator, a date, a scope, and the reason that [Q21](0021-terminological-succession-and-validity-under-merge.md) adds — is a document. The corpus has documents, and it gives them shelves, kinds, lifecycles, identifiers, and `accepted_by`. An edge promoted to a node is a second and weaker copy of all of that. No shelf holds it, and no acceptance binds it. [Spec 1](../spec/01-conceptual-model.md#facet) removed reference-valued facets on that argument, and Q4 cut the annotated prose link on it.

**The three listed options each fail on one sentence.** A resolution on the `conflicts_with` edge cannot name a resolvable adjudicator, which is the entry's own test. A correction or succession of the loser destroys the record, because the loser was not wrong. A scoped-precedence declaration regrows the authority rank, which the entry already suspected.

## Decision

An adjudication is a decision record, and it carries `overrides` against the document whose effect it displaces. [Spec 2](../spec/02-taxonomy-model.md#the-decision-relation-vocabulary) has listed `overrides` in the Kruchten set from the start, with the claim "displaces a prior decision's effect and does not retire it". That is an adjudication, defined before this question was asked.

## Consequences

Everything the open-question entry wanted then follows from rulings that exist.

- **The adjudicator is named and resolved.** The adjudicating document carries `accepted_by`, and its [warrant](../spec/01-conceptual-model.md#warrant) is `accepted`. No attribute holds an unresolved name.
- **Readers and agents inherit it.** `overrides` is in the succession family, and [reading precedence](../spec/02-taxonomy-model.md#reading-precedence-is-derived) says that the successor governs. Routing already offers a successor first ([spec 5](../spec/05-ai-integration.md#intent-time-routing)).
- **Scope comes from the endpoints and the prose.** A scoped precedence is one document with one edge, which is the thing that a scalar rank could never express.
- **The adjudication can be wrong, and then a later document supersedes it.** An attribute pair has no lifecycle.

**One consequence, and it lands on the package rather than on the base.** `overrides` declares an inverse with required reciprocity, in the way that `supersedes` does. Without it, a reader who arrives at the losing document learns nothing, and `check --fix` has no back-link to write. The package defines all twelve relations of [this vocabulary](../spec/02-taxonomy-model.md#the-decision-relation-vocabulary), the base enables five, and `overrides` is not one of them. An adopter who records an adjudication enables it by reference in one line. The relation runs between decisions, and the base already carries that kind, so the adopter needs no bundle.

**The prior art argued the other way first** ([HW-EVAL-adjacent-work §P](../evaluations/adjacent-work.md#p--provenance-endorsement-and-the-record-of-a-judgment)). Legal citators put a treatment signal on the citing relationship, which is the edge, and they have done so for over a century. Two facts turn it around. The flag is derived from a published opinion, so the opinion is the record and the flag is a projection. And two citators over the same case law disagree at a measured rate. Retraction practice lands where this ruling lands: a separate citable object that points at a work which stays in place.
