---
id: HW-OBL-0013
title: "No probe tests whether a counted tombstone stops a confident report of absence"
status: current
status_since: 2026-08-10
waiting_on: build
last_verified: 2026-08-28
summary: "Q17 claims that a counted tombstone stops an agent reporting absence with confidence, and the probe has run against a corpus that serves no tombstone."
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

The recording this record waited for landed. [The transcript of 2026-09-11](../probe-runs/regression-probe-transcript-for-2026-09-11.md) is graded and [the result](../probe-results/regression-probe-transcript-for-2026-09-11.md) carries a verdict for this probe. The verdict measures nothing about a tombstone, for two reasons that are both about the instrument. This corpus declares no export profile, so the session met no counted tombstone and searched the tree for the word instead. And `tools/probe/probe-record.sh` passes no answer to the transform, so the session that ended with the word `present` is recorded as a session that ended with no answer. So the wait moved from a recording nobody had taken to a served corpus that holds a tombstone and a recorder that observes an answer.
