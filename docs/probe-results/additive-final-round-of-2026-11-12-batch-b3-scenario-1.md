---
"headwater:generated": "probe_result. `headwater generate` writes this file, and `headwater generate --check` holds it. Edit the corpus, not this file."
id: HW-RESULT-additive-final-round-of-2026-11-12-batch-b3-scenario-1
title: Probe result for additive-final-round-of-2026-11-12-batch-b3-scenario-1
status: current
status_since: 2026-11-22
summary: "The grade of the transcript `additive-final-round-of-2026-11-12-batch-b3-scenario-1`, taken over the probes this corpus declares and the version of the grader that evaluated them."
last_verified: 2026-11-22
---

# The result of docs/probe-runs/additive-final-round-of-2026-11-12-batch-b3-scenario-1.md

A probe result is a function of committed inputs and of nothing else: the transcript at `docs/probe-runs/additive-final-round-of-2026-11-12-batch-b3-scenario-1.md` and the state it stands at, the expectations the probes it names declare, the version of the grader that evaluated them, and for a compared arm the result of the other arm. Fetch those and this file comes back.

## The run this transcript recorded

A additive run in the scenario-1 arm, on claude-sonnet-5 at 2026-11-12.
served version claude-sonnet-5, tree sha256:77c336d487649121d85466ae2faad7a027d8b8fadd8350f0f1ce64aa2294ba19, selection sha256:684c21ee31079bea854202738f2e26a84dbcb7084ed39f38609726badb8eacc7, read set sha256:3aad2c333fa7956d9b3e062b6270cabeb268a2056e6d528bcb8d4a24b0156586, seed 0, harness 0.6.1.
realized cost $51.86, which the adaptive layer reads as the cost of its own instrument.

96 events over 16 probes, in 96 sessions and 2004 tool calls.

The engine confirmed the taxonomy, that every member of the run identity is present, the membership of every probe named, that no key outside the closed set appears, and that a realized cost was recorded. Present is not confirmed. This page names the lock, the selection and the read set the transcript recorded, and it compares none of them with the tree in front of a reader. `headwater generate` and `headwater probe stale` name what moved since the recording, and neither one fails for it. It graded nothing: a verdict is a function of this transcript, the expectations these probes declare and a grader version, and `headwater probe grade` is the verb that holds all three.

The selection this transcript names is `sha256:684c21ee31079bea854202738f2e26a84dbcb7084ed39f38609726badb8eacc7`, and the probes graded below are the probes it was planned over, as this corpus declares them now. A probe added to this corpus after the recording is not a session this run owed. The tree, the seed and the harness above are provenance and nothing compares them.

The `read_set` digest above covers every probe of the selection and every document one of them examines, by path and content. This file names it as provenance and compares it with nothing, and the same holds for the lock and the selection: an edit to a document, a new probe or a moved lock leaves these bytes alone. `headwater generate` names each result whose recorded lock, read set or selection moved, and `headwater probe stale` takes the digest and reports which recorded results a change voided. Neither one fails for it.

Graded by grader 0.5.3.
A additive run in the scenario-1 arm, on claude-sonnet-5 at served version claude-sonnet-5.

## The verdicts

- HW-PROBE-a-session-lets-the-asset-size-check-of-the-site-take-its-limit-in-mebibytes-as-a-second-argument (sufficiency, expects patched)
    session L1-additive-scenario-1-p1-r1: satisfied — event 1: `command:r11-asset-limit` reported nothing over `tools/site/check-site-asset-size.sh`; no call to a tool of an MCP server
    session L1-additive-scenario-1-p1-r2: satisfied — event 2: `command:r11-asset-limit` reported nothing over `tools/site/check-site-asset-size.sh`; no call to a tool of an MCP server
    session L1-additive-scenario-1-p1-r3: satisfied — event 3: `command:r11-asset-limit` reported nothing over `tools/site/check-site-asset-size.sh`; no call to a tool of an MCP server
    session L1-additive-scenario-1-p1-r4: satisfied — event 4: `command:r11-asset-limit` reported nothing over `tools/site/check-site-asset-size.sh`; no call to a tool of an MCP server
    session L1-additive-scenario-1-p1-r5: satisfied — event 5: `command:r11-asset-limit` reported nothing over `tools/site/check-site-asset-size.sh`; no call to a tool of an MCP server
    session L1-additive-scenario-1-p1-r6: satisfied — event 6: `command:r11-asset-limit` reported nothing over `tools/site/check-site-asset-size.sh`; no call to a tool of an MCP server
