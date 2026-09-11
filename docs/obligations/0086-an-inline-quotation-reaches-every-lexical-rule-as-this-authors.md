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

**The count this record used to carry was wrong, and it is corrected here rather than inherited.** It read "two of the four suppressions over this corpus exist for that one reason". One of four did, on `99e1ecae`: the directive at `docs/evaluations/n8n-worked-example.md:427`. The two directives that carry `note=quoting Star and Griesemer` are illustrations inside an indented code block of `CLAUDE.md` and `AGENTS.md`, and no run ever read them as suppressions. That one directive named `false_positive` rather than an accepted deviation, which was the correct name for it, and this change removes it.

**The size of the reading is measured on one shelf, and it is small there.** [Q27](../spec/09-decisions.md#q27--whether-a-decision-record-is-governed-prose) bound the house language regime to `decision`, which is the kind that quotes prior art most heavily here. Every inline quotation of five characters or more on that shelf was replaced by one word, which is 37 quotations across 26 files. The finding count moved from 64 to 63. Five of the 64 findings fall on a sentence that carries a quotation mark of any kind. So this reading is a real defect of the parser and a small share of one shelf. A count of findings there is close to a count of defects.

**A second shelf gives the same answer, and it was the shelf the reading was expected to dominate.** [Q28](../spec/09-decisions.md#q28--whether-an-evaluation-is-governed-prose) bound the regime to `evaluation`, which quotes an external standard more than a decision record does. The same collapse over `docs/evaluations/` replaced 159 quotations across 15 files, and the count moved from 432 to 417. That is 15 findings in 432, against 1 in 64 on the decision shelf. The reading is larger here and it is still a small share, so a count on either shelf is close to a count of defects. Both measurements were taken by an agent and neither is accepted.

## Discharge

**Discharged on 2026-09-11 by #783, in the parse.** A quotation mark is in the source and a rule cannot see it, so `mark_inline_quotations` in `engine/crates/doc/src/body.rs` splits a run at the marks and marks what is between them as another author's. No rule in the check layer learns what a quotation mark is, which is the form [spec 3](../spec/03-authoring-and-lifecycle.md#what-a-lexical-rule-gets-wrong-and-where-posture-comes-from) asks for when it says "not an exemption that a rule declares".

**The open half was the edge cases, and the build ruled on each with a case rather than with a decision record.** Spec 3 already answered whether it means the inline case. What no document answered was the behavior of an unbalanced or a nested mark, and these are the rulings, each held by a test in `mod tests` of that file:

- A straight `"` pairs in order of appearance and does not nest, because nothing in the source says which of two straight marks opens.
- A curly `“` and `”` pair by direction, and they nest.
- An apostrophe marks nothing. A possessive and a quotation are the same character here, so a rule that read it would suppress the rest of every sentence that owns something.
- **An unclosed mark suppresses nothing.** Every run marked since the last unmatched opening returns to this author's, and the runs it split are put back together, so a stray mark leaves the block as the parse produced it. This is the conservative direction: a mark can never hide prose from a rule.
- A quotation opens in one run and closes in a later run of the same block, so a code span and a wrapped source line are both inside it. A code span between the marks stays code. A quotation never crosses a block boundary.

**What moved, measured over this corpus.** `headwater check --root . --format json` on `99e1ecae` reported 197 findings and reports 195 after the change. `voice.forbidden_construction` moved from 37 to 36, and the finding that went is `docs/decisions/0021-terminological-succession-and-validity-under-merge.md:32`. `language.controlled.not_met` moved from 153 to 152, and the finding that went is the contraction the directive at `docs/evaluations/n8n-worked-example.md:427` suppressed. The 152 sentence-length findings did not move, because `words_between` counts a quoted run as well and this change does not touch it. `warrant.evidence.unsupported` stayed at 7.

**One measurement in this record is now unreachable, and that is a cost worth naming.** The two collapse experiments above replaced every inline quotation with one word to size the reading. Neither can be repeated as written, because the engine no longer reads a quotation the way they measured. Both were taken by an agent and neither was accepted, so nothing rests on them.
