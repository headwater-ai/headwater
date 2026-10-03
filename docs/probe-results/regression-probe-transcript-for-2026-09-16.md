---
"headwater:generated": "probe_result. `headwater generate` writes this file, and `headwater generate --check` holds it. Edit the corpus, not this file."
id: HW-RESULT-regression-probe-transcript-for-2026-09-16
title: Probe result for regression-probe-transcript-for-2026-09-16
status: deprecated
status_since: 2026-09-19
summary: "The grade of the transcript `regression-probe-transcript-for-2026-09-16`, taken over the probes this corpus declares and the version of the grader that evaluated them."
last_verified: 2026-09-16
---

# The result of docs/probe-runs/regression-probe-transcript-for-2026-09-16.md

The transcript this result grades stands at `deprecated`, a state that ends its lifecycle, so the run it recorded is withdrawn. No figure below is a current finding, and this result states no direction at the 5% level.

A probe result is a function of committed inputs and of nothing else: the transcript at `docs/probe-runs/regression-probe-transcript-for-2026-09-16.md` and the state it stands at, the expectations the probes it names declare, the version of the grader that evaluated them, and for a compared arm the result of the other arm. Fetch those and this file comes back.

## The run this transcript recorded

A regression run in the present arm, on claude-haiku-4-5 at 2026-09-16.
served version claude-haiku-4-5-20251001, tree sha256:71a12f7cfbd2e0563bc4dd840c73e1a79606f579068dd3319b4fc931bfa476ef, selection sha256:aa4aaa7375349b5c8cc667a196819166519aac3f519d436f3f237afb7923530a, read set sha256:1dcd3c358af1b5c12d1e7f8f4f711a52631ac60c71bbd008bfbfa5c29315409d, seed 0, harness 0.2.0.
realized cost $1.66, which the adaptive layer reads as the cost of its own instrument.

8 events over 8 probes, in 8 sessions and 197 tool calls.

The engine confirmed the taxonomy, that every member of the run identity is present, the membership of every probe named, that no key outside the closed set appears, and that a realized cost was recorded. Present is not confirmed. This page names the lock, the selection and the read set the transcript recorded, and it compares none of them with the tree in front of a reader. `headwater generate` and `headwater probe stale` name what moved since the recording, and neither one fails for it. It graded nothing: a verdict is a function of this transcript, the expectations these probes declare and a grader version, and `headwater probe grade` is the verb that holds all three.

The selection this transcript names is `sha256:aa4aaa7375349b5c8cc667a196819166519aac3f519d436f3f237afb7923530a`, and the probes graded below are the probes it was planned over, as this corpus declares them now. A probe added to this corpus after the recording is not a session this run owed. The tree, the seed and the harness above are provenance and nothing compares them.

The `read_set` digest above covers every probe of the selection and every document one of them examines, by path and content. This file names it as provenance and compares it with nothing, and the same holds for the lock and the selection: an edit to a document, a new probe or a moved lock leaves these bytes alone. `headwater generate` names each result whose recorded lock, read set or selection moved, and `headwater probe stale` takes the digest and reports which recorded results a change voided. Neither one fails for it.

Graded by grader 0.5.0.
A regression run in the present arm, on claude-haiku-4-5 at served version claude-haiku-4-5-20251001.

## The verdicts

- HW-PROBE-a-cold-agent-reaches-the-governing-document-through-the-corpus-descriptor (discovery, expects opened)
    session regression-20260916-cold-current: not satisfied — 27 recorded calls, and none named any of the 1 document
- HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer (sufficiency, expects answered)
    session regression-20260916-tombstone: not satisfied — the session ended with no answer
- HW-PROBE-a-session-answers-from-the-register-without-opening-the-question-it-replaced (navigability, expects not_opened)
    session regression-20260916-register: satisfied — 4 recorded calls, and none named any of the 1 document
- HW-PROBE-a-session-records-an-unmeasured-claim-in-the-shape-this-corpus-checks (sufficiency, expects patched)
    session regression-20260916-unmeasured-claim: satisfied — event 4: `section.required.missing` reported nothing over `docs/obligations/0198-nothing-states-how-much-of-a-session-s-budget-the-standing-instructions-consume-before-work-starts.md`
- HW-PROBE-an-agent-reaches-the-adjudication-from-the-document-that-lost-it (navigability, expects opened)
    session regression-20260916-adjudication: satisfied — event 5, call 2: `Read /home/james/.claude/jobs/eb30b5a4/tmp/probe170/workspace-5/docs/decisions/0008-probe-cost-and-cadence.md`
- HW-PROBE-the-authoring-skill-reaches-an-agent-that-is-about-to-write-a-governed-document (discovery, expects opened)
    session regression-20260916-authoring-skill: not satisfied — 29 recorded calls, and none named any of the 1 document
- HW-PROBE-the-pointer-this-corpus-offers-for-a-task-is-the-document-a-session-opens (discovery, expects opened)
    session regression-20260916-pointer: not satisfied — 0 recorded calls, and none named any of the 1 document
- HW-PROBE-what-a-session-writes-points-back-at-the-ruling-it-rests-on (navigability, expects cited)
    session regression-20260916-points-back: satisfied — event 8: `docs/evaluations/why-corpus-counts-are-derived-not-stored.md` cites HW-DR-0049

## The rate, and the denominator it is over

4 of 8 graded sessions satisfied their expectation: 50.0%, in a 95% interval of 21.5% to 78.5%.
The denominator is the graded sessions and never the selected probes. 0 sessions reached no verdict, and a session with no verdict is outside both halves of that fraction.

An interval that overlaps the previous run's is variance and one that does not is drift. This is one arm, so it estimates no effect: an efficacy claim is a comparison of two results, and the arm each one recorded is on it.