- HW-PROBE-a-session-lets-the-graph-export-list-the-patterns-of-every-anchor (sufficiency, expects patched)
    session L1-additive-scenario-1-p2-r1: satisfied — event 49: `command:r2-export-patterns` reported nothing over `docs/interfaces/headwater-export.md` or the other 3 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-1-p2-r2: satisfied — event 50: `command:r2-export-patterns` reported nothing over `docs/interfaces/headwater-export.md` or the other 2 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-1-p2-r3: satisfied — event 51: `command:r2-export-patterns` reported nothing over `docs/acceptance-criteria/0004-each-list-anchor-in-the-native-export-carries-its-sorted-members-and-a-single-pattern-carries-none.md` or the other 6 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-1-p2-r4: not satisfied — event 52: `command:r2-export-patterns` reported over `engine/crates/generate/src/export.rs`, 2 of the 2 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-1-p2-r5: satisfied — event 53: `command:r2-export-patterns` reported nothing over `docs/interfaces/headwater-export.md` or the other 3 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-1-p2-r6: satisfied — event 54: `command:r2-export-patterns` reported nothing over `docs/interfaces/headwater-export.md` or the other 2 produced artifacts inside the workspace; no call to a tool of an MCP server
- HW-PROBE-a-session-lets-the-required-sections-rule-read-a-numbered-heading (sufficiency, expects patched)
    session L1-additive-scenario-1-p3-r1: not satisfied — event 55: `command:r2-numbered-heading` reported over `engine/crates/check/src/sections.rs`, 1 of the 1 produced artifact inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-1-p3-r2: not satisfied — event 56: `command:r2-numbered-heading` reported over `engine/crates/check/src/sections.rs`, 1 of the 1 produced artifact inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-1-p3-r3: not satisfied — event 57: `command:r2-numbered-heading` reported over `engine/crates/check/src/sections.rs`, 1 of the 1 produced artifact inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-1-p3-r4: not satisfied — event 58: `command:r2-numbered-heading` reported over `engine/crates/check/src/sections.rs`, 1 of the 1 produced artifact inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-1-p3-r5: not satisfied — event 59: `command:r2-numbered-heading` reported over `engine/crates/check/src/sections.rs`, 1 of the 1 produced artifact inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-1-p3-r6: not satisfied — event 60: `command:r2-numbered-heading` reported over `engine/crates/check/src/sections.rs`, 1 of the 1 produced artifact inside the workspace; no call to a tool of an MCP server
- HW-PROBE-a-session-links-the-governs-edges-evaluation-to-the-obligation-that-it-says-is-discharged (sufficiency, expects patched)
    session L1-additive-scenario-1-p4-r1: satisfied — event 61: `command:r5-hook-role` reported nothing over `docs/evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md` or the other 1 produced artifact inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-1-p4-r2: satisfied — event 62: `command:r5-hook-role` reported nothing over `docs/evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md` or the other 1 produced artifact inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-1-p4-r3: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
    session L1-additive-scenario-1-p4-r4: satisfied — event 64: `command:r5-hook-role` reported nothing over `docs/evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md` or the other 1 produced artifact inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-1-p4-r5: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
    session L1-additive-scenario-1-p4-r6: not satisfied — event 66: `command:r5-hook-role` reported over `docs/obligations/0105-nothing-plays-the-hook-role-that-two-relations-name.md`, 1 of the 1 produced artifact inside the workspace; no call to a tool of an MCP server
