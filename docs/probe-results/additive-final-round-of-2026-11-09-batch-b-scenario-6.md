---
"headwater:generated": "probe_result. `headwater generate` writes this file, and `headwater generate --check` holds it. Edit the corpus, not this file."
id: HW-RESULT-additive-final-round-of-2026-11-09-batch-b-scenario-6
title: Probe result for additive-final-round-of-2026-11-09-batch-b-scenario-6
status: current
status_since: 2026-11-22
summary: "The grade of the transcript `additive-final-round-of-2026-11-09-batch-b-scenario-6`, taken over the probes this corpus declares and the version of the grader that evaluated them."
last_verified: 2026-11-22
---

# The result of docs/probe-runs/additive-final-round-of-2026-11-09-batch-b-scenario-6.md

A probe result is a function of committed inputs and of nothing else: the transcript at `docs/probe-runs/additive-final-round-of-2026-11-09-batch-b-scenario-6.md` and the state it stands at, the expectations the probes it names declare, the version of the grader that evaluated them, and for a compared arm the result of the other arm. Fetch those and this file comes back.

## The run this transcript recorded

A additive run in the scenario-6 arm, on claude-sonnet-5 at 2026-11-09.
served version claude-sonnet-5, tree sha256:77c336d487649121d85466ae2faad7a027d8b8fadd8350f0f1ce64aa2294ba19, selection sha256:937fd228f03a148226864bb69ec93439368777c771ce2c6446e80b30c6acabf3, read set sha256:2b3062a5429a026f6154720c0026e70143b0e68ed970f16e136bdb3d1b64f7ec, seed 0, harness 0.6.1.
realized cost $75.61, which the adaptive layer reads as the cost of its own instrument.

84 events over 14 probes, in 84 sessions and 2683 tool calls.

The engine confirmed the taxonomy, that every member of the run identity is present, the membership of every probe named, that no key outside the closed set appears, and that a realized cost was recorded. Present is not confirmed. This page names the lock, the selection and the read set the transcript recorded, and it compares none of them with the tree in front of a reader. `headwater generate` and `headwater probe stale` name what moved since the recording, and neither one fails for it. It graded nothing: a verdict is a function of this transcript, the expectations these probes declare and a grader version, and `headwater probe grade` is the verb that holds all three.

The selection this transcript names is `sha256:937fd228f03a148226864bb69ec93439368777c771ce2c6446e80b30c6acabf3`, and the probes graded below are the probes it was planned over, as this corpus declares them now. A probe added to this corpus after the recording is not a session this run owed. The tree, the seed and the harness above are provenance and nothing compares them.

The `read_set` digest above covers every probe of the selection and every document one of them examines, by path and content. This file names it as provenance and compares it with nothing, and the same holds for the lock and the selection: an edit to a document, a new probe or a moved lock leaves these bytes alone. `headwater generate` names each result whose recorded lock, read set or selection moved, and `headwater probe stale` takes the digest and reports which recorded results a change voided. Neither one fails for it.

Graded by grader 0.5.3.
A additive run in the scenario-6 arm, on claude-sonnet-5 at served version claude-sonnet-5.

## The verdicts

- HW-PROBE-a-session-holds-the-trademark-notice-to-the-house-language-rules-from-the-overlay-that-the-task-names (sufficiency, expects patched)
    session L2-additive-scenario-6-p1-r1: satisfied — event 1: `command:r8-trademarks` reported nothing over `.headwater/overlay.yml`; no call to a tool of an MCP server
    session L2-additive-scenario-6-p1-r2: satisfied — event 2: `command:r8-trademarks` reported nothing over `.headwater/overlay.yml`; no call to a tool of an MCP server
    session L2-additive-scenario-6-p1-r3: satisfied — event 3: `command:r8-trademarks` reported nothing over `.headwater/overlay.yml`; 2 calls to a tool of an MCP server
    session L2-additive-scenario-6-p1-r4: not satisfied — event 4: `command:r8-trademarks` reported over `.headwater/overlay.yml`, 1 of the 1 produced artifact inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-6-p1-r5: satisfied — event 5: `command:r8-trademarks` reported nothing over `.headwater/overlay.yml`; no call to a tool of an MCP server
    session L2-additive-scenario-6-p1-r6: not satisfied — event 6: `command:r8-trademarks` reported over `.headwater/overlay.yml`, 1 of the 1 produced artifact inside the workspace; 1 call to a tool of an MCP server
