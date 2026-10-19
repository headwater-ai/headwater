---
"headwater:generated": "probe_result. `headwater generate` writes this file, and `headwater generate --check` holds it. Edit the corpus, not this file."
id: HW-RESULT-additive-screening-of-2026-10-18-scenario-8
title: Probe result for additive-screening-of-2026-10-18-scenario-8
status: current
status_since: 2026-10-19
summary: "The grade of the transcript `additive-screening-of-2026-10-18-scenario-8`, taken over the probes this corpus declares and the version of the grader that evaluated them."
last_verified: 2026-10-19
---

# The result of docs/probe-runs/additive-screening-of-2026-10-18-scenario-8.md

A probe result is a function of committed inputs and of nothing else: the transcript at `docs/probe-runs/additive-screening-of-2026-10-18-scenario-8.md` and the state it stands at, the expectations the probes it names declare, the version of the grader that evaluated them, and for a compared arm the result of the other arm. Fetch those and this file comes back.

## The run this transcript recorded

A additive run in the scenario-8 arm, on claude-sonnet-5 at 2026-10-18.
served version claude-sonnet-5, tree sha256:7d84079327a6496ad99d1047ec2c9d0f82a810c3765b0ae22a4ae3ea97be60e8, selection sha256:0e2113cd81af83407d78a1964326c730f04457f9a81c929f9e32bfc73585bda3, read set sha256:30284342a7371374ee5ded1720314683046f3f9f7b372ecbb70ff7e6f3e8b5d3, seed 0, harness 0.6.0.
realized cost $30.45, which the adaptive layer reads as the cost of its own instrument.

72 events over 4 probes, in 72 sessions and 1077 tool calls.

The engine confirmed the taxonomy, that every member of the run identity is present, the membership of every probe named, that no key outside the closed set appears, and that a realized cost was recorded. Present is not confirmed. This page names the lock, the selection and the read set the transcript recorded, and it compares none of them with the tree in front of a reader. `headwater generate` and `headwater probe stale` name what moved since the recording, and neither one fails for it. It graded nothing: a verdict is a function of this transcript, the expectations these probes declare and a grader version, and `headwater probe grade` is the verb that holds all three.

The selection this transcript names is `sha256:0e2113cd81af83407d78a1964326c730f04457f9a81c929f9e32bfc73585bda3`, and the probes graded below are the probes it was planned over, as this corpus declares them now. A probe added to this corpus after the recording is not a session this run owed. The tree, the seed and the harness above are provenance and nothing compares them.

The `read_set` digest above covers every probe of the selection and every document one of them examines, by path and content. This file names it as provenance and compares it with nothing, and the same holds for the lock and the selection: an edit to a document, a new probe or a moved lock leaves these bytes alone. `headwater generate` names each result whose recorded lock, read set or selection moved, and `headwater probe stale` takes the digest and reports which recorded results a change voided. Neither one fails for it.

Graded by grader 0.5.1.
A additive run in the scenario-8 arm, on claude-sonnet-5 at served version claude-sonnet-5.

## The verdicts

- HW-PROBE-a-cold-agent-reaches-the-governing-document-through-the-corpus-descriptor (discovery, expects opened)
    session L9-additive-scenario-8-p1-r1: satisfied — event 1, call 33: `Grep /mnt/build/hw-1775/screening/a/ws/L9-additive-scenario-8-p1-r1/docs/spec/12-check-layer.md`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p1-r10: not satisfied — 38 recorded calls, and none named any of the 1 document; no call to a tool of an MCP server
    session L9-additive-scenario-8-p1-r11: satisfied — event 3, call 21: `Grep /mnt/build/hw-1775/screening/a/ws/L9-additive-scenario-8-p1-r11/docs/spec/12-check-layer.md`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p1-r12: not satisfied — 13 recorded calls, and none named any of the 1 document; no call to a tool of an MCP server
    session L9-additive-scenario-8-p1-r13: not satisfied — 14 recorded calls, and none named any of the 1 document; no call to a tool of an MCP server
    session L9-additive-scenario-8-p1-r14: satisfied — event 6, call 26: `Grep /mnt/build/hw-1775/screening/a/ws/L9-additive-scenario-8-p1-r14/docs/spec/12-check-layer.md`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p1-r15: not satisfied — 16 recorded calls, and none named any of the 1 document; no call to a tool of an MCP server
    session L9-additive-scenario-8-p1-r16: satisfied — event 8, call 9: `Grep /mnt/build/hw-1775/screening/a/ws/L9-additive-scenario-8-p1-r16/docs/spec/12-check-layer.md`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p1-r17: satisfied — event 9, call 26: `Bash {"command":"sed -n '1,120p' /mnt/build/hw-1775/screening/a/ws/L9-additive-scenario-8-p1-r17/docs/spec/12-check-layer.md | grep -n \"origin\" -A 30 | head -150"}`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p1-r18: not satisfied — 21 recorded calls, and none named any of the 1 document; no call to a tool of an MCP server
    session L9-additive-scenario-8-p1-r2: not satisfied — 23 recorded calls, and none named any of the 1 document; no call to a tool of an MCP server
    session L9-additive-scenario-8-p1-r3: not satisfied — 15 recorded calls, and none named any of the 1 document; no call to a tool of an MCP server
    session L9-additive-scenario-8-p1-r4: satisfied — event 13, call 32: `Bash {"command":"cd /mnt/build/hw-1775/screening/a/ws/L9-additive-scenario-8-p1-r4\ngrep -n \"five origins\\|Shape-origin\\|Document-origin\\|Corpus-origin\\|Graph-origin\\|Engine-origin\" docs/spec/12-check-layer.md | head -20"}`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p1-r5: not satisfied — 12 recorded calls, and none named any of the 1 document; no call to a tool of an MCP server
    session L9-additive-scenario-8-p1-r6: not satisfied — 26 recorded calls, and none named any of the 1 document; no call to a tool of an MCP server
    session L9-additive-scenario-8-p1-r7: not satisfied — 12 recorded calls, and none named any of the 1 document; no call to a tool of an MCP server
    session L9-additive-scenario-8-p1-r8: satisfied — event 17, call 19: `Bash {"command":"grep -n \"posture\\|severity\" docs/spec/12-check-layer.md | head -60"}`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p1-r9: satisfied — event 18, call 20: `Grep /mnt/build/hw-1775/screening/a/ws/L9-additive-scenario-8-p1-r9/docs/spec/12-check-layer.md`; no call to a tool of an MCP server
