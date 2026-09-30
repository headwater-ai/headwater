---
id: HW-OBL-0197
status: discharged
status_since: 2026-09-30
summary: "A supersession says that nobody may rely on the document that lost, and every mechanism here acts on a relation rather than on a reading."
last_verified: 2026-09-30
title: "A superseded document claims no reliance, and nothing measures whether a session honors that"
waiting_on: measurement
provenance:
  warrant: asserted
  agency: mixed
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: reconstructed
relations:
  traces_to:
    - HW-DR-0026
    - HW-DR-0062
    - HW-RUN-campaign-of-2026-09-30-campaign-tier-present-arm-navigability
    - HW-RUN-campaign-of-2026-09-30-campaign-tier-absent-arm-navigability
---

# A superseded document claims no reliance, and nothing measures whether a session honors that

## Context

[HW-DR-0062](../decisions/0062-a-refused-recording-is-held-by-the-reliance-its-state-claims-and-not-by-promotion.md) rules that the reliance a document claims is what decides the cost of a refusal. A state whose role is `live` claims reliance, and a run that meets a refused recording in such a state fails. A state that claims none leaves the run green. So this corpus already treats reliance as a declared fact rather than as a habit of a reader.

Two mechanisms act on that fact and both act on a relation. `lifecycle.dependency.on_terminal` refuses an edge whose target stands at a terminal state, and [HW-DR-0026](../decisions/0026-q26-whether-terminality-belongs-to-a-state-or-to-a-state-and-a-regime.md) fixes where a rule reads terminality from. A `supersedes` edge writes `superseded_by` on the document that lost.

**The reading below is reconstructed from the tree rather than taken by an instrument.** The count comes from the links under `docs/` at the commit named. The two rulings it rests on stand at `asserted`, so no reader has accepted either. A later reader repeats the count rather than trusting it.

**Neither mechanism reaches a reading.** `docs/spec/09-open-questions.md` stands superseded by the decision register, and it keeps a heading for every decision, so an old citation of a heading still resolves. On 2026-09-11, at commit `d6ac7c93`, 133 links under `docs/` name an anchor in that file. A session that follows one arrives at a document this corpus says nobody may rely on. It reads a heading there and reports an answer that no rule of this engine can mark as stale.

## Obligation

A paired probe run is the instrument, over a task whose current answer both registers carry.

What it measures is avoidance rather than correctness: whether the pointers a session is offered steer it around the superseded document at all. [HW-OBL-0014](0014-no-probe-tests-whether-an-agent-reaches-the-adjudication-from.md) owns the other half, which is recovery. A session that opens the superseded document, follows the supersession, and answers correctly is still evidence that the corpus routed it wrong. Its answer hides that, so one predicate cannot separate the two outcomes and a pair of them can.

## Discharge

A graded run of that probe in both arms, with a rate and the denominator it came from.

**The instrument is a probe.** It is [HW-PROBE-a-session-answers-from-the-register-without-opening-the-question-it-replaced](../probes/a-session-answers-from-the-register-without-opening-the-question-it-replaced.md). It states in its own body why a session that recovers counts against it there.

**A rule of the check layer is the other candidate answer, and this record does not ask for one.** A rule reads a corpus and never a session. It can report that a document of this corpus cites a superseded one, and it cannot report that a session read one. The gap named here is on the session side alone.

**This record is discharged.** The batch of 2026-09-30 graded that probe in both arms of the `campaign` tier, 30 sessions in each ([#1384](https://github.com/headwater-ai/headwater/issues/1384)). The present arm satisfied `not_opened` in 28 of 30, 78.7% to 98.2% in a 95% Wilson interval. The absent arm satisfied it in 26 of 30, 70.3% to 94.7%. The two intervals overlap, so the governance layer did not measurably steer sessions away from the superseded document.

**The grade reads avoidance, as the Obligation section asks.** The expectation is `not_opened` over the superseded register, so a session that opened it and recovered is a miss. The grader counts a read made through Bash since #1396. The present arm made 34 Bash read calls in 26 sessions and 39 `Read` calls. The absent arm made 33 Bash read calls in 27 sessions and 45 `Read` calls. The corpus was not frozen between the two arms' sessions, and `docs/spec/09-open-questions.md` changed after the recording. So [the present-arm result](../probe-results/campaign-of-2026-09-30-campaign-tier-present-arm-navigability.md) grades each verdict over a moved read set. [The evaluation of that batch](../evaluations/what-the-counterfactual-campaign-of-2026-09-30-measured-by-component.md) states the limits of the whole batch.
