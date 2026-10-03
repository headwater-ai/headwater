---
"headwater:generated": "probe_result. `headwater generate` writes this file, and `headwater generate --check` holds it. Edit the corpus, not this file."
id: HW-RESULT-regression-probe-transcript-for-2026-09-11
title: Probe result for regression-probe-transcript-for-2026-09-11
status: deprecated
status_since: 2026-09-11
summary: "The grade of the transcript `regression-probe-transcript-for-2026-09-11`, taken over the probes this corpus declares and the version of the grader that evaluated them."
last_verified: 2026-09-11
---

# The result of docs/probe-runs/regression-probe-transcript-for-2026-09-11.md

The transcript this result grades stands at `deprecated`, a state that ends its lifecycle, so the run it recorded is withdrawn. No figure below is a current finding, and this result states no direction at the 5% level.

A probe result is a function of committed inputs and of nothing else: the transcript at `docs/probe-runs/regression-probe-transcript-for-2026-09-11.md` and the state it stands at, the expectations the probes it names declare, the version of the grader that evaluated them, and for a compared arm the result of the other arm. Fetch those and this file comes back.

## The run this transcript recorded

A regression run in the present arm, on claude-haiku-4-5 at 2026-09-11.
served version claude-haiku-4-5-20251001, tree sha256:57e3ccceda009a3f910ddb9a5b01a81079c399ffdd474719df8fcc7bf9bf8cb4, selection sha256:2ae2482aea0cf2c00e83fac793d72d09c287514680ef632d535f1607d12a913a, read set sha256:4552fe4c8345b84c96dfb90fcaf048fcdd9284642c4771077b49077369b2ec17, seed 0, harness 0.1.2.
realized cost $0.43, which the adaptive layer reads as the cost of its own instrument.

4 events over 4 probes, in 4 sessions and 55 tool calls.

The engine confirmed the taxonomy, that every member of the run identity is present, the membership of every probe named, that no key outside the closed set appears, and that a realized cost was recorded. Present is not confirmed. This page names the lock, the selection and the read set the transcript recorded, and it compares none of them with the tree in front of a reader. `headwater generate` and `headwater probe stale` name what moved since the recording, and neither one fails for it. It graded nothing: a verdict is a function of this transcript, the expectations these probes declare and a grader version, and `headwater probe grade` is the verb that holds all three.

The selection this transcript names is `sha256:2ae2482aea0cf2c00e83fac793d72d09c287514680ef632d535f1607d12a913a`, and the probes graded below are the probes it was planned over, as this corpus declares them now. A probe added to this corpus after the recording is not a session this run owed. The tree, the seed and the harness above are provenance and nothing compares them.

The `read_set` digest above covers every probe of the selection and every document one of them examines, by path and content. This file names it as provenance and compares it with nothing, and the same holds for the lock and the selection: an edit to a document, a new probe or a moved lock leaves these bytes alone. `headwater generate` names each result whose recorded lock, read set or selection moved, and `headwater probe stale` takes the digest and reports which recorded results a change voided. Neither one fails for it.

Graded by grader 0.5.0.
A regression run in the present arm, on claude-haiku-4-5 at served version claude-haiku-4-5-20251001.

## The verdicts

- HW-PROBE-a-cold-agent-reaches-the-governing-document-through-the-corpus-descriptor (discovery, expects opened)
    session regression-20260911-cold-current: not satisfied — 37 recorded calls, and none named any of the 1 document
- HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer (sufficiency, expects answered)
    session regression-20260911-tombstone-current: not satisfied — event 2 answered `present`, and the probe expects `absent`
- HW-PROBE-an-agent-reaches-the-adjudication-from-the-document-that-lost-it (navigability, expects opened)
    session regression-20260911-adjudication-current: not satisfied — 1 recorded call, and none named any of the 2 documents
- HW-PROBE-the-authoring-skill-reaches-an-agent-that-is-about-to-write-a-governed-document (discovery, expects opened)
    session regression-20260911-authoring-clean: not satisfied — 3 recorded calls, and none named any of the 1 document

## The rate, and the denominator it is over

0 of 4 graded sessions satisfied their expectation: 0.0%, in a 95% interval of 0.0% to 49.0%.
The denominator is the graded sessions and never the selected probes. 0 sessions reached no verdict, and a session with no verdict is outside both halves of that fraction.

An interval that overlaps the previous run's is variance and one that does not is drift. This is one arm, so it estimates no effect: an efficacy claim is a comparison of two results, and the arm each one recorded is on it.
