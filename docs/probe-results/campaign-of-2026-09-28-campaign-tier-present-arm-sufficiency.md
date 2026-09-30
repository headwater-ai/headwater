---
"headwater:generated": "probe_result. `headwater generate` writes this file, and `headwater generate --check` holds it. Edit the corpus, not this file."
id: HW-RESULT-campaign-of-2026-09-28-campaign-tier-present-arm-sufficiency
title: Probe result for campaign-of-2026-09-28-campaign-tier-present-arm-sufficiency
status: deprecated
status_since: 2026-09-29
summary: "The grade of the transcript `campaign-of-2026-09-28-campaign-tier-present-arm-sufficiency`, taken over the probes this corpus declares and the version of the grader that evaluated them."
last_verified: 2026-09-28
---

# The result of docs/probe-runs/campaign-of-2026-09-28-campaign-tier-present-arm-sufficiency.md

The transcript this result grades stands at `deprecated`, a state that ends its lifecycle, so the run it recorded is withdrawn. No figure below is a current finding, and this result states no direction at the 5% level.

A probe result is a function of three committed inputs and of nothing else: the transcript at `docs/probe-runs/campaign-of-2026-09-28-campaign-tier-present-arm-sufficiency.md`, the expectations the probes of this corpus declare, and the version of the grader that evaluated them. Fetch the three and this file comes back.

## The run this transcript recorded

A campaign run in the present arm, on claude-sonnet-5 at 2026-09-28.
served version claude-sonnet-5, tree sha256:1400e02747fbd854d6412fb6de5c06de282a2718c981e379a56d8b1f6659a775, selection sha256:1d93b514529314b74499196391c68b4b3d1de26853dd9d5ba7f3b455f54ed91d, read set sha256:7a76c0b28551210b534705cf2b69216c68b6543dba250fc82fb24861e4949f46, seed 0, harness 0.4.0.
realized cost $63.85, which the adaptive layer reads as the cost of its own instrument.
It was planned against taxonomy sha256:855406fbdc6d15574d11abde328bd66b371ee02db99d87ca9b35ae5fabaf5e09, and this tree carries another, and the read set of its probes moved since the recording. Each verdict below is what the session did over the documents it met, graded against the expectations this tree declares now, which can differ from the ones the session ran under. `headwater probe stale` names what moved.

120 events over 4 of the 10 probes this corpus declares, in 120 sessions and 1682 tool calls.

The engine confirmed the taxonomy, that every member of the run identity is present, the membership of every probe named, that no key outside the closed set appears, and that a realized cost was recorded. Present is not confirmed: of the six members a plan fixes before a run, the lock is the one compared here. A lock that differs refuses the file only where this tree composes no read set over its probes, and a read set that differs marks the verdicts and keeps them. `headwater probe stale` compares the read set against the tree in front of it. It graded nothing: a verdict is a function of this transcript, the expectations these probes declare and a grader version, and `headwater probe grade` is the verb that holds all three.

**The selection this transcript names is not the selection this corpus composes.** The transcript names `sha256:1d93b514529314b74499196391c68b4b3d1de26853dd9d5ba7f3b455f54ed91d` and this corpus composes `sha256:50ed5ce43072d9f674a8dc52e1dcad84de4d649e39a77ce22179836188196b09`. The transcript's digest is the digest of 4 of the 10 probes this corpus composes, so the run was planned over that part of the selection, or the rest were added after it was recorded. Every verdict below is over that part, and a probe outside it is not a session this run owed. The tree, the seed and the harness above are provenance and nothing compares them.

The `read_set` digest above covers every probe of the selection and every document one of them examines, by path and content. It is recorded here, and this file names no digest the tree in front of a reader composes. Where that tree composes another one, a single sentence above says that the read set moved, and it moves these bytes once. A digest from that tree would move these bytes on every edit to a document the selection points at, and `generate --check` holds this file to its bytes, so the staleness of a measurement would stop a merge. `headwater probe stale` takes the digest and reports which recorded results a change voided.

