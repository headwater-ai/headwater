---
"headwater:generated": "probe_result. `headwater generate` writes this file, and `headwater generate --check` holds it. Edit the corpus, not this file."
id: HW-RESULT-status-probe-pilot-of-2026-09-29-campaign-tier-absent-arm-sufficiency
title: Probe result for status-probe-pilot-of-2026-09-29-campaign-tier-absent-arm-sufficiency
status: current
status_since: 2026-09-29
summary: "The grade of the transcript `status-probe-pilot-of-2026-09-29-campaign-tier-absent-arm-sufficiency`, taken over the probes this corpus declares and the version of the grader that evaluated them."
last_verified: 2026-09-29
---

# The result of docs/probe-runs/status-probe-pilot-of-2026-09-29-campaign-tier-absent-arm-sufficiency.md

A probe result is a function of three committed inputs and of nothing else: the transcript at `docs/probe-runs/status-probe-pilot-of-2026-09-29-campaign-tier-absent-arm-sufficiency.md`, the expectations the probes of this corpus declare, and the version of the grader that evaluated them. Fetch the three and this file comes back.

## The run this transcript recorded

A campaign run in the absent arm, on claude-sonnet-5 at 2026-09-29.
served version claude-sonnet-5, tree sha256:7c8d9c1568c93bd1747f64dbda46742cf0c82d9e27b795aa2302de8bba2433fd, selection sha256:002ab25221c821cea88b42dbbdb3e831304d41c12788a9557d7a1da2a01efa17, read set sha256:c97c613e433de3010d256895bc70c07bca1047d280d23f2dd841dc7712e0f2b7, seed 0, harness 0.4.1.
realized cost $1.79, which the adaptive layer reads as the cost of its own instrument.

10 events over 1 of the 10 probes this corpus declares, in 10 sessions and 43 tool calls.

The engine confirmed the taxonomy, that every member of the run identity is present, the membership of every probe named, that no key outside the closed set appears, and that a realized cost was recorded. Present is not confirmed: of the six members a plan fixes before a run, the lock is the one compared here. A lock that differs refuses the file unless the read set of its probes is the one this tree composes. `headwater probe stale` compares the read set against the tree in front of it. It graded nothing: a verdict is a function of this transcript, the expectations these probes declare and a grader version, and `headwater probe grade` is the verb that holds all three.

**The selection this transcript names is not the selection this corpus composes.** The transcript names `sha256:002ab25221c821cea88b42dbbdb3e831304d41c12788a9557d7a1da2a01efa17` and this corpus composes `sha256:50ed5ce43072d9f674a8dc52e1dcad84de4d649e39a77ce22179836188196b09`. The transcript's digest is the digest of 1 of the 10 probes this corpus composes, so the run was planned over that part of the selection, or the rest were added after it was recorded. Every verdict below is over that part, and a probe outside it is not a session this run owed. The tree, the seed and the harness above are provenance and nothing compares them.

The `read_set` digest above covers every probe of the selection and every document one of them examines, by path and content. It is recorded here and compared nowhere in this file. A comparison against the tree in front of a reader would move these bytes on every edit to a document the selection points at, and `generate --check` holds this file to its bytes, so the staleness of a measurement would stop a merge. `headwater probe stale` takes the digest and reports which recorded results a change voided.

Graded by grader 0.5.0.
A campaign run in the absent arm, on claude-sonnet-5 at served version claude-sonnet-5.

## The verdicts

- HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request (sufficiency, expects answered)
    session L2-campaign-absent-p1-r1: not satisfied — the session ended with no answer
    session L2-campaign-absent-p1-r10: satisfied — event 2: answered `current HW-DR-0052`
    session L2-campaign-absent-p1-r2: satisfied — event 3: answered `current HW-DR-0052`
    session L2-campaign-absent-p1-r3: not satisfied — the session ended with no answer
    session L2-campaign-absent-p1-r4: satisfied — event 5: answered `current HW-DR-0052`
    session L2-campaign-absent-p1-r5: satisfied — event 6: answered `current HW-DR-0052`
    session L2-campaign-absent-p1-r6: satisfied — event 7: answered `current HW-DR-0052`
    session L2-campaign-absent-p1-r7: satisfied — event 8: answered `current HW-DR-0052`
    session L2-campaign-absent-p1-r8: satisfied — event 9: answered `current HW-DR-0052`
    session L2-campaign-absent-p1-r9: satisfied — event 10: answered `current HW-DR-0052`

## The rate, and the denominator it is over

8 of 10 graded sessions satisfied their expectation: 80.0%, in a 95% interval of 49.0% to 94.3%.
The denominator is the graded sessions and never the selected probes. 0 sessions reached no verdict, and a session with no verdict is outside both halves of that fraction.

An interval that overlaps the previous run's is variance and one that does not is drift. This is one arm, so it estimates no effect: an efficacy claim is a comparison of two results, and the arm each one recorded is on it.

## The comparisons this arm takes part in

What the governance changes. The treated arm is the `campaign` present arm and the control is the `campaign` absent arm, which removed the paths the `campaign` tier's `ablation` names and kept `docs/`.

- treated, `docs/probe-runs/status-probe-pilot-of-2026-09-29-campaign-tier-present-arm-sufficiency.md`: 10 of 10 graded sessions satisfied their expectation, 100.0%, in a 95% interval of 72.2% to 100.0%. 0 sessions refused by the session itself.
- control, `docs/probe-runs/status-probe-pilot-of-2026-09-29-campaign-tier-absent-arm-sufficiency.md`: 8 of 10 graded sessions satisfied their expectation, 80.0%, in a 95% interval of 49.0% to 94.3%. 0 sessions refused by the session itself.

The difference is +20.0 points, in a 95% Newcombe interval of -11.2 points to +51.0 points. The interval contains zero, so this run does not separate the two arms at the 5% level.
