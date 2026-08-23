---
id: HW-OBL-0106
title: "A shelf layout names a file at birth and no rule reads it"
status: current
status_since: 2026-08-14
waiting_on: ruling
last_verified: 2026-08-15
summary: "A shelf layout is read by the scaffolder and by nothing else, and a rule that read it would report every document that a person renamed or typed by hand."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: measure+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-authoring-and-lifecycle
    - HW-SPEC-taxonomy-model
---

# A shelf layout names a file at birth and no rule reads it

## Context

`shelves.<name>.layout` is the template that a file name on a shelf comes from. `headwater new` is its only reader in this engine, and no check reads it at all.

Two shelves that number their files declared no layout. So the verb named a record after its title alone, and `headwater new obligation_record --title "A governs edge reaches the path it names and nothing under it"` wrote `docs/obligations/a-governs-edge-reaches-the-path-it-names-and-nothing-under-it.md`. That name sorts under `a` among siblings named `0001-` through `0105-`, and a person renamed both records of [#150](https://github.com/headwater-ai/headwater/issues/150) by hand.

Both shelves now declare `{seq:04d}-{slug}.md`. `{seq}` reads the sequence of the identifier that the run mints. The number in the file name and the number in `HW-OBL-0106` are therefore one value written twice. No facet holds a third copy.

## Obligation

The declaration fixes a name at the moment of birth, and nothing holds the name after that. A file that a person renames answers to no rule, and neither does a file that a person typed by hand.

`taxonomy audit` takes the measurement, one row for each shelf that declares a layout. It renders each name again through the function that `headwater new` writes one with, and it reports how many files carry the name that comes back. A shelf that no declaration lets it measure is held apart from the count, and the row states where the absence lives. This record holds no figure, because a figure here is one that nothing re-derives.

Three causes hold the three shelves apart, and each one blocks a rule for a different reason.

No kind on the specification shelf requires a facet in the `name` role. So a check has no source for `{slug}` there, and that shelf is the arm the reading holds apart.

Every obligation record that disagrees carries a name truncated from its title, and no declaration states a width.

A decision title opens with the token `Q<n>`, and a file name that a person typed drops it. `Q4 — Relation storage` renders `0004-q4-relation-storage.md`, and the file is `0004-relation-storage.md`. The one decision that the scaffolder named keeps the token.

## Discharge

A rule that reads a layout needs three declarations that this taxonomy does not carry. It needs a facet in the `name` role on every kind of a layout-bearing shelf. It needs a stated width for a slug, or the ruling that a name is never truncated. It needs a ruling on the second numbering vocabulary that a decision title carries.

The other answer is the ruling that a layout binds the scaffolder alone. [Spec 3](../spec/03-authoring-and-lifecycle.md#templates-and-scaffolding) states that answer, and a rename after birth is then a fact outside the declaration.

The verb takes the measurement under either answer. A rule of this shape reports every document that a person renamed or typed by hand, and it repairs none of them. [Spec 12](../spec/12-check-layer.md#fixability) puts a rename below the fixability bar, since a rename breaks every inbound link that names the old path.
