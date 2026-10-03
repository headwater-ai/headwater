---
"headwater:generated": "probe_result. `headwater generate` writes this file, and `headwater generate --check` holds it. Edit the corpus, not this file."
id: HW-RESULT-second-campaign-pilot-of-2026-09-28-campaign-tier-present-arm-discovery
title: Probe result for second-campaign-pilot-of-2026-09-28-campaign-tier-present-arm-discovery
status: deprecated
status_since: 2026-09-28
summary: "The grade of the transcript `second-campaign-pilot-of-2026-09-28-campaign-tier-present-arm-discovery`, taken over the probes this corpus declares and the version of the grader that evaluated them."
last_verified: 2026-09-28
---

# The result of docs/probe-runs/second-campaign-pilot-of-2026-09-28-campaign-tier-present-arm-discovery.md

The transcript this result grades stands at `deprecated`, a state that ends its lifecycle, so the run it recorded is withdrawn. No figure below is a current finding, and this result states no direction at the 5% level.

A probe result is a function of committed inputs and of nothing else: the transcript at `docs/probe-runs/second-campaign-pilot-of-2026-09-28-campaign-tier-present-arm-discovery.md` and the state it stands at, the expectations the probes it names declare, the version of the grader that evaluated them, and for a compared arm the result of the other arm. Fetch those and this file comes back.

## The run this transcript recorded

A campaign run in the present arm, on claude-sonnet-5 at 2026-09-28.
served version claude-sonnet-5, tree sha256:d9aad3464b048f92f47d8c5779bffee5b728699fc710d7e0af2c66363e9b51b0, selection sha256:8ec515cf6d769b34f8fc840d00f902606dc575c74a4105c7599670b563be6ad5, read set sha256:3099b30a529a8aaa647dd29d7678bf8f22ac68c76030388aff2044074069ab05, seed 0, harness 0.4.0.
realized cost $0.88, which the adaptive layer reads as the cost of its own instrument.

2 events over 2 probes, in 2 sessions and 29 tool calls.

The engine confirmed the taxonomy, that every member of the run identity is present, the membership of every probe named, that no key outside the closed set appears, and that a realized cost was recorded. Present is not confirmed. This page names the lock, the selection and the read set the transcript recorded, and it compares none of them with the tree in front of a reader. `headwater generate` and `headwater probe stale` name what moved since the recording, and neither one fails for it. It graded nothing: a verdict is a function of this transcript, the expectations these probes declare and a grader version, and `headwater probe grade` is the verb that holds all three.

The selection this transcript names is `sha256:8ec515cf6d769b34f8fc840d00f902606dc575c74a4105c7599670b563be6ad5`, and the probes graded below are the probes it was planned over, as this corpus declares them now. A probe added to this corpus after the recording is not a session this run owed. The tree, the seed and the harness above are provenance and nothing compares them.

The `read_set` digest above covers every probe of the selection and every document one of them examines, by path and content. This file names it as provenance and compares it with nothing, and the same holds for the lock and the selection: an edit to a document, a new probe or a moved lock leaves these bytes alone. `headwater generate` names each result whose recorded lock, read set or selection moved, and `headwater probe stale` takes the digest and reports which recorded results a change voided. Neither one fails for it.

Graded by grader 0.5.0.
A campaign run in the present arm, on claude-sonnet-5 at served version claude-sonnet-5.

## The verdicts

- HW-PROBE-a-cold-agent-reaches-the-governing-document-through-the-corpus-descriptor (discovery, expects opened)
    session L6-campaign-present-p1-r1: satisfied — event 1, call 13: `Bash {"command":"cd /home/james/probe-batches/980-pilot2-2026-09-28/ws/L6-campaign-present-p1-r1 && sed -n '1,200p' docs/spec/12-check-layer.md 2>/dev/null | grep -n \"origin\\|Shape-origin\\|Document-origin\\|declared\" | head -60","description":"Search check-layer spec for origin/declaration terminology"}`
- HW-PROBE-the-pointer-this-corpus-offers-for-a-task-is-the-document-a-session-opens (discovery, expects opened)
    session L6-campaign-present-p2-r1: not satisfied — 12 recorded calls, and none named any of the 1 document

## The rate, and the denominator it is over

1 of 2 graded sessions satisfied their expectation: 50.0%, in a 95% interval of 9.5% to 90.5%.
The denominator is the graded sessions and never the selected probes. 0 sessions reached no verdict, and a session with no verdict is outside both halves of that fraction.

An interval that overlaps the previous run's is variance and one that does not is drift. This is one arm, so it estimates no effect: an efficacy claim is a comparison of two results, and the arm each one recorded is on it.

## The comparisons this arm takes part in

What the governance changes. The treated arm is the `campaign` present arm and the control is the `campaign` absent arm, which removed the paths the `campaign` tier's `ablation` names and kept `docs/`.

- treated, `docs/probe-runs/second-campaign-pilot-of-2026-09-28-campaign-tier-present-arm-discovery.md`: 1 of 2 graded sessions satisfied their expectation, 50.0%, in a 95% interval of 9.5% to 90.5%. 0 sessions refused by the session itself.
- control, `docs/probe-runs/second-campaign-pilot-of-2026-09-28-campaign-tier-absent-arm-discovery.md`: 0 of 2 graded sessions satisfied their expectation, 0.0%, in a 95% interval of 0.0% to 65.8%. 0 sessions refused by the session itself.

The difference is +50.0 points, in a 95% Newcombe interval of -27.3 points to +90.5 points. The transcript of at least one arm is not current, so this page states no direction at the 5% level.
