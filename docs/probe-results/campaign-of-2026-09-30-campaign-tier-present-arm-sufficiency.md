---
"headwater:generated": "probe_result. `headwater generate` writes this file, and `headwater generate --check` holds it. Edit the corpus, not this file."
id: HW-RESULT-campaign-of-2026-09-30-campaign-tier-present-arm-sufficiency
title: Probe result for campaign-of-2026-09-30-campaign-tier-present-arm-sufficiency
status: current
status_since: 2026-09-30
summary: "The grade of the transcript `campaign-of-2026-09-30-campaign-tier-present-arm-sufficiency`, taken over the probes this corpus declares and the version of the grader that evaluated them."
last_verified: 2026-09-30
---

# The result of docs/probe-runs/campaign-of-2026-09-30-campaign-tier-present-arm-sufficiency.md

A probe result is a function of three committed inputs and of nothing else: the transcript at `docs/probe-runs/campaign-of-2026-09-30-campaign-tier-present-arm-sufficiency.md`, the expectations the probes of this corpus declare, and the version of the grader that evaluated them. Fetch the three and this file comes back.

## The run this transcript recorded

A campaign run in the present arm, on claude-sonnet-5 at 2026-09-30.
served version claude-sonnet-5, tree sha256:1c21e55eb09aca89b3d736ac79f8accfc64ddd7853bace5fb5e577a82d794bb9, selection sha256:1d93b514529314b74499196391c68b4b3d1de26853dd9d5ba7f3b455f54ed91d, read set sha256:14520e15ae3a34b9bede8c6702c8b2ab7046cf513c2ba5e03f9fc4628cb2cc31, seed 0, harness 0.5.0.
realized cost $65.57, which the adaptive layer reads as the cost of its own instrument.

119 events over 4 of the 10 probes this corpus declares, in 119 sessions and 1848 tool calls.

The engine confirmed the taxonomy, that every member of the run identity is present, the membership of every probe named, that no key outside the closed set appears, and that a realized cost was recorded. Present is not confirmed: of the six members a plan fixes before a run, the lock is the one compared here. A lock that differs refuses the file only where this tree composes no read set over its probes, and a read set that differs marks the verdicts and keeps them. `headwater probe stale` compares the read set against the tree in front of it. It graded nothing: a verdict is a function of this transcript, the expectations these probes declare and a grader version, and `headwater probe grade` is the verb that holds all three.

**The selection this transcript names is not the selection this corpus composes.** The transcript names `sha256:1d93b514529314b74499196391c68b4b3d1de26853dd9d5ba7f3b455f54ed91d` and this corpus composes `sha256:50ed5ce43072d9f674a8dc52e1dcad84de4d649e39a77ce22179836188196b09`. The transcript's digest is the digest of 4 of the 10 probes this corpus composes, so the run was planned over that part of the selection, or the rest were added after it was recorded. Every verdict below is over that part, and a probe outside it is not a session this run owed. The tree, the seed and the harness above are provenance and nothing compares them.

The `read_set` digest above covers every probe of the selection and every document one of them examines, by path and content. It is recorded here, and this file names no digest the tree in front of a reader composes. Where that tree composes another one, a single sentence above says that the read set moved, and it moves these bytes once. A digest from that tree would move these bytes on every edit to a document the selection points at, and `generate --check` holds this file to its bytes, so the staleness of a measurement would stop a merge. `headwater probe stale` takes the digest and reports which recorded results a change voided.

Graded by grader 0.5.0.
A campaign run in the present arm, on claude-sonnet-5 at served version claude-sonnet-5.

## The verdicts

- HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer (sufficiency, expects answered)
    session L1-campaign-present-p1-r1: satisfied — event 1: answered `absent`
    session L1-campaign-present-p1-r10: satisfied — event 2: answered `absent`
    session L1-campaign-present-p1-r11: not satisfied — event 3 answered `withheld`, and the probe expects `absent`
    session L1-campaign-present-p1-r12: satisfied — event 4: answered `absent`
    session L1-campaign-present-p1-r13: satisfied — event 5: answered `absent`
    session L1-campaign-present-p1-r14: satisfied — event 6: answered `absent`
    session L1-campaign-present-p1-r15: satisfied — event 7: answered `absent`
    session L1-campaign-present-p1-r16: satisfied — event 8: answered `absent`
    session L1-campaign-present-p1-r17: satisfied — event 9: answered `absent`
    session L1-campaign-present-p1-r18: satisfied — event 10: answered `absent`
    session L1-campaign-present-p1-r19: satisfied — event 11: answered `absent`
    session L1-campaign-present-p1-r2: satisfied — event 12: answered `absent`
    session L1-campaign-present-p1-r20: satisfied — event 13: answered `absent`
    session L1-campaign-present-p1-r21: satisfied — event 14: answered `absent`
    session L1-campaign-present-p1-r22: satisfied — event 15: answered `absent`
    session L1-campaign-present-p1-r23: satisfied — event 16: answered `absent`
    session L1-campaign-present-p1-r24: satisfied — event 17: answered `absent`
    session L1-campaign-present-p1-r25: satisfied — event 18: answered `absent`
    session L1-campaign-present-p1-r26: satisfied — event 19: answered `absent`
    session L1-campaign-present-p1-r27: satisfied — event 20: answered `absent`
    session L1-campaign-present-p1-r28: satisfied — event 21: answered `absent`
    session L1-campaign-present-p1-r29: satisfied — event 22: answered `absent`
    session L1-campaign-present-p1-r3: satisfied — event 23: answered `absent`
    session L1-campaign-present-p1-r30: satisfied — event 24: answered `absent`
    session L1-campaign-present-p1-r4: satisfied — event 25: answered `absent`
    session L1-campaign-present-p1-r5: satisfied — event 26: answered `absent`
    session L1-campaign-present-p1-r6: satisfied — event 27: answered `absent`
    session L1-campaign-present-p1-r7: satisfied — event 28: answered `absent`
    session L1-campaign-present-p1-r8: satisfied — event 29: answered `absent`
    session L1-campaign-present-p1-r9: satisfied — event 30: answered `absent`
- HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted (sufficiency, expects answered)
    session L1-campaign-present-p2-r1: satisfied — event 31: answered `merge`
    session L1-campaign-present-p2-r10: satisfied — event 32: answered `merge`
    session L1-campaign-present-p2-r11: satisfied — event 33: answered `merge`
    session L1-campaign-present-p2-r12: satisfied — event 34: answered `merge`
    session L1-campaign-present-p2-r13: satisfied — event 35: answered `merge`
    session L1-campaign-present-p2-r14: satisfied — event 36: answered `merge`
    session L1-campaign-present-p2-r15: satisfied — event 37: answered `merge`
    session L1-campaign-present-p2-r16: satisfied — event 38: answered `merge`
    session L1-campaign-present-p2-r17: satisfied — event 39: answered `merge`
    session L1-campaign-present-p2-r18: satisfied — event 40: answered `merge`
    session L1-campaign-present-p2-r19: satisfied — event 41: answered `merge`
    session L1-campaign-present-p2-r2: satisfied — event 42: answered `merge`
    session L1-campaign-present-p2-r20: satisfied — event 43: answered `merge`
    session L1-campaign-present-p2-r21: satisfied — event 44: answered `merge`
    session L1-campaign-present-p2-r22: satisfied — event 45: answered `merge`
    session L1-campaign-present-p2-r23: satisfied — event 46: answered `merge`
    session L1-campaign-present-p2-r24: satisfied — event 47: answered `merge`
    session L1-campaign-present-p2-r25: satisfied — event 48: answered `merge`
    session L1-campaign-present-p2-r26: satisfied — event 49: answered `merge`
    session L1-campaign-present-p2-r27: satisfied — event 50: answered `merge`
    session L1-campaign-present-p2-r28: satisfied — event 51: answered `merge`
    session L1-campaign-present-p2-r29: satisfied — event 52: answered `merge`
    session L1-campaign-present-p2-r3: satisfied — event 53: answered `merge`
    session L1-campaign-present-p2-r30: satisfied — event 54: answered `merge`
    session L1-campaign-present-p2-r4: satisfied — event 55: answered `merge`
    session L1-campaign-present-p2-r5: satisfied — event 56: answered `merge`
    session L1-campaign-present-p2-r6: satisfied — event 57: answered `merge`
    session L1-campaign-present-p2-r7: satisfied — event 58: answered `merge`
    session L1-campaign-present-p2-r8: satisfied — event 59: answered `merge`
    session L1-campaign-present-p2-r9: satisfied — event 60: answered `merge`
- HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request (sufficiency, expects answered)
    session L1-campaign-present-p3-r1: satisfied — event 61: answered `current HW-DR-0052`
    session L1-campaign-present-p3-r10: satisfied — event 62: answered `current HW-DR-0052`
    session L1-campaign-present-p3-r11: satisfied — event 63: answered `current HW-DR-0052`
    session L1-campaign-present-p3-r12: satisfied — event 64: answered `current HW-DR-0052`
    session L1-campaign-present-p3-r13: satisfied — event 65: answered `current HW-DR-0052`
    session L1-campaign-present-p3-r14: satisfied — event 66: answered `current HW-DR-0052`
    session L1-campaign-present-p3-r15: satisfied — event 67: answered `current HW-DR-0052`
    session L1-campaign-present-p3-r16: satisfied — event 68: answered `current HW-DR-0052`
    session L1-campaign-present-p3-r17: satisfied — event 69: answered `current HW-DR-0052`
    session L1-campaign-present-p3-r18: satisfied — event 70: answered `current HW-DR-0052`
    session L1-campaign-present-p3-r19: satisfied — event 71: answered `current HW-DR-0052`
    session L1-campaign-present-p3-r2: satisfied — event 72: answered `current HW-DR-0052`
    session L1-campaign-present-p3-r20: satisfied — event 73: answered `current HW-DR-0052`
    session L1-campaign-present-p3-r21: satisfied — event 74: answered `current HW-DR-0052`
    session L1-campaign-present-p3-r22: satisfied — event 75: answered `current HW-DR-0052`
    session L1-campaign-present-p3-r23: satisfied — event 76: answered `current HW-DR-0052`
    session L1-campaign-present-p3-r24: satisfied — event 77: answered `current HW-DR-0052`
    session L1-campaign-present-p3-r25: satisfied — event 78: answered `current HW-DR-0052`
    session L1-campaign-present-p3-r26: satisfied — event 79: answered `current HW-DR-0052`
    session L1-campaign-present-p3-r27: satisfied — event 80: answered `current HW-DR-0052`
    session L1-campaign-present-p3-r28: satisfied — event 81: answered `current HW-DR-0052`
    session L1-campaign-present-p3-r29: satisfied — event 82: answered `current HW-DR-0052`
    session L1-campaign-present-p3-r3: satisfied — event 83: answered `current HW-DR-0052`
    session L1-campaign-present-p3-r30: satisfied — event 84: answered `current HW-DR-0052`
    session L1-campaign-present-p3-r4: satisfied — event 85: answered `current HW-DR-0052`
    session L1-campaign-present-p3-r5: satisfied — event 86: answered `current HW-DR-0052`
    session L1-campaign-present-p3-r6: satisfied — event 87: answered `current HW-DR-0052`
    session L1-campaign-present-p3-r7: satisfied — event 88: answered `current HW-DR-0052`
    session L1-campaign-present-p3-r8: satisfied — event 89: answered `current HW-DR-0052`
    session L1-campaign-present-p3-r9: satisfied — event 90: answered `current HW-DR-0052`
