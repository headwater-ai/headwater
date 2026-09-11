---
id: HW-OBL-0010
title: "The corpus descriptor exists and no probe has run against it"
status: current
status_since: 2026-09-11
waiting_on: measurement
last_verified: 2026-09-11
summary: "Two recordings stand on the shelf, a confirmation refuses both, and no probe has returned a verdict against the descriptor."
provenance:
  warrant: asserted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0014
---

<!-- headwater allow=lifecycle.transition.not_permitted scope=file until=2027-12-31 reason=accepted_deviation note=a discharge taken on a measurement that does not exist has to be retractable, and `discharged` is terminal in the `obligation` regime -->

# The corpus descriptor exists and no probe has run against it

## Context

[Q14](../spec/09-decisions.md#q14--discovery-surface) rules that the corpus descriptor is a generated projection at `.headwater/corpus.json`. A descriptor should let a cold agent reach a governing document that it otherwise misses.

## Obligation

The Discovery and Navigability probe categories are the instrument, and this corpus owes a reading from each.

## Discharge

**The artifact under test exists and the instrument is declared. No run has returned a verdict.** [HW-PROBE-a-cold-agent-reaches-the-governing-document-through-the-corpus-descriptor](../probes/a-cold-agent-reaches-the-governing-document-through-the-corpus-descriptor.md) states the task and the expectation, and `headwater probe plan` selects it. The recorder that was missing landed in #725, and [the transcript of 2026-09-11](../probe-runs/regression-probe-transcript-for-2026-09-11.md) records four sessions. A later change moved the lock under that recording, so [the result](../probe-results/regression-probe-transcript-for-2026-09-11.md) carries zero verdicts of four. The section at the end of this record states what each recording is worth.

`headwater generate` writes `.headwater/corpus.json`, and `generate --check` holds it to regeneration. The descriptor over this corpus names one root, one exclusion, the taxonomy identity and the lock hash. It names one entry point for each shelf that holds a document. `.headwater/corpus.json` is where that set is counted, and a shelf with no document on it puts nothing there. So a probe has something to run against, and no probe has returned a verdict against it.

The descriptor carries each declared export profile as well, and this repository declares none, so that list is empty and means it.

## Two recordings, and what each one reads

This record stood at `discharged` with an `accepted` warrant between 2026-08-13 and 2026-09-11, on a reading that does not exist. A regression session was recorded on 2026-09-09 and `docs/probe-runs/regression-probe-transcript-for-2026-09-09.md` holds it. The commit that landed it also edited `.headwater/overlay.yml` and `.headwater/taxonomy.lock`, so the `lock` the transcript pins was not the lock of the tree it merged into, and the first of the five confirmations refused the recording whole on that day. [The result](../probe-results/regression-probe-transcript-for-2026-09-09.md) carries zero verdicts of four.

**The warrant is lowered rather than the sentence deleted**, because a warrant states who stands behind the claim and the accepter never saw this correction. What the evidence supports is that the instrument is declared, one run was taken and the recording is unusable. That is `asserted`. A fresh recording against this tree is what returns the record to `discharged`, and `headwater generate` now reports a refused transcript rather than leaving it to a reader of the result.

**[The recording of 2026-09-11](../probe-runs/regression-probe-transcript-for-2026-09-11.md) was that recording for one day, and this record stands at `current` again.** This record read it as a discharge on 2026-09-11. `8b61593e` moved `.headwater/taxonomy.lock` later the same day, so the lock the recording pins is not the lock of this tree, and the first of the five confirmations refuses the recording whole. `0149a92c` retired it from `current` to `deprecated`, and [the result](../probe-results/regression-probe-transcript-for-2026-09-11.md) carries zero verdicts of four. Both recordings on the shelf are refused, and this corpus holds no reading.

**The transcript still records what each session did, and no rate here rests on it.** A reader who wants the calls reads the transcript and counts the events. A verdict is what the grader returns, the grader reaches no refused recording, and a rate over zero verdicts is a rate over none. What discharges this record is a recording that a confirmation does not refuse. The owner ruled on 2026-09-11 to let the current one go stale until a fresh recording is worth taking, which [HW-DR-0062](../decisions/0062-a-refused-recording-is-held-by-the-reliance-its-state-claims-and-not-by-promotion.md) carries, so this record waits on a measurement rather than on a build.
