---
id: OBL-repo-0086
title: "An inline quotation reaches every lexical rule as this author's prose"
status: current
status_since: 2026-08-13
last_verified: 2026-08-13
summary: "The parser marks a block quotation and a code span, and an inline quotation is marked as nothing."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - SPEC-HW-authoring-and-lifecycle
---

# An inline quotation reaches every lexical rule as this author's prose

## Context

[Spec 3](../spec/03-authoring-and-lifecycle.md#what-a-lexical-rule-gets-wrong-and-where-posture-comes-from) rules that "a quotation, a code span, a citation line, and a generated block are outside every voice rule by construction. This is not an exemption that a rule declares."

The parser supplies three of the four. It marks a block quotation as another author's, it marks a code span as code, and `Sentence::authored` drops both. An inline quotation inside a paragraph is marked as nothing, so a rule reads the quoted words as ours.

## Obligation

Two of the four suppressions over this corpus exist for that one reason, and each names `false_positive` rather than an accepted deviation.

## Discharge

The parser can close it, because a quotation mark is in the source and a rule cannot see it. What is open is whether spec 3 means the inline case, and what an unbalanced or a nested quotation mark then does.