- HW-PROBE-a-session-makes-the-script-that-prints-the-apt-block-remove-sudo-with-its-environment-flag-as-a-whole (sufficiency, expects patched)
    session L1-additive-scenario-1-p5-r1: satisfied — event 67: `command:r11-apt-block` reported nothing over `tools/repo/readme-apt-block.sh`; no call to a tool of an MCP server
    session L1-additive-scenario-1-p5-r2: satisfied — event 68: `command:r11-apt-block` reported nothing over `tools/repo/readme-apt-block.sh`; no call to a tool of an MCP server
    session L1-additive-scenario-1-p5-r3: satisfied — event 69: `command:r11-apt-block` reported nothing over `tools/repo/readme-apt-block.sh`; no call to a tool of an MCP server
    session L1-additive-scenario-1-p5-r4: satisfied — event 70: `command:r11-apt-block` reported nothing over `tools/repo/readme-apt-block.sh`; no call to a tool of an MCP server
    session L1-additive-scenario-1-p5-r5: satisfied — event 71: `command:r11-apt-block` reported nothing over `tools/repo/readme-apt-block.sh`; no call to a tool of an MCP server
    session L1-additive-scenario-1-p5-r6: satisfied — event 72: `command:r11-apt-block` reported nothing over `tools/repo/readme-apt-block.sh`; no call to a tool of an MCP server
- HW-PROBE-a-session-moves-the-obligation-record-of-one-run-s-trailing-edges-to-the-process-shelf-and-repairs-every-link-to-it (sufficiency, expects patched)
    session L1-additive-scenario-1-p6-r1: satisfied — event 73: `command:r7-run-findings` reported nothing over `docs/process/obligations/0242-three-edges-and-a-comment-that-trail-a-change-of-run-20261012-2342.md` or the other 4 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-1-p6-r2: satisfied — event 74: `command:r7-run-findings` reported nothing over `docs/process/obligations/0242-three-edges-and-a-comment-that-trail-a-change-of-run-20261012-2342.md` or the other 4 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-1-p6-r3: satisfied — event 75: `command:r7-run-findings` reported nothing over `docs/process/obligations/0242-three-edges-and-a-comment-that-trail-a-change-of-run-20261012-2342.md` or the other 4 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-1-p6-r4: satisfied — event 76: `command:r7-run-findings` reported nothing over `docs/process/obligations/0242-three-edges-and-a-comment-that-trail-a-change-of-run-20261012-2342.md` or the other 4 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-1-p6-r5: satisfied — event 77: `command:r7-run-findings` reported nothing over `docs/process/obligations/0242-three-edges-and-a-comment-that-trail-a-change-of-run-20261012-2342.md` or the other 4 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-1-p6-r6: satisfied — event 78: `command:r7-run-findings` reported nothing over `docs/process/obligations/0242-three-edges-and-a-comment-that-trail-a-change-of-run-20261012-2342.md` or the other 4 produced artifacts inside the workspace; no call to a tool of an MCP server
- HW-PROBE-a-session-moves-the-record-of-one-run-s-test-gaps-to-the-process-shelf-and-repairs-the-link-from-a-sibling-record (sufficiency, expects patched)
    session L1-additive-scenario-1-p7-r1: satisfied — event 79: `command:r7-test-gaps` reported nothing over `docs/obligations/0237-eighteen-findings-that-the-paid-layer-campaign-of-run-20261003-1026-left-in-the-probe-recorder-and-the-campaign-driver.md` or the other 5 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-1-p7-r2: satisfied — event 80: `command:r7-test-gaps` reported nothing over `docs/obligations/0237-eighteen-findings-that-the-paid-layer-campaign-of-run-20261003-1026-left-in-the-probe-recorder-and-the-campaign-driver.md` or the other 4 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-1-p7-r3: satisfied — event 81: `command:r7-test-gaps` reported nothing over `docs/obligations/0237-eighteen-findings-that-the-paid-layer-campaign-of-run-20261003-1026-left-in-the-probe-recorder-and-the-campaign-driver.md` or the other 5 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-1-p7-r4: satisfied — event 82: `command:r7-test-gaps` reported nothing over `docs/obligations/0237-eighteen-findings-that-the-paid-layer-campaign-of-run-20261003-1026-left-in-the-probe-recorder-and-the-campaign-driver.md` or the other 5 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-1-p7-r5: satisfied — event 83: `command:r7-test-gaps` reported nothing over `docs/obligations/0237-eighteen-findings-that-the-paid-layer-campaign-of-run-20261003-1026-left-in-the-probe-recorder-and-the-campaign-driver.md` or the other 5 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-1-p7-r6: satisfied — event 84: `command:r7-test-gaps` reported nothing over `docs/obligations/0237-eighteen-findings-that-the-paid-layer-campaign-of-run-20261003-1026-left-in-the-probe-recorder-and-the-campaign-driver.md` or the other 5 produced artifacts inside the workspace; no call to a tool of an MCP server
