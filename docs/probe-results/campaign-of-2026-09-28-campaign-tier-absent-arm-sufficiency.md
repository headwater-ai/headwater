---
"headwater:generated": "probe_result. `headwater generate` writes this file, and `headwater generate --check` holds it. Edit the corpus, not this file."
id: HW-RESULT-campaign-of-2026-09-28-campaign-tier-absent-arm-sufficiency
title: Probe result for campaign-of-2026-09-28-campaign-tier-absent-arm-sufficiency
status: current
status_since: 2026-09-28
summary: "The grade of the transcript `campaign-of-2026-09-28-campaign-tier-absent-arm-sufficiency`, taken over the probes this corpus declares and the version of the grader that evaluated them."
last_verified: 2026-09-28
---

# The result of docs/probe-runs/campaign-of-2026-09-28-campaign-tier-absent-arm-sufficiency.md

A probe result is a function of three committed inputs and of nothing else: the transcript at `docs/probe-runs/campaign-of-2026-09-28-campaign-tier-absent-arm-sufficiency.md`, the expectations the probes of this corpus declare, and the version of the grader that evaluated them. Fetch the three and this file comes back.

## The run this transcript recorded

A campaign run in the absent arm, on claude-sonnet-5 at 2026-09-28.
served version claude-sonnet-5, tree sha256:1400e02747fbd854d6412fb6de5c06de282a2718c981e379a56d8b1f6659a775, selection sha256:1d93b514529314b74499196391c68b4b3d1de26853dd9d5ba7f3b455f54ed91d, read set sha256:7a76c0b28551210b534705cf2b69216c68b6543dba250fc82fb24861e4949f46, seed 0, harness 0.4.0.
realized cost $47.47, which the adaptive layer reads as the cost of its own instrument.
It was planned against taxonomy sha256:855406fbdc6d15574d11abde328bd66b371ee02db99d87ca9b35ae5fabaf5e09, and this tree carries another. The read set of its probes is the one this tree composes, so the move reaches no document a verdict reads, and the transcript is read over this tree.

120 events over 4 of the 10 probes this corpus declares, in 120 sessions and 1277 tool calls.

The engine confirmed the taxonomy, that every member of the run identity is present, the membership of every probe named, that no key outside the closed set appears, and that a realized cost was recorded. Present is not confirmed: of the six members a plan fixes before a run, the lock is the one compared here. A lock that differs refuses the file unless the read set of its probes is the one this tree composes. `headwater probe stale` compares the read set against the tree in front of it. It graded nothing: a verdict is a function of this transcript, the expectations these probes declare and a grader version, and `headwater probe grade` is the verb that holds all three.

**The selection this transcript names is not the selection this corpus composes.** The transcript names `sha256:1d93b514529314b74499196391c68b4b3d1de26853dd9d5ba7f3b455f54ed91d` and this corpus composes `sha256:50ed5ce43072d9f674a8dc52e1dcad84de4d649e39a77ce22179836188196b09`. The transcript's digest is the digest of 4 of the 10 probes this corpus composes, so the run was planned over that part of the selection, or the rest were added after it was recorded. Every verdict below is over that part, and a probe outside it is not a session this run owed. The tree, the seed and the harness above are provenance and nothing compares them.

The `read_set` digest above covers every probe of the selection and every document one of them examines, by path and content. It is recorded here and compared nowhere in this file. A comparison against the tree in front of a reader would move these bytes on every edit to a document the selection points at, and `generate --check` holds this file to its bytes, so the staleness of a measurement would stop a merge. `headwater probe stale` takes the digest and reports which recorded results a change voided.

Graded by grader 0.4.1.
A campaign run in the absent arm, on claude-sonnet-5 at served version claude-sonnet-5.

## The verdicts

- HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer (sufficiency, expects answered)
    session L2-campaign-absent-p1-r1: satisfied — event 1: answered `absent`
    session L2-campaign-absent-p1-r10: satisfied — event 2: answered `absent`
    session L2-campaign-absent-p1-r11: satisfied — event 3: answered `absent`
    session L2-campaign-absent-p1-r12: satisfied — event 4: answered `absent`
    session L2-campaign-absent-p1-r13: satisfied — event 5: answered `absent`
    session L2-campaign-absent-p1-r14: satisfied — event 6: answered `absent`
    session L2-campaign-absent-p1-r15: satisfied — event 7: answered `absent`
    session L2-campaign-absent-p1-r16: satisfied — event 8: answered `absent`
    session L2-campaign-absent-p1-r17: not satisfied — event 9 answered `withheld`, and the probe expects `absent`
    session L2-campaign-absent-p1-r18: satisfied — event 10: answered `absent`
    session L2-campaign-absent-p1-r19: satisfied — event 11: answered `absent`
    session L2-campaign-absent-p1-r2: satisfied — event 12: answered `absent`
    session L2-campaign-absent-p1-r20: satisfied — event 13: answered `absent`
    session L2-campaign-absent-p1-r21: not satisfied — event 14 answered `withheld`, and the probe expects `absent`
    session L2-campaign-absent-p1-r22: satisfied — event 15: answered `absent`
    session L2-campaign-absent-p1-r23: satisfied — event 16: answered `absent`
    session L2-campaign-absent-p1-r24: satisfied — event 17: answered `absent`
    session L2-campaign-absent-p1-r25: satisfied — event 18: answered `absent`
    session L2-campaign-absent-p1-r26: satisfied — event 19: answered `absent`
    session L2-campaign-absent-p1-r27: satisfied — event 20: answered `absent`
    session L2-campaign-absent-p1-r28: satisfied — event 21: answered `absent`
    session L2-campaign-absent-p1-r29: satisfied — event 22: answered `absent`
    session L2-campaign-absent-p1-r3: satisfied — event 23: answered `absent`
    session L2-campaign-absent-p1-r30: satisfied — event 24: answered `absent`
    session L2-campaign-absent-p1-r4: satisfied — event 25: answered `absent`
    session L2-campaign-absent-p1-r5: satisfied — event 26: answered `absent`
    session L2-campaign-absent-p1-r6: satisfied — event 27: answered `absent`
    session L2-campaign-absent-p1-r7: not satisfied — event 28 answered `withheld`, and the probe expects `absent`
    session L2-campaign-absent-p1-r8: satisfied — event 29: answered `absent`
    session L2-campaign-absent-p1-r9: satisfied — event 30: answered `absent`
- HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted (sufficiency, expects answered)
    session L2-campaign-absent-p2-r1: satisfied — event 31: answered `merge`
    session L2-campaign-absent-p2-r10: not satisfied — event 32 answered `stamp`, and the probe expects `merge`
    session L2-campaign-absent-p2-r11: satisfied — event 33: answered `merge`
    session L2-campaign-absent-p2-r12: satisfied — event 34: answered `merge`
    session L2-campaign-absent-p2-r13: not satisfied — event 35 answered `stamp`, and the probe expects `merge`
    session L2-campaign-absent-p2-r14: satisfied — event 36: answered `merge`
    session L2-campaign-absent-p2-r15: satisfied — event 37: answered `merge`
    session L2-campaign-absent-p2-r16: satisfied — event 38: answered `merge`
    session L2-campaign-absent-p2-r17: satisfied — event 39: answered `merge`
    session L2-campaign-absent-p2-r18: satisfied — event 40: answered `merge`
    session L2-campaign-absent-p2-r19: satisfied — event 41: answered `merge`
    session L2-campaign-absent-p2-r2: satisfied — event 42: answered `merge`
    session L2-campaign-absent-p2-r20: satisfied — event 43: answered `merge`
    session L2-campaign-absent-p2-r21: satisfied — event 44: answered `merge`
    session L2-campaign-absent-p2-r22: satisfied — event 45: answered `merge`
    session L2-campaign-absent-p2-r23: satisfied — event 46: answered `merge`
    session L2-campaign-absent-p2-r24: satisfied — event 47: answered `merge`
    session L2-campaign-absent-p2-r25: satisfied — event 48: answered `merge`
    session L2-campaign-absent-p2-r26: not satisfied — event 49 answered `stamp`, and the probe expects `merge`
    session L2-campaign-absent-p2-r27: satisfied — event 50: answered `merge`
    session L2-campaign-absent-p2-r28: not satisfied — event 51 answered `stamp`, and the probe expects `merge`
    session L2-campaign-absent-p2-r29: satisfied — event 52: answered `merge`
    session L2-campaign-absent-p2-r3: satisfied — event 53: answered `merge`
    session L2-campaign-absent-p2-r30: satisfied — event 54: answered `merge`
    session L2-campaign-absent-p2-r4: satisfied — event 55: answered `merge`
    session L2-campaign-absent-p2-r5: satisfied — event 56: answered `merge`
    session L2-campaign-absent-p2-r6: not satisfied — event 57 answered `stamp`, and the probe expects `merge`
    session L2-campaign-absent-p2-r7: satisfied — event 58: answered `merge`
    session L2-campaign-absent-p2-r8: not satisfied — event 59 answered `stamp`, and the probe expects `merge`
    session L2-campaign-absent-p2-r9: satisfied — event 60: answered `merge`
- HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request (sufficiency, expects answered)
    session L2-campaign-absent-p3-r1: satisfied — event 61: answered `current`
    session L2-campaign-absent-p3-r10: satisfied — event 62: answered `current`
    session L2-campaign-absent-p3-r11: satisfied — event 63: answered `current`
    session L2-campaign-absent-p3-r12: satisfied — event 64: answered `current`
    session L2-campaign-absent-p3-r13: satisfied — event 65: answered `current`
    session L2-campaign-absent-p3-r14: satisfied — event 66: answered `current`
    session L2-campaign-absent-p3-r15: satisfied — event 67: answered `current`
    session L2-campaign-absent-p3-r16: satisfied — event 68: answered `current`
    session L2-campaign-absent-p3-r17: satisfied — event 69: answered `current`
    session L2-campaign-absent-p3-r18: satisfied — event 70: answered `current`
    session L2-campaign-absent-p3-r19: satisfied — event 71: answered `current`
    session L2-campaign-absent-p3-r2: satisfied — event 72: answered `current`
    session L2-campaign-absent-p3-r20: satisfied — event 73: answered `current`
    session L2-campaign-absent-p3-r21: satisfied — event 74: answered `current`
    session L2-campaign-absent-p3-r22: satisfied — event 75: answered `current`
    session L2-campaign-absent-p3-r23: satisfied — event 76: answered `current`
    session L2-campaign-absent-p3-r24: satisfied — event 77: answered `current`
    session L2-campaign-absent-p3-r25: satisfied — event 78: answered `current`
    session L2-campaign-absent-p3-r26: satisfied — event 79: answered `current`
    session L2-campaign-absent-p3-r27: satisfied — event 80: answered `current`
    session L2-campaign-absent-p3-r28: satisfied — event 81: answered `current`
    session L2-campaign-absent-p3-r29: satisfied — event 82: answered `current`
    session L2-campaign-absent-p3-r3: satisfied — event 83: answered `current`
    session L2-campaign-absent-p3-r30: satisfied — event 84: answered `current`
    session L2-campaign-absent-p3-r4: satisfied — event 85: answered `current`
    session L2-campaign-absent-p3-r5: satisfied — event 86: answered `current`
    session L2-campaign-absent-p3-r6: satisfied — event 87: answered `current`
    session L2-campaign-absent-p3-r7: satisfied — event 88: answered `current`
    session L2-campaign-absent-p3-r8: satisfied — event 89: answered `current`
    session L2-campaign-absent-p3-r9: satisfied — event 90: answered `current`
