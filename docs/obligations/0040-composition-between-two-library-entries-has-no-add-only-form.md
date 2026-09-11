---
id: HW-OBL-0040
title: "Composition between two library entries has no add-only form"
status: current
status_since: 2026-08-13
waiting_on: ruling
last_verified: 2026-09-11
summary: "The first library entry to claim an address owns it, so a second entry cannot reuse the vocabulary that both traditions need."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0003
---

# Composition between two library entries has no add-only form

## Context

The [decision-record entry](../taxonomies/decision-record/doctrine.md) is the second entry in the library, and it met this three times in one bundle. Its `obligation_record` kind serves the `obligation` purpose, and the design-spec entry declares that purpose. The admission criteria refuse a second declaration at one address, and the resolver refuses it too, because both operations write the same leaves.

**The wall is narrower than this record first stated, and a measurement narrowed it.** The [`diataxis` entry](../taxonomies/diataxis/doctrine.md#findings) measured the relevance canon at [line 130](https://github.com/headwater-ai/headwater/blob/5fb9518/docs/taxonomies/diataxis/doctrine.md#L130) of its doctrine. The resolver reads `require` and `optional` together when it decides the facets of a kind. An `optional` listing therefore satisfies the canon. A facet-only entry is admissible without any reach into another entry's `kinds.<k>.facets.require`. [#6](https://github.com/headwater-ai/headwater/issues/6) stated that such an entry had to reach that address, and it does not. What stands is the operation itself: a required facet on a kind that another entry declared still has no add-only form. What falls is the claim that every facet-only entry needs the operation.

## Obligation

The remedies are a rename, which puts two names on one reader intent, or a dependency. The entry declares `requires: [design-spec]` for one line, and a team that keeps only decision records inherits six kinds of a tradition it does not use.

The two other cases are lists rather than addresses. An endpoint list of concrete kinds closes a relation to a later kind, so no kind of the second entry cites evidence at all. A kind's list of required facets closes the same way.

## Discharge

The root is one. Vocabulary that more than one tradition needs cannot live in an entry, because the first entry to claim an address owns it. Either shared vocabulary belongs to the base, or the resolver admits two identical declarations at one address and the confluence proof takes a stated exception. [Spec 2](../spec/02-taxonomy-model.md#customization-by-composition) has to choose, and [Q3](../spec/09-decisions.md#q3--how-much-of-the-default-taxonomy-ships-in-the-box) is where the reasoning sits.