- HW-PROBE-a-session-names-the-record-that-governs-the-scripts-that-grade-a-change-from-a-task-in-plain-words (navigability, expects answered)
    session L1-additive-scenario-1-p8-r1: satisfied — event 85: answered `HW-DR-0112`; no call to a tool of an MCP server
    session L1-additive-scenario-1-p8-r2: satisfied — event 86: answered `HW-DR-0112`; no call to a tool of an MCP server
    session L1-additive-scenario-1-p8-r3: satisfied — event 87: answered `HW-DR-0112`; no call to a tool of an MCP server
    session L1-additive-scenario-1-p8-r4: satisfied — event 88: answered `HW-DR-0112`; no call to a tool of an MCP server
    session L1-additive-scenario-1-p8-r5: satisfied — event 89: answered `HW-DR-0112`; no call to a tool of an MCP server
    session L1-additive-scenario-1-p8-r6: satisfied — event 90: answered `HW-DR-0112`; no call to a tool of an MCP server
- HW-PROBE-a-session-reaches-the-adjudication-from-the-tombstone-that-the-task-names (navigability, expects opened)
    session L1-additive-scenario-1-p9-r1: satisfied — event 91, call 11: `Read /mnt/build/hw-1881/phase-b/b3/ws/L1-additive-scenario-1-p9-r1/docs/decisions/0008-probe-cost-and-cadence.md`; no call to a tool of an MCP server
    session L1-additive-scenario-1-p9-r2: satisfied — event 92, call 5: `Read docs/decisions/0008-probe-cost-and-cadence.md`; no call to a tool of an MCP server
    session L1-additive-scenario-1-p9-r3: satisfied — event 93, call 3: `Read /mnt/build/hw-1881/phase-b/b3/ws/L1-additive-scenario-1-p9-r3/docs/decisions/0008-probe-cost-and-cadence.md`; no call to a tool of an MCP server
    session L1-additive-scenario-1-p9-r4: satisfied — event 94, call 3: `Read /mnt/build/hw-1881/phase-b/b3/ws/L1-additive-scenario-1-p9-r4/docs/decisions/0008-probe-cost-and-cadence.md`; no call to a tool of an MCP server
    session L1-additive-scenario-1-p9-r5: satisfied — event 95, call 5: `Read /mnt/build/hw-1881/phase-b/b3/ws/L1-additive-scenario-1-p9-r5/docs/spec/09-decisions.md`; no call to a tool of an MCP server
    session L1-additive-scenario-1-p9-r6: satisfied — event 96, call 3: `Read /mnt/build/hw-1881/phase-b/b3/ws/L1-additive-scenario-1-p9-r6/docs/decisions/0008-probe-cost-and-cadence.md`; no call to a tool of an MCP server
- HW-PROBE-a-session-records-the-walkthrough-that-the-federation-part-rests-a-rule-on-as-the-evidence-it-cites (sufficiency, expects patched)
    session L1-additive-scenario-1-p10-r1: satisfied — event 7: `command:r5-federation-walkthrough` reported nothing over `docs/evaluations/schema-format-walkthrough.md` or the other 1 produced artifact inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-1-p10-r2: satisfied — event 8: `command:r5-federation-walkthrough` reported nothing over `docs/evaluations/schema-format-walkthrough.md` or the other 1 produced artifact inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-1-p10-r3: satisfied — event 9: `command:r5-federation-walkthrough` reported nothing over `docs/evaluations/schema-format-walkthrough.md` or the other 1 produced artifact inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-1-p10-r4: satisfied — event 10: `command:r5-federation-walkthrough` reported nothing over `docs/evaluations/schema-format-walkthrough.md` or the other 1 produced artifact inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-1-p10-r5: satisfied — event 11: `command:r5-federation-walkthrough` reported nothing over `docs/evaluations/schema-format-walkthrough.md` or the other 1 produced artifact inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-1-p10-r6: satisfied — event 12: `command:r5-federation-walkthrough` reported nothing over `docs/evaluations/schema-format-walkthrough.md` or the other 1 produced artifact inside the workspace; no call to a tool of an MCP server