- HW-PROBE-a-session-follows-two-citations-from-a-record-the-task-does-not-name (sufficiency, expects answered)
    session L9-additive-scenario-8-p3-r1: satisfied — event 37: answered `HW-DR-0097`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p3-r10: satisfied — event 38: answered `HW-DR-0097`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p3-r11: satisfied — event 39: answered `HW-DR-0097`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p3-r12: satisfied — event 40: answered `HW-DR-0097`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p3-r13: satisfied — event 41: answered `HW-DR-0097`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p3-r14: satisfied — event 42: answered `HW-DR-0097`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p3-r15: satisfied — event 43: answered `HW-DR-0097`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p3-r16: satisfied — event 44: answered `HW-DR-0097`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p3-r17: satisfied — event 45: answered `HW-DR-0097`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p3-r18: satisfied — event 46: answered `HW-DR-0097`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p3-r2: satisfied — event 47: answered `HW-DR-0097`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p3-r3: satisfied — event 48: answered `HW-DR-0097`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p3-r4: satisfied — event 49: answered `HW-DR-0097`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p3-r5: satisfied — event 50: answered `HW-DR-0097`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p3-r6: satisfied — event 51: answered `HW-DR-0097`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p3-r7: satisfied — event 52: answered `HW-DR-0097`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p3-r8: satisfied — event 53: answered `HW-DR-0097`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p3-r9: satisfied — event 54: answered `HW-DR-0097`; no call to a tool of an MCP server
