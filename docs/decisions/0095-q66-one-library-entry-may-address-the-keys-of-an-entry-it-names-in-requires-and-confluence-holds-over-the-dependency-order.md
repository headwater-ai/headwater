---
id: HW-DR-0095
status: current
status_since: 2026-09-27
summary: "An entry that names another in requires may add into its keys, and every other pair of entries still commutes only over disjoint leaves."
last_verified: 2026-09-27
title: "Q66 — One library entry may address the keys of an entry it names in requires, and confluence holds over the dependency order"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-OBL-0040
    - HW-DR-0040
    - HW-SPEC-taxonomy-model
    - HW-SPEC-distribution-and-federation
---

# Q66 — One library entry may address the keys of an entry it names in requires, and confluence holds over the dependency order

## Context

[HW-OBL-0040](../obligations/0040-composition-between-two-library-entries-has-no-add-only-form.md) records that two library entries have no add-only way to share vocabulary. The first entry to claim an address owns it. A second entry cannot append to a list that the first entry declares. It also cannot add a required facet to a kind of the first entry. The `brd-prd` entry met this with the `relations.realizes` endpoint list of `standards-spec`. The `diataxis` entry met it with `kinds.<k>.facets.require` on the kinds of `design-spec`.

[#523](https://github.com/headwater-ai/headwater/issues/523) measured the optional half of the case. One entry lists a facet as `optional` on the abstract kind of the base. That facet reads over the kinds of another entry, and no new operation is necessary. The [fixtures README of the `diataxis` entry](../taxonomies/diataxis/fixtures/README.md#what-the-composition-did-not-need-and-what-it-could-not-do) records the run. The required half is the operation that HW-OBL-0040 holds, and the demonstration found no way around it.

[#524](https://github.com/headwater-ai/headwater/issues/524) put the question to the owner. The owner ruled on 2026-09-22, in [this comment on the issue](https://github.com/headwater-ai/headwater/issues/524#issuecomment-5770898653). The words of the ruling are these:

> **Ruling, owner, 2026-09-22.** Asked what form composition between two library entries takes, the owner chose: "Declared-dependency form". One entry may address keys of an entry it names as a dependency. The #523 measurement showed the optional half already works with no new operation, and no general extend-appends operation is added. This discharges the question HW-OBL-0040 holds open and gives #377 and #349 a form to build against. <!-- headwater allow=language.controlled.not_met scope=block until=2027-12-31 reason=accepted_deviation note=the ruling of the owner quoted verbatim -->

The issue holds the ruling, and this record points to it. This record adds nothing to the form. It states the form in the terms of the specification and derives what the form does to the confluence claim.

## Decision

**One library entry may address keys of an entry that it names as a dependency in `requires`.** That is the whole form. The dependent entry uses the operations that exist. It appends to a list of its dependency with `add_to`, for example an endpoint list of `relations.realizes` or a list at `kinds.<k>.facets.require`. It adds a key under a mapping of its dependency with `add`. An entry that does not name the other entry in `requires` cannot address its keys.

**No general operation that extends or appends is added.** The ruling excludes it by name. The optional-facet case needs nothing new, and #523 measured that.

**For this one purpose, `requires` is read.** [HW-DR-0040](0040-q40-whether-extends-bundle-requires-and-an-overlay-s-taxonomy-key-are-a-mechanism-or-a-label.md) ruled `requires` a label. This record narrows that ruling and does not supersede it. `requires` becomes the permission that lets one entry write into the keys of another, and nothing more. It does not expand a selection, and the other three keys of Q40 stay labels.

## Consequences

**Confluence is narrowed, and it is not given up.** The owner did not rule on this point. This paragraph derives it from the form. [Spec 2](../spec/02-taxonomy-model.md#customization-by-composition) claimed that every set of overlays gives one result in any legal order. [Spec 7](../spec/07-distribution-and-federation.md#bundles-are-publisher-overlays-in-the-other-direction) claimed that every subset of add-only bundles commutes. A dependent entry can write into a key of its dependency. That write reaches a node that the dependency wrote, so the two operations do not commute. The declared dependency orders the pair, and the dependency applies first. The declared dependencies make a partial order over the selected entries. Confluence holds over every order that respects it. Two entries with no dependency between them still commute only over disjoint leaves, and the resolver still refuses them where their leaves meet.

**The resolver does not read the form yet.** Today the confluence check in `engine/crates/resolve/src/confluence.rs` compares every pair of overlays and reads no `requires` value. An engine change is owed, and HW-OBL-0040 holds it until that change lands. The obligation now waits on a build and not on a ruling.

**This record does not do three things.** It does not change the resolver. It does not fold the `brd-prd` finding that cites HW-OBL-0040 into that entry. It does not decide the kind set of the Diátaxis site entry ([#349](https://github.com/headwater-ai/headwater/issues/349)). Each of these can now build against the form above.
