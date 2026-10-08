---
"headwater:generated": "probe_result. `headwater generate` writes this file, and `headwater generate --check` holds it. Edit the corpus, not this file."
id: HW-RESULT-campaign-of-2026-10-03-campaign-tier-no-claude-md-arm-sufficiency-leak-kept-probes
title: Probe result for campaign-of-2026-10-03-campaign-tier-no-claude-md-arm-sufficiency-leak-kept-probes
status: current
status_since: 2026-10-08
summary: "The grade of the transcript `campaign-of-2026-10-03-campaign-tier-no-claude-md-arm-sufficiency-leak-kept-probes`, taken over the probes this corpus declares and the version of the grader that evaluated them."
last_verified: 2026-10-08
---

# The result of docs/probe-runs/campaign-of-2026-10-03-campaign-tier-no-claude-md-arm-sufficiency-leak-kept-probes.md

A probe result is a function of committed inputs and of nothing else: the transcript at `docs/probe-runs/campaign-of-2026-10-03-campaign-tier-no-claude-md-arm-sufficiency-leak-kept-probes.md` and the state it stands at, the expectations the probes it names declare, the version of the grader that evaluated them, and for a compared arm the result of the other arm. Fetch those and this file comes back.

## The run this transcript recorded

A campaign run in the no-claude-md arm, on claude-sonnet-5 at 2026-10-03.
served version claude-sonnet-5, tree sha256:81d5f62acb289022175d192df0a61faf5f71eb4b79e8dd1f4efae4532ebdd916, selection sha256:18ec419cfebb740a16b74470a04e90ca181b3c78bef48e6ec5969aff76f1f37d, read set sha256:f565abcbde3c4a2d20c3e27c799dda32c5d6fe6fa691eedf3012d5c5df417004, seed 0, harness 0.5.0.
realized cost $4.48, which the adaptive layer reads as the cost of its own instrument.

60 events over 2 probes, in 60 sessions and 14 tool calls.

The engine confirmed the taxonomy, that every member of the run identity is present, the membership of every probe named, that no key outside the closed set appears, and that a realized cost was recorded. Present is not confirmed. This page names the lock, the selection and the read set the transcript recorded, and it compares none of them with the tree in front of a reader. `headwater generate` and `headwater probe stale` name what moved since the recording, and neither one fails for it. It graded nothing: a verdict is a function of this transcript, the expectations these probes declare and a grader version, and `headwater probe grade` is the verb that holds all three.

The selection this transcript names is `sha256:18ec419cfebb740a16b74470a04e90ca181b3c78bef48e6ec5969aff76f1f37d`, and the probes graded below are the probes it was planned over, as this corpus declares them now. A probe added to this corpus after the recording is not a session this run owed. The tree, the seed and the harness above are provenance and nothing compares them.

The `read_set` digest above covers every probe of the selection and every document one of them examines, by path and content. This file names it as provenance and compares it with nothing, and the same holds for the lock and the selection: an edit to a document, a new probe or a moved lock leaves these bytes alone. `headwater generate` names each result whose recorded lock, read set or selection moved, and `headwater probe stale` takes the digest and reports which recorded results a change voided. Neither one fails for it.

Graded by grader 0.5.0.
A campaign run in the no-claude-md arm, on claude-sonnet-5 at served version claude-sonnet-5.

## The verdicts

- HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted (sufficiency, expects answered)
    session L12-campaign-no-claude-md-p1-r1: satisfied — event 1: answered `merge`
    session L12-campaign-no-claude-md-p1-r10: satisfied — event 2: answered `merge`
    session L12-campaign-no-claude-md-p1-r11: satisfied — event 3: answered `merge`
    session L12-campaign-no-claude-md-p1-r12: satisfied — event 4: answered `merge`
    session L12-campaign-no-claude-md-p1-r13: satisfied — event 5: answered `merge`
    session L12-campaign-no-claude-md-p1-r14: satisfied — event 6: answered `merge`
    session L12-campaign-no-claude-md-p1-r15: satisfied — event 7: answered `merge`
    session L12-campaign-no-claude-md-p1-r16: satisfied — event 8: answered `merge`
    session L12-campaign-no-claude-md-p1-r17: satisfied — event 9: answered `merge`
    session L12-campaign-no-claude-md-p1-r18: satisfied — event 10: answered `merge`
    session L12-campaign-no-claude-md-p1-r19: satisfied — event 11: answered `merge`
    session L12-campaign-no-claude-md-p1-r2: satisfied — event 12: answered `merge`
    session L12-campaign-no-claude-md-p1-r20: satisfied — event 13: answered `merge`
    session L12-campaign-no-claude-md-p1-r21: satisfied — event 14: answered `merge`
    session L12-campaign-no-claude-md-p1-r22: satisfied — event 15: answered `merge`
    session L12-campaign-no-claude-md-p1-r23: satisfied — event 16: answered `merge`
    session L12-campaign-no-claude-md-p1-r24: satisfied — event 17: answered `merge`
    session L12-campaign-no-claude-md-p1-r25: satisfied — event 18: answered `merge`
    session L12-campaign-no-claude-md-p1-r26: satisfied — event 19: answered `merge`
    session L12-campaign-no-claude-md-p1-r27: satisfied — event 20: answered `merge`
    session L12-campaign-no-claude-md-p1-r28: satisfied — event 21: answered `merge`
    session L12-campaign-no-claude-md-p1-r29: satisfied — event 22: answered `merge`
    session L12-campaign-no-claude-md-p1-r3: satisfied — event 23: answered `merge`
    session L12-campaign-no-claude-md-p1-r30: satisfied — event 24: answered `merge`
    session L12-campaign-no-claude-md-p1-r4: satisfied — event 25: answered `merge`
    session L12-campaign-no-claude-md-p1-r5: satisfied — event 26: answered `merge`
    session L12-campaign-no-claude-md-p1-r6: satisfied — event 27: answered `merge`
    session L12-campaign-no-claude-md-p1-r7: satisfied — event 28: answered `merge`
    session L12-campaign-no-claude-md-p1-r8: satisfied — event 29: answered `merge`
    session L12-campaign-no-claude-md-p1-r9: satisfied — event 30: answered `merge`
- HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request (sufficiency, expects answered)
    session L12-campaign-no-claude-md-p2-r1: satisfied — event 31: answered `current HW-DR-0052`
    session L12-campaign-no-claude-md-p2-r10: satisfied — event 32: answered `current HW-DR-0052`
    session L12-campaign-no-claude-md-p2-r11: satisfied — event 33: answered `current HW-DR-0052`
    session L12-campaign-no-claude-md-p2-r12: satisfied — event 34: answered `current HW-DR-0052`
    session L12-campaign-no-claude-md-p2-r13: satisfied — event 35: answered `current HW-DR-0052`
    session L12-campaign-no-claude-md-p2-r14: satisfied — event 36: answered `current HW-DR-0052`
    session L12-campaign-no-claude-md-p2-r15: satisfied — event 37: answered `current HW-DR-0052`
    session L12-campaign-no-claude-md-p2-r16: satisfied — event 38: answered `current HW-DR-0052`
    session L12-campaign-no-claude-md-p2-r17: satisfied — event 39: answered `current HW-DR-0052`
    session L12-campaign-no-claude-md-p2-r18: satisfied — event 40: answered `current HW-DR-0052`
    session L12-campaign-no-claude-md-p2-r19: satisfied — event 41: answered `current HW-DR-0052`
    session L12-campaign-no-claude-md-p2-r2: satisfied — event 42: answered `current HW-DR-0052`
    session L12-campaign-no-claude-md-p2-r20: satisfied — event 43: answered `current HW-DR-0052`
    session L12-campaign-no-claude-md-p2-r21: satisfied — event 44: answered `current HW-DR-0052`
    session L12-campaign-no-claude-md-p2-r22: satisfied — event 45: answered `current HW-DR-0052`
    session L12-campaign-no-claude-md-p2-r23: satisfied — event 46: answered `current HW-DR-0052`
    session L12-campaign-no-claude-md-p2-r24: satisfied — event 47: answered `current HW-DR-0052`
    session L12-campaign-no-claude-md-p2-r25: satisfied — event 48: answered `current HW-DR-0052`
    session L12-campaign-no-claude-md-p2-r26: satisfied — event 49: answered `current HW-DR-0052`
    session L12-campaign-no-claude-md-p2-r27: satisfied — event 50: answered `current HW-DR-0052`
    session L12-campaign-no-claude-md-p2-r28: satisfied — event 51: answered `current HW-DR-0052`
    session L12-campaign-no-claude-md-p2-r29: satisfied — event 52: answered `current HW-DR-0052`
    session L12-campaign-no-claude-md-p2-r3: satisfied — event 53: answered `current HW-DR-0052`
    session L12-campaign-no-claude-md-p2-r30: satisfied — event 54: answered `current HW-DR-0052`
    session L12-campaign-no-claude-md-p2-r4: satisfied — event 55: answered `current HW-DR-0052`
    session L12-campaign-no-claude-md-p2-r5: satisfied — event 56: answered `current HW-DR-0052`
    session L12-campaign-no-claude-md-p2-r6: satisfied — event 57: answered `current HW-DR-0052`
    session L12-campaign-no-claude-md-p2-r7: satisfied — event 58: answered `current HW-DR-0052`
    session L12-campaign-no-claude-md-p2-r8: satisfied — event 59: answered `current HW-DR-0052`
    session L12-campaign-no-claude-md-p2-r9: satisfied — event 60: answered `current HW-DR-0052`

## The rate, and the denominator it is over

60 of 60 graded sessions satisfied their expectation: 100.0%, in a 95% interval of 94.0% to 100.0%.
The denominator is the graded sessions and never the selected probes. 0 sessions reached no verdict, and a session with no verdict is outside both halves of that fraction.

An interval that overlaps the previous run's is variance and one that does not is drift. This is one arm, so it estimates no effect: an efficacy claim is a comparison of two results, and the arm each one recorded is on it.

## The comparisons this arm takes part in

What `CLAUDE.md` changes. The treated arm is the `campaign` present arm and the control is the `campaign` `no-claude-md` arm, which removed the paths the `no-claude-md` component's delta names. The documents are in both arms (spec 5).

- treated, `docs/probe-runs/campaign-of-2026-10-03-campaign-tier-present-arm-sufficiency-leak-kept-probes.md`: 60 of 60 graded sessions satisfied their expectation, 100.0%, in a 95% interval of 94.0% to 100.0%. 0 sessions refused by the session itself.
- control, `docs/probe-runs/campaign-of-2026-10-03-campaign-tier-no-claude-md-arm-sufficiency-leak-kept-probes.md`: 60 of 60 graded sessions satisfied their expectation, 100.0%, in a 95% interval of 94.0% to 100.0%. 0 sessions refused by the session itself.

The difference is +0.0 points, in a 95% Newcombe interval of -6.0 points to +6.0 points. The interval contains zero, so this run does not separate the two arms at the 5% level.
