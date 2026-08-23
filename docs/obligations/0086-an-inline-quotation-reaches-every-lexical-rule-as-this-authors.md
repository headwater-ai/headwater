---
id: HW-OBL-0086
title: "An inline quotation reaches every lexical rule as this author's prose"
status: current
status_since: 2026-08-13
waiting_on: ruling
last_verified: 2026-08-15
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
    - HW-SPEC-authoring-and-lifecycle
---

# An inline quotation reaches every lexical rule as this author's prose

## Context

[Spec 3](../spec/03-authoring-and-lifecycle.md#what-a-lexical-rule-gets-wrong-and-where-posture-comes-from) rules that "a quotation, a code span, a citation line, and a generated block are outside every voice rule by construction. This is not an exemption that a rule declares."

The parser supplies three of the four. It marks a block quotation as another author's, it marks a code span as code, and `Sentence::authored` drops both. An inline quotation inside a paragraph is marked as nothing, so a rule reads the quoted words as ours.

## Obligation

Two of the four suppressions over this corpus exist for that one reason, and each names `false_positive` rather than an accepted deviation.

**The size of the reading is measured on one shelf, and it is small there.** [Q27](../spec/09-decisions.md#q27--whether-a-decision-record-is-governed-prose) bound the house language regime to `decision`, which is the kind that quotes prior art most heavily here. Every inline quotation of five characters or more on that shelf was replaced by one word, which is 37 quotations across 26 files. The finding count moved from 64 to 63. Five of the 64 findings fall on a sentence that carries a quotation mark of any kind. So this reading is a real defect of the parser and a small share of one shelf. A count of findings there is close to a count of defects.

**A second shelf gives the same answer, and it was the shelf the reading was expected to dominate.** [Q28](../spec/09-decisions.md#q28--whether-an-evaluation-is-governed-prose) bound the regime to `evaluation`, which quotes an external standard more than a decision record does. The same collapse over `docs/evaluations/` replaced 159 quotations across 15 files, and the count moved from 432 to 417. That is 15 findings in 432, against 1 in 64 on the decision shelf. The reading is larger here and it is still a small share, so a count on either shelf is close to a count of defects. Both measurements were taken by an agent and neither is accepted.

## Discharge

The parser can close it, because a quotation mark is in the source and a rule cannot see it. What is open is whether spec 3 means the inline case, and what an unbalanced or a nested quotation mark then does.