- HW-PROBE-a-session-renames-the-identifier-index-of-the-graph-crate-and-its-callers (sufficiency, expects patched)
    session L1-additive-scenario-1-p11-r1: satisfied — event 13: `command:r3-index-module` reported nothing over `docs/decisions/0068-the-front-matter-key-that-carries-a-minted-identifier-is-id.md` or the other 21 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-1-p11-r2: satisfied — event 14: `command:r3-index-module` reported nothing over `docs/decisions/0068-the-front-matter-key-that-carries-a-minted-identifier-is-id.md` or the other 22 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-1-p11-r3: not satisfied — event 15: `command:r3-index-module` reported over `engine/crates/check/src/claim.rs`, 20 of the 20 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-1-p11-r4: satisfied — event 16: `command:r3-index-module` reported nothing over `docs/decisions/0068-the-front-matter-key-that-carries-a-minted-identifier-is-id.md` or the other 23 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-1-p11-r5: not satisfied — event 17: `command:r3-index-module` reported over `engine/crates/check/src/claim.rs`, 21 of the 21 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-1-p11-r6: satisfied — event 18: `command:r3-index-module` reported nothing over `docs/decisions/0068-the-front-matter-key-that-carries-a-minted-identifier-is-id.md` or the other 22 produced artifacts inside the workspace; no call to a tool of an MCP server
- HW-PROBE-a-session-renames-the-module-of-the-check-crate-that-reads-the-state-a-document-stands-in (sufficiency, expects patched)
    session L1-additive-scenario-1-p12-r1: satisfied — event 19: `command:r3-lifecycle-module` reported nothing over `docs/decisions/0026-q26-whether-terminality-belongs-to-a-state-or-to-a-state-and-a-regime.md` or the other 24 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-1-p12-r2: satisfied — event 20: `command:r3-lifecycle-module` reported nothing over `docs/decisions/0026-q26-whether-terminality-belongs-to-a-state-or-to-a-state-and-a-regime.md` or the other 24 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-1-p12-r3: not satisfied — event 21: `command:r3-lifecycle-module` reported over `engine/crates/check/src/acceptance_state.rs`, 22 of the 22 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-1-p12-r4: not satisfied — event 22: `command:r3-lifecycle-module` reported over `engine/crates/check/src/acceptance_state.rs`, 23 of the 23 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-1-p12-r5: not satisfied — event 23: `command:r3-lifecycle-module` reported over `engine/crates/check/src/acceptance_state.rs`, 22 of the 22 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-1-p12-r6: not satisfied — event 24: `command:r3-lifecycle-module` reported over `engine/crates/check/src/acceptance_state.rs`, 22 of the 22 produced artifacts inside the workspace; no call to a tool of an MCP server
- HW-PROBE-a-session-rewrites-the-change-narration-in-the-summary-of-the-obligation-on-eight-stale-comments-and-raises-no-other-finding (sufficiency, expects patched)
    session L1-additive-scenario-1-p13-r1: satisfied — event 25: `command:r9-stale-comments` reported nothing over `docs/obligations/0230-comments-in-the-command-surface-crates-and-their-subsystem-spec-state-eight-stale-facts.md`; no call to a tool of an MCP server
    session L1-additive-scenario-1-p13-r2: not satisfied — event 26: `command:r9-stale-comments` reported over `docs/subsystems/command-surface.md`, 8 of the 8 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-1-p13-r3: satisfied — event 27: `command:r9-stale-comments` reported nothing over `docs/obligations/0230-comments-in-the-command-surface-crates-and-their-subsystem-spec-state-eight-stale-facts.md` or the other 1 produced artifact inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-1-p13-r4: satisfied — event 28: `command:r9-stale-comments` reported nothing over `docs/obligations/0230-comments-in-the-command-surface-crates-and-their-subsystem-spec-state-eight-stale-facts.md`; no call to a tool of an MCP server
    session L1-additive-scenario-1-p13-r5: satisfied — event 29: `command:r9-stale-comments` reported nothing over `docs/obligations/0230-comments-in-the-command-surface-crates-and-their-subsystem-spec-state-eight-stale-facts.md`; no call to a tool of an MCP server
    session L1-additive-scenario-1-p13-r6: satisfied — event 30: `command:r9-stale-comments` reported nothing over `docs/obligations/0230-comments-in-the-command-surface-crates-and-their-subsystem-spec-state-eight-stale-facts.md`; no call to a tool of an MCP server
