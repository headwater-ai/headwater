---
"headwater:generated": "probe_result. `headwater generate` writes this file, and `headwater generate --check` holds it. Edit the corpus, not this file."
id: HW-RESULT-additive-final-round-of-2026-11-09-batch-a-scenario-2
title: Probe result for additive-final-round-of-2026-11-09-batch-a-scenario-2
status: current
status_since: 2026-11-22
summary: "The grade of the transcript `additive-final-round-of-2026-11-09-batch-a-scenario-2`, taken over the probes this corpus declares and the version of the grader that evaluated them."
last_verified: 2026-11-22
---

# The result of docs/probe-runs/additive-final-round-of-2026-11-09-batch-a-scenario-2.md

A probe result is a function of committed inputs and of nothing else: the transcript at `docs/probe-runs/additive-final-round-of-2026-11-09-batch-a-scenario-2.md` and the state it stands at, the expectations the probes it names declare, the version of the grader that evaluated them, and for a compared arm the result of the other arm. Fetch those and this file comes back.

## The run this transcript recorded

A additive run in the scenario-2 arm, on claude-sonnet-5 at 2026-11-09.
served version claude-sonnet-5, tree sha256:77c336d487649121d85466ae2faad7a027d8b8fadd8350f0f1ce64aa2294ba19, selection sha256:59d63e891e4f3d89863a84096d48ec1b0cac730eb3d7c5a2cd518fc2543f8d81, read set sha256:1826437b155ad5563b0e345cda4cf5ffe827bd0e6ab464e36996e2077fa013b9, seed 0, harness 0.6.1.
realized cost $51.83, which the adaptive layer reads as the cost of its own instrument.

66 events over 22 probes, in 66 sessions and 1838 tool calls.

The engine confirmed the taxonomy, that every member of the run identity is present, the membership of every probe named, that no key outside the closed set appears, and that a realized cost was recorded. Present is not confirmed. This page names the lock, the selection and the read set the transcript recorded, and it compares none of them with the tree in front of a reader. `headwater generate` and `headwater probe stale` name what moved since the recording, and neither one fails for it. It graded nothing: a verdict is a function of this transcript, the expectations these probes declare and a grader version, and `headwater probe grade` is the verb that holds all three.

The selection this transcript names is `sha256:59d63e891e4f3d89863a84096d48ec1b0cac730eb3d7c5a2cd518fc2543f8d81`, and the probes graded below are the probes it was planned over, as this corpus declares them now. A probe added to this corpus after the recording is not a session this run owed. The tree, the seed and the harness above are provenance and nothing compares them.

The `read_set` digest above covers every probe of the selection and every document one of them examines, by path and content. This file names it as provenance and compares it with nothing, and the same holds for the lock and the selection: an edit to a document, a new probe or a moved lock leaves these bytes alone. `headwater generate` names each result whose recorded lock, read set or selection moved, and `headwater probe stale` takes the digest and reports which recorded results a change voided. Neither one fails for it.

Graded by grader 0.5.3.
A additive run in the scenario-2 arm, on claude-sonnet-5 at served version claude-sonnet-5.

## The verdicts

- HW-PROBE-a-session-holds-the-trademark-notice-to-the-house-language-rules-from-the-overlay-that-the-task-names (sufficiency, expects patched)
    session L2-additive-scenario-2-p1-r1: satisfied — event 1: `command:r8-trademarks` reported nothing over `.headwater/overlay.yml`; no call to a tool of an MCP server
    session L2-additive-scenario-2-p1-r2: not satisfied — event 2: `command:r8-trademarks` reported over `.headwater/overlay.yml`, 1 of the 1 produced artifact inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-2-p1-r3: satisfied — event 3: `command:r8-trademarks` reported nothing over `.headwater/overlay.yml`; no call to a tool of an MCP server
- HW-PROBE-a-session-lets-the-asset-size-check-of-the-site-take-its-limit-in-mebibytes-as-a-second-argument (sufficiency, expects patched)
    session L2-additive-scenario-2-p2-r1: satisfied — event 34: `command:r11-asset-limit` reported nothing over `tools/site/check-site-asset-size.sh`; no call to a tool of an MCP server
    session L2-additive-scenario-2-p2-r2: satisfied — event 35: `command:r11-asset-limit` reported nothing over `tools/site/check-site-asset-size.sh`; no call to a tool of an MCP server
    session L2-additive-scenario-2-p2-r3: satisfied — event 36: `command:r11-asset-limit` reported nothing over `tools/site/check-site-asset-size.sh`; no call to a tool of an MCP server
