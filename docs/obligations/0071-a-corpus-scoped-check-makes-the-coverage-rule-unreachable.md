---
id: HW-OBL-0071
title: "A corpus-scoped check makes the coverage rule unreachable"
status: current
status_since: 2026-08-12
waiting_on: build
last_verified: 2026-08-14
summary: "A corpus-scoped instance reads every document, and coverage counts routing rather than reading."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-check-layer
    - HW-SPEC-assurance-model
---

# A corpus-scoped check makes the coverage rule unreachable

## Context

[Spec 12](../spec/12-check-layer.md#instances-and-why-coverage-needs-them) offers `Corpus` as a scope. It also rules that a classified document with zero instances is a finding, which is OB-COV-2 of [spec 4](../spec/04-assurance-model.md#no-silent-passes-every-document-is-accounted-for). The two meet badly. A corpus-scoped instance reads every document, so coverage counts every document as checked, and that finding can never fire again.

## Obligation

What is open is the rule for the first real corpus-scoped check. Either such an instance counts toward coverage for no document, or it counts only for the documents that its findings name.

## Discharge

The first of the two options holds. A corpus-scoped instance counts toward coverage for no document.

Coverage counts routing, and routing is the generation step: a template, a declaration, and one instance per target. A document-scoped instance covers its document, and an edge-scoped one covers both endpoints. A corpus-scoped instance has one target, and that target is the corpus, so it accounts against no document whatever it read. `Grain::routes` in `engine/crates/check/src/scope.rs` carries the ruling. The match over the five grains is exhaustive, so a sixth grain answers this question rather than inherits an answer.

[Spec 12](../spec/12-check-layer.md#instances-and-why-coverage-needs-them) states it, and one test holds it. `identifier.claimed_twice` is the first corpus-scoped check, it reads `check/spec/03-no-instance.md`, and the coverage finding against that file survives.

The second option is refused. An instance that counted only for the documents its findings name makes coverage a function of the verdict. A rule that finds nothing then covers nothing, and a corpus with a defect reports higher coverage than a clean one.

One cost stands. A rule that instantiates over every typed document reaches the unreachable end by a shorter route. Such a rule routes every document to a check before anything is read. The generated Shape checks instantiate per kind for that reason, so a kind that forbids what a rule reads gets no instance.

**What happens next is the closure of this record, which is why it reads `build`.** The question is answered, `Grain::routes` carries the answer, spec 12 states it and one test holds it. Nothing further is owed by the engine or by the specification. What remains is the state movement of this record to `discharged`, and that is the owner's to take rather than an agent's.