- HW-PROBE-a-session-records-which-obligation-an-evaluation-discharged (sufficiency, expects patched)
    session L9-additive-scenario-8-p4-r1: satisfied — event 55: `relation.reciprocity.missing` reported nothing over `docs/evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p4-r10: satisfied — event 56: `relation.reciprocity.missing` reported nothing over `docs/evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p4-r11: satisfied — event 57: `relation.reciprocity.missing` reported nothing over `docs/evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p4-r12: satisfied — event 58: `relation.reciprocity.missing` reported nothing over `docs/evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p4-r13: satisfied — event 59: `relation.reciprocity.missing` reported nothing over `docs/evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p4-r14: not satisfied — `relation.reciprocity.missing` reported over each of the 1 produced artifact; no call to a tool of an MCP server
    session L9-additive-scenario-8-p4-r15: satisfied — event 61: `relation.reciprocity.missing` reported nothing over `docs/evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p4-r16: satisfied — event 62: `relation.reciprocity.missing` reported nothing over `docs/evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p4-r17: satisfied — event 63: `relation.reciprocity.missing` reported nothing over `docs/evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p4-r18: satisfied — event 64: `relation.reciprocity.missing` reported nothing over `docs/evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p4-r2: satisfied — event 65: `relation.reciprocity.missing` reported nothing over `docs/evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p4-r3: satisfied — event 66: `relation.reciprocity.missing` reported nothing over `docs/evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p4-r4: satisfied — event 67: `relation.reciprocity.missing` reported nothing over `docs/evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p4-r5: satisfied — event 68: `relation.reciprocity.missing` reported nothing over `docs/evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p4-r6: satisfied — event 69: `relation.reciprocity.missing` reported nothing over `docs/evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p4-r7: not satisfied — `relation.reciprocity.missing` reported over each of the 1 produced artifact; no call to a tool of an MCP server
    session L9-additive-scenario-8-p4-r8: satisfied — event 71: `relation.reciprocity.missing` reported nothing over `docs/evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p4-r9: satisfied — event 72: `relation.reciprocity.missing` reported nothing over `docs/evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md`; no call to a tool of an MCP server
- HW-PROBE-an-agent-reaches-the-adjudication-when-the-decision-a-tombstone-links-is-superseded (navigability, expects opened)
    session L9-additive-scenario-8-p2-r1: not satisfied — 3 recorded calls, and none named any of the 1 document; no call to a tool of an MCP server
    session L9-additive-scenario-8-p2-r10: satisfied — event 20, call 5: `Read /mnt/build/hw-1775/screening/a/ws/L9-additive-scenario-8-p2-r10/docs/decisions/0087-no-paragraph-limit-joins-the-language-rule-because-most-paragraphs-past-six-sentences-hold-one-topic.md`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p2-r11: not satisfied — 3 recorded calls, and none named any of the 1 document; no call to a tool of an MCP server
    session L9-additive-scenario-8-p2-r12: not satisfied — 4 recorded calls, and none named any of the 1 document; no call to a tool of an MCP server
    session L9-additive-scenario-8-p2-r13: satisfied — event 23, call 4: `Read /mnt/build/hw-1775/screening/a/ws/L9-additive-scenario-8-p2-r13/docs/decisions/0087-no-paragraph-limit-joins-the-language-rule-because-most-paragraphs-past-six-sentences-hold-one-topic.md`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p2-r14: satisfied — event 24, call 6: `Read /mnt/build/hw-1775/screening/a/ws/L9-additive-scenario-8-p2-r14/docs/decisions/0087-no-paragraph-limit-joins-the-language-rule-because-most-paragraphs-past-six-sentences-hold-one-topic.md`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p2-r15: satisfied — event 25, call 5: `Read /mnt/build/hw-1775/screening/a/ws/L9-additive-scenario-8-p2-r15/docs/decisions/0087-no-paragraph-limit-joins-the-language-rule-because-most-paragraphs-past-six-sentences-hold-one-topic.md`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p2-r16: not satisfied — 6 recorded calls, and none named any of the 1 document; no call to a tool of an MCP server
    session L9-additive-scenario-8-p2-r17: satisfied — event 27, call 5: `Read /mnt/build/hw-1775/screening/a/ws/L9-additive-scenario-8-p2-r17/docs/decisions/0087-no-paragraph-limit-joins-the-language-rule-because-most-paragraphs-past-six-sentences-hold-one-topic.md`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p2-r18: satisfied — event 28, call 4: `Read /mnt/build/hw-1775/screening/a/ws/L9-additive-scenario-8-p2-r18/docs/decisions/0087-no-paragraph-limit-joins-the-language-rule-because-most-paragraphs-past-six-sentences-hold-one-topic.md`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p2-r2: not satisfied — 3 recorded calls, and none named any of the 1 document; no call to a tool of an MCP server
    session L9-additive-scenario-8-p2-r3: satisfied — event 30, call 5: `Read /mnt/build/hw-1775/screening/a/ws/L9-additive-scenario-8-p2-r3/docs/decisions/0087-no-paragraph-limit-joins-the-language-rule-because-most-paragraphs-past-six-sentences-hold-one-topic.md`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p2-r4: satisfied — event 31, call 5: `Read /mnt/build/hw-1775/screening/a/ws/L9-additive-scenario-8-p2-r4/docs/decisions/0087-no-paragraph-limit-joins-the-language-rule-because-most-paragraphs-past-six-sentences-hold-one-topic.md`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p2-r5: not satisfied — 3 recorded calls, and none named any of the 1 document; no call to a tool of an MCP server
    session L9-additive-scenario-8-p2-r6: satisfied — event 33, call 4: `Read /mnt/build/hw-1775/screening/a/ws/L9-additive-scenario-8-p2-r6/docs/decisions/0087-no-paragraph-limit-joins-the-language-rule-because-most-paragraphs-past-six-sentences-hold-one-topic.md`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p2-r7: satisfied — event 34, call 5: `Read /mnt/build/hw-1775/screening/a/ws/L9-additive-scenario-8-p2-r7/docs/decisions/0087-no-paragraph-limit-joins-the-language-rule-because-most-paragraphs-past-six-sentences-hold-one-topic.md`; no call to a tool of an MCP server
    session L9-additive-scenario-8-p2-r8: not satisfied — 3 recorded calls, and none named any of the 1 document; no call to a tool of an MCP server
    session L9-additive-scenario-8-p2-r9: not satisfied — 3 recorded calls, and none named any of the 1 document; no call to a tool of an MCP server

72 of 72 sessions that recorded their calls made no call to a tool of an MCP server. 0 sessions recorded no calls.

## The rate, and the denominator it is over

52 of 72 graded sessions satisfied their expectation: 72.2%, in a 95% interval of 61.0% to 81.2%.
The denominator is the graded sessions and never the selected probes. 0 sessions reached no verdict, and a session with no verdict is outside both halves of that fraction.

An interval that overlaps the previous run's is variance and one that does not is drift. This is one arm, so it estimates no effect: an efficacy claim is a comparison of two results, and the arm each one recorded is on it.
