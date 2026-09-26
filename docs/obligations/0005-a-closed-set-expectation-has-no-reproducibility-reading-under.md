---
id: HW-OBL-0005
title: "A closed-set expectation has no reproducibility reading under grading"
status: discharged
status_since: 2026-09-16
waiting_on: build
last_verified: 2026-09-16
summary: "Q8 claims that a closed-set expectation makes a probe verdict reproducible, and two runs of the one closed-set probe this corpus declares disagree."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0008
---

# A closed-set expectation has no reproducibility reading under grading

## Context

[Q8](../spec/09-decisions.md#q8--probe-cost-and-cadence) rules that a probe is a document with a declared expectation. A closed-set expectation should make a probe verdict reproducible under grading.

## Obligation

`generate --check` over a result document is the instrument, and this corpus owes a reading of it.

## Discharge

`headwater generate --check` exists and it now holds a result document of this repository. [The result of 2026-09-11](../probe-results/regression-probe-transcript-for-2026-09-11.md) carried four verdicts of four for one day, and one of the four probes declares a closed answer set. A later change moved the lock under that recording, so the first confirmation refuses it and that result carries zero verdicts of four now.

**A second run of the same probe landed, and the instrument has a reading at last.** [The result of 2026-09-16](../probe-results/regression-probe-transcript-for-2026-09-16.md) carries a verdict for [HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer](../probes/a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer.md) too: `not satisfied`. That is because the session ended with no answer in the closed set rather than with the bare word `present` the 2026-09-11 session wrote. The two verdicts disagree, over the same probe and the same grader.

**The disagreement is a reproducibility finding, and it is not evidence that the grader is unsound.** Each session ran once, against a different draft of the same task, and the closed-set derivation is deliberately narrow. It matches only a whole trimmed final answer, never a substring or a markup character. The 2026-09-11 session closed with the bare word. The 2026-09-16 session closed with the same word set in bold Markdown, which the derivation correctly refuses to read as a match. So the disagreement traces to what the two sessions wrote, not to two runs of one grader over one transcript. It says nothing about whether the grader itself reproduces. What it does show is that a closed-set expectation graded this narrowly is not reproducible across sessions. Those sessions differ only in how they format an answer the reader would recognize as the same word. That is the claim Q8 makes, read once: this reading does not support it.
