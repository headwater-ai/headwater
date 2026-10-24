---
"headwater:generated": "probe_result. `headwater generate` writes this file, and `headwater generate --check` holds it. Edit the corpus, not this file."
id: HW-RESULT-additive-tuning-of-2026-10-24-skills-round-1-variant-4-scenario-4
title: Probe result for additive-tuning-of-2026-10-24-skills-round-1-variant-4-scenario-4
status: current
status_since: 2026-10-24
summary: "The grade of the transcript `additive-tuning-of-2026-10-24-skills-round-1-variant-4-scenario-4`, taken over the probes this corpus declares and the version of the grader that evaluated them."
last_verified: 2026-10-24
---

# The result of docs/probe-runs/additive-tuning-of-2026-10-24-skills-round-1-variant-4-scenario-4.md

A probe result is a function of committed inputs and of nothing else: the transcript at `docs/probe-runs/additive-tuning-of-2026-10-24-skills-round-1-variant-4-scenario-4.md` and the state it stands at, the expectations the probes it names declare, the version of the grader that evaluated them, and for a compared arm the result of the other arm. Fetch those and this file comes back.

## The run this transcript recorded

A additive run in the scenario-4 arm, on claude-sonnet-5 at 2026-10-24.
served version claude-sonnet-5, tree sha256:bc9ff145ae7bc2a53ece57ca1442ff3d5ef1528cca5641cbfc877419836a844a, selection sha256:8d4747a05d1213146c97d37a574a457ce8aeee431a3f9f6a9c67c79021d1f437, read set sha256:f18e4fd7eafa09aabd6204d8a41112a4af8892eb03c0b5d7fb9d3bf5f927b3a3, seed 0, harness 0.6.1.
realized cost $18.54, which the adaptive layer reads as the cost of its own instrument.

25 events over 5 probes, in 25 sessions and 704 tool calls.

The engine confirmed the taxonomy, that every member of the run identity is present, the membership of every probe named, that no key outside the closed set appears, and that a realized cost was recorded. Present is not confirmed. This page names the lock, the selection and the read set the transcript recorded, and it compares none of them with the tree in front of a reader. `headwater generate` and `headwater probe stale` name what moved since the recording, and neither one fails for it. It graded nothing: a verdict is a function of this transcript, the expectations these probes declare and a grader version, and `headwater probe grade` is the verb that holds all three.

The selection this transcript names is `sha256:8d4747a05d1213146c97d37a574a457ce8aeee431a3f9f6a9c67c79021d1f437`, and the probes graded below are the probes it was planned over, as this corpus declares them now. A probe added to this corpus after the recording is not a session this run owed. The tree, the seed and the harness above are provenance and nothing compares them.

The `read_set` digest above covers every probe of the selection and every document one of them examines, by path and content. This file names it as provenance and compares it with nothing, and the same holds for the lock and the selection: an edit to a document, a new probe or a moved lock leaves these bytes alone. `headwater generate` names each result whose recorded lock, read set or selection moved, and `headwater probe stale` takes the digest and reports which recorded results a change voided. Neither one fails for it.

Graded by grader 0.5.1.
A additive run in the scenario-4 arm, on claude-sonnet-5 at served version claude-sonnet-5.

## The verdicts

- HW-PROBE-a-session-records-which-obligation-an-evaluation-discharged (sufficiency, expects patched)
    session L1-additive-scenario-4-p2-r1: satisfied — event 6: `relation.reciprocity.missing` reported nothing over `docs/evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md`; no call to a tool of an MCP server
    session L1-additive-scenario-4-p2-r2: satisfied — event 7: `relation.reciprocity.missing` reported nothing over `docs/evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md`; no call to a tool of an MCP server
    session L1-additive-scenario-4-p2-r3: satisfied — event 8: `relation.reciprocity.missing` reported nothing over `docs/evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md`; no call to a tool of an MCP server
    session L1-additive-scenario-4-p2-r4: satisfied — event 9: `relation.reciprocity.missing` reported nothing over `docs/evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md`; no call to a tool of an MCP server
    session L1-additive-scenario-4-p2-r5: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
- HW-PROBE-a-session-that-changes-a-verb-s-default-reaches-the-verb-s-interface-contract (navigability, expects opened)
    session L1-additive-scenario-4-p3-r1: satisfied — event 11, call 5: `Read /mnt/build/hw-1777/skills-r1-v4/a/ws/L1-additive-scenario-4-p3-r1/docs/interfaces/headwater-route.md`; no call to a tool of an MCP server
    session L1-additive-scenario-4-p3-r2: satisfied — event 12, call 5: `Read /mnt/build/hw-1777/skills-r1-v4/a/ws/L1-additive-scenario-4-p3-r2/docs/interfaces/headwater-route.md`; no call to a tool of an MCP server
    session L1-additive-scenario-4-p3-r3: satisfied — event 13, call 7: `Bash {"command":"grep -n -i \"five\\|budget\\|pointers\" docs/interfaces/headwater-route.md","description":"Search headwater-route.md interface doc for budget/default mentions"}`; no call to a tool of an MCP server
    session L1-additive-scenario-4-p3-r4: satisfied — event 14, call 4: `Read /mnt/build/hw-1777/skills-r1-v4/a/ws/L1-additive-scenario-4-p3-r4/docs/interfaces/headwater-route.md`; no call to a tool of an MCP server
    session L1-additive-scenario-4-p3-r5: satisfied — event 15, call 12: `Read /mnt/build/hw-1777/skills-r1-v4/a/ws/L1-additive-scenario-4-p3-r5/docs/interfaces/headwater-route.md`; no call to a tool of an MCP server