- HW-PROBE-a-session-records-an-unmeasured-claim-in-the-shape-this-corpus-checks (sufficiency, expects patched)
    session L2-campaign-absent-p4-r1: satisfied — event 91: `section.required.missing` reported nothing over `docs/obligations/0223-c1-standing-context-is-charged-to-every-session-and-no-document-prices-it.md`
    session L2-campaign-absent-p4-r10: satisfied — event 92: `section.required.missing` reported nothing over `docs/obligations/0223-nothing-states-how-much-of-a-sessions-standing-instructions-cost-before-its-first-turn-of-work.md`
    session L2-campaign-absent-p4-r11: satisfied — event 93: `section.required.missing` reported nothing over `docs/obligations/0223-claude-md-costs-6-500-tokens-on-every-dispatch-and-nothing-turns-that-into-a-share-of-a-session-s-budget.md`
    session L2-campaign-absent-p4-r12: satisfied — event 94: `section.required.missing` reported nothing over `docs/obligations/0224-the-token-cost-of-c1-standing-context-has-no-stated-figure.md`
    session L2-campaign-absent-p4-r13: satisfied — event 95: `section.required.missing` reported nothing over `docs/obligations/0223-the-always-on-cost-of-standing-context-has-no-stated-size.md`
    session L2-campaign-absent-p4-r14: satisfied — event 96: `section.required.missing` reported nothing over `docs/obligations/0224-spec-5-says-standing-context-costs-a-session-and-no-document-states-how-much.md`
    session L2-campaign-absent-p4-r15: satisfied — event 97: `section.required.missing` reported nothing over `docs/obligations/0219-the-context-cost-of-claude-md-and-agents-md-before-a-session-s-first-turn-is-unmeasured.md`
    session L2-campaign-absent-p4-r16: satisfied — event 98: `section.required.missing` reported nothing over `docs/obligations/0224-a-sessions-standing-instructions-have-a-token-cost-and-nothing-here-has-ever-measured-one.md`
    session L2-campaign-absent-p4-r17: satisfied — event 99: `section.required.missing` reported nothing over `docs/obligations/0224-the-size-of-the-always-on-prompt-this-repository-pays-every-session-is-unmeasured.md`
    session L2-campaign-absent-p4-r18: satisfied — event 100: `section.required.missing` reported nothing over `docs/obligations/0224-nothing-states-how-much-of-a-sessions-budget-the-standing-context-spends-before-any-work-starts.md`
    session L2-campaign-absent-p4-r19: satisfied — event 101: `section.required.missing` reported nothing over `docs/obligations/0224-standing-instructions-cost-tokens-before-a-session-does-any-work.md`
    session L2-campaign-absent-p4-r2: satisfied — event 102: `section.required.missing` reported nothing over `docs/obligations/0223-the-standing-context-file-costs-its-size-on-every-session-and-nothing-states-what-that-size-is.md`
    session L2-campaign-absent-p4-r20: satisfied — event 103: `section.required.missing` reported nothing over `docs/obligations/0224-session-cost-cents-prices-every-arm-alike-and-no-comparison-reads-a-transcripts-realized-cost-by-arm.md`
    session L2-campaign-absent-p4-r21: satisfied — event 104: `section.required.missing` reported nothing over `docs/obligations/0223-nothing-states-how-much-of-a-session-s-budget-this-repository-s-standing-instructions-consume-before-work-starts.md`
    session L2-campaign-absent-p4-r22: satisfied — event 105: `section.required.missing` reported nothing over `docs/obligations/0223-each-standing-instruction-file-declares-its-own-ceiling-and-nothing-sums-what-a-session-pays-before-it-does-any-work.md`
    session L2-campaign-absent-p4-r23: satisfied — event 106: `section.required.missing` reported nothing over `docs/obligations/0223-nothing-states-how-much-of-a-session-s-context-budget-this-repository-s-standing-instructions-spend-before-work-starts.md`
    session L2-campaign-absent-p4-r24: satisfied — event 107: `section.required.missing` reported nothing over `docs/obligations/0224-a-declared-session-cost-cents-prices-a-whole-session-and-no-reading-separates-what-standing-instructions-spend-before-work-starts.md`
    session L2-campaign-absent-p4-r25: not satisfied — the session produced no artifact, so the oracle had nothing to read
    session L2-campaign-absent-p4-r26: satisfied — event 109: `section.required.missing` reported nothing over `docs/obligations/0224-spec-16-says-spec-5-prices-the-standing-instructions-cost-and-spec-5-states-the-argument-with-no-number.md`
    session L2-campaign-absent-p4-r27: satisfied — event 110: `section.required.missing` reported nothing over `docs/obligations/0224-nothing-states-how-many-tokens-of-a-sessions-standing-instructions.md`
    session L2-campaign-absent-p4-r28: satisfied — event 111: `section.required.missing` reported nothing over `docs/obligations/0224-nothing-states-how-much-of-a-sessions-budget-the-standing-instructions-of-this-repository-consume-before-any-work-starts.md`
    session L2-campaign-absent-p4-r29: not satisfied — the session produced no artifact, so the oracle had nothing to read
    session L2-campaign-absent-p4-r3: satisfied — event 113: `section.required.missing` reported nothing over `docs/obligations/0223-nothing-states-what-hw-run-policy-costs-a-session-before-its-work-begins.md`
    session L2-campaign-absent-p4-r30: satisfied — event 114: `section.required.missing` reported nothing over `docs/obligations/0223-no-record-measures-how-many-tokens-claude-md-costs-a-dispatch-before-any-work-starts.md`
    session L2-campaign-absent-p4-r4: satisfied — event 115: `section.required.missing` reported nothing over `docs/obligations/0224-nothing-states-how-much-of-a-sessions-budget-the-standing-instructions-of-this-repository-consume-before-any-work-starts.md`
    session L2-campaign-absent-p4-r5: satisfied — event 116: `section.required.missing` reported nothing over `docs/obligations/0224-claude-md-s-cost-lands-on-every-dispatch-and-no-document-states-its-size.md`
    session L2-campaign-absent-p4-r6: not satisfied — the session produced no artifact, so the oracle had nothing to read
    session L2-campaign-absent-p4-r7: satisfied — event 118: `section.required.missing` reported nothing over `docs/obligations/0224-spec-16-says-standing-context-costs-its-size-on-every-session-and-no-document-states-that-size.md`
    session L2-campaign-absent-p4-r8: not satisfied — the session produced no artifact, so the oracle had nothing to read
    session L2-campaign-absent-p4-r9: not satisfied — the session produced no artifact, so the oracle had nothing to read