- HW-PROBE-a-session-lets-the-graph-export-list-the-patterns-of-every-anchor (sufficiency, expects patched)
    session L2-additive-scenario-6-p2-r1: satisfied — event 37: `command:r2-export-patterns` reported nothing over `docs/acceptance-criteria/0004-each-list-anchor-in-the-native-export-carries-its-sorted-members-and-a-single-pattern-carries-none.md` or the other 8 produced artifacts inside the workspace; 1 call to a tool of an MCP server
    session L2-additive-scenario-6-p2-r2: satisfied — event 38: `command:r2-export-patterns` reported nothing over `docs/acceptance-criteria/0004-each-anchor-in-the-native-export-carries-its-sorted-members-and-a-single-pattern-lists-that-one-pattern.md` or the other 6 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-6-p2-r3: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
    session L2-additive-scenario-6-p2-r4: satisfied — event 40: `command:r2-export-patterns` reported nothing over `docs/acceptance-criteria/0004-each-anchor-in-the-native-export-carries-its-sorted-members-and-a-single-pattern-lists-that-one.md` or the other 5 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-6-p2-r5: satisfied — event 41: `command:r2-export-patterns` reported nothing over `docs/acceptance-criteria/0004-each-list-anchor-in-the-native-export-carries-its-sorted-members-and-a-single-pattern-carries-none.md` or the other 6 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-6-p2-r6: satisfied — event 42: `command:r2-export-patterns` reported nothing over `docs/acceptance-criteria/0004-each-list-anchor-in-the-native-export-carries-its-sorted-members-and-a-single-pattern-carries-none.md` or the other 5 produced artifacts inside the workspace; no call to a tool of an MCP server
- HW-PROBE-a-session-lets-the-required-sections-rule-read-a-numbered-heading (sufficiency, expects patched)
    session L2-additive-scenario-6-p3-r1: satisfied — event 43: `command:r2-numbered-heading` reported nothing over `engine/crates/check/src/sections.rs` or the other 2 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-6-p3-r2: not satisfied — event 44: `command:r2-numbered-heading` reported over `engine/crates/check/fixtures/editions.ledger`, 2 of the 2 produced artifacts inside the workspace; 1 call to a tool of an MCP server
    session L2-additive-scenario-6-p3-r3: not satisfied — event 45: `command:r2-numbered-heading` reported over `engine/crates/check/src/sections.rs`, 1 of the 1 produced artifact inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-6-p3-r4: not satisfied — event 46: `command:r2-numbered-heading` reported over `engine/crates/check/src/sections.rs`, 1 of the 1 produced artifact inside the workspace; 1 call to a tool of an MCP server
    session L2-additive-scenario-6-p3-r5: satisfied — event 47: `command:r2-numbered-heading` reported nothing over `engine/crates/check/fixtures/editions.ledger` or the other 1 produced artifact inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-6-p3-r6: not satisfied — event 48: `command:r2-numbered-heading` reported over `engine/crates/check/src/sections.rs`, 1 of the 1 produced artifact inside the workspace; 1 call to a tool of an MCP server
- HW-PROBE-a-session-records-as-owed-that-nothing-reports-a-pass-rate-for-each-phrasing-of-a-probe (sufficiency, expects patched)
    session L2-additive-scenario-6-p4-r1: not satisfied — event 49: `command:r4-phrasing-rate` reported over `docs/obligations/0243-no-report-breaks-the-additive-campaign-s-pooled-phrasing-result-into-a-rate-for-each-phrasing.md`, 2 of the 2 produced artifacts inside the workspace; 2 calls to a tool of an MCP server
    session L2-additive-scenario-6-p4-r2: satisfied — event 50: `command:r4-phrasing-rate` reported nothing over `docs/obligations/0243-no-rate-per-phrasing-exists-because-the-campaign-draws-one-phrasing-per-session-and-pools-the-result.md`; 1 call to a tool of an MCP server
    session L2-additive-scenario-6-p4-r3: satisfied — event 51: `command:r4-phrasing-rate` reported nothing over `docs/obligations/0243-a-probe-s-phrasings-are-pooled-and-no-result-splits-them-so-one-phrasing-s-pass-rate-can-hide-inside-another-s.md` or the other 1 produced artifact inside the workspace; 1 call to a tool of an MCP server
    session L2-additive-scenario-6-p4-r4: satisfied — event 52: `command:r4-phrasing-rate` reported nothing over `docs/obligations/0243-a-pooled-phrasing-verdict-carries-no-pass-rate-per-wording-so-a-misleading-or-answer-giving-phrasing-is-invisible-in-it.md`; 1 call to a tool of an MCP server
    session L2-additive-scenario-6-p4-r5: satisfied — event 53: `command:r4-phrasing-rate` reported nothing over `docs/obligations/0243-the-additive-campaign-pools-a-probe-s-phrasing-draw-into-one-rate-and-no-rate-exists-per-phrasing.md`; 1 call to a tool of an MCP server
    session L2-additive-scenario-6-p4-r6: satisfied — event 54: `command:r4-phrasing-rate` reported nothing over `docs/obligations/0243-a-probe-s-phrasings-are-pooled-and-no-result-splits-them.md` or the other 1 produced artifact inside the workspace; 1 call to a tool of an MCP server