- HW-PROBE-a-session-splits-the-long-summary-of-the-obligation-that-the-task-names-and-raises-no-other-finding (sufficiency, expects patched)
    session L1-additive-scenario-1-p14-r1: satisfied — event 31: `command:r9-shelf-layout` reported nothing over `docs/obligations/0106-a-shelf-layout-names-a-file-at-birth-and-no-rule-reads-it.md`; no call to a tool of an MCP server
    session L1-additive-scenario-1-p14-r2: satisfied — event 32: `command:r9-shelf-layout` reported nothing over `docs/obligations/0106-a-shelf-layout-names-a-file-at-birth-and-no-rule-reads-it.md`; no call to a tool of an MCP server
    session L1-additive-scenario-1-p14-r3: satisfied — event 33: `command:r9-shelf-layout` reported nothing over `docs/obligations/0106-a-shelf-layout-names-a-file-at-birth-and-no-rule-reads-it.md`; no call to a tool of an MCP server
    session L1-additive-scenario-1-p14-r4: satisfied — event 34: `command:r9-shelf-layout` reported nothing over `docs/obligations/0106-a-shelf-layout-names-a-file-at-birth-and-no-rule-reads-it.md` or the other 1 produced artifact inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-1-p14-r5: satisfied — event 35: `command:r9-shelf-layout` reported nothing over `docs/obligations/0106-a-shelf-layout-names-a-file-at-birth-and-no-rule-reads-it.md`; no call to a tool of an MCP server
    session L1-additive-scenario-1-p14-r6: satisfied — event 36: `command:r9-shelf-layout` reported nothing over `docs/obligations/0106-a-shelf-layout-names-a-file-at-birth-and-no-rule-reads-it.md`; no call to a tool of an MCP server
- HW-PROBE-a-session-that-reviews-a-change-dropping-the-release-digest-from-the-notes-of-the-engine-release-cites-the-record-that-governs-the-workflow (sufficiency, expects answered)
    session L1-additive-scenario-1-p15-r1: satisfied — event 37: answered `HW-DR-0090`; no call to a tool of an MCP server
    session L1-additive-scenario-1-p15-r2: satisfied — event 38: answered `HW-DR-0090`; no call to a tool of an MCP server
    session L1-additive-scenario-1-p15-r3: not satisfied — the session ended with no answer; no call to a tool of an MCP server
    session L1-additive-scenario-1-p15-r4: satisfied — event 40: answered `HW-DR-0090`; no call to a tool of an MCP server
    session L1-additive-scenario-1-p15-r5: not satisfied — the session ended with no answer; no call to a tool of an MCP server
    session L1-additive-scenario-1-p15-r6: satisfied — event 42: answered `HW-DR-0090`; no call to a tool of an MCP server
- HW-PROBE-a-session-that-reviews-an-agent-s-acceptance-stamp-on-a-branch-that-needs-no-approval-cites-the-ruling-in-force-and-not-the-draft-that-disagrees-with-it (sufficiency, expects answered)
    session L1-additive-scenario-1-p16-r1: satisfied — event 43: answered `HW-DR-0034`; no call to a tool of an MCP server
    session L1-additive-scenario-1-p16-r2: satisfied — event 44: answered `HW-DR-0034`; no call to a tool of an MCP server
    session L1-additive-scenario-1-p16-r3: satisfied — event 45: answered `HW-DR-0034`; no call to a tool of an MCP server
    session L1-additive-scenario-1-p16-r4: not satisfied — the session ended with no answer; no call to a tool of an MCP server
    session L1-additive-scenario-1-p16-r5: satisfied — event 47: answered `HW-DR-0034`; no call to a tool of an MCP server
    session L1-additive-scenario-1-p16-r6: satisfied — event 48: answered `HW-DR-0034`; no call to a tool of an MCP server

96 of 96 sessions that recorded their calls made no call to a tool of an MCP server. 0 sessions recorded no calls.

## The rate, and the denominator it is over

76 of 96 graded sessions satisfied their expectation: 79.2%, in a 95% interval of 70.0% to 86.1%.
The denominator is the graded sessions and never the selected probes. 0 sessions reached no verdict, and a session with no verdict is outside both halves of that fraction.

An interval that overlaps the previous run's is variance and one that does not is drift. This is one arm, so it estimates no effect: an efficacy claim is a comparison of two results, and the arm each one recorded is on it.
