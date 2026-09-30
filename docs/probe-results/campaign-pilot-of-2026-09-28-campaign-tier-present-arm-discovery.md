---
"headwater:generated": "probe_result. `headwater generate` writes this file, and `headwater generate --check` holds it. Edit the corpus, not this file."
id: HW-RESULT-campaign-pilot-of-2026-09-28-campaign-tier-present-arm-discovery
title: Probe result for campaign-pilot-of-2026-09-28-campaign-tier-present-arm-discovery
status: current
status_since: 2026-09-28
summary: "The grade of the transcript `campaign-pilot-of-2026-09-28-campaign-tier-present-arm-discovery`, taken over the probes this corpus declares and the version of the grader that evaluated them."
last_verified: 2026-09-28
---

# The result of docs/probe-runs/campaign-pilot-of-2026-09-28-campaign-tier-present-arm-discovery.md

A probe result is a function of three committed inputs and of nothing else: the transcript at `docs/probe-runs/campaign-pilot-of-2026-09-28-campaign-tier-present-arm-discovery.md`, the expectations the probes of this corpus declare, and the version of the grader that evaluated them. Fetch the three and this file comes back.

## The run this transcript recorded

A campaign run in the present arm, on claude-sonnet-5 at 2026-09-28.
served version claude-sonnet-5, tree sha256:a25a8d3faf10c1e6e60b7962d4fd61866b5abef14f8266d7e7efcea1a107963f, selection sha256:8ec515cf6d769b34f8fc840d00f902606dc575c74a4105c7599670b563be6ad5, read set sha256:3e0a8778b6e5993b8faa4e0929404aa6169eb207f3dcbbb5f4e64365591b24c5, seed 0, harness 0.4.0.
realized cost $2.42, which the adaptive layer reads as the cost of its own instrument.
It was planned against taxonomy sha256:62144f6124525b73c49768d80ccdc41d7324fff0b39836367db4db85aa502cb8, and this tree carries another, and the read set of its probes moved since the recording. Each verdict below is what the session did over the documents it met, graded against the expectations this tree declares now, which can differ from the ones the session ran under. `headwater probe stale` names what moved.

6 events over 2 of the 10 probes this corpus declares, in 6 sessions and 75 tool calls.

The engine confirmed the taxonomy, that every member of the run identity is present, the membership of every probe named, that no key outside the closed set appears, and that a realized cost was recorded. Present is not confirmed: of the six members a plan fixes before a run, the lock is the one compared here. A lock that differs refuses the file only where this tree composes no read set over its probes, and a read set that differs marks the verdicts and keeps them. `headwater probe stale` compares the read set against the tree in front of it. It graded nothing: a verdict is a function of this transcript, the expectations these probes declare and a grader version, and `headwater probe grade` is the verb that holds all three.

**The selection this transcript names is not the selection this corpus composes.** The transcript names `sha256:8ec515cf6d769b34f8fc840d00f902606dc575c74a4105c7599670b563be6ad5` and this corpus composes `sha256:50ed5ce43072d9f674a8dc52e1dcad84de4d649e39a77ce22179836188196b09`. The transcript's digest is the digest of 2 of the 10 probes this corpus composes, so the run was planned over that part of the selection, or the rest were added after it was recorded. Every verdict below is over that part, and a probe outside it is not a session this run owed. The tree, the seed and the harness above are provenance and nothing compares them.

The `read_set` digest above covers every probe of the selection and every document one of them examines, by path and content. It is recorded here, and this file names no digest the tree in front of a reader composes. Where that tree composes another one, a single sentence above says that the read set moved, and it moves these bytes once. A digest from that tree would move these bytes on every edit to a document the selection points at, and `generate --check` holds this file to its bytes, so the staleness of a measurement would stop a merge. `headwater probe stale` takes the digest and reports which recorded results a change voided.

Graded by grader 0.5.0.
A campaign run in the present arm, on claude-sonnet-5 at served version claude-sonnet-5.

## The verdicts

- HW-PROBE-a-cold-agent-reaches-the-governing-document-through-the-corpus-descriptor (discovery, expects opened)
    session L6-campaign-present-p1-r1: not satisfied — 19 recorded calls, and none named any of the 1 document
    session L6-campaign-present-p1-r2: not satisfied — 15 recorded calls, and none named any of the 1 document
    session L6-campaign-present-p1-r3: not satisfied — 14 recorded calls, and none named any of the 1 document
- HW-PROBE-the-pointer-this-corpus-offers-for-a-task-is-the-document-a-session-opens (discovery, expects opened)
    session L6-campaign-present-p2-r1: not satisfied — 19 recorded calls, and none named any of the 1 document
    session L6-campaign-present-p2-r2: not satisfied — 3 recorded calls, and none named any of the 1 document
    session L6-campaign-present-p2-r3: not satisfied — 5 recorded calls, and none named any of the 1 document

## The rate, and the denominator it is over

0 of 6 graded sessions satisfied their expectation: 0.0%, in a 95% interval of 0.0% to 39.0%.
The denominator is the graded sessions and never the selected probes. 0 sessions reached no verdict, and a session with no verdict is outside both halves of that fraction.

An interval that overlaps the previous run's is variance and one that does not is drift. This is one arm, so it estimates no effect: an efficacy claim is a comparison of two results, and the arm each one recorded is on it.

## The comparisons this arm takes part in

What the governance changes. The treated arm is the `campaign` present arm and the control is the `campaign` absent arm, which removed the paths the `campaign` tier's `ablation` names and kept `docs/`.

- treated, `docs/probe-runs/campaign-pilot-of-2026-09-28-campaign-tier-present-arm-discovery.md`: 0 of 6 graded sessions satisfied their expectation, 0.0%, in a 95% interval of 0.0% to 39.0%. 0 sessions refused by the session itself.
- control, `docs/probe-runs/campaign-pilot-of-2026-09-28-campaign-tier-absent-arm-discovery.md`: 2 of 6 graded sessions satisfied their expectation, 33.3%, in a 95% interval of 9.7% to 70.0%. 0 sessions refused by the session itself.

The difference is -33.3 points, in a 95% Newcombe interval of -70.0 points to +12.3 points. The interval contains zero, so this run does not separate the two arms at the 5% level.
