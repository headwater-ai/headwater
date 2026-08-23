---
id: HW-OBL-0054
title: "The templates and the fixtures of the first library entry disagree with Q4"
status: current
status_since: 2026-08-12
waiting_on: build
last_verified: 2026-08-14
summary: "Every template of the design-spec entry writes a relation at the top level with a path as the target, which Q4 refused."
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

[Q4](../spec/09-decisions.md#q4--relation-storage) and the [glossary](../spec/glossary.md#relation) both put a relation under a `relations:` block in front matter. Every template and every fixture in the [design-spec entry](../taxonomies/design-spec/templates/) writes the relation name at the top level, with a path as the target.

## Obligation

So the one surface that an author copies teaches the shape that the decision refused. The templates owe a rewrite to the declared shape.

## Discharge

The entry predates neither ruling, which makes this a review gap in the admission criteria. What closes it is an edit to the templates and a criterion that reads them.

**The scaffolder reads no template file, so the defect is a teaching defect and only that.** [Spec 3](../spec/03-authoring-and-lifecycle.md#templates-and-scaffolding) rules that a template is derived from the kind declaration, and `headwater new` derives one. A file under `templates/` reaches an author who opens it and reaches nothing else. So a wrong shape there teaches a wrong shape and produces no wrong document, which is a smaller cost than this record read it as. It is also a cost that no check catches, because a taxonomy package sits outside the corpus root of every corpus it types.
