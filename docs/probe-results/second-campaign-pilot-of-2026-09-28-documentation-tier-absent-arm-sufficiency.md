---
"headwater:generated": "probe_result. `headwater generate` writes this file, and `headwater generate --check` holds it. Edit the corpus, not this file."
id: HW-RESULT-second-campaign-pilot-of-2026-09-28-documentation-tier-absent-arm-sufficiency
title: Probe result for second-campaign-pilot-of-2026-09-28-documentation-tier-absent-arm-sufficiency
status: current
status_since: 2026-09-28
summary: "The grade of the transcript `second-campaign-pilot-of-2026-09-28-documentation-tier-absent-arm-sufficiency`, taken over the probes this corpus declares and the version of the grader that evaluated them."
last_verified: 2026-09-28
---

# The result of docs/probe-runs/second-campaign-pilot-of-2026-09-28-documentation-tier-absent-arm-sufficiency.md

A probe result is a function of three committed inputs and of nothing else: the transcript at `docs/probe-runs/second-campaign-pilot-of-2026-09-28-documentation-tier-absent-arm-sufficiency.md`, the expectations the probes of this corpus declare, and the version of the grader that evaluated them. Fetch the three and this file comes back.

## The run this transcript recorded

A documentation run in the absent arm, on claude-sonnet-5 at 2026-09-28.
served version claude-sonnet-5, tree sha256:d9aad3464b048f92f47d8c5779bffee5b728699fc710d7e0af2c66363e9b51b0, selection sha256:1d93b514529314b74499196391c68b4b3d1de26853dd9d5ba7f3b455f54ed91d, read set sha256:7a76c0b28551210b534705cf2b69216c68b6543dba250fc82fb24861e4949f46, seed 0, harness 0.4.0.
realized cost $0.55, which the adaptive layer reads as the cost of its own instrument.
It was planned against taxonomy sha256:62144f6124525b73c49768d80ccdc41d7324fff0b39836367db4db85aa502cb8, and this tree carries another, and the read set of its probes moved since the recording. Each verdict below is what the session did over the documents it met, graded against the expectations this tree declares now, which can differ from the ones the session ran under. `headwater probe stale` names what moved.

4 events over 4 of the 10 probes this corpus declares, in 4 sessions and 15 tool calls.

The engine confirmed the taxonomy, that every member of the run identity is present, the membership of every probe named, that no key outside the closed set appears, and that a realized cost was recorded. Present is not confirmed: of the six members a plan fixes before a run, the lock is the one compared here. A lock that differs refuses the file only where this tree composes no read set over its probes, and a read set that differs marks the verdicts and keeps them. `headwater probe stale` compares the read set against the tree in front of it. It graded nothing: a verdict is a function of this transcript, the expectations these probes declare and a grader version, and `headwater probe grade` is the verb that holds all three.

**The selection this transcript names is not the selection this corpus composes.** The transcript names `sha256:1d93b514529314b74499196391c68b4b3d1de26853dd9d5ba7f3b455f54ed91d` and this corpus composes `sha256:50ed5ce43072d9f674a8dc52e1dcad84de4d649e39a77ce22179836188196b09`. The transcript's digest is the digest of 4 of the 10 probes this corpus composes, so the run was planned over that part of the selection, or the rest were added after it was recorded. Every verdict below is over that part, and a probe outside it is not a session this run owed. The tree, the seed and the harness above are provenance and nothing compares them.

The `read_set` digest above covers every probe of the selection and every document one of them examines, by path and content. It is recorded here, and this file names no digest the tree in front of a reader composes. Where that tree composes another one, a single sentence above says that the read set moved, and it moves these bytes once. A digest from that tree would move these bytes on every edit to a document the selection points at, and `generate --check` holds this file to its bytes, so the staleness of a measurement would stop a merge. `headwater probe stale` takes the digest and reports which recorded results a change voided.

Graded by grader 0.5.0.
A documentation run in the absent arm, on claude-sonnet-5 at served version claude-sonnet-5.

## The verdicts

- HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer (sufficiency, expects answered)
    session L3-documentation-absent-p1-r1: satisfied — event 1: answered `absent`
- HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted (sufficiency, expects answered)
    session L3-documentation-absent-p2-r1: not satisfied — event 2 answered `stamp`, and the probe expects `merge`
- HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request (sufficiency, expects answered)
    session L3-documentation-absent-p3-r1: not satisfied — event 3 answered `current`, which the probe does not declare
- HW-PROBE-a-session-records-an-unmeasured-claim-in-the-shape-this-corpus-checks (sufficiency, expects patched)
    session L3-documentation-absent-p4-r1: not satisfied — the session produced no artifact, so the oracle had nothing to read

## The rate, and the denominator it is over

1 of 4 graded sessions satisfied their expectation: 25.0%, in a 95% interval of 4.6% to 69.9%.
The denominator is the graded sessions and never the selected probes. 0 sessions reached no verdict, and a session with no verdict is outside both halves of that fraction.

An interval that overlaps the previous run's is variance and one that does not is drift. This is one arm, so it estimates no effect: an efficacy claim is a comparison of two results, and the arm each one recorded is on it.

## The comparisons this arm takes part in

What the documents change. The treated arm is the `campaign` absent arm and the control is the `documentation` absent arm. Both removed the governance, and only the control removed `docs/`, so this is the effect of the documents alone (spec 5).

- treated, `docs/probe-runs/second-campaign-pilot-of-2026-09-28-campaign-tier-absent-arm-sufficiency.md`: 3 of 4 graded sessions satisfied their expectation, 75.0%, in a 95% interval of 30.1% to 95.4%. 0 sessions refused by the session itself.
- control, `docs/probe-runs/second-campaign-pilot-of-2026-09-28-documentation-tier-absent-arm-sufficiency.md`: 1 of 4 graded sessions satisfied their expectation, 25.0%, in a 95% interval of 4.6% to 69.9%. 0 sessions refused by the session itself.

The difference is +50.0 points, in a 95% Newcombe interval of -13.5 points to +78.9 points. The interval contains zero, so this run does not separate the two arms at the 5% level.

What the documents and the governance change together. The treated arm is a present arm and the control is the `documentation` absent arm.

- treated, `docs/probe-runs/second-campaign-pilot-of-2026-09-28-campaign-tier-present-arm-sufficiency.md`: 2 of 4 graded sessions satisfied their expectation, 50.0%, in a 95% interval of 15.0% to 85.0%. 0 sessions refused by the session itself.
- control, `docs/probe-runs/second-campaign-pilot-of-2026-09-28-documentation-tier-absent-arm-sufficiency.md`: 1 of 4 graded sessions satisfied their expectation, 25.0%, in a 95% interval of 4.6% to 69.9%. 0 sessions refused by the session itself.

The difference is +25.0 points, in a 95% Newcombe interval of -32.0 points to +65.5 points. The interval contains zero, so this run does not separate the two arms at the 5% level.