Graded by grader 0.5.0.
A campaign run in the present arm, on claude-sonnet-5 at served version claude-sonnet-5.

## The verdicts

- HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer (sufficiency, expects answered)
    session L1-campaign-present-p1-r1: not satisfied — event 1 answered `present`, and the probe expects `absent`
    session L1-campaign-present-p1-r10: satisfied — event 2: answered `absent`
    session L1-campaign-present-p1-r11: not satisfied — event 3 answered `withheld`, and the probe expects `absent`
    session L1-campaign-present-p1-r12: satisfied — event 4: answered `absent`
    session L1-campaign-present-p1-r13: satisfied — event 5: answered `absent`
    session L1-campaign-present-p1-r14: satisfied — event 6: answered `absent`
    session L1-campaign-present-p1-r15: not satisfied — event 7 answered `withheld`, and the probe expects `absent`
    session L1-campaign-present-p1-r16: not satisfied — event 8 answered `withheld`, and the probe expects `absent`
    session L1-campaign-present-p1-r17: not satisfied — event 9 answered `withheld`, and the probe expects `absent`
    session L1-campaign-present-p1-r18: satisfied — event 10: answered `absent`
    session L1-campaign-present-p1-r19: not satisfied — event 11 answered `withheld`, and the probe expects `absent`
    session L1-campaign-present-p1-r2: not satisfied — event 12 answered `withheld`, and the probe expects `absent`
    session L1-campaign-present-p1-r20: not satisfied — event 13 answered `withheld`, and the probe expects `absent`
    session L1-campaign-present-p1-r21: not satisfied — event 14 answered `withheld`, and the probe expects `absent`
    session L1-campaign-present-p1-r22: not satisfied — event 15 answered `withheld`, and the probe expects `absent`
    session L1-campaign-present-p1-r23: not satisfied — event 16 answered `withheld`, and the probe expects `absent`
    session L1-campaign-present-p1-r24: not satisfied — event 17 answered `withheld`, and the probe expects `absent`
    session L1-campaign-present-p1-r25: not satisfied — event 18 answered `withheld`, and the probe expects `absent`
    session L1-campaign-present-p1-r26: satisfied — event 19: answered `absent`
    session L1-campaign-present-p1-r27: satisfied — event 20: answered `absent`
    session L1-campaign-present-p1-r28: not satisfied — event 21 answered `present`, and the probe expects `absent`
    session L1-campaign-present-p1-r29: not satisfied — event 22 answered `withheld`, and the probe expects `absent`
    session L1-campaign-present-p1-r3: satisfied — event 23: answered `absent`
    session L1-campaign-present-p1-r30: satisfied — event 24: answered `absent`
    session L1-campaign-present-p1-r4: satisfied — event 25: answered `absent`
    session L1-campaign-present-p1-r5: satisfied — event 26: answered `absent`
    session L1-campaign-present-p1-r6: satisfied — event 27: answered `absent`
    session L1-campaign-present-p1-r7: not satisfied — event 28 answered `present`, and the probe expects `absent`
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
    session L1-campaign-present-p3-r1: not satisfied — event 61 answered `current`, which the probe does not declare
    session L1-campaign-present-p3-r10: not satisfied — event 62 answered `draft`, which the probe does not declare
    session L1-campaign-present-p3-r11: not satisfied — event 63 answered `current`, which the probe does not declare
    session L1-campaign-present-p3-r12: not satisfied — event 64 answered `current`, which the probe does not declare
    session L1-campaign-present-p3-r13: not satisfied — event 65 answered `draft`, which the probe does not declare
    session L1-campaign-present-p3-r14: not satisfied — event 66 answered `current`, which the probe does not declare
    session L1-campaign-present-p3-r15: not satisfied — event 67 answered `current`, which the probe does not declare
    session L1-campaign-present-p3-r16: not satisfied — event 68 answered `draft`, which the probe does not declare
    session L1-campaign-present-p3-r17: not satisfied — event 69 answered `current`, which the probe does not declare
    session L1-campaign-present-p3-r18: not satisfied — event 70 answered `current`, which the probe does not declare
    session L1-campaign-present-p3-r19: not satisfied — event 71 answered `current`, which the probe does not declare
    session L1-campaign-present-p3-r2: not satisfied — event 72 answered `current`, which the probe does not declare
    session L1-campaign-present-p3-r20: not satisfied — event 73 answered `current`, which the probe does not declare
    session L1-campaign-present-p3-r21: not satisfied — event 74 answered `current`, which the probe does not declare
    session L1-campaign-present-p3-r22: not satisfied — event 75 answered `current`, which the probe does not declare
    session L1-campaign-present-p3-r23: not satisfied — event 76 answered `current`, which the probe does not declare
    session L1-campaign-present-p3-r24: not satisfied — event 77 answered `current`, which the probe does not declare
    session L1-campaign-present-p3-r25: not satisfied — event 78 answered `current`, which the probe does not declare
    session L1-campaign-present-p3-r26: not satisfied — event 79 answered `draft`, which the probe does not declare
    session L1-campaign-present-p3-r27: not satisfied — event 80 answered `current`, which the probe does not declare
    session L1-campaign-present-p3-r28: not satisfied — event 81 answered `draft`, which the probe does not declare
    session L1-campaign-present-p3-r29: not satisfied — event 82 answered `current`, which the probe does not declare
    session L1-campaign-present-p3-r3: not satisfied — event 83 answered `current`, which the probe does not declare
    session L1-campaign-present-p3-r30: not satisfied — event 84 answered `draft`, which the probe does not declare
    session L1-campaign-present-p3-r4: not satisfied — event 85 answered `draft`, which the probe does not declare
    session L1-campaign-present-p3-r5: not satisfied — event 86 answered `current`, which the probe does not declare
    session L1-campaign-present-p3-r6: not satisfied — event 87 answered `current`, which the probe does not declare
    session L1-campaign-present-p3-r7: not satisfied — event 88 answered `draft`, which the probe does not declare
    session L1-campaign-present-p3-r8: not satisfied — event 89 answered `draft`, which the probe does not declare
    session L1-campaign-present-p3-r9: not satisfied — event 90 answered `current`, which the probe does not declare
