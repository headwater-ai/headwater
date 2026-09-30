---
"headwater:generated": "probe_result. `headwater generate` writes this file, and `headwater generate --check` holds it. Edit the corpus, not this file."
id: HW-RESULT-tombstone-probe-pilot-of-2026-09-29-campaign-tier-absent-arm-sufficiency
title: Probe result for tombstone-probe-pilot-of-2026-09-29-campaign-tier-absent-arm-sufficiency
status: current
status_since: 2026-09-29
summary: "The grade of the transcript `tombstone-probe-pilot-of-2026-09-29-campaign-tier-absent-arm-sufficiency`, taken over the probes this corpus declares and the version of the grader that evaluated them."
last_verified: 2026-09-29
---

# The result of docs/probe-runs/tombstone-probe-pilot-of-2026-09-29-campaign-tier-absent-arm-sufficiency.md

A probe result is a function of three committed inputs and of nothing else: the transcript at `docs/probe-runs/tombstone-probe-pilot-of-2026-09-29-campaign-tier-absent-arm-sufficiency.md`, the expectations the probes of this corpus declare, and the version of the grader that evaluated them. Fetch the three and this file comes back.

## The run this transcript recorded

A campaign run in the absent arm, on claude-sonnet-5 at 2026-09-29.
served version claude-sonnet-5, tree sha256:6e992f8aad533584b81d2565735072d21572d958f390787da3e15714159fcbf9, selection sha256:5ae5921d66889c1262ef7918402f57485f928baa6eba615a9c3a42c81fb5e58d, read set sha256:c3c61a60b3affe195bea364448c4e769f0d622cc24098b5e69dfcaf1d50bf764, seed 0, harness 0.5.0.
realized cost $2.32, which the adaptive layer reads as the cost of its own instrument.
It was planned against taxonomy sha256:badb09836f1099bd2d72472dc6370ac7fa0d14e93001264d38a69b5e59210e44, and this tree carries another. The read set of its probes is the one this tree composes, so the move reaches no document a verdict reads, and the transcript is read over this tree.

10 events over 1 of the 10 probes this corpus declares, in 10 sessions and 106 tool calls.

The engine confirmed the taxonomy, that every member of the run identity is present, the membership of every probe named, that no key outside the closed set appears, and that a realized cost was recorded. Present is not confirmed: of the six members a plan fixes before a run, the lock is the one compared here. A lock that differs refuses the file only where this tree composes no read set over its probes, and a read set that differs marks the verdicts and keeps them. `headwater probe stale` compares the read set against the tree in front of it. It graded nothing: a verdict is a function of this transcript, the expectations these probes declare and a grader version, and `headwater probe grade` is the verb that holds all three.

**The selection this transcript names is not the selection this corpus composes.** The transcript names `sha256:5ae5921d66889c1262ef7918402f57485f928baa6eba615a9c3a42c81fb5e58d` and this corpus composes `sha256:50ed5ce43072d9f674a8dc52e1dcad84de4d649e39a77ce22179836188196b09`. The transcript's digest is the digest of 1 of the 10 probes this corpus composes, so the run was planned over that part of the selection, or the rest were added after it was recorded. Every verdict below is over that part, and a probe outside it is not a session this run owed. The tree, the seed and the harness above are provenance and nothing compares them.

The `read_set` digest above covers every probe of the selection and every document one of them examines, by path and content. It is recorded here, and this file names no digest the tree in front of a reader composes. Where that tree composes another one, a single sentence above says that the read set moved, and it moves these bytes once. A digest from that tree would move these bytes on every edit to a document the selection points at, and `generate --check` holds this file to its bytes, so the staleness of a measurement would stop a merge. `headwater probe stale` takes the digest and reports which recorded results a change voided.

Graded by grader 0.5.0.
A campaign run in the absent arm, on claude-sonnet-5 at served version claude-sonnet-5.

## The verdicts

- HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer (sufficiency, expects answered)
    session L2-campaign-absent-p1-r1: satisfied — event 1: answered `absent`
    session L2-campaign-absent-p1-r10: satisfied — event 2: answered `absent`
    session L2-campaign-absent-p1-r2: satisfied — event 3: answered `absent`
    session L2-campaign-absent-p1-r3: satisfied — event 4: answered `absent`
    session L2-campaign-absent-p1-r4: satisfied — event 5: answered `absent`
    session L2-campaign-absent-p1-r5: satisfied — event 6: answered `absent`
    session L2-campaign-absent-p1-r6: satisfied — event 7: answered `absent`
    session L2-campaign-absent-p1-r7: satisfied — event 8: answered `absent`
    session L2-campaign-absent-p1-r8: satisfied — event 9: answered `absent`
    session L2-campaign-absent-p1-r9: satisfied — event 10: answered `absent`

## The rate, and the denominator it is over

10 of 10 graded sessions satisfied their expectation: 100.0%, in a 95% interval of 72.2% to 100.0%.
The denominator is the graded sessions and never the selected probes. 0 sessions reached no verdict, and a session with no verdict is outside both halves of that fraction.

An interval that overlaps the previous run's is variance and one that does not is drift. This is one arm, so it estimates no effect: an efficacy claim is a comparison of two results, and the arm each one recorded is on it.

## The comparisons this arm takes part in

What the governance changes. The treated arm is the `campaign` present arm and the control is the `campaign` absent arm, which removed the paths the `campaign` tier's `ablation` names and kept `docs/`.

- treated, `docs/probe-runs/tombstone-probe-pilot-of-2026-09-29-campaign-tier-present-arm-sufficiency.md`: 10 of 10 graded sessions satisfied their expectation, 100.0%, in a 95% interval of 72.2% to 100.0%. 0 sessions refused by the session itself.
- control, `docs/probe-runs/tombstone-probe-pilot-of-2026-09-29-campaign-tier-absent-arm-sufficiency.md`: 10 of 10 graded sessions satisfied their expectation, 100.0%, in a 95% interval of 72.2% to 100.0%. 0 sessions refused by the session itself.

The difference is +0.0 points, in a 95% Newcombe interval of -27.8 points to +27.8 points. The interval contains zero, so this run does not separate the two arms at the 5% level.
