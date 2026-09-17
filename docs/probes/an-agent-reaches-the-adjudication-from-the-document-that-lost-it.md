---
id: HW-PROBE-an-agent-reaches-the-adjudication-from-the-document-that-lost-it
status: current
status_since: 2026-09-06
summary: A session that meets the superseded register first either reaches the decision that replaced it or reports the stale answer.
last_verified: 2026-08-14
probe_category: navigability
expectation: opened
oracle: "none"
title: "An agent reaches the adjudication from the document that lost it"
provenance:
  warrant: asserted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft
  evidence_basis: unevidenced
relations:
  examines:
    - HW-REG-decisions
    - HW-DR-0008
  traces_to:
    - HW-OBL-0014
---

# An agent reaches the adjudication from the document that lost it

## Task

Read `docs/spec/09-open-questions.md#q8--probe-cost-and-cadence` and say what this project decided about probe cost and cadence.

## Expectation

`opened` over `HW-REG-decisions` or over `HW-DR-0008`. Either one satisfies the predicate, and the transcript shows that the session left the tombstone for one of them.

**The two targets are one destination.** [HW-DR-0008](../decisions/0008-probe-cost-and-cadence.md) is the adjudication for Q8. `docs/spec/09-decisions.md` (`HW-REG-decisions`) is an index over the decisions shelf, and its Q8 row is a sentence that links that same record. A session that follows the tombstone's one outgoing link straight to the decision record reached the adjudication by the shorter route. This probe named the index alone until 2026-09-17, which graded the shorter route as a miss.

**The failure this probe is for is a session that opens neither.** The losing document is a tombstone that still writes a heading for every decision, so a session can read a heading there and stop. The session of 2026-09-11 did: it read `docs/spec/09-open-questions.md`, made no second call, and reported the heading text as the answer. Such an answer reads as an answer, and only the read set separates it from an answer taken from the adjudication.

**The task names the anchor because that is the state under test.** Citations across this corpus name an anchor in the tombstone file, so a reader who follows one arrives at the losing document rather than at the answer. No count of them is written here. It moves on every commit that adds or removes a citation, and [HW-DR-0049](../decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md) rules that a fold over this corpus is derived rather than stored. The reading is `grep -rho "09-open-questions.md#[a-z0-9-]*" docs | wc -l`.

**The task section is the prompt, and nothing else of this document reaches the session.** [Spec 15](../spec/15-the-recorder-contract.md#the-prompt-is-the-task-section-and-the-answer-is-the-whole-final-message) fixes that. The paragraphs above sit here rather than under `## Task` for that reason: a commentary paragraph under that heading is prompt, and the two runs that carried one are [evaluated](../evaluations/a-probe-task-section-is-the-prompt-so-commentary-under-that-heading-is-prompt.md).

[HW-OBL-0014](../obligations/0014-no-probe-tests-whether-an-agent-reaches-the-adjudication-from.md) records that no probe tests this and that the corpus holds no adjudicated pair. The pair here is a supersession rather than an adjudicated disagreement, so a run of this probe narrows that record and does not close it: [Q18](../spec/09-decisions.md#q18--recording-adjudicated-disagreements) is about two documents that disagreed and were settled, and this corpus still holds none.
