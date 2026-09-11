---
id: HW-OBL-0013
title: "No probe tests whether a counted tombstone stops a confident report of absence"
status: current
status_since: 2026-08-10
waiting_on: build
last_verified: 2026-08-28
summary: "Q17 claims that a counted tombstone stops an agent reporting absence with confidence, and no probe has run."
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

What is missing is the same thing [HW-OBL-0010](0010-the-corpus-descriptor-exists-and-no-probe-has-run-against.md) now names. The recorder landed and `docs/probe-runs/` holds one transcript, recorded on 2026-09-09, which the first of the five confirmations refuses whole, so it grades to no verdict for this probe or any other. This corpus also still declares one export profile, so a session served this corpus never meets a tombstone. So the wait moved from an instrument nobody had declared to a recording that a confirmation does not refuse.
