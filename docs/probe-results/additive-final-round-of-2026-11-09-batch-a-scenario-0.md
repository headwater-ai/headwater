---
"headwater:generated": "probe_result. `headwater generate` writes this file, and `headwater generate --check` holds it. Edit the corpus, not this file."
id: HW-RESULT-additive-final-round-of-2026-11-09-batch-a-scenario-0
title: Probe result for additive-final-round-of-2026-11-09-batch-a-scenario-0
status: current
status_since: 2026-11-22
summary: "The grade of the transcript `additive-final-round-of-2026-11-09-batch-a-scenario-0`, taken over the probes this corpus declares and the version of the grader that evaluated them."
last_verified: 2026-11-22
---

# The result of docs/probe-runs/additive-final-round-of-2026-11-09-batch-a-scenario-0.md

A probe result is a function of committed inputs and of nothing else: the transcript at `docs/probe-runs/additive-final-round-of-2026-11-09-batch-a-scenario-0.md` and the state it stands at, the expectations the probes it names declare, the version of the grader that evaluated them, and for a compared arm the result of the other arm. Fetch those and this file comes back.

## The run this transcript recorded

A additive run in the scenario-0 arm, on claude-sonnet-5 at 2026-11-09.
served version claude-sonnet-5, tree sha256:77c336d487649121d85466ae2faad7a027d8b8fadd8350f0f1ce64aa2294ba19, selection sha256:59d63e891e4f3d89863a84096d48ec1b0cac730eb3d7c5a2cd518fc2543f8d81, read set sha256:1826437b155ad5563b0e345cda4cf5ffe827bd0e6ab464e36996e2077fa013b9, seed 0, harness 0.6.1.
realized cost $39.29, which the adaptive layer reads as the cost of its own instrument.

65 events over 22 probes, in 65 sessions and 1562 tool calls.

The engine confirmed the taxonomy, that every member of the run identity is present, the membership of every probe named, that no key outside the closed set appears, and that a realized cost was recorded. Present is not confirmed. This page names the lock, the selection and the read set the transcript recorded, and it compares none of them with the tree in front of a reader. `headwater generate` and `headwater probe stale` name what moved since the recording, and neither one fails for it. It graded nothing: a verdict is a function of this transcript, the expectations these probes declare and a grader version, and `headwater probe grade` is the verb that holds all three.

The selection this transcript names is `sha256:59d63e891e4f3d89863a84096d48ec1b0cac730eb3d7c5a2cd518fc2543f8d81`, and the probes graded below are the probes it was planned over, as this corpus declares them now. A probe added to this corpus after the recording is not a session this run owed. The tree, the seed and the harness above are provenance and nothing compares them.

The `read_set` digest above covers every probe of the selection and every document one of them examines, by path and content. This file names it as provenance and compares it with nothing, and the same holds for the lock and the selection: an edit to a document, a new probe or a moved lock leaves these bytes alone. `headwater generate` names each result whose recorded lock, read set or selection moved, and `headwater probe stale` takes the digest and reports which recorded results a change voided. Neither one fails for it.

Graded by grader 0.5.3.
A additive run in the scenario-0 arm, on claude-sonnet-5 at served version claude-sonnet-5.

## The verdicts

- HW-PROBE-a-session-holds-the-trademark-notice-to-the-house-language-rules-from-the-overlay-that-the-task-names (sufficiency, expects patched)
    session L1-additive-scenario-0-p1-r1: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
    session L1-additive-scenario-0-p1-r2: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
    session L1-additive-scenario-0-p1-r3: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
- HW-PROBE-a-session-lets-the-asset-size-check-of-the-site-take-its-limit-in-mebibytes-as-a-second-argument (sufficiency, expects patched)
    session L1-additive-scenario-0-p2-r1: satisfied — event 34: `command:r11-asset-limit` reported nothing over `tools/site/check-site-asset-size.sh`; no call to a tool of an MCP server
    session L1-additive-scenario-0-p2-r2: satisfied — event 35: `command:r11-asset-limit` reported nothing over `tools/site/check-site-asset-size.sh`; no call to a tool of an MCP server
    session L1-additive-scenario-0-p2-r3: satisfied — event 36: `command:r11-asset-limit` reported nothing over `tools/site/check-site-asset-size.sh`; no call to a tool of an MCP server
