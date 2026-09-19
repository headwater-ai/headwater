---
id: HW-OBL-0200
status: current
status_since: 2026-09-19
summary: "Three current probe-run transcripts moved to deprecated when a taxonomy change staled their lock, and no fresh session has run against the new one."
last_verified: 2026-09-19
title: "Three graded regression runs are owed a fresh recording against a moved taxonomy lock"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-sonnet-5
  activity: draft
  evidence_basis: evidenced
waiting_on: measurement
relations:
  traces_to:
    - HW-RUN-regression-probe-transcript-for-2026-09-16
    - HW-RUN-regression-probe-transcript-for-2026-09-17
    - HW-RUN-regression-probe-transcript-for-2026-09-17-after-the-probe-corrections
---

# Three graded regression runs are owed a fresh recording against a moved taxonomy lock

## Context

[PR #963](https://github.com/headwater-ai/headwater/pull/963) (issue [#935](https://github.com/headwater-ai/headwater/issues/935)) declares a `verification` kind and a relation over it. That change moves `.headwater/taxonomy.lock`. Three probe-run transcripts stood at `current` and pinned the digest from before that move: 2026-09-16, 2026-09-17, and 2026-09-17-after-the-probe-corrections. `headwater generate --check` refuses to regenerate a `current` transcript's result against a digest the transcript was not planned against. Its own refusal names two remedies: record the session again, or retire the transcript to a terminal state. This corpus took the second remedy, so all three now stand at `deprecated`.

The remedy taken says nothing was re-verified. It says only that nobody may rely on the old recording anymore. [HW-DR-0062](../decisions/0062-a-refused-recording-is-held-by-the-reliance-its-state-claims-and-not-by-promotion.md) separates a transcript's state from a re-run of it. This record states the second half that ruling implies: a state change is not a measurement.

## Obligation

Two live artifacts in this corpus depend on evidence one of the three retired runs carries, and no later run has repeated it. [HW-OBL-0198](0198-nothing-states-how-much-of-a-session-s-budget-the-standing-instructions-consume-before-work-starts.md) traces to the 2026-09-16 recording. [The pointer-probe evaluation](../evaluations/the-pointer-probe-grades-a-session-against-the-router-s-own-first-pick-so-a-correct-session-that-declines-a-wrong-pointer-fails.md) traces to the 2026-09-17-after-the-probe-corrections recording. Neither claim is false because its recording moved to `deprecated`, but neither has been checked against the tree this corpus now carries.

This corpus owes a fresh regression recording, run with `tools/probe/probe-record.sh`, over the same eight probes the retired runs graded. That recording runs against the lock `verification` and `proven_by` now carry. A later taxonomy change repeats this obligation for whatever it retires in turn. No mechanism today assigns that recording to a person or a stage of the build order, before or after such a change lands.

## Discharge

A new `docs/probe-runs/` transcript, `status: current`, planned against the lock this corpus carries once #935 merges, with a `docs/probe-results/` projection `headwater generate` writes from it. The two artifacts named above are read again by that recording's own grading, and this record is discharged when they are.
