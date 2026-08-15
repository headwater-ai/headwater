---
id: HW-PROBE-an-agent-reaches-the-adjudication-from-the-document-that-lost-it
status: draft
status_since: 2026-08-14
summary: A session that meets the superseded register first either reaches the decision that replaced it or reports the stale answer.
last_verified: 2026-08-14
probe_category: navigability
expectation: opened
oracle: "none"
provenance:
  warrant: asserted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft
  evidence_basis: unevidenced
relations:
  examines:
    - HW-REG-decisions
  traces_to:
    - HW-OBL-0014
---

# An agent reaches the adjudication from the document that lost it

## Task

Read `docs/spec/09-open-questions.md#q8--probe-cost-and-cadence` and say what this project decided about probe cost and cadence.

The task names the anchor because that is the state under test. 136 citations in this corpus name an anchor in that file, so a reader who follows one arrives at the losing document rather than at the answer.

## Expectation

`opened` over `HW-REG-decisions`. The transcript shows that the session read the decision register, which is where the answer moved.

**The losing document is a tombstone that still writes a heading for every decision, so a session can read a heading there and stop.** That is the failure this probe is for: an answer taken from the superseded document reads as an answer, and only the read set separates it from an answer taken from the adjudication.

[HW-OBL-0014](../obligations/0014-no-probe-tests-whether-an-agent-reaches-the-adjudication-from.md) records that no probe tests this and that the corpus holds no adjudicated pair. The pair here is a supersession rather than an adjudicated disagreement, so a run of this probe narrows that record and does not close it: [Q18](../spec/09-decisions.md#q18--recording-adjudicated-disagreements) is about two documents that disagreed and were settled, and this corpus still holds none.