- HW-PROBE-a-session-lets-the-graph-export-list-the-patterns-of-every-anchor (sufficiency, expects patched)
    session L1-additive-scenario-0-p3-r1: satisfied — event 46: `command:r2-export-patterns` reported nothing over `engine/crates/generate/src/export.rs` or the other 1 produced artifact inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-0-p3-r2: not satisfied — event 47: `command:r2-export-patterns` reported over `engine/crates/generate/src/export.rs`, 2 of the 2 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-0-p3-r3: not satisfied — event 48: `command:r2-export-patterns` reported over `engine/crates/generate/src/export.rs`, 2 of the 2 produced artifacts inside the workspace; no call to a tool of an MCP server
- HW-PROBE-a-session-lets-the-required-sections-rule-read-a-numbered-heading (sufficiency, expects patched)
    session L1-additive-scenario-0-p4-r1: not satisfied — event 49: `command:r2-numbered-heading` reported over `engine/crates/check/src/sections.rs`, 1 of the 1 produced artifact inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-0-p4-r2: not satisfied — event 50: `command:r2-numbered-heading` reported over `engine/crates/check/src/sections.rs`, 1 of the 1 produced artifact inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-0-p4-r3: not satisfied — event 51: `command:r2-numbered-heading` reported over `engine/crates/check/src/sections.rs`, 1 of the 1 produced artifact inside the workspace; no call to a tool of an MCP server
- HW-PROBE-a-session-links-the-governs-edges-evaluation-to-the-obligation-that-it-says-is-discharged (sufficiency, expects patched)
    session L1-additive-scenario-0-p5-r1: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
    session L1-additive-scenario-0-p5-r2: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
- HW-PROBE-a-session-makes-the-script-that-prints-the-apt-block-remove-sudo-with-its-environment-flag-as-a-whole (sufficiency, expects patched)
    session L1-additive-scenario-0-p6-r1: satisfied — event 54: `command:r11-apt-block` reported nothing over `tools/repo/readme-apt-block.sh`; no call to a tool of an MCP server
    session L1-additive-scenario-0-p6-r2: satisfied — event 55: `command:r11-apt-block` reported nothing over `tools/repo/readme-apt-block.sh`; no call to a tool of an MCP server
    session L1-additive-scenario-0-p6-r3: satisfied — event 56: `command:r11-apt-block` reported nothing over `tools/repo/readme-apt-block.sh`; no call to a tool of an MCP server
- HW-PROBE-a-session-moves-the-obligation-record-of-one-run-s-trailing-edges-to-the-process-shelf-and-repairs-every-link-to-it (sufficiency, expects patched)
    session L1-additive-scenario-0-p7-r1: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
    session L1-additive-scenario-0-p7-r2: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
    session L1-additive-scenario-0-p7-r3: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
- HW-PROBE-a-session-moves-the-record-of-one-run-s-test-gaps-to-the-process-shelf-and-repairs-the-link-from-a-sibling-record (sufficiency, expects patched)
    session L1-additive-scenario-0-p8-r1: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
    session L1-additive-scenario-0-p8-r2: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
    session L1-additive-scenario-0-p8-r3: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
- HW-PROBE-a-session-names-the-record-that-governs-the-scripts-that-grade-a-change-from-a-task-in-plain-words (navigability, expects answered)
    session L1-additive-scenario-0-p9-r1: satisfied — event 63: answered `HW-DR-0112`; no call to a tool of an MCP server
    session L1-additive-scenario-0-p9-r2: satisfied — event 64: answered `HW-DR-0112`; no call to a tool of an MCP server
    session L1-additive-scenario-0-p9-r3: satisfied — event 65: answered `HW-DR-0112`; no call to a tool of an MCP server
