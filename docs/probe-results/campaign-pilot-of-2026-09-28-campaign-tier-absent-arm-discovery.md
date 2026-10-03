---
"headwater:generated": "probe_result. `headwater generate` writes this file, and `headwater generate --check` holds it. Edit the corpus, not this file."
id: HW-RESULT-campaign-pilot-of-2026-09-28-campaign-tier-absent-arm-discovery
title: Probe result for campaign-pilot-of-2026-09-28-campaign-tier-absent-arm-discovery
status: deprecated
status_since: 2026-09-28
summary: "The grade of the transcript `campaign-pilot-of-2026-09-28-campaign-tier-absent-arm-discovery`, taken over the probes this corpus declares and the version of the grader that evaluated them."
last_verified: 2026-09-28
---

# The result of docs/probe-runs/campaign-pilot-of-2026-09-28-campaign-tier-absent-arm-discovery.md

The transcript this result grades stands at `deprecated`, a state that ends its lifecycle, so the run it recorded is withdrawn. No figure below is a current finding, and this result states no direction at the 5% level.

A probe result is a function of committed inputs and of nothing else: the transcript at `docs/probe-runs/campaign-pilot-of-2026-09-28-campaign-tier-absent-arm-discovery.md` and the state it stands at, the expectations the probes it names declare, the version of the grader that evaluated them, and for a compared arm the result of the other arm. Fetch those and this file comes back.

## The run this transcript recorded

A campaign run in the absent arm, on claude-sonnet-5 at 2026-09-28.
served version claude-sonnet-5, tree sha256:a25a8d3faf10c1e6e60b7962d4fd61866b5abef14f8266d7e7efcea1a107963f, selection sha256:8ec515cf6d769b34f8fc840d00f902606dc575c74a4105c7599670b563be6ad5, read set sha256:3e0a8778b6e5993b8faa4e0929404aa6169eb207f3dcbbb5f4e64365591b24c5, seed 0, harness 0.4.0.
realized cost $3.18, which the adaptive layer reads as the cost of its own instrument.

6 events over 2 probes, in 6 sessions and 128 tool calls.

The engine confirmed the taxonomy, that every member of the run identity is present, the membership of every probe named, that no key outside the closed set appears, and that a realized cost was recorded. Present is not confirmed. This page names the lock, the selection and the read set the transcript recorded, and it compares none of them with the tree in front of a reader. `headwater generate` and `headwater probe stale` name what moved since the recording, and neither one fails for it. It graded nothing: a verdict is a function of this transcript, the expectations these probes declare and a grader version, and `headwater probe grade` is the verb that holds all three.

The selection this transcript names is `sha256:8ec515cf6d769b34f8fc840d00f902606dc575c74a4105c7599670b563be6ad5`, and the probes graded below are the probes it was planned over, as this corpus declares them now. A probe added to this corpus after the recording is not a session this run owed. The tree, the seed and the harness above are provenance and nothing compares them.

The `read_set` digest above covers every probe of the selection and every document one of them examines, by path and content. This file names it as provenance and compares it with nothing, and the same holds for the lock and the selection: an edit to a document, a new probe or a moved lock leaves these bytes alone. `headwater generate` names each result whose recorded lock, read set or selection moved, and `headwater probe stale` takes the digest and reports which recorded results a change voided. Neither one fails for it.

Graded by grader 0.5.0.
A campaign run in the absent arm, on claude-sonnet-5 at served version claude-sonnet-5.

## The verdicts

- HW-PROBE-a-cold-agent-reaches-the-governing-document-through-the-corpus-descriptor (discovery, expects opened)
    session L7-campaign-absent-p1-r1: satisfied — event 1, call 19: `Bash {"command":"ls docs/spec/ | head -30 && echo \"---\" && sed -n '1,50p' docs/spec/12-check-layer.md 2>/dev/null"}`
    session L7-campaign-absent-p1-r2: satisfied — event 2, call 8: `Bash {"command":"sed -n '1,400p' docs/spec/12-check-layer.md | grep -n \"summary\\|severity\\|rule\" -i | head -80"}`
    session L7-campaign-absent-p1-r3: not satisfied — 11 recorded calls, and none named any of the 1 document
- HW-PROBE-the-pointer-this-corpus-offers-for-a-task-is-the-document-a-session-opens (discovery, expects opened)
    session L7-campaign-absent-p2-r1: not satisfied — 11 recorded calls, and none named any of the 1 document
    session L7-campaign-absent-p2-r2: not satisfied — 8 recorded calls, and none named any of the 1 document
    session L7-campaign-absent-p2-r3: not satisfied — 18 recorded calls, and none named any of the 1 document

## The rate, and the denominator it is over

2 of 6 graded sessions satisfied their expectation: 33.3%, in a 95% interval of 9.7% to 70.0%.
The denominator is the graded sessions and never the selected probes. 0 sessions reached no verdict, and a session with no verdict is outside both halves of that fraction.

An interval that overlaps the previous run's is variance and one that does not is drift. This is one arm, so it estimates no effect: an efficacy claim is a comparison of two results, and the arm each one recorded is on it.

## The comparisons this arm takes part in

What the governance changes. The treated arm is the `campaign` present arm and the control is the `campaign` absent arm, which removed the paths the `campaign` tier's `ablation` names and kept `docs/`.

- treated, `docs/probe-runs/campaign-pilot-of-2026-09-28-campaign-tier-present-arm-discovery.md`: 0 of 6 graded sessions satisfied their expectation, 0.0%, in a 95% interval of 0.0% to 39.0%. 0 sessions refused by the session itself.
- control, `docs/probe-runs/campaign-pilot-of-2026-09-28-campaign-tier-absent-arm-discovery.md`: 2 of 6 graded sessions satisfied their expectation, 33.3%, in a 95% interval of 9.7% to 70.0%. 0 sessions refused by the session itself.

The difference is -33.3 points, in a 95% Newcombe interval of -70.0 points to +12.3 points. The transcript of at least one arm is not current, so this page states no direction at the 5% level.
