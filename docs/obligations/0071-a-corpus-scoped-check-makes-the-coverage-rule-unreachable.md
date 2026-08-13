---
id: OBL-repo-0071
title: "A corpus-scoped check makes the coverage rule unreachable"
status: current
status_since: 2026-08-12
last_verified: 2026-08-13
summary: "A corpus-scoped instance reads every document, so the zero-instance finding can never fire again."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - SPEC-HW-check-layer
    - SPEC-HW-assurance-model
---

# A corpus-scoped check makes the coverage rule unreachable

## Context

[Spec 12](../spec/12-check-layer.md#instances-and-why-coverage-needs-them) offers `Corpus` as a scope. It also rules that a classified document with zero instances is a finding, which is OB-COV-2 of [spec 4](../spec/04-assurance-model.md#no-silent-passes-every-document-is-accounted-for). The two meet badly. A corpus-scoped instance reads every document, so coverage counts every document as checked, and that finding can never fire again.

## Obligation

What is open is the rule for the first real corpus-scoped check. Either such an instance counts toward coverage for no document, or it counts only for the documents that its findings name.

## Discharge

The engine avoids the collision today by keeping the coverage rule as the runner's accounting rather than a check over a view. A rule that instantiates over every typed document reaches the same end by a shorter route. It routes every document to a check before anything is read. The generated Shape checks instantiate per kind for that reason, so a kind that forbids what a rule reads gets no instance.