- HW-PROBE-a-session-reaches-the-adjudication-from-the-tombstone-that-the-task-names (navigability, expects opened)
    session L1-additive-scenario-0-p10-r1: not satisfied — 6 recorded calls, and none named any of the 2 documents; no call to a tool of an MCP server
    session L1-additive-scenario-0-p10-r2: not satisfied — 11 recorded calls, and none named any of the 2 documents; no call to a tool of an MCP server
    session L1-additive-scenario-0-p10-r3: not satisfied — 4 recorded calls, and none named any of the 2 documents; no call to a tool of an MCP server
- HW-PROBE-a-session-records-as-owed-that-nothing-reports-a-pass-rate-for-each-phrasing-of-a-probe (sufficiency, expects patched)
    session L1-additive-scenario-0-p11-r1: not satisfied — event 7: `command:r4-phrasing-rate` reported over `docs/obligations/0302-the-additive-campaigns-wording-pool-reports-no-pass-rate-per-wording.md`, 1 of the 1 produced artifact inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-0-p11-r2: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
    session L1-additive-scenario-0-p11-r3: not satisfied — event 9: `command:r4-phrasing-rate` reported over `docs/obligations/0219-a-probes-phrasings-pool-into-one-rate-and-no-result-splits-them.md`, 2 of the 2 produced artifacts inside the workspace; no call to a tool of an MCP server
- HW-PROBE-a-session-records-as-owed-the-axis-of-an-adopter-s-own-repository-that-the-named-decision-defers (sufficiency, expects patched)
    session L1-additive-scenario-0-p12-r1: not satisfied — event 10: `command:r4-adopter-axis` reported over `docs/obligations/0224-the-additive-campaign-defers-the-axis-of-an-adopters-own-repository-and-nothing-records-it-as-owed.md`, 1 of the 1 produced artifact inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-0-p12-r2: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
    session L1-additive-scenario-0-p12-r3: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
- HW-PROBE-a-session-records-the-walkthrough-that-the-federation-part-rests-a-rule-on-as-the-evidence-it-cites (sufficiency, expects patched)
    session L1-additive-scenario-0-p13-r1: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
    session L1-additive-scenario-0-p13-r2: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
    session L1-additive-scenario-0-p13-r3: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
- HW-PROBE-a-session-renames-the-identifier-index-of-the-graph-crate-and-its-callers (sufficiency, expects patched)
    session L1-additive-scenario-0-p14-r1: not satisfied — event 16: `command:r3-index-module` reported over `engine/crates/check/src/claim.rs`, 21 of the 21 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-0-p14-r2: not satisfied — event 17: `command:r3-index-module` reported over `engine/crates/check/src/claim.rs`, 21 of the 21 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-0-p14-r3: not satisfied — event 18: `command:r3-index-module` reported over `engine/crates/check/src/claim.rs`, 21 of the 21 produced artifacts inside the workspace; no call to a tool of an MCP server
- HW-PROBE-a-session-renames-the-module-of-the-check-crate-that-reads-the-state-a-document-stands-in (sufficiency, expects patched)
    session L1-additive-scenario-0-p15-r1: not satisfied — event 19: `command:r3-lifecycle-module` reported over `engine/crates/check/src/acceptance_state.rs`, 22 of the 22 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-0-p15-r2: not satisfied — event 20: `command:r3-lifecycle-module` reported over `engine/crates/check/src/acceptance_state.rs`, 22 of the 22 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-0-p15-r3: not satisfied — event 21: `command:r3-lifecycle-module` reported over `engine/crates/check/src/acceptance_state.rs`, 22 of the 22 produced artifacts inside the workspace; no call to a tool of an MCP server
