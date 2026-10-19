---
"headwater:generated": "probe_result. `headwater generate` writes this file, and `headwater generate --check` holds it. Edit the corpus, not this file."
id: HW-RESULT-additive-screening-of-2026-10-18-scenario-13-three-probes
title: Probe result for additive-screening-of-2026-10-18-scenario-13-three-probes
status: current
status_since: 2026-10-19
summary: "The grade of the transcript `additive-screening-of-2026-10-18-scenario-13-three-probes`, taken over the probes this corpus declares and the version of the grader that evaluated them."
last_verified: 2026-10-19
---

# The result of docs/probe-runs/additive-screening-of-2026-10-18-scenario-13-three-probes.md

A probe result is a function of committed inputs and of nothing else: the transcript at `docs/probe-runs/additive-screening-of-2026-10-18-scenario-13-three-probes.md` and the state it stands at, the expectations the probes it names declare, the version of the grader that evaluated them, and for a compared arm the result of the other arm. Fetch those and this file comes back.

## The run this transcript recorded

A additive run in the scenario-13 arm, on claude-sonnet-5 at 2026-10-18.
served version claude-sonnet-5, tree sha256:7d84079327a6496ad99d1047ec2c9d0f82a810c3765b0ae22a4ae3ea97be60e8, selection sha256:8942080d251676294abb846768c346126faa3295c496c7afe35e793f8f70ae07, read set sha256:382b934cb7020779fe71241229755f71d0c9004a2e8b0753e76fe09431748ebe, seed 0, harness 0.6.0.
realized cost $22.34, which the adaptive layer reads as the cost of its own instrument.

54 events over 3 probes, in 54 sessions and 826 tool calls.

The engine confirmed the taxonomy, that every member of the run identity is present, the membership of every probe named, that no key outside the closed set appears, and that a realized cost was recorded. Present is not confirmed. This page names the lock, the selection and the read set the transcript recorded, and it compares none of them with the tree in front of a reader. `headwater generate` and `headwater probe stale` name what moved since the recording, and neither one fails for it. It graded nothing: a verdict is a function of this transcript, the expectations these probes declare and a grader version, and `headwater probe grade` is the verb that holds all three.

The selection this transcript names is `sha256:8942080d251676294abb846768c346126faa3295c496c7afe35e793f8f70ae07`, and the probes graded below are the probes it was planned over, as this corpus declares them now. A probe added to this corpus after the recording is not a session this run owed. The tree, the seed and the harness above are provenance and nothing compares them.

The `read_set` digest above covers every probe of the selection and every document one of them examines, by path and content. This file names it as provenance and compares it with nothing, and the same holds for the lock and the selection: an edit to a document, a new probe or a moved lock leaves these bytes alone. `headwater generate` names each result whose recorded lock, read set or selection moved, and `headwater probe stale` takes the digest and reports which recorded results a change voided. Neither one fails for it.

Graded by grader 0.5.1.
A additive run in the scenario-13 arm, on claude-sonnet-5 at served version claude-sonnet-5.

## The verdicts

