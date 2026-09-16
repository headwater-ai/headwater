---
"headwater:generated": "probe_result. `headwater generate` writes this file, and `headwater generate --check` holds it. Edit the corpus, not this file."
id: HW-RESULT-regression-probe-transcript-for-2026-09-17
title: Probe result for regression-probe-transcript-for-2026-09-17
status: current
status_since: 2026-09-16
summary: "The grade of the transcript `regression-probe-transcript-for-2026-09-17`, taken over the probes this corpus declares and the version of the grader that evaluated them."
last_verified: 2026-09-16
---

# The result of docs/probe-runs/regression-probe-transcript-for-2026-09-17.md

A probe result is a function of three committed inputs and of nothing else: the transcript at `docs/probe-runs/regression-probe-transcript-for-2026-09-17.md`, the expectations the probes of this corpus declare, and the version of the grader that evaluated them. Fetch the three and this file comes back.

## The run this transcript recorded

A regression run in the present arm, on claude-sonnet-5 at 2026-09-16.
served version claude-sonnet-5, tree sha256:56e6e8bcf645e06dbb50ecfb4050783c7afc0e876585e7103aae58ea17877501, selection sha256:aa4aaa7375349b5c8cc667a196819166519aac3f519d436f3f237afb7923530a, read set sha256:8f891e3c74fd78d9bc1e9fab3509ff6b89eb9a132d51ca1ead9a8c6feb746b27, seed 0, harness 0.2.0.
realized cost $3.90, which the adaptive layer reads as the cost of its own instrument.

8 events over 8 of the 8 probes this corpus declares, in 8 sessions and 121 tool calls.

The engine confirmed the taxonomy, that every member of the run identity is present, the membership of every probe named, that no key outside the closed set appears, and that a realized cost was recorded. Present is not confirmed: of the six members a plan fixes before a run, the lock is the one compared here, and it refuses the file. `headwater probe stale` compares the read set against the tree in front of it. It graded nothing: a verdict is a function of this transcript, the expectations these probes declare and a grader version, and `headwater probe grade` is the verb that holds all three.

The selection this transcript names is the selection this corpus composes, so the probes graded below are the probes this run was planned over. The tree, the seed and the harness above are provenance: nothing compares them, and the tree in particular is not compared because a result that tracked it would need rewriting after an edit to any document of this corpus.

The `read_set` digest above covers every probe of the selection and every document one of them examines, by path and content. It is recorded here and compared nowhere in this file. A comparison against the tree in front of a reader would move these bytes on every edit to a document the selection points at, and `generate --check` holds this file to its bytes, so the staleness of a measurement would stop a merge. `headwater probe stale` takes the digest and reports which recorded results a change voided.

Graded by grader 0.2.0.
A regression run in the present arm, on claude-sonnet-5 at served version claude-sonnet-5.

## The verdicts

- HW-PROBE-a-cold-agent-reaches-the-governing-document-through-the-corpus-descriptor (discovery, expects opened)
    session regression-20260917-cold-current: not satisfied — 18 recorded calls, and none named any of the 1 document
- HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer (sufficiency, expects answered)
    session regression-20260917-tombstone: not satisfied — the session ended with no answer
- HW-PROBE-a-session-answers-from-the-register-without-opening-the-question-it-replaced (navigability, expects not_opened)
    session regression-20260917-register: satisfied — 7 recorded calls, and none named any of the 1 document
- HW-PROBE-a-session-records-an-unmeasured-claim-in-the-shape-this-corpus-checks (sufficiency, expects patched)
    session regression-20260917-unmeasured-claim: satisfied — event 7: `section.required.missing` reported nothing over `docs/obligations/0198-nothing-states-how-much-of-a-session-s-budget-the-standing-instructions-consume-before-work-starts.md`
- HW-PROBE-an-agent-reaches-the-adjudication-from-the-document-that-lost-it (navigability, expects opened)
    session regression-20260917-adjudication: not satisfied — 4 recorded calls, and none named any of the 1 document
- HW-PROBE-the-authoring-skill-reaches-an-agent-that-is-about-to-write-a-governed-document (discovery, expects opened)
    session regression-20260917-authoring-skill: not satisfied — 11 recorded calls, and none named any of the 1 document
- HW-PROBE-the-pointer-this-corpus-offers-for-a-task-is-the-document-a-session-opens (discovery, expects opened)
    session regression-20260917-pointer: not satisfied — 25 recorded calls, and none named any of the 1 document
- HW-PROBE-what-a-session-writes-points-back-at-the-ruling-it-rests-on (navigability, expects cited)
    session regression-20260917-points-back: satisfied — event 8: `docs/evaluations/why-corpus-counts-are-derived-not-stored.md` cites HW-DR-0049

## The rate, and the denominator it is over

3 of 8 graded sessions satisfied their expectation: 37.5%, in a 95% interval of 13.7% to 69.4%.
The denominator is the graded sessions and never the selected probes. 0 sessions reached no verdict, and a session with no verdict is outside both halves of that fraction.

An interval that overlaps the previous run's is variance and one that does not is drift. This is one arm, so it estimates no effect: an efficacy claim is a comparison of two results, and the arm each one recorded is on it.
