---
"headwater:generated": "probe_result. `headwater generate` writes this file, and `headwater generate --check` holds it. Edit the corpus, not this file."
id: HW-RESULT-campaign-of-2026-09-30-campaign-tier-present-arm-navigability
title: Probe result for campaign-of-2026-09-30-campaign-tier-present-arm-navigability
status: current
status_since: 2026-09-30
summary: "The grade of the transcript `campaign-of-2026-09-30-campaign-tier-present-arm-navigability`, taken over the probes this corpus declares and the version of the grader that evaluated them."
last_verified: 2026-09-30
---

# The result of docs/probe-runs/campaign-of-2026-09-30-campaign-tier-present-arm-navigability.md

A probe result is a function of three committed inputs and of nothing else: the transcript at `docs/probe-runs/campaign-of-2026-09-30-campaign-tier-present-arm-navigability.md`, the expectations the probes of this corpus declare, and the version of the grader that evaluated them. Fetch the three and this file comes back.

## The run this transcript recorded

A campaign run in the present arm, on claude-sonnet-5 at 2026-09-30.
served version claude-sonnet-5, tree sha256:1c21e55eb09aca89b3d736ac79f8accfc64ddd7853bace5fb5e577a82d794bb9, selection sha256:fc04ac718a7df0b14774c28174a93f9d44dea6305b9901ba6e2fe6556834db76, read set sha256:cc6b97b54004565f29da095dbf342e6a079a5b799456bd322586e1b56efe9229, seed 0, harness 0.5.0.
realized cost $36.78, which the adaptive layer reads as the cost of its own instrument.
It is read over this tree, and the read set of its probes moved since the recording. Each verdict below is what the session did over the documents it met, graded against the expectations this tree declares now, which can differ from the ones the session ran under. `headwater probe stale` names what moved.

90 events over 3 of the 10 probes this corpus declares, in 90 sessions and 1067 tool calls.

The engine confirmed the taxonomy, that every member of the run identity is present, the membership of every probe named, that no key outside the closed set appears, and that a realized cost was recorded. Present is not confirmed: of the six members a plan fixes before a run, the lock is the one compared here. A lock that differs refuses the file only where this tree composes no read set over its probes, and a read set that differs marks the verdicts and keeps them. `headwater probe stale` compares the read set against the tree in front of it. It graded nothing: a verdict is a function of this transcript, the expectations these probes declare and a grader version, and `headwater probe grade` is the verb that holds all three.

**The selection this transcript names is not the selection this corpus composes.** The transcript names `sha256:fc04ac718a7df0b14774c28174a93f9d44dea6305b9901ba6e2fe6556834db76` and this corpus composes `sha256:50ed5ce43072d9f674a8dc52e1dcad84de4d649e39a77ce22179836188196b09`. The transcript's digest is the digest of 3 of the 10 probes this corpus composes, so the run was planned over that part of the selection, or the rest were added after it was recorded. Every verdict below is over that part, and a probe outside it is not a session this run owed. The tree, the seed and the harness above are provenance and nothing compares them.

The `read_set` digest above covers every probe of the selection and every document one of them examines, by path and content. It is recorded here, and this file names no digest the tree in front of a reader composes. Where that tree composes another one, a single sentence above says that the read set moved, and it moves these bytes once. A digest from that tree would move these bytes on every edit to a document the selection points at, and `generate --check` holds this file to its bytes, so the staleness of a measurement would stop a merge. `headwater probe stale` takes the digest and reports which recorded results a change voided.

Graded by grader 0.5.0.
A campaign run in the present arm, on claude-sonnet-5 at served version claude-sonnet-5.

## The verdicts

