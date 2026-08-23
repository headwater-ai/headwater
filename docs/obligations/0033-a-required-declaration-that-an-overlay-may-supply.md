---
id: HW-OBL-0033
title: "A required declaration that an overlay may supply cannot be required of a source"
status: current
status_since: 2026-08-12
waiting_on: ruling
last_verified: 2026-08-13
summary: "The meta-schema has one notion of a required member, and two facet acceptance tests need a second."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0002
---

# A required declaration that an overlay may supply cannot be required of a source

## Context

The meta-schema has one notion of a required member, and it holds a single source to it. Two of the [facet acceptance tests](../spec/02-taxonomy-model.md#facet-acceptance-tests) need a second notion. A facet declares `volatility`, and every value of an enum carries guidance, and an overlay may supply either one at `facets.status.volatility`. So a base package that omits one is not yet wrong, and the resolved taxonomy that omits it is.

The engine runs both under `taxonomy validate` over the result, which is where spec 2 lists them.

## Obligation

What stays open is the facet case and the vocabulary for it. `volatility` and `guidance` are optional members of the meta-schema, because the file has no way to say that a rule over the result requires them. So a reader cannot tell a member that nothing requires from one that `taxonomy validate` requires, and the two sit a line apart.

## Discharge

The two new roots met this question and settled half of it. Every member of an obligation and of a control is required of the source that writes the entry. No source writes a partial entry. The meta-schema reads the value of an overlay operation against the shape at the address it names. The resolver refuses a `remove` of a required member.

The facet case differs because a base declares a facet and an overlay refines it. No adopter completes a publisher's obligation, and an adopter with a different invariant declares one of their own.