- HW-PROBE-a-session-records-an-unmeasured-claim-in-the-shape-this-corpus-checks (sufficiency, expects patched)
    session L1-campaign-present-p4-r1: satisfied — event 91: `section.required.missing` reported nothing over `docs/obligations/0224-nothing-states-how-much-of-a-session-s-budget-the-standing-instructions-consume-before-any-work-starts.md`
    session L1-campaign-present-p4-r10: satisfied — event 92: `section.required.missing` reported nothing over `docs/obligations/0224-nothing-states-how-much-of-a-session-s-budget-this-repository-s-standing-instructions-consume-before-any-work-starts.md`
    session L1-campaign-present-p4-r11: satisfied — event 93: `section.required.missing` reported nothing over `docs/obligations/0224-the-standing-instruction-cost-of-a-session-is-named-and-never-measured.md`
    session L1-campaign-present-p4-r12: satisfied — event 94: `section.required.missing` reported nothing over `docs/obligations/0224-nothing-states-what-a-session-s-standing-instructions-cost-before-work-starts.md`
    session L1-campaign-present-p4-r13: satisfied — event 95: `section.required.missing` reported nothing over `docs/obligations/0224-nothing-states-how-much-of-a-session-s-budget-the-standing-instructions-of-this-repository-consume-before-any-work-starts.md`
    session L1-campaign-present-p4-r14: satisfied — event 96: `section.required.missing` reported nothing over `docs/obligations/0224-claude-md-has-no-reading-of-the-session-budget-it-spends-before-a-dispatch-s-first-turn.md`
    session L1-campaign-present-p4-r15: satisfied — event 97: `section.required.missing` reported nothing over `docs/obligations/0224-the-session-budget-cost-of-this-repository-s-standing-instructions-before-any-work-starts-is-unstated.md`
    session L1-campaign-present-p4-r16: satisfied — event 98: `section.required.missing` reported nothing over `docs/obligations/0224-this-repository-s-standing-instructions-have-no-reading-of-the-session-budget-they-consume-before-work-starts.md`
    session L1-campaign-present-p4-r17: not satisfied — the session produced no artifact, so the oracle had nothing to read
    session L1-campaign-present-p4-r18: satisfied — event 100: `section.required.missing` reported nothing over `docs/obligations/0224-nothing-states-the-token-floor-claude-md-the-skills-and-the-hooks-spend-before-a-session-does-any-task-work.md`
    session L1-campaign-present-p4-r19: not satisfied — the session produced no artifact, so the oracle had nothing to read
    session L1-campaign-present-p4-r2: satisfied — event 102: `section.required.missing` reported nothing over `docs/obligations/0224-this-repository-s-standing-instructions-have-no-measured-cost-before-a-session-opens-a-document.md`
    session L1-campaign-present-p4-r20: satisfied — event 103: `section.required.missing` reported nothing over `docs/obligations/0224-the-standing-instructions-of-this-repository-spend-an-unmeasured-part-of-a-session-s-budget-before-work-starts.md`
    session L1-campaign-present-p4-r21: satisfied — event 104: `section.required.missing` reported nothing over `docs/obligations/0224-nothing-states-how-much-of-a-session-s-budget-claude-md-and-its-named-skills-spend-before-any-work-starts.md`
    session L1-campaign-present-p4-r22: satisfied — event 105: `section.required.missing` reported nothing over `docs/obligations/0224-the-standing-context-a-session-pays-before-any-work-starts-has-no-measured-size.md`
    session L1-campaign-present-p4-r23: satisfied — event 106: `section.required.missing` reported nothing over `docs/obligations/0224-nothing-states-how-much-of-a-session-s-budget-the-standing-instructions-of-this-repository-consume-before-any-work-starts.md`
    session L1-campaign-present-p4-r24: satisfied — event 107: `section.required.missing` reported nothing over `docs/obligations/0224-nothing-states-how-much-of-a-session-s-budget-the-standing-instructions-consume-before-any-work-starts.md`
    session L1-campaign-present-p4-r25: satisfied — event 108: `section.required.missing` reported nothing over `docs/obligations/0224-nothing-states-how-much-of-a-session-s-budget-the-standing-instructions-of-this-repository-consume-before-any-work-starts.md`
    session L1-campaign-present-p4-r26: satisfied — event 109: `section.required.missing` reported nothing over `docs/obligations/0224-nothing-states-how-much-of-a-session-s-budget-the-standing-instructions-of-this-repository-consume-before-any-work-starts.md`
    session L1-campaign-present-p4-r27: satisfied — event 110: `section.required.missing` reported nothing over `docs/obligations/0224-nothing-states-how-much-of-a-session-s-budget-the-standing-instructions-consume-before-any-work-starts.md`
    session L1-campaign-present-p4-r28: satisfied — event 111: `section.required.missing` reported nothing over `docs/obligations/0224-nothing-states-how-much-of-a-session-s-budget-its-standing-instructions-consume-before-any-work-starts.md`
    session L1-campaign-present-p4-r29: satisfied — event 112: `section.required.missing` reported nothing over `docs/obligations/0224-nothing-states-how-much-of-a-session-s-budget-the-standing-instructions-of-this-repository-consume-before-any-work-starts.md`
    session L1-campaign-present-p4-r3: satisfied — event 113: `section.required.missing` reported nothing over `docs/obligations/0224-nothing-states-how-much-of-a-session-s-budget-the-standing-instructions-of-this-repository-consume-before-any-work-starts.md`
    session L1-campaign-present-p4-r30: satisfied — event 114: `section.required.missing` reported nothing over `docs/obligations/0224-nothing-states-how-much-of-a-session-s-budget-the-standing-instructions-of-this-repository-consume-before-any-work-starts.md`
    session L1-campaign-present-p4-r4: satisfied — event 115: `section.required.missing` reported nothing over `docs/obligations/0224-the-session-budget-cost-of-this-repository-s-standing-instructions-before-any-work-starts-is-unmeasured.md`
    session L1-campaign-present-p4-r5: satisfied — event 116: `section.required.missing` reported nothing over `docs/obligations/0224-nothing-states-how-much-of-a-session-s-budget-the-standing-instructions-of-this-repository-consume-before-any-work-starts.md`
    session L1-campaign-present-p4-r6: satisfied — event 117: `section.required.missing` reported nothing over `docs/obligations/0224-nothing-states-how-much-of-a-session-s-budget-the-standing-instructions-of-this-repository-consume-before-any-work-starts.md`
    session L1-campaign-present-p4-r7: satisfied — event 118: `section.required.missing` reported nothing over `docs/obligations/0224-the-session-budget-that-standing-instructions-consume-before-any-work-starts-is-unstated.md`
    session L1-campaign-present-p4-r8: not satisfied — the session produced no artifact, so the oracle had nothing to read
    session L1-campaign-present-p4-r9: satisfied — event 120: `section.required.missing` reported nothing over `docs/obligations/0224-nothing-states-how-much-of-a-session-s-budget-the-standing-instructions-consume-before-work-starts.md`