- HW-PROBE-a-session-records-as-owed-the-axis-of-an-adopter-s-own-repository-that-the-named-decision-defers (sufficiency, expects patched)
    session L2-additive-scenario-6-p5-r1: satisfied — event 55: `command:r4-adopter-axis` reported nothing over `docs/obligations/0243-the-additive-campaign-measures-no-probe-against-an-adopter-s-own-repository.md`; 1 call to a tool of an MCP server
    session L2-additive-scenario-6-p5-r2: satisfied — event 56: `command:r4-adopter-axis` reported nothing over `docs/obligations/0243-the-additive-campaign-measures-nothing-in-a-repository-other-than-this-one.md` or the other 1 produced artifact inside the workspace; 2 calls to a tool of an MCP server
    session L2-additive-scenario-6-p5-r3: satisfied — event 57: `command:r4-adopter-axis` reported nothing over `docs/obligations/0243-the-additive-campaign-runs-only-on-this-repository-so-an-adopter-s-own-repository-is-untested.md` or the other 1 produced artifact inside the workspace; 1 call to a tool of an MCP server
    session L2-additive-scenario-6-p5-r4: satisfied — event 58: `command:r4-adopter-axis` reported nothing over `docs/obligations/0243-an-adopter-s-own-repository-is-a-circumstance-axis-the-additive-campaign-defers.md`; 2 calls to a tool of an MCP server
    session L2-additive-scenario-6-p5-r5: satisfied — event 59: `command:r4-adopter-axis` reported nothing over `docs/obligations/0243-the-additive-campaign-measures-nothing-in-a-repository-other-than-this-one.md` or the other 1 produced artifact inside the workspace; 2 calls to a tool of an MCP server
    session L2-additive-scenario-6-p5-r6: satisfied — event 60: `command:r4-adopter-axis` reported nothing over `docs/obligations/0243-the-additive-campaign-defers-testing-against-an-adopter-s-own-repository-and-records-it-as-debt.md` or the other 1 produced artifact inside the workspace; no call to a tool of an MCP server
- HW-PROBE-a-session-renames-the-identifier-index-of-the-graph-crate-and-its-callers (sufficiency, expects patched)
    session L2-additive-scenario-6-p6-r1: satisfied — event 61: `command:r3-index-module` reported nothing over `docs/decisions/0068-the-front-matter-key-that-carries-a-minted-identifier-is-id.md` or the other 22 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-6-p6-r2: satisfied — event 62: `command:r3-index-module` reported nothing over `docs/decisions/0068-the-front-matter-key-that-carries-a-minted-identifier-is-id.md` or the other 22 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-6-p6-r3: satisfied — event 63: `command:r3-index-module` reported nothing over `docs/decisions/0068-the-front-matter-key-that-carries-a-minted-identifier-is-id.md` or the other 24 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-6-p6-r4: satisfied — event 64: `command:r3-index-module` reported nothing over `docs/decisions/0068-the-front-matter-key-that-carries-a-minted-identifier-is-id.md` or the other 23 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-6-p6-r5: satisfied — event 65: `command:r3-index-module` reported nothing over `docs/decisions/0068-the-front-matter-key-that-carries-a-minted-identifier-is-id.md` or the other 22 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-6-p6-r6: satisfied — event 66: `command:r3-index-module` reported nothing over `docs/decisions/0068-the-front-matter-key-that-carries-a-minted-identifier-is-id.md` or the other 23 produced artifacts inside the workspace; no call to a tool of an MCP server
