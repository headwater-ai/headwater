---
id: HW-OBL-0010
title: "The corpus descriptor exists and no probe has run against it"
status: discharged
status_since: 2026-09-16
waiting_on: measurement
last_verified: 2026-09-30
summary: "A third recording is not refused, and both the discovery and navigability probe categories now carry a verdict against the descriptor."
provenance:
  warrant: asserted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0014
    - HW-RUN-campaign-of-2026-09-30-campaign-tier-present-arm-discovery
    - HW-RUN-campaign-of-2026-09-30-campaign-tier-absent-arm-discovery
---

<!-- headwater allow=lifecycle.transition.not_permitted scope=file until=2027-12-31 reason=accepted_deviation note=a discharge taken on a measurement that does not exist has to be retractable, and `discharged` is terminal in the `obligation` regime -->

# The corpus descriptor exists and no probe has run against it

## Context

[Q14](../spec/09-decisions.md#q14--discovery-surface) rules that the corpus descriptor is a generated projection at `.headwater/corpus.json`. A descriptor should let a cold agent reach a governing document that it otherwise misses.

## Obligation

The Discovery and Navigability probe categories are the instrument, and this corpus owes a reading from each.

## Discharge

**The artifact under test exists and the instrument is declared. No run has returned a verdict.** [HW-PROBE-a-cold-agent-reaches-the-governing-document-through-the-corpus-descriptor](../probes/a-cold-agent-reaches-the-governing-document-through-the-corpus-descriptor.md) states the task and the expectation, and `headwater probe plan` selects it. The recorder that was missing landed in #725, and [the transcript of 2026-09-11](../probe-runs/regression-probe-transcript-for-2026-09-11.md) records four sessions. A later change moved the lock under that recording, and for a time [the result](../probe-results/regression-probe-transcript-for-2026-09-11.md) carried zero verdicts of four. The sections at the end of this record states what each recording is worth.

`headwater generate` writes `.headwater/corpus.json`, and `generate --check` holds it to regeneration. The descriptor over this corpus names one root, one exclusion, the taxonomy identity and the lock hash. It names one entry point for each shelf that holds a document. `.headwater/corpus.json` is where that set is counted, and a shelf with no document on it puts nothing there. So a probe has something to run against, and no probe has returned a verdict against it.

The descriptor carries each declared export profile as well, and this repository declares none, so that list is empty and means it.

## Two recordings, and what each one reads

This record stood at `discharged` with an `accepted` warrant between 2026-08-13 and 2026-09-11, on a reading that does not exist. A regression session was recorded on 2026-09-09 and `docs/probe-runs/regression-probe-transcript-for-2026-09-09.md` holds it. The commit that landed it also edited `.headwater/overlay.yml` and `.headwater/taxonomy.lock`. So the `lock` the transcript pins was not the lock of the tree it merged into. The first of the five confirmations refused the recording whole on that day. Since #1338 (PR #1448), a moved lock marks the verdicts and keeps them, so [the result](../probe-results/regression-probe-transcript-for-2026-09-09.md) carries four verdicts again. Its descriptor session is `not satisfied`.

**The warrant is lowered rather than the sentence deleted**, because a warrant states who stands behind the claim and the accepter never saw this correction. What the evidence supports is that the instrument is declared, one run was taken and the recording is unusable. That is `asserted`. A fresh recording against this tree is what returns the record to `discharged`. `headwater generate` now reports a refused transcript rather than leaving it to a reader of the result.

**[The recording of 2026-09-11](../probe-runs/regression-probe-transcript-for-2026-09-11.md) was that recording for one day, and this record stands at `current` again.** This record read it as a discharge on 2026-09-11. `8b61593e` moved `.headwater/taxonomy.lock` later the same day. So the lock the recording pins is not the lock of this tree, and the first of the five confirmations refuses the recording whole. `0149a92c` retired it from `current` to `deprecated`. On that day both recordings on the shelf were refused, and this corpus held no reading. Since #1338, [the result](../probe-results/regression-probe-transcript-for-2026-09-11.md) carries four verdicts again, graded over a moved read set, and its descriptor session is `not satisfied`.

**Each of these two recordings is one session per probe, and no rate here rests on either.** A verdict graded over a moved read set says what the session did over the documents it met. It says nothing about the tree in front of a reader. What discharged this record is the recording of 2026-09-16 below, which a confirmation did not refuse. The owner ruled on 2026-09-11 to let the current one go stale until a fresh recording is worth taking, which [HW-DR-0062](../decisions/0062-a-refused-recording-is-held-by-the-reliance-its-state-claims-and-not-by-promotion.md) carries.

## A third recording carries a verdict for both categories

[The transcript of 2026-09-16](../probe-runs/regression-probe-transcript-for-2026-09-16.md) is a recording the first confirmation does not refuse. [The result](../probe-results/regression-probe-transcript-for-2026-09-16.md) carries a verdict for every probe of the current regression selection. The descriptor session itself — [HW-PROBE-a-cold-agent-reaches-the-governing-document-through-the-corpus-descriptor](../probes/a-cold-agent-reaches-the-governing-document-through-the-corpus-descriptor.md) — made 27 recorded calls and named the governing document in none of them. The verdict is `not satisfied`.

**The negative reading discharges this record, because the obligation is for a reading and never for a passing one.** The Discovery category now has three readings from this run and Navigability has three. Two of the three navigability probes were satisfied (a `not_opened` reading over the register, and a `cited` reading over an evaluation the session produced) and one was not (the adjudication probe below). Every Discovery reading in the run was `not satisfied`. Both categories the obligation names have a verdict, so the instrument this record asked for has run.

**These recordings predate the sealed workspace (#1229).** Each session ran in a workspace that kept every probe file and every recorded run, so a session could read its own expectation. No session has run since in a workspace that [`tools/probe/seal.sh`](../../tools/probe/seal.sh) sealed. The discharge stands, because the obligation was for a reading rather than for a clean one.

## The sealed batch of 2026-09-30

The batch of 2026-09-30 graded the descriptor probe 30 times in each arm of the `campaign` tier, in sealed workspaces ([#1384](https://github.com/headwater-ai/headwater/issues/1384)). The present arm reached the governing document in 7 of 30 sessions (11.8% to 40.9% in a 95% Wilson interval). The absent arm reached it in 4 of 30 (5.3% to 29.7%). Some sessions named a path outside their workspace. Without them, the rates are 7 of 25 and 4 of 29. So the discharge now rests on a sealed reading as well. That reading does not show that the governance layer helps a cold session reach the document. The corpus was not frozen between the two arms' sessions. `docs/spec/12-check-layer.md` changed after the recording, so each verdict is graded over a moved read set. [The evaluation of that batch](../evaluations/what-the-counterfactual-campaign-of-2026-09-30-measured-by-component.md) states every limit and the cost. This record stays discharged.
