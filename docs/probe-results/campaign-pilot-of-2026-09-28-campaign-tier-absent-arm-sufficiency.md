---
"headwater:generated": "probe_result. `headwater generate` writes this file, and `headwater generate --check` holds it. Edit the corpus, not this file."
id: HW-RESULT-campaign-pilot-of-2026-09-28-campaign-tier-absent-arm-sufficiency
title: Probe result for campaign-pilot-of-2026-09-28-campaign-tier-absent-arm-sufficiency
status: current
status_since: 2026-09-28
summary: "The grade of the transcript `campaign-pilot-of-2026-09-28-campaign-tier-absent-arm-sufficiency`, taken over the probes this corpus declares and the version of the grader that evaluated them."
last_verified: 2026-09-28
---

# The result of docs/probe-runs/campaign-pilot-of-2026-09-28-campaign-tier-absent-arm-sufficiency.md

A probe result is a function of three committed inputs and of nothing else: the transcript at `docs/probe-runs/campaign-pilot-of-2026-09-28-campaign-tier-absent-arm-sufficiency.md`, the expectations the probes of this corpus declare, and the version of the grader that evaluated them. Fetch the three and this file comes back.

## The run this transcript recorded

A campaign run in the absent arm, on claude-sonnet-5 at 2026-09-28.
served version claude-sonnet-5, tree sha256:a25a8d3faf10c1e6e60b7962d4fd61866b5abef14f8266d7e7efcea1a107963f, selection sha256:1d93b514529314b74499196391c68b4b3d1de26853dd9d5ba7f3b455f54ed91d, read set sha256:644c0ee478dc2a9bddfbd5193e8a0a3a493b5c7c8a1d452ecc82288cce3f6131, seed 0, harness 0.4.0.
realized cost $2.19, which the adaptive layer reads as the cost of its own instrument.

12 events over 4 of the 10 probes this corpus declares, in 12 sessions and 53 tool calls.

The engine confirmed the taxonomy, that every member of the run identity is present, the membership of every probe named, that no key outside the closed set appears, and that a realized cost was recorded. Present is not confirmed: of the six members a plan fixes before a run, the lock is the one compared here, and it refuses the file. `headwater probe stale` compares the read set against the tree in front of it. It graded nothing: a verdict is a function of this transcript, the expectations these probes declare and a grader version, and `headwater probe grade` is the verb that holds all three.

**The selection this transcript names is not the selection this corpus composes.** The transcript names `sha256:1d93b514529314b74499196391c68b4b3d1de26853dd9d5ba7f3b455f54ed91d` and this corpus composes `sha256:50ed5ce43072d9f674a8dc52e1dcad84de4d649e39a77ce22179836188196b09`. The transcript's digest is the digest of 4 of the 10 probes this corpus composes, so the run was planned over that part of the selection, or the rest were added after it was recorded. Every verdict below is over that part, and a probe outside it is not a session this run owed. The tree, the seed and the harness above are provenance and nothing compares them.

The `read_set` digest above covers every probe of the selection and every document one of them examines, by path and content. It is recorded here and compared nowhere in this file. A comparison against the tree in front of a reader would move these bytes on every edit to a document the selection points at, and `generate --check` holds this file to its bytes, so the staleness of a measurement would stop a merge. `headwater probe stale` takes the digest and reports which recorded results a change voided.

Graded by grader 0.4.0.
A campaign run in the absent arm, on claude-sonnet-5 at served version claude-sonnet-5.

## The verdicts

- HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer (sufficiency, expects answered)
    session L2-campaign-absent-p1-r1: satisfied — event 1: answered `absent`
    session L2-campaign-absent-p1-r2: not satisfied — the session ended with no answer
    session L2-campaign-absent-p1-r3: satisfied — event 3: answered `absent`
- HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted (sufficiency, expects answered)
    session L2-campaign-absent-p2-r1: not satisfied — event 4 answered `stamp`, and the probe expects `merge`
    session L2-campaign-absent-p2-r2: satisfied — event 5: answered `merge`
    session L2-campaign-absent-p2-r3: satisfied — event 6: answered `merge`
- HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request (sufficiency, expects answered)
    session L2-campaign-absent-p3-r1: satisfied — event 7: answered `current`
    session L2-campaign-absent-p3-r2: satisfied — event 8: answered `current`
    session L2-campaign-absent-p3-r3: satisfied — event 9: answered `current`
- HW-PROBE-a-session-records-an-unmeasured-claim-in-the-shape-this-corpus-checks (sufficiency, expects patched)
    session L2-campaign-absent-p4-r1: not satisfied — `section.required.missing` reported over each of the 0 produced artifacts
    session L2-campaign-absent-p4-r2: not satisfied — `section.required.missing` reported over each of the 0 produced artifacts
    session L2-campaign-absent-p4-r3: not satisfied — `section.required.missing` reported over each of the 0 produced artifacts

## The rate, and the denominator it is over

7 of 12 graded sessions satisfied their expectation: 58.3%, in a 95% interval of 32.0% to 80.7%.
The denominator is the graded sessions and never the selected probes. 0 sessions reached no verdict, and a session with no verdict is outside both halves of that fraction.

An interval that overlaps the previous run's is variance and one that does not is drift. This is one arm, so it estimates no effect: an efficacy claim is a comparison of two results, and the arm each one recorded is on it.

## The comparisons this arm takes part in

What the governance changes. The treated arm is the `campaign` present arm and the control is the `campaign` absent arm, which removed the paths the `campaign` tier's `ablation` names and kept `docs/`.

- treated, `docs/probe-runs/campaign-pilot-of-2026-09-28-campaign-tier-present-arm-sufficiency.md`: 2 of 12 graded sessions satisfied their expectation, 16.7%, in a 95% interval of 4.7% to 44.8%. 0 sessions refused by the session itself.
- control, `docs/probe-runs/campaign-pilot-of-2026-09-28-campaign-tier-absent-arm-sufficiency.md`: 7 of 12 graded sessions satisfied their expectation, 58.3%, in a 95% interval of 32.0% to 80.7%. 0 sessions refused by the session itself.

The difference is -41.7 points, in a 95% Newcombe interval of -67.0 points to -3.1 points. The interval is below zero, so the treated arm satisfied less often at the 5% level.

What the documents change. The treated arm is the `campaign` absent arm and the control is the `documentation` absent arm. Both removed the governance, and only the control removed `docs/`, so this is the effect of the documents alone (spec 5).

- treated, `docs/probe-runs/campaign-pilot-of-2026-09-28-campaign-tier-absent-arm-sufficiency.md`: 7 of 12 graded sessions satisfied their expectation, 58.3%, in a 95% interval of 32.0% to 80.7%. 0 sessions refused by the session itself.
- control, `docs/probe-runs/campaign-pilot-of-2026-09-28-documentation-tier-absent-arm-sufficiency.md`: 3 of 12 graded sessions satisfied their expectation, 25.0%, in a 95% interval of 8.9% to 53.2%. 0 sessions refused by the session itself.

The difference is +33.3 points, in a 95% Newcombe interval of -5.3 points to +60.9 points. The interval contains zero, so this run does not separate the two arms at the 5% level.