- HW-PROBE-a-session-renames-the-module-of-the-check-crate-that-reads-the-state-a-document-stands-in (sufficiency, expects patched)
    session L2-additive-scenario-6-p7-r1: satisfied — event 67: `command:r3-lifecycle-module` reported nothing over `docs/decisions/0026-q26-whether-terminality-belongs-to-a-state-or-to-a-state-and-a-regime.md` or the other 23 produced artifacts inside the workspace; 4 calls to a tool of an MCP server
    session L2-additive-scenario-6-p7-r2: satisfied — event 68: `command:r3-lifecycle-module` reported nothing over `docs/decisions/0026-q26-whether-terminality-belongs-to-a-state-or-to-a-state-and-a-regime.md` or the other 24 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-6-p7-r3: satisfied — event 69: `command:r3-lifecycle-module` reported nothing over `docs/decisions/0026-q26-whether-terminality-belongs-to-a-state-or-to-a-state-and-a-regime.md` or the other 24 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-6-p7-r4: satisfied — event 70: `command:r3-lifecycle-module` reported nothing over `docs/decisions/0026-q26-whether-terminality-belongs-to-a-state-or-to-a-state-and-a-regime.md` or the other 24 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-6-p7-r5: satisfied — event 71: `command:r3-lifecycle-module` reported nothing over `docs/decisions/0026-q26-whether-terminality-belongs-to-a-state-or-to-a-state-and-a-regime.md` or the other 25 produced artifacts inside the workspace; 4 calls to a tool of an MCP server
    session L2-additive-scenario-6-p7-r6: satisfied — event 72: `command:r3-lifecycle-module` reported nothing over `docs/decisions/0026-q26-whether-terminality-belongs-to-a-state-or-to-a-state-and-a-regime.md` or the other 23 produced artifacts inside the workspace; 1 call to a tool of an MCP server
- HW-PROBE-a-session-retires-a-word-with-its-replacement-so-that-the-check-offers-the-substitution (sufficiency, expects patched)
    session L2-additive-scenario-6-p8-r1: satisfied — event 73: `command:r8-utilize` reported nothing over `.headwater/overlay.yml`; no call to a tool of an MCP server
    session L2-additive-scenario-6-p8-r2: not satisfied — event 74: `command:r8-utilize` reported over `.headwater/overlay.yml`, 1 of the 1 produced artifact inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-6-p8-r3: satisfied — event 75: `command:r8-utilize` reported nothing over `.headwater/overlay.yml`; no call to a tool of an MCP server
    session L2-additive-scenario-6-p8-r4: satisfied — event 76: `command:r8-utilize` reported nothing over `.headwater/overlay.yml`; no call to a tool of an MCP server
    session L2-additive-scenario-6-p8-r5: satisfied — event 77: `command:r8-utilize` reported nothing over `.headwater/overlay.yml`; 2 calls to a tool of an MCP server
    session L2-additive-scenario-6-p8-r6: satisfied — event 78: `command:r8-utilize` reported nothing over `docs/spec/02-taxonomy-model.md` or the other 1 produced artifact inside the workspace; no call to a tool of an MCP server
- HW-PROBE-a-session-rewrites-the-change-narration-in-the-summary-of-the-obligation-on-eight-stale-comments-and-raises-no-other-finding (sufficiency, expects patched)
    session L2-additive-scenario-6-p9-r1: satisfied — event 79: `command:r9-stale-comments` reported nothing over `docs/obligations/0230-comments-in-the-command-surface-crates-and-their-subsystem-spec-state-eight-stale-facts.md`; no call to a tool of an MCP server
    session L2-additive-scenario-6-p9-r2: satisfied — event 80: `command:r9-stale-comments` reported nothing over `docs/obligations/0230-comments-in-the-command-surface-crates-and-their-subsystem-spec-state-eight-stale-facts.md`; 3 calls to a tool of an MCP server
    session L2-additive-scenario-6-p9-r3: satisfied — event 81: `command:r9-stale-comments` reported nothing over `docs/obligations/0230-comments-in-the-command-surface-crates-and-their-subsystem-spec-state-eight-stale-facts.md`; no call to a tool of an MCP server
    session L2-additive-scenario-6-p9-r4: satisfied — event 82: `command:r9-stale-comments` reported nothing over `docs/obligations/0230-comments-in-the-command-surface-crates-and-their-subsystem-spec-state-eight-stale-facts.md`; no call to a tool of an MCP server
    session L2-additive-scenario-6-p9-r5: satisfied — event 83: `command:r9-stale-comments` reported nothing over `docs/obligations/0230-comments-in-the-command-surface-crates-and-their-subsystem-spec-state-eight-stale-facts.md`; 1 call to a tool of an MCP server
    session L2-additive-scenario-6-p9-r6: satisfied — event 84: `command:r9-stale-comments` reported nothing over `docs/obligations/0230-comments-in-the-command-surface-crates-and-their-subsystem-spec-state-eight-stale-facts.md`; no call to a tool of an MCP server
