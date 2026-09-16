---
id: HW-OBL-0013
title: "No probe tests whether a counted tombstone stops a confident report of absence"
status: current
status_since: 2026-08-10
waiting_on: build
last_verified: 2026-09-16
summary: "Q17 claims a counted tombstone stops a confident report of absence, and a third run against a corpus with no tombstone recorded no answer."
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

[The transcript of 2026-09-16](../probe-runs/regression-probe-transcript-for-2026-09-16.md) is not refused, and [the result](../probe-results/regression-probe-transcript-for-2026-09-16.md) carries a verdict for this probe: `not satisfied`, because the session ended with no answer in the closed set. It searched `.headwater/overlay.yml` for an export filter and found the `site` profile with none, then closed with a sentence ending in the word `absent` set in bold Markdown rather than the bare word the recorder's derivation matches; spec 15 rules that the derivation strips a trailing period and never a substring or a markup character, so the session's answer did not survive the transform.

**This run corrects the premise this record stated.** `.headwater/overlay.yml` declares one export profile, `site`, and this record's earlier readings said the corpus declares none. The accurate gap is narrower: the one profile this corpus serves withholds nothing, so no session that reaches it meets a counted tombstone. The wait moves from "an export profile that serves one" to a filter configuration on the profile that already exists.