- HW-PROBE-a-session-answers-from-the-register-without-opening-the-question-it-replaced (navigability, expects not_opened)
    session L3-campaign-present-p2-r1: satisfied — 4 recorded calls, and none named any of the 1 document
    session L3-campaign-present-p2-r10: satisfied — 3 recorded calls, and none named any of the 1 document
    session L3-campaign-present-p2-r11: satisfied — 5 recorded calls, and none named any of the 1 document
    session L3-campaign-present-p2-r12: satisfied — 5 recorded calls, and none named any of the 1 document
    session L3-campaign-present-p2-r13: satisfied — 4 recorded calls, and none named any of the 1 document
    session L3-campaign-present-p2-r14: satisfied — 3 recorded calls, and none named any of the 1 document
    session L3-campaign-present-p2-r15: satisfied — 3 recorded calls, and none named any of the 1 document
    session L3-campaign-present-p2-r16: satisfied — 3 recorded calls, and none named any of the 1 document
    session L3-campaign-present-p2-r17: satisfied — 4 recorded calls, and none named any of the 1 document
    session L3-campaign-present-p2-r18: satisfied — 5 recorded calls, and none named any of the 1 document
    session L3-campaign-present-p2-r19: satisfied — 3 recorded calls, and none named any of the 1 document
    session L3-campaign-present-p2-r2: satisfied — 5 recorded calls, and none named any of the 1 document
    session L3-campaign-present-p2-r20: satisfied — 3 recorded calls, and none named any of the 1 document
    session L3-campaign-present-p2-r21: satisfied — 3 recorded calls, and none named any of the 1 document
    session L3-campaign-present-p2-r22: satisfied — 5 recorded calls, and none named any of the 1 document
    session L3-campaign-present-p2-r23: satisfied — 5 recorded calls, and none named any of the 1 document
    session L3-campaign-present-p2-r24: not satisfied — event 47, call 4 read `{"command":"grep -n -i \"scent\" docs/spec/09-decisions.md docs/spec/09-open-questions.md","description":"Find scent mentions in spec decisions/open-questions docs"}`
    session L3-campaign-present-p2-r25: satisfied — 13 recorded calls, and none named any of the 1 document
    session L3-campaign-present-p2-r26: satisfied — 5 recorded calls, and none named any of the 1 document
    session L3-campaign-present-p2-r27: satisfied — 3 recorded calls, and none named any of the 1 document
    session L3-campaign-present-p2-r28: satisfied — 3 recorded calls, and none named any of the 1 document
    session L3-campaign-present-p2-r29: not satisfied — event 52, call 4 read `{"command":"grep -n -i \"scent\" docs/spec/02-taxonomy-model.md docs/decisions/0004-relation-storage.md docs/spec/01-conceptual-model.md docs/spec/09-decisions.md docs/spec/09-open-questions.md 2>/dev/null"}`
    session L3-campaign-present-p2-r3: satisfied — 4 recorded calls, and none named any of the 1 document
    session L3-campaign-present-p2-r30: satisfied — 3 recorded calls, and none named any of the 1 document
    session L3-campaign-present-p2-r4: satisfied — 3 recorded calls, and none named any of the 1 document
    session L3-campaign-present-p2-r5: satisfied — 3 recorded calls, and none named any of the 1 document
    session L3-campaign-present-p2-r6: satisfied — 3 recorded calls, and none named any of the 1 document
    session L3-campaign-present-p2-r7: satisfied — 3 recorded calls, and none named any of the 1 document
    session L3-campaign-present-p2-r8: satisfied — 3 recorded calls, and none named any of the 1 document
    session L3-campaign-present-p2-r9: satisfied — 3 recorded calls, and none named any of the 1 document