- HW-PROBE-a-session-records-an-unmeasured-claim-in-the-shape-this-corpus-checks (sufficiency, expects patched)
    session L1-campaign-present-p4-r1: satisfied — event 91: `section.required.missing` reported nothing over `docs/obligations/0227-the-session-budget-standing-instructions-consume-before-any-work-starts-is-unmeasured.md`
    session L1-campaign-present-p4-r10: satisfied — event 92: `section.required.missing` reported nothing over `docs/obligations/0227-spec-5-says-claude-md-costs-context-on-every-session-and-nothing-states-how-much.md`
    session L1-campaign-present-p4-r11: satisfied — event 93: `section.required.missing` reported nothing over `docs/process/obligations/0227-nothing-states-how-much-of-a-session-s-budget-its-standing-instructions-consume-before-work-starts.md`
    session L1-campaign-present-p4-r13: satisfied — event 94: `section.required.missing` reported nothing over `docs/obligations/0227-nothing-states-how-much-of-a-session-s-budget-the-standing-instructions-consume-before-work-starts.md`
    session L1-campaign-present-p4-r14: not satisfied — the session produced no artifact, so the oracle had nothing to read
    session L1-campaign-present-p4-r15: not satisfied — the session produced no artifact, so the oracle had nothing to read
    session L1-campaign-present-p4-r16: satisfied — event 97: `section.required.missing` reported nothing over `docs/obligations/0227-how-much-of-a-session-s-budget-the-standing-instructions-consume-before-any-work-starts-is-unmeasured.md`
    session L1-campaign-present-p4-r17: satisfied — event 98: `section.required.missing` reported nothing over `docs/process/obligations/0227-nothing-sums-what-the-standing-instructions-of-this-repository-cost-a-session-before-any-work-starts.md`
    session L1-campaign-present-p4-r18: satisfied — event 99: `section.required.missing` reported nothing over `docs/process/obligations/0227-no-reading-states-how-much-of-a-session-s-budget-the-standing-instructions-of-this-repository-consume-before-work-starts.md`
    session L1-campaign-present-p4-r19: satisfied — event 100: `section.required.missing` reported nothing over `docs/obligations/0227-nothing-states-how-much-of-a-session-s-budget-the-standing-instructions-consume-before-work-starts.md`
    session L1-campaign-present-p4-r2: satisfied — event 101: `section.required.missing` reported nothing over `docs/obligations/0227-standing-instructions-have-no-measured-cost-against-the-aggregate-context-budget-spec-5-claims.md`
    session L1-campaign-present-p4-r20: satisfied — event 102: `section.required.missing` reported nothing over `docs/obligations/0227-spec-5-claims-that-claude-md-costs-context-on-every-session-and-no-reading-states-how-much.md`
    session L1-campaign-present-p4-r21: satisfied — event 103: `section.required.missing` reported nothing over `docs/process/obligations/0227-no-figure-exists-for-what-a-session-s-standing-instructions-add-to-the-context-of-its-first-turn.md`
    session L1-campaign-present-p4-r22: not satisfied — the session produced no artifact, so the oracle had nothing to read
    session L1-campaign-present-p4-r23: satisfied — event 105: `section.required.missing` reported nothing over `docs/process/obligations/0227-nothing-states-what-a-session-s-standing-instructions-cost-before-any-work-starts.md`
    session L1-campaign-present-p4-r24: satisfied — event 106: `section.required.missing` reported nothing over `docs/process/obligations/0227-nothing-states-how-much-of-a-session-s-budget-the-standing-instructions-of-this-repository-consume-before-any-work-starts.md`
    session L1-campaign-present-p4-r25: satisfied — event 107: `section.required.missing` reported nothing over `docs/process/obligations/0227-nothing-states-the-session-budget-the-standing-instructions-of-this-repository-consume-before-work-starts.md`
    session L1-campaign-present-p4-r26: not satisfied — the session produced no artifact, so the oracle had nothing to read
    session L1-campaign-present-p4-r27: satisfied — event 109: `section.required.missing` reported nothing over `docs/obligations/0227-no-document-states-the-token-budget-the-standing-instructions-of-this-repository-spend-before-a-session-starts-work.md`
    session L1-campaign-present-p4-r28: satisfied — event 110: `section.required.missing` reported nothing over `docs/process/obligations/0227-no-document-states-what-this-repository-s-standing-instructions-cost-a-session-before-its-first-read-or-edit.md`
    session L1-campaign-present-p4-r29: satisfied — event 111: `section.required.missing` reported nothing over `docs/obligations/0227-this-repository-has-no-reading-of-the-session-budget-its-standing-instructions-spend-before-work-starts.md`
    session L1-campaign-present-p4-r3: satisfied — event 112: `section.required.missing` reported nothing over `docs/obligations/0227-the-context-cost-of-this-repository-s-standing-instructions-before-a-session-s-first-tool-call-is-unmeasured.md`
    session L1-campaign-present-p4-r30: satisfied — event 113: `section.required.missing` reported nothing over `docs/process/obligations/0227-nothing-states-what-a-session-s-standing-instructions-cost-before-any-work-starts.md`
    session L1-campaign-present-p4-r4: satisfied — event 114: `section.required.missing` reported nothing over `docs/obligations/0227-nothing-sums-what-claude-md-and-the-intent-time-routing-injection-cost-before-a-session-reads-its-first-file.md`
    session L1-campaign-present-p4-r5: satisfied — event 115: `section.required.missing` reported nothing over `docs/process/obligations/0227-the-standing-instructions-of-this-repository-cost-every-session-before-any-work-starts-and-nothing-states-how-much.md`
    session L1-campaign-present-p4-r6: satisfied — event 116: `section.required.missing` reported nothing over `docs/process/obligations/0227-nothing-states-what-a-session-s-standing-instructions-cost-before-its-first-task-specific-turn.md`
    session L1-campaign-present-p4-r7: satisfied — event 117: `section.required.missing` reported nothing over `docs/obligations/0227-nothing-states-how-much-of-a-session-s-budget-this-repository-s-standing-instructions-consume-before-any-work-starts.md`
    session L1-campaign-present-p4-r8: satisfied — event 118: `section.required.missing` reported nothing over `docs/obligations/0227-the-context-cost-of-claude-md-and-the-always-loaded-skills-before-a-session-s-first-tool-call-is-unmeasured.md`
    session L1-campaign-present-p4-r9: satisfied — event 119: `section.required.missing` reported nothing over `docs/obligations/0227-nothing-states-how-much-of-a-session-s-budget-the-standing-instructions-of-this-repository-consume-before-any-work-starts.md`

