---
id: OBL-repo-0044
title: "A generated index labels a document with its identifier"
status: current
status_since: 2026-08-13
last_verified: 2026-08-13
summary: "The index of the specification shelf reads `SPEC-HW-vision-and-scope` where the table it replaced read \"Vision and scope\"."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - SPEC-HW-taxonomy-model
    - SPEC-HW-authoring-and-lifecycle
---

# A generated index labels a document with its identifier

## Context

A shelf index prints one row for each document, and it needs a word to put in the link. The [taxonomy](../spec/02-taxonomy-model.md#the-thirteen-declarations) declares `summary` and it declares no title. So the emitter prints the [identifier](../spec/03-authoring-and-lifecycle.md#identifiers), and [the index of this shelf](../spec/README.md) reads `SPEC-HW-vision-and-scope` where the table it replaced read "Vision and scope".

The heading inside each file holds the name that a reader wants, and a body scan is the wrong place to take it from. A heading is prose. An author rewrites one without a thought for the index, and the index then moves under a `--check` gate for a reason nobody meant.

## Obligation

Either a kind declares a title facet, or every generated index of documents is a list of identifiers.

## Discharge

**A second entry took the first answer, and it does not reach the shelf that showed the defect.** The [decision-record entry](../taxonomies/decision-record/doctrine.md) declares `facets.title` and requires it on both of its kinds. A log of decisions that reads `DR-repo-0007` is not a log that anybody reads. The facet reaches those kinds and the base `decision`, and it reaches nothing else. The design-spec kinds already declare their required facets as a list, and no `add` writes into a list that exists. So the entry that met the defect cannot repair it where it was measured.

**A generated document has no cue at all, which is the same defect one step further.** The identity block of a projection holds two scalars. So such a document declares no facet in the `scent` role, and none in the `name` role. [The index of the specification shelf](../spec/README.md) carries one such row. It names `REG-HW-open-questions` and says nothing else about the redirect map that 136 citations reach.

The general remedy landed at the meta-schema. `name` is the sixth entry of the closed facet-role registry of [spec 2](../spec/02-taxonomy-model.md#the-meta-schema), and `facets.title` claims it. A projection reads the facet by that role and never by its name. What stays open is the shelf that showed the defect. No design-spec kind declares a facet in the `name` role, so the index of the specification shelf still prints an identifier in every row.
