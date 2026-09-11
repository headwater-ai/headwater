---
id: HW-OBL-0044
title: "A generated index labels a document with its identifier"
status: discharged
status_since: 2026-09-11
waiting_on: build
last_verified: 2026-09-11
summary: "The index of the specification shelf reads `HW-SPEC-vision-and-scope` where the table it replaced read \"Vision and scope\"."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-taxonomy-model
    - HW-SPEC-authoring-and-lifecycle
---

# A generated index labels a document with its identifier

## Context

A shelf index prints one row for each document, and it needs a word to put in the link. The [taxonomy](../spec/02-taxonomy-model.md#the-thirteen-declarations) declares `summary` and it declares no title. So the emitter prints the [identifier](../spec/03-authoring-and-lifecycle.md#identifiers), and [the index of this shelf](../spec/README.md) reads `HW-SPEC-vision-and-scope` where the table it replaced read "Vision and scope".

The heading inside each file holds the name that a reader wants, and a body scan is the wrong place to take it from. A heading is prose. An author rewrites one without a thought for the index, and the index then moves under a `--check` gate for a reason nobody meant.

## Obligation

Either a kind declares a title facet, or every generated index of documents is a list of identifiers.

## Discharge

**A second entry took the first answer, and it does not reach the shelf that showed the defect.** The [decision-record entry](../taxonomies/decision-record/doctrine.md) declares `facets.title` and requires it on both of its kinds. A log of decisions that reads `HW-DR-0007` is not a log that anybody reads. The facet reaches those kinds and the base `decision`, and it reaches nothing else. The design-spec kinds already declare their required facets as a list, and no `add` writes into a list that exists. So the entry that met the defect cannot repair it where it was measured.

**A generated document had no name, which is the same defect one step further.** The identity block of a projection held two scalars, `id` and `kind`. A document that block wrote declared no facet in the `name` role, whatever its kind required, and [`label`](../../engine/crates/generate/src/lib.rs) then fell through to the identifier. Three rows of this corpus were in that state: `docs/spec/README.md` named `HW-REG-open-questions`, `docs/probe-results/README.md` named `HW-RESULT-regression-probe-transcript-for-2026-09-09`, and `docs/probe-runs/README.md` named `HW-RUN-regression-probe-transcript-for-2026-09-09`. The same three strings reached the navigation sidebar and the browser tab, because MkDocs serves a navigation label as the page title.

The general remedy landed at the meta-schema. `name` is the sixth entry of the closed facet-role registry of [spec 2](../spec/02-taxonomy-model.md#the-meta-schema), and `facets.title` claims it. A projection reads the facet by that role and never by its name.

## Discharge, 2026-09-11

**The clause is met, and this record closes.** The earlier paragraph here said that no design-spec kind declares a facet in the `name` role. That was true when it was written. It is not true now. `design_spec` requires `title` in the lock, and 15 of the 16 rows of `docs/spec/README.md` carried a title and a summary. The record still said that every row was an identifier. The one exception was the generated row, which is the half that stayed open.

`projection_identity` now admits a third scalar, `name`, and the emitter writes it under whichever facet this taxonomy puts in the `name` role. The three declarations that needed one carry one. A case of `headwater-generate` takes the measurement. Zero of the 333 leaf entries of `.headwater/nav.yml` match `HW-<SCHEME>-`, and zero of the 300 rows across the 12 generated shelf-index pages match it. The two counts were 3 and 3 before the change.

One honest limit. The identity block still writes no facet in the `scent` role, so the tombstone's row on `docs/spec/README.md` carries a label and no summary. It is the one row of 16 in that state. The clause of this record is about the name, so the name closes it. [#227](https://github.com/headwater-ai/headwater/issues/227) holds the wider question of what a generated document owes a reader.