- HW-PROBE-a-session-lets-the-graph-export-list-the-patterns-of-every-anchor (sufficiency, expects patched)
    session L2-additive-scenario-2-p3-r1: satisfied — event 46: `command:r2-export-patterns` reported nothing over `docs/acceptance-criteria/0004-each-list-anchor-in-the-native-export-carries-its-sorted-members-and-a-single-pattern-carries-none.md` or the other 6 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-2-p3-r2: satisfied — event 47: `command:r2-export-patterns` reported nothing over `docs/acceptance-criteria/0004-each-list-anchor-in-the-native-export-carries-its-sorted-members-and-a-single-pattern-carries-none.md` or the other 5 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-2-p3-r3: satisfied — event 48: `command:r2-export-patterns` reported nothing over `docs/interfaces/headwater-export.md` or the other 2 produced artifacts inside the workspace; no call to a tool of an MCP server
- HW-PROBE-a-session-lets-the-required-sections-rule-read-a-numbered-heading (sufficiency, expects patched)
    session L2-additive-scenario-2-p4-r1: not satisfied — event 49: `command:r2-numbered-heading` reported over `engine/crates/check/src/sections.rs`, 1 of the 1 produced artifact inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-2-p4-r2: not satisfied — event 50: `command:r2-numbered-heading` reported over `engine/crates/check/fixtures/check/spec/25-numbered-headings.md`, 2 of the 2 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-2-p4-r3: not satisfied — event 51: `command:r2-numbered-heading` reported over `engine/crates/check/fixtures/check/spec/08-contract-met.md`, 2 of the 2 produced artifacts inside the workspace; no call to a tool of an MCP server
- HW-PROBE-a-session-links-the-governs-edges-evaluation-to-the-obligation-that-it-says-is-discharged (sufficiency, expects patched)
    session L2-additive-scenario-2-p5-r1: satisfied — event 52: `command:r5-hook-role` reported nothing over `docs/evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md` or the other 1 produced artifact inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-2-p5-r2: satisfied — event 53: `command:r5-hook-role` reported nothing over `docs/evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md` or the other 1 produced artifact inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-2-p5-r3: satisfied — event 54: `command:r5-hook-role` reported nothing over `docs/evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md` or the other 1 produced artifact inside the workspace; no call to a tool of an MCP server
- HW-PROBE-a-session-makes-the-script-that-prints-the-apt-block-remove-sudo-with-its-environment-flag-as-a-whole (sufficiency, expects patched)
    session L2-additive-scenario-2-p6-r1: satisfied — event 55: `command:r11-apt-block` reported nothing over `tools/repo/readme-apt-block.sh`; no call to a tool of an MCP server
    session L2-additive-scenario-2-p6-r2: satisfied — event 56: `command:r11-apt-block` reported nothing over `tools/repo/readme-apt-block.sh`; no call to a tool of an MCP server
    session L2-additive-scenario-2-p6-r3: satisfied — event 57: `command:r11-apt-block` reported nothing over `tools/repo/readme-apt-block.sh`; no call to a tool of an MCP server
- HW-PROBE-a-session-moves-the-obligation-record-of-one-run-s-trailing-edges-to-the-process-shelf-and-repairs-every-link-to-it (sufficiency, expects patched)
    session L2-additive-scenario-2-p7-r1: satisfied — event 58: `command:r7-run-findings` reported nothing over `docs/process/obligations/0242-three-edges-and-a-comment-that-trail-a-change-of-run-20261012-2342.md` or the other 3 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-2-p7-r2: satisfied — event 59: `command:r7-run-findings` reported nothing over `docs/process/obligations/0242-three-edges-and-a-comment-that-trail-a-change-of-run-20261012-2342.md` or the other 2 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-2-p7-r3: satisfied — event 60: `command:r7-run-findings` reported nothing over `docs/process/obligations/0242-three-edges-and-a-comment-that-trail-a-change-of-run-20261012-2342.md` or the other 6 produced artifacts inside the workspace; no call to a tool of an MCP server
