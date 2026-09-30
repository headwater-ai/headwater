---
"headwater:generated": "probe_result. `headwater generate` writes this file, and `headwater generate --check` holds it. Edit the corpus, not this file."
id: HW-RESULT-second-campaign-pilot-of-2026-09-28-campaign-tier-absent-arm-navigability
title: Probe result for second-campaign-pilot-of-2026-09-28-campaign-tier-absent-arm-navigability
status: deprecated
status_since: 2026-09-28
summary: "The grade of the transcript `second-campaign-pilot-of-2026-09-28-campaign-tier-absent-arm-navigability`, taken over the probes this corpus declares and the version of the grader that evaluated them."
last_verified: 2026-09-28
---

# The result of docs/probe-runs/second-campaign-pilot-of-2026-09-28-campaign-tier-absent-arm-navigability.md

The transcript this result grades stands at `deprecated`, a state that ends its lifecycle, so the run it recorded is withdrawn. No figure below is a current finding, and this result states no direction at the 5% level.

A probe result is a function of three committed inputs and of nothing else: the transcript at `docs/probe-runs/second-campaign-pilot-of-2026-09-28-campaign-tier-absent-arm-navigability.md`, the expectations the probes of this corpus declare, and the version of the grader that evaluated them. Fetch the three and this file comes back.

## The run this transcript recorded

A campaign run in the absent arm, on claude-sonnet-5 at 2026-09-28.
served version claude-sonnet-5, tree sha256:d9aad3464b048f92f47d8c5779bffee5b728699fc710d7e0af2c66363e9b51b0, selection sha256:fc04ac718a7df0b14774c28174a93f9d44dea6305b9901ba6e2fe6556834db76, read set sha256:00323cb8f153c3f22044099b6eeca18c5787b93cdb0d89262d285b926703ee49, seed 0, harness 0.4.0.
realized cost $0.69, which the adaptive layer reads as the cost of its own instrument.
It was planned against taxonomy sha256:62144f6124525b73c49768d80ccdc41d7324fff0b39836367db4db85aa502cb8, and this tree carries another, and the read set of its probes moved since the recording. Each verdict below is what the session did over the documents it met, graded against the expectations this tree declares now, which can differ from the ones the session ran under. `headwater probe stale` names what moved.

3 events over 3 of the 10 probes this corpus declares, in 3 sessions and 18 tool calls.

The engine confirmed the taxonomy, that every member of the run identity is present, the membership of every probe named, that no key outside the closed set appears, and that a realized cost was recorded. Present is not confirmed: of the six members a plan fixes before a run, the lock is the one compared here. A lock that differs refuses the file only where this tree composes no read set over its probes, and a read set that differs marks the verdicts and keeps them. `headwater probe stale` compares the read set against the tree in front of it. It graded nothing: a verdict is a function of this transcript, the expectations these probes declare and a grader version, and `headwater probe grade` is the verb that holds all three.

**The selection this transcript names is not the selection this corpus composes.** The transcript names `sha256:fc04ac718a7df0b14774c28174a93f9d44dea6305b9901ba6e2fe6556834db76` and this corpus composes `sha256:50ed5ce43072d9f674a8dc52e1dcad84de4d649e39a77ce22179836188196b09`. The transcript's digest is the digest of 3 of the 10 probes this corpus composes, so the run was planned over that part of the selection, or the rest were added after it was recorded. Every verdict below is over that part, and a probe outside it is not a session this run owed. The tree, the seed and the harness above are provenance and nothing compares them.

The `read_set` digest above covers every probe of the selection and every document one of them examines, by path and content. It is recorded here, and this file names no digest the tree in front of a reader composes. Where that tree composes another one, a single sentence above says that the read set moved, and it moves these bytes once. A digest from that tree would move these bytes on every edit to a document the selection points at, and `generate --check` holds this file to its bytes, so the staleness of a measurement would stop a merge. `headwater probe stale` takes the digest and reports which recorded results a change voided.

Graded by grader 0.5.0.
A campaign run in the absent arm, on claude-sonnet-5 at served version claude-sonnet-5.

## The verdicts

- HW-PROBE-a-session-answers-from-the-register-without-opening-the-question-it-replaced (navigability, expects not_opened)
    session L5-campaign-absent-p2-r1: satisfied — 4 recorded calls, and none named any of the 1 document
- HW-PROBE-an-agent-reaches-the-adjudication-from-the-document-that-lost-it (navigability, expects opened)
    session L5-campaign-absent-p1-r1: satisfied — event 1, call 2: `Read /home/james/probe-batches/980-pilot2-2026-09-28/ws/L5-campaign-absent-p1-r1/docs/decisions/0008-probe-cost-and-cadence.md`
- HW-PROBE-what-a-session-writes-points-back-at-the-ruling-it-rests-on (navigability, expects cited)
    session L5-campaign-absent-p3-r1: not satisfied — 0 produced artifacts, and none cites any of the 1 identifier

## The rate, and the denominator it is over

2 of 3 graded sessions satisfied their expectation: 66.7%, in a 95% interval of 20.8% to 93.9%.
The denominator is the graded sessions and never the selected probes. 0 sessions reached no verdict, and a session with no verdict is outside both halves of that fraction.

An interval that overlaps the previous run's is variance and one that does not is drift. This is one arm, so it estimates no effect: an efficacy claim is a comparison of two results, and the arm each one recorded is on it.

## The comparisons this arm takes part in

What the governance changes. The treated arm is the `campaign` present arm and the control is the `campaign` absent arm, which removed the paths the `campaign` tier's `ablation` names and kept `docs/`.

- treated, `docs/probe-runs/second-campaign-pilot-of-2026-09-28-campaign-tier-present-arm-navigability.md`: 3 of 3 graded sessions satisfied their expectation, 100.0%, in a 95% interval of 43.9% to 100.0%. 0 sessions refused by the session itself.
- control, `docs/probe-runs/second-campaign-pilot-of-2026-09-28-campaign-tier-absent-arm-navigability.md`: 2 of 3 graded sessions satisfied their expectation, 66.7%, in a 95% interval of 20.8% to 93.9%. 0 sessions refused by the session itself.

The difference is +33.3 points, in a 95% Newcombe interval of -29.1 points to +79.2 points. The transcript of at least one arm is not current, so this page states no direction at the 5% level.
