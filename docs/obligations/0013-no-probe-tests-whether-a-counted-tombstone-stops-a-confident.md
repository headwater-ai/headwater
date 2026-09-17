---
id: HW-OBL-0013
title: "No probe tests whether a counted tombstone stops a confident report of absence"
status: current
status_since: 2026-08-10
waiting_on: build
last_verified: 2026-09-17
summary: "Q17 claims a counted tombstone stops a confident report of absence, and the recorder discarded the answer of both runs that reached one."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0017
---

# No probe tests whether a counted tombstone stops a confident report of absence

## Context

[Q17](../spec/09-decisions.md#q17--governed-access-and-the-solution-layer) rules how governed access and the solution layer work. A `counted` tombstone should stop an agent reporting absence with confidence.

## Obligation

A probe over a withheld answer is the instrument.

## Discharge

**The instrument now exists, and this corpus still withholds nothing.** [HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer](../probes/a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer.md) states the task and the closed answer set, and `headwater probe plan` selects it. A fixture at `engine/crates/cli/fixtures/answered-export/` proves the mechanism. A `filtered` profile withholds one document behind a counted tombstone, a `control` profile carries it, and only the control artifact states the recovery word.

The recording this record waited for landed and then went stale, twice. [The transcript of 2026-09-11](../probe-runs/regression-probe-transcript-for-2026-09-11.md) was graded for one day, a later change moved the lock under it, and [that result](../probe-results/regression-probe-transcript-for-2026-09-11.md) carries no verdict now. That session searched the tree for the word, found it in a fixture under `engine/`, and answered `present`, reading the search and not a tombstone, because this corpus still declares no export profile.

**Two graded sessions converged on the answer, and the recorder discarded both.** [The transcript of 2026-09-16](../probe-runs/regression-probe-transcript-for-2026-09-16.md) and [the transcript of 2026-09-17](../probe-runs/regression-probe-transcript-for-2026-09-17.md) each carry a verdict of `not satisfied` here. Each result reads "the session ended with no answer". Neither session was undecided. Both searched `.headwater/overlay.yml` for an export filter, both found the `site` profile with none, and both closed on `absent`. The 2026-09-16 session set the word in bold Markdown after three paragraphs of justification. The 2026-09-17 session wrote the bare word after two sentences of justification. [Spec 15](../spec/15-the-recorder-contract.md#the-prompt-is-the-task-section-and-the-answer-is-the-whole-final-message) rules that the derivation compares the whole final message and never a substring of one, so neither answer survived the transform. The probe now states the output form in its own task text, which is the remedy, and no recording since carries a reading of it.

The earlier account here read those two results as an undecided session. It was wrong in the direction that matters. Both sessions reached the closed set and the instrument lost the value. So neither run says whether a tombstone would have moved the answer.

**This run corrects the premise this record stated.** `.headwater/overlay.yml` declares one export profile, `site`, and this record's earlier readings said the corpus declares none. The accurate gap is narrower: the one profile this corpus serves withholds nothing, so no session that reaches it meets a counted tombstone. The wait moves from "an export profile that serves one" to a filter configuration on the profile that already exists.
