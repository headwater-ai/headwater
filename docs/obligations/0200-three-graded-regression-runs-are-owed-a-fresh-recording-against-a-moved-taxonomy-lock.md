---
id: HW-OBL-0200
status: discharged
status_since: 2026-10-03
summary: "Three regression transcripts went to deprecated when their lock moved. A sealed recording of their eight probes on claude-sonnet-5 replaced them on 2026-10-03."
last_verified: 2026-10-03
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
    - HW-RUN-regression-of-2026-10-03-regression-tier-present-arm-discovery
    - HW-RUN-regression-of-2026-10-03-regression-tier-present-arm-navigability
    - HW-RUN-regression-of-2026-10-03-regression-tier-present-arm-sufficiency
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

**A fresh recording must now run in a sealed workspace (#1229).** [`tools/probe/seal.sh`](https://github.com/headwater-ai/headwater/blob/main/tools/probe/seal.sh) removes the three probe shelves and `.headwater/export.json`. It removes every document under `docs/` that names a selected probe, and every answer key that `.headwater/probe.yml` declares for one. For each document that it removes, it also removes the identifier claim under `.headwater/ids/`. It also removes each line, in any file of the workspace, that holds the identifier or the file name of that document. A JSON file loses the array element that names the document. A slug of one word, or a file name that another file shares such as `README`, removes no line. It removes every file outside `docs/` that names a selected probe, unless `folds:` in `.headwater/probe.yml` declares the file a fold. A fold stays as a file. `tools/probe/probe-record.sh` refuses a workspace that still holds the instrument, or a file that names the probe and is not a fold. The three runs that this record names ran before the seal existed. So a recording that repeats them as they ran does not discharge this record.

**Discharged on 2026-10-03 by [PR #1680](https://github.com/headwater-ai/headwater/pull/1680) for [#1474](https://github.com/headwater-ai/headwater/issues/1474).** `tools/probe/campaign.sh` recorded the eight probes of the retired runs in a sealed workspace on `claude-sonnet-5`, at commit `a46bd95e`. The three transcripts are at `status: current`: [discovery](../probe-runs/regression-of-2026-10-03-regression-tier-present-arm-discovery.md), [navigability](../probe-runs/regression-of-2026-10-03-regression-tier-present-arm-navigability.md) and [sufficiency](../probe-runs/regression-of-2026-10-03-regression-tier-present-arm-sufficiency.md). `headwater generate` wrote a `docs/probe-results/` projection of each. The eight sessions spent 437 cents. No recorded call named a path on a probe shelf, and no session called `WebSearch` or `WebFetch`. Of the two artifacts above, HW-OBL-0198 was already discharged on 2026-10-01. The pointer-probe evaluation was read again against the new pointer session. The verdict is `not satisfied` again, for the same reason, so the evaluation stands and now states that reading.

The new recording repeats the configuration of the 2026-09-17 run after the probe corrections. The 2026-09-17 run before the corrections cannot be recorded again, because only the corrected probes exist. No sealed recording of the 2026-09-16 configuration on `claude-haiku-4-5` exists. Whether one is owed is a spend question for the owner on #1474, and this record does not wait on it.
