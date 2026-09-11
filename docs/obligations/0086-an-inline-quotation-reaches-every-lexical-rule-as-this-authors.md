---
id: HW-OBL-0086
title: "An inline quotation reaches every lexical rule as this author's prose"
status: discharged
status_since: 2026-09-11
waiting_on: build
last_verified: 2026-09-11
summary: "The parse now marks an inline quotation, and an unclosed mark suppresses nothing."
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

**The count in the sentence above is wrong, and the correct one is here.** It says two of the four suppressions over this corpus exist for that reason. One of four does, on `99e1ecae`: the directive at `docs/evaluations/n8n-worked-example.md:427`. The two directives that carry `note=quoting Star and Griesemer` sit inside an indented code block of `CLAUDE.md` and of `AGENTS.md`. They are illustrations, and no run reads them as suppressions. The one real directive names `false_positive` rather than an accepted deviation, which is the right name for it. This change removes it.

**The size of the reading is measured on one shelf, and it is small there.** [Q27](../spec/09-decisions.md#q27--whether-a-decision-record-is-governed-prose) bound the house language regime to `decision`, which is the kind that quotes prior art most heavily here. Every inline quotation of five characters or more on that shelf was replaced by one word, which is 37 quotations across 26 files. The finding count moved from 64 to 63. Five of the 64 findings fall on a sentence that carries a quotation mark of any kind. So this reading is a real defect of the parser and a small share of one shelf. A count of findings there is close to a count of defects.

**A second shelf gives the same answer, and it was the shelf the reading was expected to dominate.** [Q28](../spec/09-decisions.md#q28--whether-an-evaluation-is-governed-prose) bound the regime to `evaluation`, which quotes an external standard more than a decision record does. The same collapse over `docs/evaluations/` replaced 159 quotations across 15 files, and the count moved from 432 to 417. That is 15 findings in 432, against 1 in 64 on the decision shelf. The reading is larger here and it is still a small share, so a count on either shelf is close to a count of defects. Both measurements were taken by an agent and neither is accepted.

## Discharge

**Discharged on 2026-09-11 by #783, in the parse.** A quotation mark is in the source, and a rule cannot see it. So `mark_inline_quotations` in `engine/crates/doc/src/body.rs` splits a run at the marks. It marks what is between them as another author's. No rule in the check layer knows what a quotation mark is. That is the form [spec 3](../spec/03-authoring-and-lifecycle.md#what-a-lexical-rule-gets-wrong-and-where-posture-comes-from) asks for.

**The open half was the edge cases.** The build rules on each with a case rather than with a decision record. Spec 3 answers whether it means the inline case. No document answers the behavior of an unbalanced or a nested mark. These are the rulings, and a test in `mod tests` of that file holds each one:

- A straight `"` pairs in order of appearance and does not nest, because nothing in the source says which of two straight marks opens.
- A curly `“` and `”` pair by direction, and they nest.
- An apostrophe marks nothing. A possessive and a quotation are the same character here. A rule that read it would suppress the rest of every sentence that owns something.
- **An unclosed mark suppresses nothing.** Every run marked since the last unmatched opening returns to this author's. The runs it split go back together, so a stray mark leaves the block as the parse produced it. A mark can never hide prose from a rule.
- A quotation opens in one run and closes in a later run of the same block. A code span and a wrapped source line are both inside it. A code span between the marks stays code. A quotation never crosses a block boundary.

**What moves, measured over this corpus.** `headwater check --root . --format json` reports 197 findings on `99e1ecae` and 195 on the same corpus after the change. `voice.forbidden_construction` goes from 37 to 36. The one finding that goes is `docs/decisions/0021-terminological-succession-and-validity-under-merge.md:32`. `language.controlled.not_met` goes from 153 to 152. The one finding that goes is the contraction under the directive at `docs/evaluations/n8n-worked-example.md:427`. The 152 sentence-length findings hold, because `words_between` counts a quoted run too and this change does not touch it. `warrant.evidence.unsupported` holds at 7.

**One measurement in this record is out of reach, and that is a cost worth naming.** The two collapse experiments above replace every inline quotation with one word to size the reading. Neither is repeatable as written, because the engine reads a quotation by a different rule than the one they measured. An agent took both and nobody accepted either, so nothing rests on them.
