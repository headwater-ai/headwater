---
id: HW-OBL-0014
title: "No probe tests whether an agent reaches the adjudication from the losing document"
status: current
status_since: 2026-08-10
waiting_on: build
last_verified: 2026-08-13
summary: "Q18 claims that an agent meeting the losing document first reaches the adjudication, and one run says that it did not."
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
---

# No probe tests whether an agent reaches the adjudication from the losing document

## Context

[Q18](../spec/09-decisions.md#q18--recording-adjudicated-disagreements) rules how an adjudicated disagreement is recorded. An agent that meets the losing document first should reach the adjudication.

## Obligation

A probe over a settled pair is the instrument.

## Discharge

The harness exists and this corpus still holds no adjudicated pair. [HW-PROBE-an-agent-reaches-the-adjudication-from-the-document-that-lost-it](../probes/an-agent-reaches-the-adjudication-from-the-document-that-lost-it.md) runs over a supersession instead: the tombstone at `docs/spec/09-open-questions.md` still writes a heading for every decision, and 136 citations reach it. A run of that probe narrows this record and closes none of it. [Q18](../spec/09-decisions.md#q18--recording-adjudicated-disagreements) is about two documents that disagreed and were settled. [The run of 2026-09-11](../probe-results/regression-probe-transcript-for-2026-09-11.md) returned the first verdict, and a later change moved the lock under that recording, so the result carries none now. The transcript still records what the session did: it read `docs/spec/09-open-questions.md`, made no second call, and answered from the tombstone. The verdict was `not satisfied` over one session, and it went with the grade. That is a reading of the supersession and not of Q18: this corpus still holds no adjudicated pair, and one session is one session.