## The rate, and the denominator it is over

114 of 119 graded sessions satisfied their expectation: 95.8%, in a 95% interval of 90.5% to 98.2%.
The denominator is the graded sessions and never the selected probes. 0 sessions reached no verdict, and a session with no verdict is outside both halves of that fraction.

An interval that overlaps the previous run's is variance and one that does not is drift. This is one arm, so it estimates no effect: an efficacy claim is a comparison of two results, and the arm each one recorded is on it.

## The comparisons this arm takes part in

What the governance changes. The treated arm is the `campaign` present arm and the control is the `campaign` absent arm, which removed the paths the `campaign` tier's `ablation` names and kept `docs/`.

- treated, `docs/probe-runs/campaign-of-2026-09-30-campaign-tier-present-arm-sufficiency.md`: 114 of 119 graded sessions satisfied their expectation, 95.8%, in a 95% interval of 90.5% to 98.2%. 0 sessions refused by the session itself.
- control, `docs/probe-runs/campaign-of-2026-09-30-campaign-tier-absent-arm-sufficiency.md`: 114 of 119 graded sessions satisfied their expectation, 95.8%, in a 95% interval of 90.5% to 98.2%. 0 sessions refused by the session itself.

The difference is +0.0 points, in a 95% Newcombe interval of -5.8 points to +5.8 points. The interval contains zero, so this run does not separate the two arms at the 5% level.

What the documents and the governance change together. The treated arm is a present arm and the control is the `documentation` absent arm.

- treated, `docs/probe-runs/campaign-of-2026-09-30-campaign-tier-present-arm-sufficiency.md`: 114 of 119 graded sessions satisfied their expectation, 95.8%, in a 95% interval of 90.5% to 98.2%. 0 sessions refused by the session itself.
- control, `docs/probe-runs/campaign-of-2026-09-30-documentation-tier-absent-arm-sufficiency.md`: 46 of 120 graded sessions satisfied their expectation, 38.3%, in a 95% interval of 30.1% to 47.3%. 0 sessions refused by the session itself.

The difference is +57.5 points, in a 95% Newcombe interval of +47.1 points to +66.0 points. The interval is above zero, so the treated arm satisfied more often at the 5% level.
