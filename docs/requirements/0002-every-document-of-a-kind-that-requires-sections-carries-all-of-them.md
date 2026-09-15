---
id: HW-REQ-0002
status: current
status_since: 2026-09-12
summary: "A document of a kind that declares required sections carries a heading for every one of them, and `section.required.missing` settles it."
last_verified: 2026-09-12
title: "Every document of a kind that requires sections carries all of them"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: draft
  evidence_basis: evidenced
relations:
  verified_by:
    - HW-AC-0002
    - section.required.missing
---

# Every document of a kind that requires sections carries all of them

## Context

A kind declares the sections a document of it must carry. `sections.require` is where a taxonomy states them. The value of that declaration goes to a reader rather than to a writer. A shelf whose documents answer the same headings reads in one pass. A reader who wants the consequences of a decision looks under one heading on every decision record.

A declaration alone does not give a reader that. Two more things do. The first is a rule that reports a document which does not carry the headings. The second is a corpus in which no such document stands. This requirement states the second, so that the corpus property has a document of its own.

This requirement is also the worked instance of an anchor that verifies. Its second verifier is a check rule and not a document. That is the capability [#411](https://github.com/headwater-ai/headwater/issues/411) added.

## Requirement

Every document of a kind that declares required sections carries a heading for each of those sections.

The scope is every document under `docs/` that the census gives a kind. A document that no shelf reaches has no kind. No kind then declares sections for it, so nothing here applies to one.

The requirement is about the heading and never about the prose under it. A heading with nothing under it satisfies this requirement and satisfies no reader. The `Fit criterion` section of [HW-AC-0002](../acceptance-criteria/0002-every-required-section-of-every-governed-document-has-a-heading.md) states that limit.

The requirement holds while the taxonomy holds. A kind that adds a required section makes every document of it non-compliant on the day of the edit. That is the correct reading: the corpus owes the new section.

## Verification

Two artifacts verify this requirement, and they answer two different questions.

`section.required.missing` is the check rule. It reads the headings of a document against the section contract of its kind. It reports an error for each missing one. `.githooks/pre-commit` runs it on every commit, and CI runs it on every pull request.

The entry above points at the rule through the anchor kind `check_rule`. So the edge is in the graph, and a reader of `headwater explain HW-REQ-0002` meets it. That is what re-establishes the property on every change.

[HW-AC-0002](../acceptance-criteria/0002-every-required-section-of-every-governed-document-has-a-heading.md) is the acceptance criterion. It states the fit criterion the rule is measured against, and its method is `test`. A test of the engine holds the rule to its own behavior. A rule that could report nothing would leave this requirement true on paper and unverified in fact.