- HW-PROBE-a-session-splits-the-long-summary-of-the-obligation-that-the-task-names-and-raises-no-other-finding (sufficiency, expects patched)
    session L2-additive-scenario-6-p10-r1: satisfied — event 7: `command:r9-shelf-layout` reported nothing over `docs/obligations/0106-a-shelf-layout-names-a-file-at-birth-and-no-rule-reads-it.md`; 1 call to a tool of an MCP server
    session L2-additive-scenario-6-p10-r2: satisfied — event 8: `command:r9-shelf-layout` reported nothing over `docs/obligations/0106-a-shelf-layout-names-a-file-at-birth-and-no-rule-reads-it.md` or the other 1 produced artifact inside the workspace; 2 calls to a tool of an MCP server
    session L2-additive-scenario-6-p10-r3: satisfied — event 9: `command:r9-shelf-layout` reported nothing over `docs/obligations/0106-a-shelf-layout-names-a-file-at-birth-and-no-rule-reads-it.md`; 1 call to a tool of an MCP server
    session L2-additive-scenario-6-p10-r4: satisfied — event 10: `command:r9-shelf-layout` reported nothing over `docs/obligations/0106-a-shelf-layout-names-a-file-at-birth-and-no-rule-reads-it.md`; 3 calls to a tool of an MCP server
    session L2-additive-scenario-6-p10-r5: satisfied — event 11: `command:r9-shelf-layout` reported nothing over `docs/obligations/0106-a-shelf-layout-names-a-file-at-birth-and-no-rule-reads-it.md`; no call to a tool of an MCP server
    session L2-additive-scenario-6-p10-r6: satisfied — event 12: `command:r9-shelf-layout` reported nothing over `docs/obligations/0106-a-shelf-layout-names-a-file-at-birth-and-no-rule-reads-it.md`; 2 calls to a tool of an MCP server
- HW-PROBE-a-session-supersedes-the-record-that-keeps-a-grammar-checker-out-of-the-engine-and-not-the-older-record-that-it-upholds (sufficiency, expects patched)
    session L2-additive-scenario-6-p11-r1: satisfied — event 13: `command:r6-harper` reported nothing over `docs/decisions/0096-harper-does-not-become-part-of-the-engine-now-and-q41-stands.md` or the other 4 produced artifacts inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-6-p11-r2: satisfied — event 14: `command:r6-harper` reported nothing over `docs/decisions/0096-harper-does-not-become-part-of-the-engine-now-and-q41-stands.md` or the other 1 produced artifact inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-6-p11-r3: satisfied — event 15: `command:r6-harper` reported nothing over `docs/decisions/0096-harper-does-not-become-part-of-the-engine-now-and-q41-stands.md` or the other 2 produced artifacts inside the workspace; 4 calls to a tool of an MCP server
    session L2-additive-scenario-6-p11-r4: not satisfied — event 16: `command:r6-harper` reported over `docs/decisions/0114-114-harper-core-links-as-an-advisory-backend-of-the-language-regime-and-hw-dr-0096-is-superseded.md`, 1 of the 1 produced artifact inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-6-p11-r5: satisfied — event 17: `command:r6-harper` reported nothing over `docs/decisions/0096-harper-does-not-become-part-of-the-engine-now-and-q41-stands.md` or the other 1 produced artifact inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-6-p11-r6: satisfied — event 18: `command:r6-harper` reported nothing over `docs/decisions/0096-harper-does-not-become-part-of-the-engine-now-and-q41-stands.md` or the other 1 produced artifact inside the workspace; 1 call to a tool of an MCP server