- HW-PROBE-a-session-moves-the-record-of-one-run-s-test-gaps-to-the-process-shelf-and-repairs-the-link-from-a-sibling-record (sufficiency, expects patched)
    session L2-additive-scenario-2-p8-r1: satisfied — event 61: `command:r7-test-gaps` reported nothing over `docs/obligations/0237-eighteen-findings-that-the-paid-layer-campaign-of-run-20261003-1026-left-in-the-probe-recorder-and-the-campaign-driver.md` or the other 4 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-2-p8-r2: satisfied — event 62: `command:r7-test-gaps` reported nothing over `docs/obligations/0237-eighteen-findings-that-the-paid-layer-campaign-of-run-20261003-1026-left-in-the-probe-recorder-and-the-campaign-driver.md` or the other 4 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-2-p8-r3: satisfied — event 63: `command:r7-test-gaps` reported nothing over `docs/obligations/0237-eighteen-findings-that-the-paid-layer-campaign-of-run-20261003-1026-left-in-the-probe-recorder-and-the-campaign-driver.md` or the other 7 produced artifacts inside the workspace; no call to a tool of an MCP server
- HW-PROBE-a-session-names-the-record-that-governs-the-scripts-that-grade-a-change-from-a-task-in-plain-words (navigability, expects answered)
    session L2-additive-scenario-2-p9-r1: satisfied — event 64: answered `HW-DR-0112`; no call to a tool of an MCP server
    session L2-additive-scenario-2-p9-r2: not satisfied — event 65 answered `none`, and the probe expects `HW-DR-0112`; no call to a tool of an MCP server
    session L2-additive-scenario-2-p9-r3: satisfied — event 66: answered `HW-DR-0112`; no call to a tool of an MCP server
- HW-PROBE-a-session-reaches-the-adjudication-from-the-tombstone-that-the-task-names (navigability, expects opened)
    session L2-additive-scenario-2-p10-r1: satisfied — event 4, call 2: `Read docs/decisions/0008-probe-cost-and-cadence.md`; no call to a tool of an MCP server
    session L2-additive-scenario-2-p10-r2: satisfied — event 5, call 3: `Read /mnt/build/hw-1881/phase-b/a/ws/L2-additive-scenario-2-p10-r2/docs/decisions/0008-probe-cost-and-cadence.md`; no call to a tool of an MCP server
    session L2-additive-scenario-2-p10-r3: satisfied — event 6, call 2: `Read /mnt/build/hw-1881/phase-b/a/ws/L2-additive-scenario-2-p10-r3/docs/decisions/0008-probe-cost-and-cadence.md`; no call to a tool of an MCP server
- HW-PROBE-a-session-records-as-owed-that-nothing-reports-a-pass-rate-for-each-phrasing-of-a-probe (sufficiency, expects patched)
    session L2-additive-scenario-2-p11-r1: not satisfied — event 7: `command:r4-phrasing-rate` reported over `docs/obligations/0243-nothing-reports-a-pass-rate-for-each-phrasing-of-a-probe-so-a-misleading-or-answer-giving-phrasing-hides-in-the-pooled-rate.md`, 2 of the 2 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-2-p11-r2: satisfied — event 8: `command:r4-phrasing-rate` reported nothing over `docs/obligations/0243-the-campaign-pools-every-phrasing-of-a-probe-s-task-and-no-rate-exists-per-phrasing.md`; no call to a tool of an MCP server
    session L2-additive-scenario-2-p11-r3: not satisfied — event 9: `command:r4-phrasing-rate` reported over `docs/obligations/0243-a-probes-phrasings-are-pooled-and-no-result-splits-them.md`, 2 of the 2 produced artifacts inside the workspace; no call to a tool of an MCP server
- HW-PROBE-a-session-records-as-owed-the-axis-of-an-adopter-s-own-repository-that-the-named-decision-defers (sufficiency, expects patched)
    session L2-additive-scenario-2-p12-r1: satisfied — event 10: `command:r4-adopter-axis` reported nothing over `docs/obligations/0243-the-additive-campaign-s-circumstance-axes-defer-an-adopter-s-own-repository.md`; no call to a tool of an MCP server
    session L2-additive-scenario-2-p12-r2: satisfied — event 11: `command:r4-adopter-axis` reported nothing over `docs/obligations/0243-the-additive-campaign-measures-nothing-in-a-repository-other-than-this-one.md` or the other 1 produced artifact inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-2-p12-r3: satisfied — event 12: `command:r4-adopter-axis` reported nothing over `docs/obligations/0243-the-additive-campaign-defers-an-adopters-own-repository-and-only-this-one-has-ever-run-it.md` or the other 1 produced artifact inside the workspace; no call to a tool of an MCP server
