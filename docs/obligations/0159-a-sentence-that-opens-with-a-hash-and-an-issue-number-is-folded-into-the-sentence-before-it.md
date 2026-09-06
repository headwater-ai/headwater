---
id: HW-OBL-0159
status: current
status_since: 2026-09-06
summary: "`opens_a_sentence` admits a digit and not a hash, so a sentence that cites an issue as `#346` is counted with the sentence before it and reported at their combined length."
last_verified: 2026-08-28
title: "A sentence that opens with a hash and an issue number is folded into the sentence before it"
waiting_on: build
provenance:
  warrant: asserted
  agency: mixed
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  governs:
    - engine/crates/doc/src/sentences.rs
---

# A sentence that opens with a hash and an issue number is folded into the sentence before it

## Context

`opens_a_sentence` in `engine/crates/doc/src/sentences.rs` decides whether a terminator ends a sentence. It admits a capital, a digit, and one of `"`, `“`, `(`, `[`, a backtick, `*`, `_` and `§`. It does not admit `#`. The guard exists to stop an abbreviation from splitting one sentence into two, which [Q5](../spec/09-decisions.md#q5--voice-checking-depth) measured as the largest source of false findings.

A hash followed by a number is how this corpus cites an issue in running prose, and the citation often opens a sentence. When it does, the terminator before it ends nothing. Both sentences reach `language.controlled.not_met` as one, and the reported word count is their sum.

**Measured, in a scratch paragraph appended to `docs/spec/06-engine-architecture.md` and then removed.** Two sentences of 15 and 16 words, where the second opens with `#346`, are reported as one sentence of 31 words at column 1. The same pair with the second sentence opening with a word is reported as two sentences and no finding. A code span at the head of a sentence splits correctly, because a backtick is in the set.

## Obligation

The finding is wrong and its remediation is impossible. The rule reports a sentence that is already two sentences, and the fix line says to split a sentence that is already split. An author who follows the fix line rewrites text that met the limit before the edit.

The character set is a literal in one function, and no declaration stands behind it. Nothing states which openers a language regime admits. So an adopting corpus whose house style opens a sentence with an omitted character meets the same fold, with no way to see why. A hash is the case this repository can measure, because this repository cites issues that way. It is not the only character the literal omits.

This is a defect in a shipped rule rather than a gap in a fixture set. [HW-OBL-0153](0153-the-rewritten-sentence-splitter-has-no-dedicated-conformance-fixtures.md) records that the splitter has no data-driven corpus, and a corpus that held this case would have caught it.

## Discharge

Admit `#` in `opens_a_sentence`, with the same reasoning the digit already carries, and add the pair above to the splitter's cases. Then read the corpus for a sentence whose reported length falls once the fold is gone. A finding that this defect created is a sentence somebody may already have rewritten.

Decide at the same time whether the opener set stays a literal or becomes something a regime declares. The second is the wider fix and it is the one an adopting corpus needs. The first is enough for the character this repository writes.
