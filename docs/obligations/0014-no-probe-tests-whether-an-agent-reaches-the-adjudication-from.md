---
id: HW-OBL-0014
title: "No probe tests whether an agent reaches the adjudication from the losing document"
status: current
status_since: 2026-08-10
waiting_on: build
last_verified: 2026-09-30
summary: "Q18 claims that an agent meeting the losing document reaches the adjudication, and three of four recorded sessions did, against a mis-specified probe."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0018
    - HW-RUN-campaign-of-2026-09-30-campaign-tier-present-arm-navigability
    - HW-RUN-campaign-of-2026-09-30-campaign-tier-absent-arm-navigability
---

# No probe tests whether an agent reaches the adjudication from the losing document

## Context

[Q18](../spec/09-decisions.md#q18--recording-adjudicated-disagreements) rules how an adjudicated disagreement is recorded. An agent that meets the losing document first should reach the adjudication.

## Obligation

A probe over a settled pair is the instrument.

## Discharge

The harness exists and this corpus still holds no adjudicated pair. [HW-PROBE-an-agent-reaches-the-adjudication-from-the-document-that-lost-it](../probes/an-agent-reaches-the-adjudication-from-the-document-that-lost-it.md) runs over a supersession instead: the tombstone at `docs/spec/09-open-questions.md` still writes a heading for every decision, and citations across this corpus reach it. A run of that probe narrows this record and closes none of it. [Q18](../spec/09-decisions.md#q18--recording-adjudicated-disagreements) is about two documents that disagreed and were settled, and this corpus holds no such pair.

**Sessions reach the adjudication, and the probe named the wrong target.** Four sessions are recorded against this probe, on 2026-09-09, 2026-09-11, 2026-09-16 and 2026-09-17. Three of the four read `docs/spec/09-open-questions.md` and then read `docs/decisions/0008-probe-cost-and-cadence.md`. That document is [HW-DR-0008](../decisions/0008-probe-cost-and-cadence.md), which is the adjudication for Q8 itself. The probe examined `HW-REG-decisions` alone, and that document is a generated index whose Q8 row is one sentence linking the same record. So the three sessions followed the tombstone's one outgoing link to the answer and graded as misses. The probe now examines both identifiers, and the grader satisfies `opened` on any one of the documents a probe examines. A regeneration re-grades the two recordings that carry a verdict. [2026-09-16](../probe-results/regression-probe-transcript-for-2026-09-16.md) and [2026-09-17](../probe-results/regression-probe-transcript-for-2026-09-17.md) now report `satisfied` here, each on a witness that names the call. `headwater probe stale` reports both results stale, because the read set moved with the edit.

**One of the four is the failure this probe is for.** [The run of 2026-09-11](../probe-results/regression-probe-transcript-for-2026-09-11.md) records a session that read `docs/spec/09-open-questions.md`, made no second call, and answered from the heading text of the tombstone. A later change moved the lock under that recording. Since #1338 (PR #1448), a moved lock marks the verdicts of a result and keeps them. So that result carries its verdict again, graded over a moved read set: `not satisfied`, because the one call named neither document. That session is the one the probe was written to catch. It is the only one of the four that opened neither the register nor the record.

This remains a reading of the supersession and not of Q18. Q18 rules that an adjudication is a decision document which carries `overrides`, and no document of this corpus carries that key. So the instrument the obligation names still has nothing to run against, and this record waits on a build.

**Every recording that this record cites predates the sealed workspace (#1229).** Each session ran in a workspace that kept every probe file, so a session could read its own expectation. A reading that counts for this record comes from a workspace that [`tools/probe/seal.sh`](../../tools/probe/seal.sh) sealed.

**A sealed batch now reads the probe, and every session reached the adjudication.** The batch of 2026-09-30 graded it in both arms of the `campaign` tier ([#1384](https://github.com/headwater-ai/headwater/issues/1384)). The present arm satisfied `opened` in 30 of 30 sessions, and the absent arm in 30 of 30. Each is 88.6% to 100.0% in a 95% Wilson interval. The present arm made 21 Bash read calls in 15 sessions and 33 `Read` calls. The absent arm made 39 Bash read calls in 22 sessions and 41 `Read` calls. The pair was still a supersession and not an adjudication, so this is a reading of recovery from a superseded document and not of Q18.

**The reading has limits.** Both arms are at the ceiling, so the probe cannot show what the governance layer adds. The corpus was not frozen between the two arms' sessions. `docs/spec/09-open-questions.md` changed after the recording, so each verdict is graded over a moved read set. [The evaluation of that batch](../evaluations/what-the-counterfactual-campaign-of-2026-09-30-measured-by-component.md) states every limit and the cost. This record stays open by the owner's ruling of 2026-09-30, pending [#1472](https://github.com/headwater-ai/headwater/issues/1472), and it waits on a document that carries `overrides`.
