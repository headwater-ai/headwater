---
id: HW-OBL-0040
title: "Composition between two library entries has no add-only form"
status: discharged
status_since: 2026-09-29
waiting_on: build
last_verified: 2026-09-29
summary: "The resolver lets an entry add into an entry it names in requires, orders the pair, and refuses a selection that lacks the named entry."
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

**The second instance refused the dependency rather than paid it.** The [`brd-prd` entry](../taxonomies/brd-prd/doctrine.md#findings) wants an edge from `prd` to `functional_spec`. The natural spelling adds `prd` to the `relations.realizes` endpoint list that the `standards-spec` entry declared, and that is not an `add`. The alternative is a new relation and `requires: [standards-spec]`. That dependency buys three kinds, a purpose, a facet and two relations. It also buys two shelves, three identifier schemes and three obligations, for one edge. The entry refused it, so the pipeline that both entries describe is declared by neither of them. The third finding at [line 167](https://github.com/headwater-ai/headwater/blob/5fb9518/docs/taxonomies/brd-prd/doctrine.md#L167) holds the argument.

The two other cases are lists rather than addresses. An endpoint list of concrete kinds closes a relation to a later kind, so no kind of the second entry cites evidence at all. A kind's list of required facets closes the same way.

## Discharge

The root is one. Vocabulary that more than one tradition needs cannot live in an entry, because the first entry to claim an address owns it.

**The owner ruled the form on 2026-09-22.** [HW-DR-0095](../decisions/0095-q67-one-library-entry-may-address-the-keys-of-an-entry-it-names-in-requires-and-confluence-holds-over-the-dependency-order.md) records the ruling. One entry may address keys of an entry that it names as a dependency in `requires`, with `add_to` and `add`. No general operation that extends or appends is added. [Spec 2](../spec/02-taxonomy-model.md#customization-by-composition) states the narrowed confluence claim.

**The resolver change landed with [#1246](https://github.com/headwater-ai/headwater/issues/1246), and this record is discharged.** The resolver reads `requires` from each source that a selection chose as a bundle. It applies each bundle after every bundle that the bundle names, so the lock does not depend on the order of the `bundles:` list. Its confluence check admits an `add` or an `add_to` from the dependent entry into a key of an entry that it names directly. It still refuses the same write from an entry that names no dependency, or that reaches the other entry only through a third. A selection that lacks a named entry is refused before the merge where the dependent writes into it, and the refusal names both entries and the address. A cycle in `requires` is refused, and the refusal names each entry in it. The cases `dependent-add-to`, `dependent-add-to-without-requires`, `missing-dependency-add-to`, `missing-dependency-add`, `requires-cycle` and `transitive-requires-grants-no-write` under `engine/crates/resolve/fixtures/cases/` hold each of these.

The library entries that met this wall have not used the form yet. The `diataxis` and `brd-prd` findings that cite this record can now build against it, and a release of the package carries that work.