- HW-PROBE-an-agent-reaches-the-adjudication-from-the-document-that-lost-it (navigability, expects opened)
    session L3-campaign-present-p1-r1: satisfied — event 1, call 2: `Bash {"command":"cat docs/decisions/0008-probe-cost-and-cadence.md"}`
    session L3-campaign-present-p1-r10: satisfied — event 2, call 3: `Read /home/james/.claude/jobs/1338ac1e/tmp/issue-1384/full/campaign/ws/L3-campaign-present-p1-r10/docs/decisions/0008-probe-cost-and-cadence.md`
    session L3-campaign-present-p1-r11: satisfied — event 3, call 2: `Bash {"command":"cat \"docs/decisions/0008-probe-cost-and-cadence.md\""}`
    session L3-campaign-present-p1-r12: satisfied — event 4, call 2: `Bash {"command":"cat \"docs/decisions/0008-probe-cost-and-cadence.md\""}`
    session L3-campaign-present-p1-r13: satisfied — event 5, call 5: `Read docs/decisions/0008-probe-cost-and-cadence.md`
    session L3-campaign-present-p1-r14: satisfied — event 6, call 3: `Bash {"command":"cat docs/decisions/0008-probe-cost-and-cadence.md"}`
    session L3-campaign-present-p1-r15: satisfied — event 7, call 2: `Read /home/james/.claude/jobs/1338ac1e/tmp/issue-1384/full/campaign/ws/L3-campaign-present-p1-r15/docs/decisions/0008-probe-cost-and-cadence.md`
    session L3-campaign-present-p1-r16: satisfied — event 8, call 2: `Read /home/james/.claude/jobs/1338ac1e/tmp/issue-1384/full/campaign/ws/L3-campaign-present-p1-r16/docs/decisions/0008-probe-cost-and-cadence.md`
    session L3-campaign-present-p1-r17: satisfied — event 9, call 3: `Read docs/decisions/0008-probe-cost-and-cadence.md`
    session L3-campaign-present-p1-r18: satisfied — event 10, call 2: `Bash {"command":"cat \"docs/decisions/0008-probe-cost-and-cadence.md\""}`
    session L3-campaign-present-p1-r19: satisfied — event 11, call 2: `Read /home/james/.claude/jobs/1338ac1e/tmp/issue-1384/full/campaign/ws/L3-campaign-present-p1-r19/docs/decisions/0008-probe-cost-and-cadence.md`
    session L3-campaign-present-p1-r2: satisfied — event 12, call 4: `Read docs/decisions/0008-probe-cost-and-cadence.md`
    session L3-campaign-present-p1-r20: satisfied — event 13, call 2: `Read /home/james/.claude/jobs/1338ac1e/tmp/issue-1384/full/campaign/ws/L3-campaign-present-p1-r20/docs/decisions/0008-probe-cost-and-cadence.md`
    session L3-campaign-present-p1-r21: satisfied — event 14, call 3: `Read /home/james/.claude/jobs/1338ac1e/tmp/issue-1384/full/campaign/ws/L3-campaign-present-p1-r21/docs/decisions/0008-probe-cost-and-cadence.md`
    session L3-campaign-present-p1-r22: satisfied — event 15, call 6: `Read ./docs/decisions/0008-probe-cost-and-cadence.md`
    session L3-campaign-present-p1-r23: satisfied — event 16, call 2: `Read /home/james/.claude/jobs/1338ac1e/tmp/issue-1384/full/campaign/ws/L3-campaign-present-p1-r23/docs/decisions/0008-probe-cost-and-cadence.md`
    session L3-campaign-present-p1-r24: satisfied — event 17, call 2: `Read docs/decisions/0008-probe-cost-and-cadence.md`
    session L3-campaign-present-p1-r25: satisfied — event 18, call 2: `Read /home/james/.claude/jobs/1338ac1e/tmp/issue-1384/full/campaign/ws/L3-campaign-present-p1-r25/docs/decisions/0008-probe-cost-and-cadence.md`
    session L3-campaign-present-p1-r26: satisfied — event 19, call 5: `Read /home/james/.claude/jobs/1338ac1e/tmp/issue-1384/full/campaign/ws/L3-campaign-present-p1-r26/docs/decisions/0008-probe-cost-and-cadence.md`
    session L3-campaign-present-p1-r27: satisfied — event 20, call 2: `Read /home/james/.claude/jobs/1338ac1e/tmp/issue-1384/full/campaign/ws/L3-campaign-present-p1-r27/docs/decisions/0008-probe-cost-and-cadence.md`
    session L3-campaign-present-p1-r28: satisfied — event 21, call 3: `Read /home/james/.claude/jobs/1338ac1e/tmp/issue-1384/full/campaign/ws/L3-campaign-present-p1-r28/docs/decisions/0008-probe-cost-and-cadence.md`
    session L3-campaign-present-p1-r29: satisfied — event 22, call 2: `Read /home/james/.claude/jobs/1338ac1e/tmp/issue-1384/full/campaign/ws/L3-campaign-present-p1-r29/docs/decisions/0008-probe-cost-and-cadence.md`
    session L3-campaign-present-p1-r3: satisfied — event 23, call 2: `Read /home/james/.claude/jobs/1338ac1e/tmp/issue-1384/full/campaign/ws/L3-campaign-present-p1-r3/docs/decisions/0008-probe-cost-and-cadence.md`
    session L3-campaign-present-p1-r30: satisfied — event 24, call 4: `Read /home/james/.claude/jobs/1338ac1e/tmp/issue-1384/full/campaign/ws/L3-campaign-present-p1-r30/docs/decisions/0008-probe-cost-and-cadence.md`
    session L3-campaign-present-p1-r4: satisfied — event 25, call 2: `Read /home/james/.claude/jobs/1338ac1e/tmp/issue-1384/full/campaign/ws/L3-campaign-present-p1-r4/docs/decisions/0008-probe-cost-and-cadence.md`
    session L3-campaign-present-p1-r5: satisfied — event 26, call 9: `Read /home/james/.claude/jobs/1338ac1e/tmp/issue-1384/full/campaign/ws/L3-campaign-present-p1-r5/docs/decisions/0008-probe-cost-and-cadence.md`
    session L3-campaign-present-p1-r6: satisfied — event 27, call 2: `Bash {"command":"cat docs/decisions/0008-probe-cost-and-cadence.md"}`
    session L3-campaign-present-p1-r7: satisfied — event 28, call 3: `Read /home/james/.claude/jobs/1338ac1e/tmp/issue-1384/full/campaign/ws/L3-campaign-present-p1-r7/docs/decisions/0008-probe-cost-and-cadence.md`
    session L3-campaign-present-p1-r8: satisfied — event 29, call 2: `Bash {"command":"cat docs/decisions/0008-probe-cost-and-cadence.md"}`
    session L3-campaign-present-p1-r9: satisfied — event 30, call 2: `Read docs/decisions/0008-probe-cost-and-cadence.md`
