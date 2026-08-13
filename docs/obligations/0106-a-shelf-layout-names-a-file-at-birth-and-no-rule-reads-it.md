---
id: OBL-repo-0106
title: "A shelf layout names a file at birth and no rule reads it"
status: current
status_since: 2026-08-14
last_verified: 2026-08-14
summary: "A shelf layout is read by the scaffolder and by nothing else, and a rule that read it would report 86 of the 142 identified documents on the three shelves that declare one."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: measure+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - SPEC-HW-authoring-and-lifecycle
    - SPEC-HW-taxonomy-model
---

# A shelf layout names a file at birth and no rule reads it

## Context

`shelves.<name>.layout` is the template that a file name on a shelf comes from. `headwater new` is its only reader in this engine, and no check reads it at all.

Two shelves that number their files declared no layout. So the verb named a record after its title alone, and `headwater new obligation_record --title "A governs edge reaches the path it names and nothing under it"` wrote `docs/obligations/a-governs-edge-reaches-the-path-it-names-and-nothing-under-it.md`. That name sorts under `a` among siblings named `0001-` through `0105-`, and a person renamed both records of [#150](https://github.com/headwater-ai/headwater/issues/150) by hand.

Both shelves now declare `{seq:04d}-{slug}.md`. `{seq}` reads the sequence of the identifier that the run mints. The number in the file name and the number in `OBL-repo-0106` are therefore one value written twice. No facet holds a third copy.

## Obligation

The declaration fixes a name at the moment of birth, and nothing holds the name after that. A file that a person renames answers to no rule, and neither does a file that a person typed by hand.

The measurement over this repository, at the three shelves that declare a layout:

| shelf | layout | identified documents | names the layout renders |
|---|---|---|---|
| `spec_series` | `{sequence:02d}-{slug}.md` | 16 | 0 |
| `obligations` | `{seq:04d}-{slug}.md` | 105 | 56 |
| `decisions` | `{seq:04d}-{slug}.md` | 21 | 0 |

Three causes hold the three rows apart, and each one blocks a rule for a different reason.

`design_spec` requires no facet in the `name` role, so 15 of the 16 spec documents give a check no source for `{slug}`. The sixteenth declares no `sequence` either.

Every one of the 49 obligation records that disagrees carries a name truncated from its title, and no declaration states a width.

Every decision title opens with the token `Q<n>`, which the file name drops. `Q4 — Relation storage` renders `0004-q4-relation-storage.md`, and the file is `0004-relation-storage.md`.

## Discharge

A rule that reads a layout needs three declarations that this taxonomy does not carry. It needs a facet in the `name` role on every kind of a layout-bearing shelf. It needs a stated width for a slug, or the ruling that a name is never truncated. It needs a ruling on the second numbering vocabulary that a decision title carries.

The other answer is the ruling that a layout binds the scaffolder alone. [Spec 3](../spec/03-authoring-and-lifecycle.md#templates-and-scaffolding) states that answer, and a rename after birth is then a fact outside the declaration.

This record holds the measurement under either answer, because a rule of this shape reports 86 of 142 documents today and repairs none of them. [Spec 12](../spec/12-check-layer.md#fixability) puts a rename below the fixability bar, since a rename breaks every inbound link that names the old path.
