---
id: HW-OBL-0054
title: "The templates and the fixtures of the first library entry disagree with Q4"
status: current
status_since: 2026-08-12
waiting_on: build
last_verified: 2026-09-07
summary: "The fixture corpus of the first library entry writes a relation at the top level with a path as the target, which Q4 refused."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-glossary
---

# The templates and the fixtures of the first library entry disagree with Q4

## Context

[Q4](../spec/09-decisions.md#q4--relation-storage) and the [glossary](../spec/glossary.md#relation) both put a relation under a `relations:` block in front matter. Both also rule that a target is an identifier and never a path.

**This record said "every template" and it was three of thirteen.** Measured on 2026-09-07, across the whole library and not the design-spec entry alone. Three templates wrote four relation names at the top level: `applied_in` in `review_prompt.md`, `applies` and `assesses` in `review_record.md`, and `cited_by` in `evaluation.md`. Two of those three are design-spec templates, out of the five that entry ships. No template target was a path. Every one of the four was a `{{placeholder}}` string, so the second half of the sentence above was never true of a template.

**The path targets are in the fixture corpus, and that half stands.** Six of the sixty-five files under `docs/taxonomies/*/fixtures/` write seven relation names at the top level, and each of those seven targets is a path. `docs/evaluations/queue-durability.md` writes `cited_by: [docs/spec/09-decisions.md]`, which Q4 refuses twice over.

## Obligation

So the one surface that an author copies taught the shape that the decision refused. **That half is closed.** [#507](https://github.com/headwater-ai/headwater/issues/507) moved the four keys under `relations:`, and `headwater taxonomy publish` now refuses a template that writes a declared relation name at the top level.

What remains is the fixture corpus, and its reader is this repository alone. A published artifact carries `bundle.yml`, `doctrine.md` and `templates/` and nothing else, so no adopter ever opens a fixture.

## Discharge

The entry predates neither ruling, which makes this a review gap in the admission criteria. The template half closed on an edit and a criterion that reads it.

What closes the rest is an edit to those six documents and an identifier for each document they point at. No document under `fixtures/design-spec/corpus/` declares an `id`, so a target that is an identifier needs one minted for each of the eleven. The fixture README also states in prose what a run over that corpus must report. So this is a change to what the corpus exercises and not only to where a key sits. That is why it is not folded into the template edit.

**The scaffolder reads no template file, so the defect is a teaching defect and only that.** [Spec 3](../spec/03-authoring-and-lifecycle.md#templates-and-scaffolding) rules that a template is derived from the kind declaration, and `headwater new` derives one. A file under `templates/` reaches an author who opens it and reaches nothing else. So a wrong shape there teaches a wrong shape and produces no wrong document, which is a smaller cost than this record read it as. It is also a cost that no check catches, because a taxonomy package sits outside the corpus root of every corpus it types.