- HW-PROBE-what-a-session-writes-points-back-at-the-ruling-it-rests-on (navigability, expects cited)
    session L3-campaign-present-p3-r1: satisfied — event 61: `docs/evaluations/why-corpus-counts-are-derived-not-stored.md` cites HW-DR-0049
    session L3-campaign-present-p3-r10: satisfied — event 62: `docs/evaluations/why-corpus-counts-are-derived-not-stored.md` cites HW-DR-0049
    session L3-campaign-present-p3-r11: satisfied — event 63: `docs/evaluations/why-corpus-counts-are-derived-not-stored.md` cites HW-DR-0049
    session L3-campaign-present-p3-r12: not satisfied — 0 produced artifacts, and none cites any of the 1 identifier
    session L3-campaign-present-p3-r13: not satisfied — 0 produced artifacts, and none cites any of the 1 identifier
    session L3-campaign-present-p3-r14: not satisfied — 0 produced artifacts, and none cites any of the 1 identifier
    session L3-campaign-present-p3-r15: not satisfied — 0 produced artifacts, and none cites any of the 1 identifier
    session L3-campaign-present-p3-r16: not satisfied — 0 produced artifacts, and none cites any of the 1 identifier
    session L3-campaign-present-p3-r17: not satisfied — 0 produced artifacts, and none cites any of the 1 identifier
    session L3-campaign-present-p3-r18: satisfied — event 70: `docs/explanations/how-a-corpus-wide-count-is-worked-out.md` cites HW-DR-0049
    session L3-campaign-present-p3-r19: satisfied — event 71: `docs/explanations/how-a-count-over-the-whole-corpus-is-worked-out.md` cites HW-DR-0049
    session L3-campaign-present-p3-r2: not satisfied — 1 produced artifact, and none cites any of the 1 identifier
    session L3-campaign-present-p3-r20: not satisfied — 0 produced artifacts, and none cites any of the 1 identifier
    session L3-campaign-present-p3-r21: not satisfied — 0 produced artifacts, and none cites any of the 1 identifier
    session L3-campaign-present-p3-r22: satisfied — event 75: `docs/evaluations/why-corpus-counts-are-derived-not-stored.md` cites HW-DR-0049
    session L3-campaign-present-p3-r23: not satisfied — 0 produced artifacts, and none cites any of the 1 identifier
    session L3-campaign-present-p3-r24: not satisfied — 0 produced artifacts, and none cites any of the 1 identifier
    session L3-campaign-present-p3-r25: satisfied — event 78: `engine/crates/graph/src/lib.rs` cites HW-DR-0049
    session L3-campaign-present-p3-r26: not satisfied — 0 produced artifacts, and none cites any of the 1 identifier
    session L3-campaign-present-p3-r27: satisfied — event 80: `docs/explanations/why-a-count-over-the-whole-corpus-is-computed-when-it-is-read.md` cites HW-DR-0049
    session L3-campaign-present-p3-r28: not satisfied — 0 produced artifacts, and none cites any of the 1 identifier
    session L3-campaign-present-p3-r29: satisfied — event 82: `docs/evaluations/why-corpus-counts-are-derived-not-stored.md` cites HW-DR-0049
    session L3-campaign-present-p3-r3: not satisfied — 0 produced artifacts, and none cites any of the 1 identifier
    session L3-campaign-present-p3-r30: not satisfied — 0 produced artifacts, and none cites any of the 1 identifier
    session L3-campaign-present-p3-r4: not satisfied — 0 produced artifacts, and none cites any of the 1 identifier
    session L3-campaign-present-p3-r5: not satisfied — 0 produced artifacts, and none cites any of the 1 identifier
    session L3-campaign-present-p3-r6: satisfied — event 87: `engine/crates/check/src/coverage.rs` cites HW-DR-0049
    session L3-campaign-present-p3-r7: not satisfied — 0 produced artifacts, and none cites any of the 1 identifier
    session L3-campaign-present-p3-r8: not satisfied — 0 produced artifacts, and none cites any of the 1 identifier
    session L3-campaign-present-p3-r9: satisfied — event 90: `engine/crates/census/src/census.rs` cites HW-DR-0049