- HW-PROBE-a-session-records-the-walkthrough-that-the-federation-part-rests-a-rule-on-as-the-evidence-it-cites (sufficiency, expects patched)
    session L2-additive-scenario-2-p13-r1: satisfied — event 13: `command:r5-federation-walkthrough` reported nothing over `docs/evaluations/schema-format-walkthrough.md` or the other 1 produced artifact inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-2-p13-r2: satisfied — event 14: `command:r5-federation-walkthrough` reported nothing over `docs/evaluations/schema-format-walkthrough.md` or the other 1 produced artifact inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-2-p13-r3: satisfied — event 15: `command:r5-federation-walkthrough` reported nothing over `docs/evaluations/schema-format-walkthrough.md` or the other 1 produced artifact inside the workspace; no call to a tool of an MCP server
- HW-PROBE-a-session-renames-the-identifier-index-of-the-graph-crate-and-its-callers (sufficiency, expects patched)
    session L2-additive-scenario-2-p14-r1: satisfied — event 16: `command:r3-index-module` reported nothing over `docs/decisions/0068-the-front-matter-key-that-carries-a-minted-identifier-is-id.md` or the other 23 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-2-p14-r2: satisfied — event 17: `command:r3-index-module` reported nothing over `docs/decisions/0068-the-front-matter-key-that-carries-a-minted-identifier-is-id.md` or the other 21 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-2-p14-r3: satisfied — event 18: `command:r3-index-module` reported nothing over `docs/decisions/0068-the-front-matter-key-that-carries-a-minted-identifier-is-id.md` or the other 23 produced artifacts inside the workspace; no call to a tool of an MCP server
- HW-PROBE-a-session-renames-the-module-of-the-check-crate-that-reads-the-state-a-document-stands-in (sufficiency, expects patched)
    session L2-additive-scenario-2-p15-r1: satisfied — event 19: `command:r3-lifecycle-module` reported nothing over `docs/decisions/0026-q26-whether-terminality-belongs-to-a-state-or-to-a-state-and-a-regime.md` or the other 24 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-2-p15-r2: not satisfied — event 20: `command:r3-lifecycle-module` reported over `engine/crates/check/src/acceptance_state.rs`, 19 of the 19 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-2-p15-r3: satisfied — event 21: `command:r3-lifecycle-module` reported nothing over `docs/decisions/0026-q26-whether-terminality-belongs-to-a-state-or-to-a-state-and-a-regime.md` or the other 23 produced artifacts inside the workspace; no call to a tool of an MCP server
- HW-PROBE-a-session-retires-a-word-with-its-replacement-so-that-the-check-offers-the-substitution (sufficiency, expects patched)
    session L2-additive-scenario-2-p16-r1: satisfied — event 22: `command:r8-utilize` reported nothing over `docs/spec/02-taxonomy-model.md` or the other 1 produced artifact inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-2-p16-r2: satisfied — event 23: `command:r8-utilize` reported nothing over `.headwater/overlay.yml`; no call to a tool of an MCP server
    session L2-additive-scenario-2-p16-r3: satisfied — event 24: `command:r8-utilize` reported nothing over `.headwater/overlay.yml`; no call to a tool of an MCP server
- HW-PROBE-a-session-rewrites-the-change-narration-in-the-summary-of-the-obligation-on-eight-stale-comments-and-raises-no-other-finding (sufficiency, expects patched)
    session L2-additive-scenario-2-p17-r1: satisfied — event 25: `command:r9-stale-comments` reported nothing over `docs/obligations/0230-comments-in-the-command-surface-crates-and-their-subsystem-spec-state-eight-stale-facts.md`; no call to a tool of an MCP server
    session L2-additive-scenario-2-p17-r2: satisfied — event 26: `command:r9-stale-comments` reported nothing over `docs/obligations/0230-comments-in-the-command-surface-crates-and-their-subsystem-spec-state-eight-stale-facts.md`; no call to a tool of an MCP server
    session L2-additive-scenario-2-p17-r3: satisfied — event 27: `command:r9-stale-comments` reported nothing over `docs/obligations/0230-comments-in-the-command-surface-crates-and-their-subsystem-spec-state-eight-stale-facts.md` or the other 1 produced artifact inside the workspace; no call to a tool of an MCP server