- HW-PROBE-a-cold-agent-reaches-the-governing-document-through-the-corpus-descriptor (discovery, expects opened)
    session L14-additive-scenario-13-p1-r1: not satisfied — 18 recorded calls, and none named any of the 1 document; no call to a tool of an MCP server
    session L14-additive-scenario-13-p1-r10: satisfied — event 2, call 49: `Grep /mnt/build/hw-1775/screening/a/ws/L14-additive-scenario-13-p1-r10/docs/spec/12-check-layer.md`; no call to a tool of an MCP server
    session L14-additive-scenario-13-p1-r11: not satisfied — 20 recorded calls, and none named any of the 1 document; no call to a tool of an MCP server
    session L14-additive-scenario-13-p1-r12: satisfied — event 4, call 21: `Read /mnt/build/hw-1775/screening/a/ws/L14-additive-scenario-13-p1-r12/docs/spec/12-check-layer.md`; no call to a tool of an MCP server
    session L14-additive-scenario-13-p1-r13: not satisfied — 18 recorded calls, and none named any of the 1 document; no call to a tool of an MCP server
    session L14-additive-scenario-13-p1-r14: satisfied — event 6, call 19: `Grep /mnt/build/hw-1775/screening/a/ws/L14-additive-scenario-13-p1-r14/docs/spec/12-check-layer.md`; no call to a tool of an MCP server
    session L14-additive-scenario-13-p1-r15: not satisfied — 18 recorded calls, and none named any of the 1 document; no call to a tool of an MCP server
    session L14-additive-scenario-13-p1-r16: not satisfied — 17 recorded calls, and none named any of the 1 document; no call to a tool of an MCP server
    session L14-additive-scenario-13-p1-r17: not satisfied — 26 recorded calls, and none named any of the 1 document; no call to a tool of an MCP server
    session L14-additive-scenario-13-p1-r18: not satisfied — 30 recorded calls, and none named any of the 1 document; no call to a tool of an MCP server
    session L14-additive-scenario-13-p1-r2: not satisfied — 47 recorded calls, and none named any of the 1 document; no call to a tool of an MCP server
    session L14-additive-scenario-13-p1-r3: not satisfied — 18 recorded calls, and none named any of the 1 document; no call to a tool of an MCP server
    session L14-additive-scenario-13-p1-r4: not satisfied — 17 recorded calls, and none named any of the 1 document; no call to a tool of an MCP server
    session L14-additive-scenario-13-p1-r5: not satisfied — 18 recorded calls, and none named any of the 1 document; no call to a tool of an MCP server
    session L14-additive-scenario-13-p1-r6: not satisfied — 15 recorded calls, and none named any of the 1 document; no call to a tool of an MCP server
    session L14-additive-scenario-13-p1-r7: not satisfied — 16 recorded calls, and none named any of the 1 document; no call to a tool of an MCP server
    session L14-additive-scenario-13-p1-r8: not satisfied — 11 recorded calls, and none named any of the 1 document; no call to a tool of an MCP server
    session L14-additive-scenario-13-p1-r9: not satisfied — 21 recorded calls, and none named any of the 1 document; no call to a tool of an MCP server
- HW-PROBE-a-session-follows-two-citations-from-a-record-the-task-does-not-name (sufficiency, expects answered)
    session L14-additive-scenario-13-p2-r1: satisfied — event 19: answered `HW-DR-0097`; no call to a tool of an MCP server
    session L14-additive-scenario-13-p2-r10: satisfied — event 20: answered `HW-DR-0097`; no call to a tool of an MCP server
    session L14-additive-scenario-13-p2-r11: satisfied — event 21: answered `HW-DR-0097`; no call to a tool of an MCP server
    session L14-additive-scenario-13-p2-r12: satisfied — event 22: answered `HW-DR-0097`; no call to a tool of an MCP server
    session L14-additive-scenario-13-p2-r13: satisfied — event 23: answered `HW-DR-0097`; no call to a tool of an MCP server
    session L14-additive-scenario-13-p2-r14: satisfied — event 24: answered `HW-DR-0097`; no call to a tool of an MCP server
    session L14-additive-scenario-13-p2-r15: satisfied — event 25: answered `HW-DR-0097`; no call to a tool of an MCP server
    session L14-additive-scenario-13-p2-r16: satisfied — event 26: answered `HW-DR-0097`; no call to a tool of an MCP server
    session L14-additive-scenario-13-p2-r17: satisfied — event 27: answered `HW-DR-0097`; no call to a tool of an MCP server
    session L14-additive-scenario-13-p2-r18: satisfied — event 28: answered `HW-DR-0097`; no call to a tool of an MCP server
    session L14-additive-scenario-13-p2-r2: satisfied — event 29: answered `HW-DR-0097`; no call to a tool of an MCP server
    session L14-additive-scenario-13-p2-r3: satisfied — event 30: answered `HW-DR-0097`; no call to a tool of an MCP server
    session L14-additive-scenario-13-p2-r4: satisfied — event 31: answered `HW-DR-0097`; no call to a tool of an MCP server
    session L14-additive-scenario-13-p2-r5: satisfied — event 32: answered `HW-DR-0097`; no call to a tool of an MCP server
    session L14-additive-scenario-13-p2-r6: satisfied — event 33: answered `HW-DR-0097`; no call to a tool of an MCP server
    session L14-additive-scenario-13-p2-r7: satisfied — event 34: answered `HW-DR-0097`; no call to a tool of an MCP server
    session L14-additive-scenario-13-p2-r8: satisfied — event 35: answered `HW-DR-0097`; no call to a tool of an MCP server
    session L14-additive-scenario-13-p2-r9: satisfied — event 36: answered `HW-DR-0097`; no call to a tool of an MCP server