- HW-PROBE-a-session-supersedes-the-standing-ruling-on-figures-of-a-hand-built-page-when-the-task-names-the-record-that-ruling-replaced (navigability, expects patched)
    session L2-additive-scenario-6-p12-r1: satisfied — event 19: `command:r6-figure-page` reported nothing over `docs/decisions/0097-a-figure-on-a-hand-built-page-is-measured-when-the-site-is-published-and-the-committed-page-carries-none.md` or the other 1 produced artifact inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-6-p12-r2: not satisfied — event 20: `command:r6-figure-page` reported over `docs/decisions/0114-a-hand-built-page-carries-a-typed-figure-beside-a-measured-on-date-marker-and-publishing-checks-it-against-a-run.md`, 1 of the 1 produced artifact inside the workspace; 2 calls to a tool of an MCP server
    session L2-additive-scenario-6-p12-r3: not satisfied — event 21: `command:r6-figure-page` reported over `docs/decisions/0114-a-typed-figure-on-a-hand-built-page-stands-beside-a-measured-on-date-marker-and-the-publish-job-checks-it-against-a-run.md`, 1 of the 1 produced artifact inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-6-p12-r4: not satisfied — event 22: `command:r6-figure-page` reported over `docs/decisions/0114-a-figure-on-a-hand-built-page-may-be-typed-when-a-measured-on-date-marker-stands-beside-it-and-the-publish-job-checks-it-against-a-fresh-run.md`, 1 of the 1 produced artifact inside the workspace; no call to a tool of an MCP server
    session L2-additive-scenario-6-p12-r5: not satisfied — the session produced no artifact, so the oracle had nothing to read; 2 calls to a tool of an MCP server
    session L2-additive-scenario-6-p12-r6: not satisfied — event 24: `command:r6-figure-page` reported over `docs/decisions/0114-a-typed-figure-on-a-hand-built-page-stands-beside-a-measured-on-date-marker-and-the-publish-job-checks-it-against-a-run.md`, 1 of the 1 produced artifact inside the workspace; no call to a tool of an MCP server
- HW-PROBE-a-session-that-reviews-a-change-dropping-the-release-digest-from-the-notes-of-the-engine-release-cites-the-record-that-governs-the-workflow (sufficiency, expects answered)
    session L2-additive-scenario-6-p13-r1: satisfied — event 25: answered `HW-DR-0090`; no call to a tool of an MCP server
    session L2-additive-scenario-6-p13-r2: satisfied — event 26: answered `HW-DR-0090`; no call to a tool of an MCP server
    session L2-additive-scenario-6-p13-r3: satisfied — event 27: answered `HW-DR-0090`; no call to a tool of an MCP server
    session L2-additive-scenario-6-p13-r4: satisfied — event 28: answered `HW-DR-0090`; no call to a tool of an MCP server
    session L2-additive-scenario-6-p13-r5: satisfied — event 29: answered `HW-DR-0090`; no call to a tool of an MCP server
    session L2-additive-scenario-6-p13-r6: satisfied — event 30: answered `HW-DR-0090`; no call to a tool of an MCP server
- HW-PROBE-a-session-that-reviews-an-agent-s-acceptance-stamp-on-a-branch-that-needs-no-approval-cites-the-ruling-in-force-and-not-the-draft-that-disagrees-with-it (sufficiency, expects answered)
    session L2-additive-scenario-6-p14-r1: satisfied — event 31: answered `HW-DR-0034`; 4 calls to a tool of an MCP server
    session L2-additive-scenario-6-p14-r2: satisfied — event 32: answered `HW-DR-0034`; 5 calls to a tool of an MCP server
    session L2-additive-scenario-6-p14-r3: satisfied — event 33: answered `HW-DR-0034`; no call to a tool of an MCP server
    session L2-additive-scenario-6-p14-r4: satisfied — event 34: answered `HW-DR-0034`; 3 calls to a tool of an MCP server
    session L2-additive-scenario-6-p14-r5: not satisfied — event 35 answered `HW-OBL-0167`, and the probe expects `HW-DR-0034`; 4 calls to a tool of an MCP server
    session L2-additive-scenario-6-p14-r6: satisfied — event 36: answered `HW-DR-0034`; 1 call to a tool of an MCP server

47 of 84 sessions that recorded their calls made no call to a tool of an MCP server. 0 sessions recorded no calls.

## The rate, and the denominator it is over

68 of 84 graded sessions satisfied their expectation: 81.0%, in a 95% interval of 71.3% to 87.9%.
The denominator is the graded sessions and never the selected probes. 0 sessions reached no verdict, and a session with no verdict is outside both halves of that fraction.

An interval that overlaps the previous run's is variance and one that does not is drift. This is one arm, so it estimates no effect: an efficacy claim is a comparison of two results, and the arm each one recorded is on it.
