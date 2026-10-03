---
"headwater:generated": "probe_result. `headwater generate` writes this file, and `headwater generate --check` holds it. Edit the corpus, not this file."
id: HW-RESULT-campaign-pilot-of-2026-09-28-campaign-tier-present-arm-navigability
title: Probe result for campaign-pilot-of-2026-09-28-campaign-tier-present-arm-navigability
status: deprecated
status_since: 2026-09-28
summary: "The grade of the transcript `campaign-pilot-of-2026-09-28-campaign-tier-present-arm-navigability`, taken over the probes this corpus declares and the version of the grader that evaluated them."
last_verified: 2026-09-28
---

# The result of docs/probe-runs/campaign-pilot-of-2026-09-28-campaign-tier-present-arm-navigability.md

The transcript this result grades stands at `deprecated`, a state that ends its lifecycle, so the run it recorded is withdrawn. No figure below is a current finding, and this result states no direction at the 5% level.

A probe result is a function of committed inputs and of nothing else: the transcript at `docs/probe-runs/campaign-pilot-of-2026-09-28-campaign-tier-present-arm-navigability.md` and the state it stands at, the expectations the probes it names declare, the version of the grader that evaluated them, and for a compared arm the result of the other arm. Fetch those and this file comes back.

## The run this transcript recorded

A campaign run in the present arm, on claude-sonnet-5 at 2026-09-28.
served version claude-sonnet-5, tree sha256:a25a8d3faf10c1e6e60b7962d4fd61866b5abef14f8266d7e7efcea1a107963f, selection sha256:fc04ac718a7df0b14774c28174a93f9d44dea6305b9901ba6e2fe6556834db76, read set sha256:8929d61762fc7e209ea77b36f1abc73da6d2ef947f4f79a2dd4103c336128209, seed 0, harness 0.4.0.
realized cost $2.78, which the adaptive layer reads as the cost of its own instrument.

9 events over 3 probes, in 9 sessions and 75 tool calls.

The engine confirmed the taxonomy, that every member of the run identity is present, the membership of every probe named, that no key outside the closed set appears, and that a realized cost was recorded. Present is not confirmed. This page names the lock, the selection and the read set the transcript recorded, and it compares none of them with the tree in front of a reader. `headwater generate` and `headwater probe stale` name what moved since the recording, and neither one fails for it. It graded nothing: a verdict is a function of this transcript, the expectations these probes declare and a grader version, and `headwater probe grade` is the verb that holds all three.

The selection this transcript names is `sha256:fc04ac718a7df0b14774c28174a93f9d44dea6305b9901ba6e2fe6556834db76`, and the probes graded below are the probes it was planned over, as this corpus declares them now. A probe added to this corpus after the recording is not a session this run owed. The tree, the seed and the harness above are provenance and nothing compares them.

The `read_set` digest above covers every probe of the selection and every document one of them examines, by path and content. This file names it as provenance and compares it with nothing, and the same holds for the lock and the selection: an edit to a document, a new probe or a moved lock leaves these bytes alone. `headwater generate` names each result whose recorded lock, read set or selection moved, and `headwater probe stale` takes the digest and reports which recorded results a change voided. Neither one fails for it.

Graded by grader 0.5.0.
A campaign run in the present arm, on claude-sonnet-5 at served version claude-sonnet-5.

## The verdicts

- HW-PROBE-a-session-answers-from-the-register-without-opening-the-question-it-replaced (navigability, expects not_opened)
    session L4-campaign-present-p2-r1: satisfied — 5 recorded calls, and none named any of the 1 document
    session L4-campaign-present-p2-r2: satisfied — 6 recorded calls, and none named any of the 1 document
    session L4-campaign-present-p2-r3: satisfied — 7 recorded calls, and none named any of the 1 document
- HW-PROBE-an-agent-reaches-the-adjudication-from-the-document-that-lost-it (navigability, expects opened)
    session L4-campaign-present-p1-r1: satisfied — event 1, call 4: `Read /home/james/probe-batches/980-pilot-2026-09-28/ws/L4-campaign-present-p1-r1/docs/decisions/0008-probe-cost-and-cadence.md`
    session L4-campaign-present-p1-r2: satisfied — event 2, call 4: `Read /home/james/probe-batches/980-pilot-2026-09-28/ws/L4-campaign-present-p1-r2/docs/decisions/0008-probe-cost-and-cadence.md`
    session L4-campaign-present-p1-r3: satisfied — event 3, call 6: `Read /home/james/probe-batches/980-pilot-2026-09-28/ws/L4-campaign-present-p1-r3/docs/decisions/0008-probe-cost-and-cadence.md`
- HW-PROBE-what-a-session-writes-points-back-at-the-ruling-it-rests-on (navigability, expects cited)
    session L4-campaign-present-p3-r1: not satisfied — 0 produced artifacts, and none cites any of the 1 identifier
    session L4-campaign-present-p3-r2: satisfied — event 8: `docs/evaluations/why-corpus-counts-are-derived-not-stored.md` cites HW-DR-0049
    session L4-campaign-present-p3-r3: not satisfied — 0 produced artifacts, and none cites any of the 1 identifier

## The rate, and the denominator it is over

7 of 9 graded sessions satisfied their expectation: 77.8%, in a 95% interval of 45.3% to 93.7%.
The denominator is the graded sessions and never the selected probes. 0 sessions reached no verdict, and a session with no verdict is outside both halves of that fraction.

An interval that overlaps the previous run's is variance and one that does not is drift. This is one arm, so it estimates no effect: an efficacy claim is a comparison of two results, and the arm each one recorded is on it.

## The comparisons this arm takes part in

What the governance changes. The treated arm is the `campaign` present arm and the control is the `campaign` absent arm, which removed the paths the `campaign` tier's `ablation` names and kept `docs/`.

- treated, `docs/probe-runs/campaign-pilot-of-2026-09-28-campaign-tier-present-arm-navigability.md`: 7 of 9 graded sessions satisfied their expectation, 77.8%, in a 95% interval of 45.3% to 93.7%. 0 sessions refused by the session itself.
- control, `docs/probe-runs/campaign-pilot-of-2026-09-28-campaign-tier-absent-arm-navigability.md`: 5 of 9 graded sessions satisfied their expectation, 55.6%, in a 95% interval of 26.7% to 81.1%. 0 sessions refused by the session itself.

The difference is +22.2 points, in a 95% Newcombe interval of -19.1 points to +55.2 points. The transcript of at least one arm is not current, so this page states no direction at the 5% level.