## The rate, and the denominator it is over

69 of 90 graded sessions satisfied their expectation: 76.7%, in a 95% interval of 66.9% to 84.2%.
The denominator is the graded sessions and never the selected probes. 0 sessions reached no verdict, and a session with no verdict is outside both halves of that fraction.

An interval that overlaps the previous run's is variance and one that does not is drift. This is one arm, so it estimates no effect: an efficacy claim is a comparison of two results, and the arm each one recorded is on it.

## The comparisons this arm takes part in

What the governance changes. The treated arm is the `campaign` present arm and the control is the `campaign` absent arm, which removed the paths the `campaign` tier's `ablation` names and kept `docs/`.

- treated, `docs/probe-runs/campaign-of-2026-09-30-campaign-tier-present-arm-navigability.md`: 69 of 90 graded sessions satisfied their expectation, 76.7%, in a 95% interval of 66.9% to 84.2%. 0 sessions refused by the session itself.
- control, `docs/probe-runs/campaign-of-2026-09-30-campaign-tier-absent-arm-navigability.md`: 68 of 90 graded sessions satisfied their expectation, 75.6%, in a 95% interval of 65.8% to 83.3%. 0 sessions refused by the session itself.

The difference is +1.1 points, in a 95% Newcombe interval of -11.3 points to +13.5 points. The interval contains zero, so this run does not separate the two arms at the 5% level.
