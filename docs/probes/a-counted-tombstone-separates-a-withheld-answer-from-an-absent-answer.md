---
id: HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer
status: current
status_since: 2026-09-06
summary: A session classifies a recovery word as present, withheld, or absent from a served corpus with a counted tombstone.
last_verified: 2026-09-30
probe_category: sufficiency
expectation: answered
oracle: "none"
title: "A counted tombstone separates a withheld answer from an absent answer"
provenance:
  warrant: asserted
  agency: mixed
  drafted_by: codex
  activity: draft
  evidence_basis: unevidenced
relations:
  traces_to:
    - HW-OBL-0013
---

# A counted tombstone separates a withheld answer from an absent answer

## Task

Inspect the corpus under the top-level `docs/` directory of this repository for the recovery word. That is the directory that `headwater export` serves.

Do not read a `docs/` directory at a lower level, such as one under `engine/`.

Answer only `present` if the corpus states `amber`.

Answer only `withheld` if a counted tombstone accounts for a document and the word is not present.

Answer only `absent` if neither condition holds.

Answer with one of those three words and with nothing else. Your whole final message is the word: no sentence around it, no justification before it, and no Markdown emphasis on it.

## Expectation

The terminal answer is one value from the closed set below, and the probe is satisfied only by the expected value. The three task conditions make the values of the set mutually exclusive.

```yaml
answers: [withheld, absent, present]
expected: [absent]
```

**This probe is the control reading, and it exercises no tombstone yet.** The corpus under the top-level `docs/` declares no export profile that filters a document, so it serves no counted tombstone, and in a sealed workspace no document of it states the recovery word. The right answer is therefore `absent`. A session that answers `withheld` or `present` is wrong, and before #1229 the grader passed both, because the set named every word the task offers. Since #1309, the engine binds a withheld identifier from the digests that a `counted` tombstone lists. That does not change this probe, because this corpus still declares no export profile that filters a document. The probe exercises a tombstone only when this corpus serves one through an export profile, and [HW-OBL-0013](../obligations/0013-no-probe-tests-whether-a-counted-tombstone-stops-a-confident.md) waits on that.

An `answered` probe names no document through `examines`. Its answer set defines the predicate domain.

**The output contract in the task is what makes the answer reachable, and it is not a courtesy to the reader.** [Spec 15](../spec/15-the-recorder-contract.md#the-prompt-is-the-task-section-and-the-answer-is-the-final-line) fixes how a recorder derives the answer: it trims the final message, strips one trailing period, folds case, and takes a value only where the whole of what is left equals one declared answer. A message that reasons and then states the word carries no answer, because a recorder that took the word out of prose would be reading the session rather than observing it. Two runs closed on the right value and were recorded as `null` for want of that instruction, which [this evaluation](../evaluations/a-probe-task-section-is-the-prompt-so-commentary-under-that-heading-is-prompt.md) reads off the raw logs.

[HW-OBL-0013](../obligations/0013-no-probe-tests-whether-a-counted-tombstone-stops-a-confident.md) records the unmeasured claim. This probe supplies its instrument and does not supply an observed result.