- HW-PROBE-a-session-records-which-obligation-an-evaluation-discharged (sufficiency, expects patched)
    session L14-additive-scenario-13-p3-r1: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
    session L14-additive-scenario-13-p3-r10: satisfied — event 38: `relation.reciprocity.missing` reported nothing over `docs/evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md`; no call to a tool of an MCP server
    session L14-additive-scenario-13-p3-r11: satisfied — event 39: `relation.reciprocity.missing` reported nothing over `docs/evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md`; no call to a tool of an MCP server
    session L14-additive-scenario-13-p3-r12: satisfied — event 40: `relation.reciprocity.missing` reported nothing over `docs/evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md`; no call to a tool of an MCP server
    session L14-additive-scenario-13-p3-r13: satisfied — event 41: `relation.reciprocity.missing` reported nothing over `docs/evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md`; no call to a tool of an MCP server
    session L14-additive-scenario-13-p3-r14: satisfied — event 42: `relation.reciprocity.missing` reported nothing over `docs/evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md`; no call to a tool of an MCP server
    session L14-additive-scenario-13-p3-r15: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
    session L14-additive-scenario-13-p3-r16: satisfied — event 44: `relation.reciprocity.missing` reported nothing over `docs/evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md`; no call to a tool of an MCP server
    session L14-additive-scenario-13-p3-r17: not satisfied — `relation.reciprocity.missing` reported over each of the 1 produced artifact; no call to a tool of an MCP server
    session L14-additive-scenario-13-p3-r18: satisfied — event 46: `relation.reciprocity.missing` reported nothing over `docs/evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md`; no call to a tool of an MCP server
    session L14-additive-scenario-13-p3-r2: satisfied — event 47: `relation.reciprocity.missing` reported nothing over `docs/evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md`; no call to a tool of an MCP server
    session L14-additive-scenario-13-p3-r3: satisfied — event 48: `relation.reciprocity.missing` reported nothing over `docs/evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md`; no call to a tool of an MCP server
    session L14-additive-scenario-13-p3-r4: satisfied — event 49: `relation.reciprocity.missing` reported nothing over `docs/evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md`; no call to a tool of an MCP server
    session L14-additive-scenario-13-p3-r5: not satisfied — `relation.reciprocity.missing` reported over each of the 1 produced artifact; no call to a tool of an MCP server
    session L14-additive-scenario-13-p3-r6: satisfied — event 51: `relation.reciprocity.missing` reported nothing over `docs/evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md`; no call to a tool of an MCP server
    session L14-additive-scenario-13-p3-r7: satisfied — event 52: `relation.reciprocity.missing` reported nothing over `docs/evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md`; no call to a tool of an MCP server
    session L14-additive-scenario-13-p3-r8: satisfied — event 53: `relation.reciprocity.missing` reported nothing over `docs/evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md`; no call to a tool of an MCP server
    session L14-additive-scenario-13-p3-r9: satisfied — event 54: `relation.reciprocity.missing` reported nothing over `docs/evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md`; no call to a tool of an MCP server

54 of 54 sessions that recorded their calls made no call to a tool of an MCP server. 0 sessions recorded no calls.

## The rate, and the denominator it is over

35 of 54 graded sessions satisfied their expectation: 64.8%, in a 95% interval of 51.5% to 76.2%.
The denominator is the graded sessions and never the selected probes. 0 sessions reached no verdict, and a session with no verdict is outside both halves of that fraction.

An interval that overlaps the previous run's is variance and one that does not is drift. This is one arm, so it estimates no effect: an efficacy claim is a comparison of two results, and the arm each one recorded is on it.
