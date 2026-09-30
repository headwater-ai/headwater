---
id: HW-OBL-0005
title: "A closed-set expectation has no reproducibility reading under grading"
status: discharged
status_since: 2026-09-16
waiting_on: build
last_verified: 2026-09-30
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
    - HW-RUN-campaign-of-2026-09-30-campaign-tier-present-arm-sufficiency
    - HW-RUN-campaign-of-2026-09-30-campaign-tier-absent-arm-sufficiency
    - HW-RUN-campaign-of-2026-09-30-documentation-tier-absent-arm-sufficiency
---

# A closed-set expectation has no reproducibility reading under grading

## Context

[Q8](../spec/09-decisions.md#q8--probe-cost-and-cadence) rules that a probe is a document with a declared expectation. A closed-set expectation should make a probe verdict reproducible under grading.

## Obligation

`generate --check` over a result document is the instrument, and this corpus owes a reading of it.

## Discharge

`headwater generate --check` exists and it now holds a result document of this repository. [The result of 2026-09-11](../probe-results/regression-probe-transcript-for-2026-09-11.md) carried four verdicts of four for one day, and one of the four probes declares a closed answer set. A later change moved the lock under that recording, and for a time the first confirmation refused it. Since #1338 (PR #1448), a moved lock marks the verdicts of a result and keeps them. So that result carries four verdicts of four again, graded over a moved read set.

**A second run of the same probe landed, and the instrument has a reading at last.** [The result of 2026-09-16](../probe-results/regression-probe-transcript-for-2026-09-16.md) carries a verdict for [HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer](../probes/a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer.md) too: `not satisfied`. That is because the session ended with no answer in the closed set rather than with the bare word `present` the 2026-09-11 session wrote. The two verdicts disagree, over the same probe and the same grader.

**The disagreement is a reproducibility finding, and it is not evidence that the grader is unsound.** Each session ran once, against a different draft of the same task, and the closed-set derivation is deliberately narrow. It matches only a whole trimmed final answer, never a substring or a markup character. The 2026-09-11 session closed with the bare word. The 2026-09-16 session closed with the same word set in bold Markdown, which the derivation correctly refuses to read as a match. So the disagreement traces to what the two sessions wrote, not to two runs of one grader over one transcript. It says nothing about whether the grader itself reproduces. What it does show concerns sessions that differ only in how they format one answer. A reader would recognize both forms as the same word. A closed-set expectation graded this narrowly is not reproducible across such sessions. That is the claim Q8 makes, read once: this reading does not support it.

**These readings predate the sealed workspace and the expected value (#1229).** Both runs ran in a workspace that kept every probe file, and the grader then passed every value of the closed set. So the disagreement stands as a finding about the answer format. Neither run is evidence about whether a session reaches the right value. The discharge stands, because the obligation was for a reading.

**The sealed batch of 2026-09-30 is the reproducibility reading that the two runs above could not give.** It ran each closed-set probe 30 times in each of three arms, in sealed workspaces, over one grader ([#1384](https://github.com/headwater-ai/headwater/issues/1384)). The status probe gave its expected value in 30 of 30 `campaign` present sessions and 29 of 30 `campaign` absent sessions. The `documentation` absent arm gave it in 12 of 30. The tombstone probe gave `absent` in 29 of 30, 30 of 30 and 30 of 30. Since #1394, the recorder takes a two-token answer in either order, so a difference of word order does not split one answer into two. Within one arm, the sessions agree closely, and the difference between arms is what the arm removed. [The evaluation of that batch](../evaluations/what-the-counterfactual-campaign-of-2026-09-30-measured-by-component.md) states its limits. One of them bears on this record. The present arm answered the status probe from the always-loaded skill description, with no tool call in 30 of 30 sessions. The corpus was not frozen between the two arms' sessions. This record stays discharged.