- HW-PROBE-a-session-retires-a-word-with-its-replacement-so-that-the-check-offers-the-substitution (sufficiency, expects patched)
    session L1-additive-scenario-0-p16-r1: not satisfied — event 22: `command:r8-utilize` reported over `taxonomy-source/headwater-standard/assemblies/starter/assembly.yml`, 3 of the 3 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-0-p16-r2: not satisfied — event 23: `command:r8-utilize` reported over `engine/crates/check/fixtures/check.taxonomy.yml`, 2 of the 2 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-0-p16-r3: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
- HW-PROBE-a-session-rewrites-the-change-narration-in-the-summary-of-the-obligation-on-eight-stale-comments-and-raises-no-other-finding (sufficiency, expects patched)
    session L1-additive-scenario-0-p17-r1: not satisfied — event 25: `command:r9-stale-comments` reported over `engine/crates/cli/src/lib.rs`, 3 of the 3 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L1-additive-scenario-0-p17-r2: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
    session L1-additive-scenario-0-p17-r3: not satisfied — event 27: `command:r9-stale-comments` reported over `engine/crates/cli/src/lib.rs`, 4 of the 4 produced artifacts inside the workspace; no call to a tool of an MCP server
- HW-PROBE-a-session-splits-the-long-summary-of-the-obligation-that-the-task-names-and-raises-no-other-finding (sufficiency, expects patched)
    session L1-additive-scenario-0-p18-r1: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
    session L1-additive-scenario-0-p18-r2: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
    session L1-additive-scenario-0-p18-r3: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
- HW-PROBE-a-session-supersedes-the-record-that-keeps-a-grammar-checker-out-of-the-engine-and-not-the-older-record-that-it-upholds (sufficiency, expects patched)
    session L1-additive-scenario-0-p19-r1: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
    session L1-additive-scenario-0-p19-r2: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
    session L1-additive-scenario-0-p19-r3: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
- HW-PROBE-a-session-supersedes-the-standing-ruling-on-figures-of-a-hand-built-page-when-the-task-names-the-record-that-ruling-replaced (navigability, expects patched)
    session L1-additive-scenario-0-p20-r1: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
    session L1-additive-scenario-0-p20-r2: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
    session L1-additive-scenario-0-p20-r3: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
- HW-PROBE-a-session-that-reviews-a-change-dropping-the-release-digest-from-the-notes-of-the-engine-release-cites-the-record-that-governs-the-workflow (sufficiency, expects answered)
    session L1-additive-scenario-0-p21-r1: satisfied — event 40: answered `HW-DR-0090`; no call to a tool of an MCP server
    session L1-additive-scenario-0-p21-r2: satisfied — event 41: answered `HW-DR-0090`; no call to a tool of an MCP server
    session L1-additive-scenario-0-p21-r3: satisfied — event 42: answered `HW-DR-0090`; no call to a tool of an MCP server
- HW-PROBE-a-session-that-reviews-an-agent-s-acceptance-stamp-on-a-branch-that-needs-no-approval-cites-the-ruling-in-force-and-not-the-draft-that-disagrees-with-it (sufficiency, expects answered)
    session L1-additive-scenario-0-p22-r1: not satisfied — event 43 answered `HW-OBL-0108`, and the probe expects `HW-DR-0034`; no call to a tool of an MCP server
    session L1-additive-scenario-0-p22-r2: not satisfied — event 44 answered `HW-OBL-0108`, and the probe expects `HW-DR-0034`; no call to a tool of an MCP server
    session L1-additive-scenario-0-p22-r3: not satisfied — the session ended with no answer; no call to a tool of an MCP server

65 of 65 sessions that recorded their calls made no call to a tool of an MCP server. 0 sessions recorded no calls.

## The rate, and the denominator it is over

13 of 65 graded sessions satisfied their expectation: 20.0%, in a 95% interval of 12.1% to 31.3%.
The denominator is the graded sessions and never the selected probes. 0 sessions reached no verdict, and a session with no verdict is outside both halves of that fraction.

An interval that overlaps the previous run's is variance and one that does not is drift. This is one arm, so it estimates no effect: an efficacy claim is a comparison of two results, and the arm each one recorded is on it.
