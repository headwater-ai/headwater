---
"headwater:generated": "probe_result. `headwater generate` writes this file, and `headwater generate --check` holds it. Edit the corpus, not this file."
id: HW-RESULT-regression-of-2026-10-03-regression-tier-present-arm-sufficiency
title: Probe result for regression-of-2026-10-03-regression-tier-present-arm-sufficiency
status: current
status_since: 2026-10-03
summary: "The grade of the transcript `regression-of-2026-10-03-regression-tier-present-arm-sufficiency`, taken over the probes this corpus declares and the version of the grader that evaluated them."
last_verified: 2026-10-03
---

# The result of docs/probe-runs/regression-of-2026-10-03-regression-tier-present-arm-sufficiency.md

A probe result is a function of committed inputs and of nothing else: the transcript at `docs/probe-runs/regression-of-2026-10-03-regression-tier-present-arm-sufficiency.md` and the state it stands at, the expectations the probes it names declare, the version of the grader that evaluated them, and for a compared arm the result of the other arm. Fetch those and this file comes back.

## The run this transcript recorded

A regression run in the present arm, on claude-sonnet-5 at 2026-10-03.
served version claude-sonnet-5, tree sha256:1170ba43bc6abce3e6f97e334983a8bb64279e2e49e5647b63ccfd372e211425, selection sha256:d0fddc9f0c2dd6ec76cbaa0a7a483989929218b5457cef93846a007b0420940b, read set sha256:4b05d45fef40465836aad67df91f018e9bc23f0fea8aeb72ac5cf9ea9764b7d3, seed 0, harness 0.5.0.
realized cost $1.84, which the adaptive layer reads as the cost of its own instrument.

2 events over 2 probes, in 2 sessions and 52 tool calls.

The engine confirmed the taxonomy, that every member of the run identity is present, the membership of every probe named, that no key outside the closed set appears, and that a realized cost was recorded. Present is not confirmed. This page names the lock, the selection and the read set the transcript recorded, and it compares none of them with the tree in front of a reader. `headwater generate` and `headwater probe stale` name what moved since the recording, and neither one fails for it. It graded nothing: a verdict is a function of this transcript, the expectations these probes declare and a grader version, and `headwater probe grade` is the verb that holds all three.

The selection this transcript names is `sha256:d0fddc9f0c2dd6ec76cbaa0a7a483989929218b5457cef93846a007b0420940b`, and the probes graded below are the probes it was planned over, as this corpus declares them now. A probe added to this corpus after the recording is not a session this run owed. The tree, the seed and the harness above are provenance and nothing compares them.

The `read_set` digest above covers every probe of the selection and every document one of them examines, by path and content. This file names it as provenance and compares it with nothing, and the same holds for the lock and the selection: an edit to a document, a new probe or a moved lock leaves these bytes alone. `headwater generate` names each result whose recorded lock, read set or selection moved, and `headwater probe stale` takes the digest and reports which recorded results a change voided. Neither one fails for it.

Graded by grader 0.5.0.
A regression run in the present arm, on claude-sonnet-5 at served version claude-sonnet-5.

## The verdicts

- HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer (sufficiency, expects answered)
    session L3-regression-present-p1-r1: satisfied — event 1: answered `absent`
- HW-PROBE-a-session-records-an-unmeasured-claim-in-the-shape-this-corpus-checks (sufficiency, expects patched)
    session L3-regression-present-p2-r1: satisfied — event 2: `section.required.missing` reported nothing over `docs/process/obligations/0233-nothing-sums-what-a-session-s-standing-instructions-cost-before-its-first-turn.md`

## The rate, and the denominator it is over

2 of 2 graded sessions satisfied their expectation: 100.0%, in a 95% interval of 34.2% to 100.0%.
The denominator is the graded sessions and never the selected probes. 0 sessions reached no verdict, and a session with no verdict is outside both halves of that fraction.

An interval that overlaps the previous run's is variance and one that does not is drift. This is one arm, so it estimates no effect: an efficacy claim is a comparison of two results, and the arm each one recorded is on it.