## The rate, and the denominator it is over

106 of 120 graded sessions satisfied their expectation: 88.3%, in a 95% interval of 81.4% to 92.9%.
The denominator is the graded sessions and never the selected probes. 0 sessions reached no verdict, and a session with no verdict is outside both halves of that fraction.

An interval that overlaps the previous run's is variance and one that does not is drift. This is one arm, so it estimates no effect: an efficacy claim is a comparison of two results, and the arm each one recorded is on it.

## The comparisons this arm takes part in

What the governance changes. The treated arm is the `campaign` present arm and the control is the `campaign` absent arm, which removed the paths the `campaign` tier's `ablation` names and kept `docs/`.

- treated, `docs/probe-runs/campaign-of-2026-09-28-campaign-tier-present-arm-sufficiency.md`: 92 of 120 graded sessions satisfied their expectation, 76.7%, in a 95% interval of 68.3% to 83.3%. 0 sessions refused by the session itself.
- control, `docs/probe-runs/campaign-of-2026-09-28-campaign-tier-absent-arm-sufficiency.md`: 106 of 120 graded sessions satisfied their expectation, 88.3%, in a 95% interval of 81.4% to 92.9%. 0 sessions refused by the session itself.

The difference is -11.7 points, in a 95% Newcombe interval of -21.2 points to -2.0 points. The interval is below zero, so the treated arm satisfied less often at the 5% level.

What the documents change. The treated arm is the `campaign` absent arm and the control is the `documentation` absent arm. Both removed the governance, and only the control removed `docs/`, so this is the effect of the documents alone (spec 5).

- treated, `docs/probe-runs/campaign-of-2026-09-28-campaign-tier-absent-arm-sufficiency.md`: 106 of 120 graded sessions satisfied their expectation, 88.3%, in a 95% interval of 81.4% to 92.9%. 0 sessions refused by the session itself.
- control, `docs/probe-runs/campaign-of-2026-09-28-documentation-tier-absent-arm-sufficiency.md`: 50 of 120 graded sessions satisfied their expectation, 41.7%, in a 95% interval of 33.2% to 50.6%. 0 sessions refused by the session itself.

The difference is +46.7 points, in a 95% Newcombe interval of +35.3 points to +56.3 points. The interval is above zero, so the treated arm satisfied more often at the 5% level.