- HW-PROBE-a-session-that-changes-the-recorder-reaches-the-recorder-contract (navigability, expects opened)
    session L1-additive-scenario-4-p4-r1: not satisfied — 3 recorded calls, and none named any of the 1 document; no call to a tool of an MCP server
    session L1-additive-scenario-4-p4-r2: not satisfied — 5 recorded calls, and none named any of the 1 document; no call to a tool of an MCP server
    session L1-additive-scenario-4-p4-r3: not satisfied — 7 recorded calls, and none named any of the 1 document; no call to a tool of an MCP server
    session L1-additive-scenario-4-p4-r4: satisfied — event 19, call 7: `Read /mnt/build/hw-1777/skills-r1-v4/a/ws/L1-additive-scenario-4-p4-r4/docs/spec/15-the-recorder-contract.md`; no call to a tool of an MCP server
    session L1-additive-scenario-4-p4-r5: satisfied — event 20, call 7: `Read /mnt/build/hw-1777/skills-r1-v4/a/ws/L1-additive-scenario-4-p4-r5/docs/spec/15-the-recorder-contract.md`; no call to a tool of an MCP server
- HW-PROBE-a-session-that-moves-a-shelf-reaches-the-relocation-guide (discovery, expects opened)
    session L1-additive-scenario-4-p5-r1: satisfied — event 21, call 10: `Read /mnt/build/hw-1777/skills-r1-v4/a/ws/L1-additive-scenario-4-p5-r1/docs/how-to/relocate-a-shelf.md`; no call to a tool of an MCP server
    session L1-additive-scenario-4-p5-r2: satisfied — event 22, call 5: `Read /mnt/build/hw-1777/skills-r1-v4/a/ws/L1-additive-scenario-4-p5-r2/docs/how-to/relocate-a-shelf.md`; no call to a tool of an MCP server
    session L1-additive-scenario-4-p5-r3: satisfied — event 23, call 6: `Read /mnt/build/hw-1777/skills-r1-v4/a/ws/L1-additive-scenario-4-p5-r3/docs/how-to/relocate-a-shelf.md`; no call to a tool of an MCP server
    session L1-additive-scenario-4-p5-r4: satisfied — event 24, call 8: `Read /mnt/build/hw-1777/skills-r1-v4/a/ws/L1-additive-scenario-4-p5-r4/docs/how-to/relocate-a-shelf.md`; no call to a tool of an MCP server
    session L1-additive-scenario-4-p5-r5: satisfied — event 25, call 6: `Read /mnt/build/hw-1777/skills-r1-v4/a/ws/L1-additive-scenario-4-p5-r5/docs/how-to/relocate-a-shelf.md`; no call to a tool of an MCP server
- HW-PROBE-an-agent-reaches-the-adjudication-when-the-decision-a-tombstone-links-is-superseded (navigability, expects opened)
    session L1-additive-scenario-4-p1-r1: satisfied — event 1, call 8: `Bash {"command":"./engine/target/dev-release/headwater explain \"docs/decisions/0069-a-paragraph-limit-counts-sentences-under-the-language-rule-and-never-words.md\" --root . 2>&1\necho \"---\"\n./engine/target/dev-release/headwater explain \"docs/decisions/0087-no-paragraph-limit-joins-the-language-rule-because-most-paragraphs-past-six-sentences-hold-one-topic.md\" --root . 2>&1","description":"Explain decision records 0069 and 0087"}`; no call to a tool of an MCP server
    session L1-additive-scenario-4-p1-r2: satisfied — event 2, call 6: `Read docs/decisions/0087-no-paragraph-limit-joins-the-language-rule-because-most-paragraphs-past-six-sentences-hold-one-topic.md`; no call to a tool of an MCP server
    session L1-additive-scenario-4-p1-r3: satisfied — event 3, call 8: `Bash {"command":"cat /mnt/build/hw-1777/skills-r1-v4/a/ws/L1-additive-scenario-4-p1-r3/docs/decisions/0087-no-paragraph-limit-joins-the-language-rule-because-most-paragraphs-past-six-sentences-hold-one-topic.md","description":"Read decision 0087, the superseding record"}`; no call to a tool of an MCP server
    session L1-additive-scenario-4-p1-r4: satisfied — event 4, call 13: `Read docs/decisions/0087-no-paragraph-limit-joins-the-language-rule-because-most-paragraphs-past-six-sentences-hold-one-topic.md`; no call to a tool of an MCP server
    session L1-additive-scenario-4-p1-r5: not satisfied — 7 recorded calls, and none named any of the 1 document; no call to a tool of an MCP server

25 of 25 sessions that recorded their calls made no call to a tool of an MCP server. 0 sessions recorded no calls.

## The rate, and the denominator it is over

20 of 25 graded sessions satisfied their expectation: 80.0%, in a 95% interval of 60.9% to 91.1%.
The denominator is the graded sessions and never the selected probes. 0 sessions reached no verdict, and a session with no verdict is outside both halves of that fraction.

An interval that overlaps the previous run's is variance and one that does not is drift. This is one arm, so it estimates no effect: an efficacy claim is a comparison of two results, and the arm each one recorded is on it.
