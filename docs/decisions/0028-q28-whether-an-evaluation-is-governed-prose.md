---
id: HW-DR-0028
status: draft
status_since: 2026-08-15
summary: An evaluation is governed prose, and the overlay binds the house language regime to it. The shelf carries 432 advisory findings, and the measurement that explains them is whether a rule was reading the prose when it was written rather than who wrote it.
last_verified: 2026-08-15
title: "Q28 — Whether an evaluation is governed prose"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  governs:
    - .headwater/overlay.yml
---

# Q28 — Whether an evaluation is governed prose

## Context

[Q27](../spec/09-decisions.md#q27--whether-a-decision-record-is-governed-prose) bound the house language regime to `decision` and left one arm open. `evaluation` was the arm. An evaluation binds the `narrative` voice regime, and every bound kind binds `declarative`. Q27 adopted that line and said what was wrong with it. It is a line about voice rather than about lexical rules, and nothing states that a narrative document is free of a word limit.

`docs/evaluations/` holds 15 records and 68,266 words of prose. No lexical rule read one word of it. [#247](https://github.com/headwater-ai/headwater/issues/247) measured what a binding would report and counted 432 findings. A count is not a characterization, which is the bar Q27 set for itself, and this record meets it for the shelf Q27 deferred.

## Decision

**An evaluation is governed prose.** `.headwater/overlay.yml` binds `kinds.evaluation.language` to `ste_house` and the lock records it. Four reasons follow, and the first three are measurements.

**First, the 432 are one rule at one severity, and almost all of them are real.** Every finding is `language.controlled.not_met` and every one is advisory. 422 name a sentence past the 25-word limit and 10 name a semicolon in running prose. The four error classes are a contraction, a British spelling, a hard-wrapped block, and a retired term that names a replacement. This shelf produces none of the four, so `headwater check --strict` exits 0 with the binding in place.

The findings fall on running prose rather than on a table. 408 fall on a paragraph, 21 on a list item and 3 on a table cell. None falls on a heading. The sentences are long rather than runaway. The median is 30 words and the longest is 52. 236 of the 422 sit within five words of the limit.

**Second, the quotation reading accounts for 15 findings in 432.** [HW-OBL-0086](../obligations/0086-an-inline-quotation-reaches-every-lexical-rule-as-this-authors.md) records that an inline quotation reaches every lexical rule as the author's prose. An evaluation quotes an external standard more than a decision record does, so the reading had to be priced here rather than inherited. Every inline quotation of five characters or more was replaced by one word, which is 159 quotations across the 15 files. The count moved from 432 to 417. Q27 measured the same experiment on the decision shelf at 64 to 63. The reading is real on both shelves and it is small on both.

**Third, the reading that explains the rate is not authorship, and this shelf is what shows it.** Q27 found 60 of its 64 findings on the five most recent records and read that as an era of authorship. This shelf refutes that reading directly. Every one of the 15 evaluations was drafted by an agent, and 14 of the 15 carry `accepted_by: j.baxter`, so a human read and accepted them. They still carry 432 findings.

The measurement that separates every group in this corpus is whether a lexical rule was reading the prose when it was written. Each row below counts prose words as whitespace-separated tokens outside front matter and outside a fenced code block.

| group | documents | prose words | findings | per 1000 words | was a rule reading it |
|---|---|---|---|---|---|
| `docs/spec/` | 18 | 118,921 | 9 | 0.1 | yes, throughout |
| `docs/obligations/` | 127 | 34,204 | 0 | 0.0 | yes, carried through the conversion |
| `docs/decisions/` 0001–0021 | 21 | 16,161 | 4 | 0.2 | yes, as prose of spec 9 |
| `docs/decisions/` 0027 | 1 | 1,408 | 0 | 0.0 | yes, written under Q27 |
| `docs/decisions/` 0022–0026 | 5 | 5,872 | 60 | 10.2 | no |
| `docs/evaluations/` | 15 | 68,266 | 432 | 6.3 | no |

The four clean groups differ by a factor of 40 from the two dirty ones. Every document in every row was drafted by the same model family. Acceptance does not separate them either. Records 0001 to 0021 are accepted and clean, and 14 of the 15 evaluations are accepted and are not. What separates them is the history of the file. Records 0001 to 0021 were carried out of `docs/spec/09-open-questions.md`, which the editorial pass of 2026-08-09 rewrote under the regime. The obligation records carried their prose off a bound register, and the overlay moved the binding with it. Record 0027 was written after Q27 bound the kind. The two dirty groups were written where no rule was looking.

**So a binding does not clean a shelf and it was never going to.** It stops the next document arriving dirty, and that is the whole of what this line buys.

**Fourth, an evaluation is prose this repository wrote, and the taxonomy already says so.** 14 of the 15 evaluations declare `cited_by`, and 12 are cited by path from `docs/spec/`. No review record and no probe declares `cited_by` at all. Q27 excused `docs/reviews/` as text this repository recorded rather than wrote, and excused a probe as the instrument of a measurement. Neither excuse reaches a document the specification cites as its own evidence.

## Consequences

**What a writer meets.** An evaluation now answers to the six rules that read prose. A hard wrap or a British spelling in one stops a commit. `headwater check --fix` writes the spelling and leaves the wrap, because the source-form rule carries no patch. The word limit and the semicolon stay advisory, and the `ste-editor` skill is the route to prose that clears them.

**The 432 stand, and nothing works them down.** This record promises no cleanup. The corpus has never worked an advisory count down and the count has never fallen on its own. What the binding changes is the 16th evaluation, and record 0027 is the measured case of that effect on the other shelf.

**A reader meets all 523 or none, and that is a ruling rather than a gap.** [Q12](0012-migration-path-for-an-existing-corpus.md) refused a flag that scopes which findings count, on two grounds that survive this change. A flag that decides which findings count makes two runs over one tree disagree, and it turns an unchecked document into an unreported one. [HW-OBL-0080](../obligations/0080-changed-only-is-the-content-addressed-cache-under-another-name.md) records that no binary ever carried one. `headwater check --change` carries a manifest and it reports the whole corpus, which was measured on this branch at 523 findings with one document named. So an advisory count here is read in full, and the number a reader acts on is the error count.

**Why no adoption pair holds any of them, and the reason is measured rather than argued.** The `adoption` block of the lock holds a `(document, rule)` pair under a task with an owner and an expiry. A pending finding does not fail a strict run. The grain of the pair is the whole rule. `language.controlled.not_met` carries the word limit and the semicolon at warning, and the contraction and the British spelling at error. A pair taken out for the word-limit backlog therefore covers the errors too.

That was measured on a scratch copy of this tree. A British spelling was appended to `docs/evaluations/language-choice.md`. With the binding and no pair, the strict run exits 1. With one pair on that document and the same rule, the strict run exits 0 and the error is reported as migration-pending under the task. So the debt mechanism would buy quiet by returning the one thing the binding is for. Q27 declined a pair because a pair would suppress nothing. The sharper reason is that a pair would suppress something.

**What the binding refuses that nothing refused before.** The same British spelling on the unbound tree moved the corpus from 91 findings to 91, and the strict run exited 0. On the bound tree it moved 523 to 524, and the strict run exited 1. The instrument is the injection rather than the count.

**Where `evaluation` sits, and which arms are now empty.** Every kind this regime binds also binds the `declarative` voice regime, except this one. `evaluation` is the first `narrative` kind under a controlled profile, which is the line Q27 adopted and this record retires. Six kinds stay unbound. `docs/reviews/` exempts `review_prompt` and `review_record` by design. The words of a probe are the instrument of a measurement, which covers `probe` and `probe_result`. `probe_transcript` and `specification` hold no document here.

**What reopens this.** Two conditions, and each one is an event.

The first is an evaluation that has to quote at length. 15 findings of the 432 come from an inline quotation, and a record that quotes a standard for a paragraph would move that share. A block quotation reaches no lexical rule, and a directive naming `reason=false_positive` states the exception where a reader meets it. An evaluation that can use neither is the case that reopens this.

The second is the next evaluation. It lands under this binding. If it still carries findings at the rate of the 15 below it, the third reason above is wrong and the binding only reports. Record 0027 is the same test on the decision shelf and it came back at zero.

**What this record does not settle.** Whether an agent may write the acceptance stamp of a document it drafted is [HW-OBL-0108](../obligations/0108-an-agent-writes-the-acceptance-stamp-of-every-document-in-this-corpus.md). Whether `warrant: proposed` is a value of that facet at all is [HW-OBL-0125](../obligations/0125-nine-documents-state-a-warrant-the-closed-set-does-not-hold-and-no-check-reads-one.md). The grain of an adoption pair is a defect of the debt mechanism rather than of this ruling. It is filed rather than repaired here.

**This record is checked prose and unaccepted prose.** Every lexical rule of this repository read it, and no human has.
