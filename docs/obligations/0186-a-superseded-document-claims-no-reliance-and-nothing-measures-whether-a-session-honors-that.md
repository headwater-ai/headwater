---
id: HW-OBL-0186
status: current
status_since: 2026-09-11
summary: "A supersession says that nobody may rely on the document that lost, and every mechanism here acts on a relation rather than on a reading."
last_verified: 2026-09-11
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

**The instrument exists and no run has taken a reading with it.** The probe is [HW-PROBE-a-session-answers-from-the-register-without-opening-the-question-it-replaced](../probes/a-session-answers-from-the-register-without-opening-the-question-it-replaced.md). It states in its own body why a session that recovers counts against it there.

**A rule of the check layer is the other candidate answer, and this record does not ask for one.** A rule reads a corpus and never a session. It can report that a document of this corpus cites a superseded one, and it cannot report that a session read one. The gap named here is on the session side alone.
