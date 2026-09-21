---
id: HW-DR-0079
status: current
status_since: 2026-09-21
summary: "A full entry that serves neither core purpose names or extends a core-serving kind, and a facet-only entry ships no templates/ directory."
last_verified: 2026-09-21
title: "Criterion 5 admits naming or extending a core-serving kind, and a facet-only entry ships no templates directory under criterion 2"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-sonnet-5
  activity: measure+draft
  evidence_basis: evidenced
---

# Criterion 5 admits naming or extending a core-serving kind, and a facet-only entry ships no templates directory under criterion 2

## Context

[Criterion 5](../taxonomies/README.md#admission-criteria) asks a full library entry to name which of its kinds serve the `rationale` and `behavior` purposes. Two admitted entries serve neither purpose with a kind of their own. [`decision-record`](../taxonomies/decision-record/doctrine.md#the-core-declared) and [`brd-prd`](../taxonomies/brd-prd/doctrine.md#the-reading-criterion-5-does-not-supply) each state that reading in their own doctrine, without naming the gap in the criterion. A third bundle, [`evidence-and-obligation`](../taxonomies/evidence-and-obligation/doctrine.md#the-core-declared), takes a weaker reading again. It names no base declaration for either purpose. [HW-OBL-0193](../obligations/0193-admission-criterion-5-has-no-reading-for-an-entry-that-serves-neither-core-purpose.md) records that criterion 5 has no stated answer for any of the three.

[Criterion 2](../taxonomies/README.md#what-an-entry-ships) asks an entry to ship one template for every concrete kind it adds. The [`diataxis` entry](../taxonomies/diataxis/doctrine.md#findings) adds no concrete kind, and it ships no `templates/` directory. [HW-OBL-0186](../obligations/0186-a-facet-only-entry-has-no-kind-and-the-entry-anatomy-asks-for-two-things-that-presume-one.md) records that the anatomy names no content for that case.

[#520](https://github.com/headwater-ai/headwater/issues/520) asked the owner to rule on both gaps together. [Spec 2](../spec/02-taxonomy-model.md#the-immutable-core) states the invariant core in resolved-taxonomy language. After resolution, some kind serves the `rationale` purpose, and the check runs against the resolved taxonomy rather than against one overlay's own declarations. Criterion 5, as written, asks more of a single entry than the core it enforces asks of a resolved taxonomy.

## Decision

**Criterion 5 admits two shapes for a full entry that serves neither core purpose with a kind of its own.** The first shape names the base declaration that already serves each of `rationale` and `behavior`. The second shape extends a base kind that already serves the purpose, by writing into that kind's `facets`, rather than leaving the base kind untouched. Neither shape lets an entry invent a kind for the purpose. Criterion 1 already refuses a structure invented for the library. `decision-record` takes the second shape: it writes into `kinds.decision.facets` rather than only naming `kinds.decision`. `brd-prd` takes the first shape: it names the base `decision` and `specification` kinds and adds neither. Both readings ground in spec 2's own resolved-taxonomy language. This decision states the criterion that way, rather than as a concession to the two entries that already read it this way.

`evidence-and-obligation`'s admission text names no base declaration for either purpose. It takes neither of the two shapes above, and its reading is weaker than both. This does not change its admission. Criteria 1 and 4 refuse the entry independently, on grounds this decision does not touch. The library index states plainly that the entry's admission text does not comply with the reworded criterion 5. The entry's compliance does not imply itself.

**Criterion 2 admits `templates/` absent entirely from a facet-only entry that adds no concrete kind.** "One template per concrete kind the entry adds" already yields zero templates for zero added kinds. This reading matches the convention the library index already states for `fixtures/`: no directory ships empty. One sentence is added to criterion 2's anatomy. An entry that extends a base kind it did not add, rather than adding a new one, may ship a template fragment for that kind. This covers `decision-record`'s actual shape, alongside `diataxis`'s absent directory.

`decision-record`, `brd-prd` and `evidence-and-obligation` are the three entries that took a criterion-5 reading before this ruling. `diataxis` is the entry that took the criterion-2 reading.

## Consequences

The [library index](../taxonomies/README.md#admission-criteria) states both readings in the admission criteria themselves. The two deferral paragraphs that named this issue now cite this decision, and each states the ruled reading in its place. No entry's own directory changes. This decision reads entries that already exist against a reworded criterion, and none of the three named entries needs a changed shape.

[HW-OBL-0193](../obligations/0193-admission-criterion-5-has-no-reading-for-an-entry-that-serves-neither-core-purpose.md) discharges. Criterion 5 now states what it asks of an entry that serves neither core purpose, and the reason.

[HW-OBL-0186](../obligations/0186-a-facet-only-entry-has-no-kind-and-the-entry-anatomy-asks-for-two-things-that-presume-one.md) discharges only its criterion-2 half. Its criterion-4 half stays open. A facet-only entry's worked corpus is typed under a kind it borrows from another entry. Whether that counts as a demonstration or a borrowing is a question this decision does not reach.