- HW-PROBE-a-session-splits-the-long-summary-of-the-obligation-that-the-task-names-and-raises-no-other-finding (sufficiency, expects patched)
    session L2-additive-scenario-2-p18-r1: satisfied — event 28: `command:r9-shelf-layout` reported nothing over `docs/obligations/0106-a-shelf-layout-names-a-file-at-birth-and-no-rule-reads-it.md`; no call to a tool of an MCP server
    session L2-additive-scenario-2-p18-r2: satisfied — event 29: `command:r9-shelf-layout` reported nothing over `docs/obligations/0106-a-shelf-layout-names-a-file-at-birth-and-no-rule-reads-it.md`; no call to a tool of an MCP server
    session L2-additive-scenario-2-p18-r3: not satisfied — event 30: `command:r9-shelf-layout` reported over `docs/obligations/0106-a-shelf-layout-names-a-file-at-birth-and-no-rule-reads-it.md`, 1 of the 1 produced artifact inside the workspace; no call to a tool of an MCP server
- HW-PROBE-a-session-supersedes-the-record-that-keeps-a-grammar-checker-out-of-the-engine-and-not-the-older-record-that-it-upholds (sufficiency, expects patched)
    session L2-additive-scenario-2-p19-r1: satisfied — event 31: `command:r6-harper` reported nothing over `docs/decisions/0096-harper-does-not-become-part-of-the-engine-now-and-q41-stands.md` or the other 3 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-2-p19-r2: satisfied — event 32: `command:r6-harper` reported nothing over `docs/decisions/0096-harper-does-not-become-part-of-the-engine-now-and-q41-stands.md` or the other 2 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-2-p19-r3: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
- HW-PROBE-a-session-supersedes-the-standing-ruling-on-figures-of-a-hand-built-page-when-the-task-names-the-record-that-ruling-replaced (navigability, expects patched)
    session L2-additive-scenario-2-p20-r1: satisfied — event 37: `command:r6-figure-page` reported nothing over `docs/decisions/0097-a-figure-on-a-hand-built-page-is-measured-when-the-site-is-published-and-the-committed-page-carries-none.md` or the other 1 produced artifact inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-2-p20-r2: satisfied — event 38: `command:r6-figure-page` reported nothing over `docs/decisions/0097-a-figure-on-a-hand-built-page-is-measured-when-the-site-is-published-and-the-committed-page-carries-none.md` or the other 1 produced artifact inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-2-p20-r3: satisfied — event 39: `command:r6-figure-page` reported nothing over `docs/decisions/0097-a-figure-on-a-hand-built-page-is-measured-when-the-site-is-published-and-the-committed-page-carries-none.md` or the other 1 produced artifact inside the workspace; no call to a tool of an MCP server
- HW-PROBE-a-session-that-reviews-a-change-dropping-the-release-digest-from-the-notes-of-the-engine-release-cites-the-record-that-governs-the-workflow (sufficiency, expects answered)
    session L2-additive-scenario-2-p21-r1: satisfied — event 40: answered `HW-DR-0090`; no call to a tool of an MCP server
    session L2-additive-scenario-2-p21-r2: satisfied — event 41: answered `HW-DR-0090`; no call to a tool of an MCP server
    session L2-additive-scenario-2-p21-r3: satisfied — event 42: answered `HW-DR-0090`; no call to a tool of an MCP server
- HW-PROBE-a-session-that-reviews-an-agent-s-acceptance-stamp-on-a-branch-that-needs-no-approval-cites-the-ruling-in-force-and-not-the-draft-that-disagrees-with-it (sufficiency, expects answered)
    session L2-additive-scenario-2-p22-r1: satisfied — event 43: answered `HW-DR-0034`; no call to a tool of an MCP server
    session L2-additive-scenario-2-p22-r2: satisfied — event 44: answered `HW-DR-0034`; no call to a tool of an MCP server
    session L2-additive-scenario-2-p22-r3: satisfied — event 45: answered `HW-DR-0034`; no call to a tool of an MCP server

66 of 66 sessions that recorded their calls made no call to a tool of an MCP server. 0 sessions recorded no calls.

## The rate, and the denominator it is over

56 of 66 graded sessions satisfied their expectation: 84.8%, in a 95% interval of 74.3% to 91.6%.
The denominator is the graded sessions and never the selected probes. 0 sessions reached no verdict, and a session with no verdict is outside both halves of that fraction.

An interval that overlaps the previous run's is variance and one that does not is drift. This is one arm, so it estimates no effect: an efficacy claim is a comparison of two results, and the arm each one recorded is on it.