## The rate, and the denominator it is over

71 of 120 graded sessions satisfied their expectation: 59.2%, in a 95% interval of 50.2% to 67.5%.
The denominator is the graded sessions and never the selected probes. 0 sessions reached no verdict, and a session with no verdict is outside both halves of that fraction.

An interval that overlaps the previous run's is variance and one that does not is drift. This is one arm, so it estimates no effect: an efficacy claim is a comparison of two results, and the arm each one recorded is on it.

## The comparisons this arm takes part in

What the governance changes. The treated arm is the `campaign` present arm and the control is the `campaign` absent arm, which removed the paths the `campaign` tier's `ablation` names and kept `docs/`.

- treated, `docs/probe-runs/campaign-of-2026-09-28-campaign-tier-present-arm-sufficiency.md`: 71 of 120 graded sessions satisfied their expectation, 59.2%, in a 95% interval of 50.2% to 67.5%. 0 sessions refused by the session itself.
- control, `docs/probe-runs/campaign-of-2026-09-28-campaign-tier-absent-arm-sufficiency.md`: 76 of 120 graded sessions satisfied their expectation, 63.3%, in a 95% interval of 54.4% to 71.4%. 0 sessions refused by the session itself.

The difference is -4.2 points, in a 95% Newcombe interval of -16.2 points to +8.1 points. The transcript of at least one arm is not current, so this page states no direction at the 5% level.

What the documents and the governance change together. The treated arm is a present arm and the control is the `documentation` absent arm.

- treated, `docs/probe-runs/campaign-of-2026-09-28-campaign-tier-present-arm-sufficiency.md`: 71 of 120 graded sessions satisfied their expectation, 59.2%, in a 95% interval of 50.2% to 67.5%. 0 sessions refused by the session itself.
- control, `docs/probe-runs/campaign-of-2026-09-28-documentation-tier-absent-arm-sufficiency.md`: 22 of 120 graded sessions satisfied their expectation, 18.3%, in a 95% interval of 12.4% to 26.2%. 0 sessions refused by the session itself.

The difference is +40.8 points, in a 95% Newcombe interval of +28.9 points to +51.1 points. The transcript of at least one arm is not current, so this page states no direction at the 5% level.
