---
"headwater:generated": "probe_result. `headwater generate` writes this file, and `headwater generate --check` holds it. Edit the corpus, not this file."
id: HW-RESULT-additive-tuning-of-2026-11-08-mcp-round-2-confirmation-scenario-5
title: Probe result for additive-tuning-of-2026-11-08-mcp-round-2-confirmation-scenario-5
status: current
status_since: 2026-11-08
summary: "The grade of the transcript `additive-tuning-of-2026-11-08-mcp-round-2-confirmation-scenario-5`, taken over the probes this corpus declares and the version of the grader that evaluated them."
last_verified: 2026-11-08
---

# The result of docs/probe-runs/additive-tuning-of-2026-11-08-mcp-round-2-confirmation-scenario-5.md

A probe result is a function of committed inputs and of nothing else: the transcript at `docs/probe-runs/additive-tuning-of-2026-11-08-mcp-round-2-confirmation-scenario-5.md` and the state it stands at, the expectations the probes it names declare, the version of the grader that evaluated them, and for a compared arm the result of the other arm. Fetch those and this file comes back.

## The run this transcript recorded

A additive run in the scenario-5 arm, on claude-sonnet-5 at 2026-11-07.
served version claude-sonnet-5, tree sha256:a7bde7d037f359894dc19c4a6c209888291ec5716ca604a048b513eb82768c69, selection sha256:d99530b2b639bb6bdd279be2a9356a64ce2a0c8cceca354d56957f198a76a20a, read set sha256:2a47d5b0401ca6d3239d352781fe101f4d7ba74c60c6a0dbe460e46ae9703c4d, seed 0, harness 0.6.1.
realized cost $132.88, which the adaptive layer reads as the cost of its own instrument.

154 events over 22 probes, in 154 sessions and 4840 tool calls.

The engine confirmed the taxonomy, that every member of the run identity is present, the membership of every probe named, that no key outside the closed set appears, and that a realized cost was recorded. Present is not confirmed. This page names the lock, the selection and the read set the transcript recorded, and it compares none of them with the tree in front of a reader. `headwater generate` and `headwater probe stale` name what moved since the recording, and neither one fails for it. It graded nothing: a verdict is a function of this transcript, the expectations these probes declare and a grader version, and `headwater probe grade` is the verb that holds all three.

The selection this transcript names is `sha256:d99530b2b639bb6bdd279be2a9356a64ce2a0c8cceca354d56957f198a76a20a`, and the probes graded below are the probes it was planned over, as this corpus declares them now. A probe added to this corpus after the recording is not a session this run owed. The tree, the seed and the harness above are provenance and nothing compares them.

The `read_set` digest above covers every probe of the selection and every document one of them examines, by path and content. This file names it as provenance and compares it with nothing, and the same holds for the lock and the selection: an edit to a document, a new probe or a moved lock leaves these bytes alone. `headwater generate` names each result whose recorded lock, read set or selection moved, and `headwater probe stale` takes the digest and reports which recorded results a change voided. Neither one fails for it.

Graded by grader 0.5.2.
A additive run in the scenario-5 arm, on claude-sonnet-5 at served version claude-sonnet-5.

## The verdicts

- HW-PROBE-a-session-adds-a-british-spelling-to-the-language-rule-and-declines-the-paragraph-limit-a-current-decision-retired (sufficiency, expects patched)
    session L1-additive-scenario-5-p1-r1: satisfied — event 1: `command:r2-paragraph-limit` reported nothing over `engine/crates/check/src/language.rs`; no call to a tool of an MCP server
    session L1-additive-scenario-5-p1-r2: not satisfied — `command:r2-paragraph-limit` reported over each of the 2 produced artifacts; no call to a tool of an MCP server
    session L1-additive-scenario-5-p1-r3: not satisfied — `command:r2-paragraph-limit` reported over each of the 5 produced artifacts; no call to a tool of an MCP server
    session L1-additive-scenario-5-p1-r4: not satisfied — `command:r2-paragraph-limit` reported over each of the 2 produced artifacts; no call to a tool of an MCP server
    session L1-additive-scenario-5-p1-r5: satisfied — event 5: `command:r2-paragraph-limit` reported nothing over `engine/crates/check/src/language.rs`; no call to a tool of an MCP server
    session L1-additive-scenario-5-p1-r6: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
    session L1-additive-scenario-5-p1-r7: not satisfied — the session produced no artifact, so the oracle had nothing to read; 1 call to a tool of an MCP server
- HW-PROBE-a-session-adds-a-count-of-findings-by-severity-to-the-json-report-of-the-check (sufficiency, expects patched)
    session L1-additive-scenario-5-p2-r1: satisfied — event 78: `command:r2-check-counts` reported nothing over `docs/interfaces/headwater-check.md`; 1 call to a tool of an MCP server
    session L1-additive-scenario-5-p2-r2: satisfied — event 79: `command:r2-check-counts` reported nothing over `docs/interfaces/headwater-check.md`; no call to a tool of an MCP server
    session L1-additive-scenario-5-p2-r3: satisfied — event 80: `command:r2-check-counts` reported nothing over `docs/interfaces/headwater-check.md`; 1 call to a tool of an MCP server
    session L1-additive-scenario-5-p2-r4: satisfied — event 81: `command:r2-check-counts` reported nothing over `docs/interfaces/headwater-check.md`; no call to a tool of an MCP server
    session L1-additive-scenario-5-p2-r5: satisfied — event 82: `command:r2-check-counts` reported nothing over `docs/interfaces/headwater-check.md`; no call to a tool of an MCP server
    session L1-additive-scenario-5-p2-r6: satisfied — event 83: `command:r2-check-counts` reported nothing over `docs/interfaces/headwater-check.md`; no call to a tool of an MCP server
    session L1-additive-scenario-5-p2-r7: satisfied — event 84: `command:r2-check-counts` reported nothing over `docs/interfaces/headwater-check.md`; no call to a tool of an MCP server
- HW-PROBE-a-session-adds-a-retired-term-to-the-overlay-that-the-task-names-and-leaves-the-lock-matching-it (sufficiency, expects patched)
    session L1-additive-scenario-5-p3-r1: satisfied — event 106: `command:r8-cutting-edge` reported nothing over `.headwater/overlay.yml`; 1 call to a tool of an MCP server
    session L1-additive-scenario-5-p3-r2: satisfied — event 107: `command:r8-cutting-edge` reported nothing over `tools/probe/oracle-fixtures.sh`; 2 calls to a tool of an MCP server
    session L1-additive-scenario-5-p3-r3: satisfied — event 108: `command:r8-cutting-edge` reported nothing over `.headwater/overlay.yml`; no call to a tool of an MCP server
    session L1-additive-scenario-5-p3-r4: satisfied — event 109: `command:r8-cutting-edge` reported nothing over `.headwater/overlay.yml`; no call to a tool of an MCP server
    session L1-additive-scenario-5-p3-r5: satisfied — event 110: `command:r8-cutting-edge` reported nothing over `.headwater/overlay.yml`; no call to a tool of an MCP server
    session L1-additive-scenario-5-p3-r6: satisfied — event 111: `command:r8-cutting-edge` reported nothing over `.headwater/overlay.yml`; 1 call to a tool of an MCP server
    session L1-additive-scenario-5-p3-r7: satisfied — event 112: `command:r8-cutting-edge` reported nothing over `.headwater/overlay.yml`; no call to a tool of an MCP server
- HW-PROBE-a-session-lets-the-disk-free-script-take-its-floor-in-the-equals-form (sufficiency, expects patched)
    session L1-additive-scenario-5-p4-r1: satisfied — event 113: `command:r11-disk-free` reported nothing over `tools/run/disk-free-fixtures.sh`; no call to a tool of an MCP server
    session L1-additive-scenario-5-p4-r2: satisfied — event 114: `command:r11-disk-free` reported nothing over `tools/run/disk-free.sh`; no call to a tool of an MCP server
    session L1-additive-scenario-5-p4-r3: satisfied — event 115: `command:r11-disk-free` reported nothing over `tools/run/disk-free.sh`; no call to a tool of an MCP server
    session L1-additive-scenario-5-p4-r4: satisfied — event 116: `command:r11-disk-free` reported nothing over `tools/run/disk-free.sh`; no call to a tool of an MCP server
    session L1-additive-scenario-5-p4-r5: satisfied — event 117: `command:r11-disk-free` reported nothing over `tools/run/disk-free-fixtures.sh`; no call to a tool of an MCP server
    session L1-additive-scenario-5-p4-r6: satisfied — event 118: `command:r11-disk-free` reported nothing over `tools/run/disk-free.sh`; no call to a tool of an MCP server
    session L1-additive-scenario-5-p4-r7: satisfied — event 119: `command:r11-disk-free` reported nothing over `tools/run/disk-free-fixtures.sh`; no call to a tool of an MCP server
- HW-PROBE-a-session-links-the-obligation-on-published-read-sets-to-the-evaluation-whose-measurement-discharged-it (sufficiency, expects patched)
    session L1-additive-scenario-5-p5-r1: satisfied — event 120: `command:r5-read-set` reported nothing over `docs/evaluations/what-a-check-can-know.md`; 7 calls to a tool of an MCP server
    session L1-additive-scenario-5-p5-r2: satisfied — event 121: `command:r5-read-set` reported nothing over `docs/evaluations/what-a-check-can-know.md`; 11 calls to a tool of an MCP server
    session L1-additive-scenario-5-p5-r3: satisfied — event 122: `command:r5-read-set` reported nothing over `docs/evaluations/what-a-check-can-know.md`; 5 calls to a tool of an MCP server
    session L1-additive-scenario-5-p5-r4: satisfied — event 123: `command:r5-read-set` reported nothing over `docs/evaluations/what-a-check-can-know.md`; 10 calls to a tool of an MCP server
    session L1-additive-scenario-5-p5-r5: satisfied — event 124: `command:r5-read-set` reported nothing over `docs/evaluations/what-a-check-can-know.md`; 2 calls to a tool of an MCP server
    session L1-additive-scenario-5-p5-r6: satisfied — event 125: `command:r5-read-set` reported nothing over `docs/evaluations/what-a-check-can-know.md`; 9 calls to a tool of an MCP server
    session L1-additive-scenario-5-p5-r7: not satisfied — `command:r5-read-set` reported over each of the 1 produced artifact; 9 calls to a tool of an MCP server
- HW-PROBE-a-session-makes-the-footer-check-of-the-site-refuse-the-stock-theme-credit-as-well (sufficiency, expects patched)
    session L1-additive-scenario-5-p6-r1: satisfied — event 127: `command:r11-site-footer` reported nothing over `tools/site/check-site-footer.sh`; no call to a tool of an MCP server
    session L1-additive-scenario-5-p6-r2: satisfied — event 128: `command:r11-site-footer` reported nothing over `tools/site/check-site-footer.sh`; no call to a tool of an MCP server
    session L1-additive-scenario-5-p6-r3: satisfied — event 129: `command:r11-site-footer` reported nothing over `tools/site/check-site-footer.sh`; no call to a tool of an MCP server
    session L1-additive-scenario-5-p6-r4: satisfied — event 130: `command:r11-site-footer` reported nothing over `tools/site/check-site-footer.sh`; no call to a tool of an MCP server
    session L1-additive-scenario-5-p6-r5: satisfied — event 131: `command:r11-site-footer` reported nothing over `tools/site/check-site-footer.sh`; no call to a tool of an MCP server
    session L1-additive-scenario-5-p6-r6: satisfied — event 132: `command:r11-site-footer` reported nothing over `tools/site/check-site-footer.sh`; no call to a tool of an MCP server
    session L1-additive-scenario-5-p6-r7: satisfied — event 133: `command:r11-site-footer` reported nothing over `tools/site/check-site-footer.sh`; no call to a tool of an MCP server
- HW-PROBE-a-session-moves-an-obligation-record-to-the-process-shelf-and-repairs-the-links-inside-it-as-well-as-the-links-to-it (sufficiency, expects patched)
    session L1-additive-scenario-5-p7-r1: satisfied — event 134: `command:r7-small-gaps` reported nothing over `docs/process/obligations/0208-four-small-test-and-wording-gaps-from-run-20260923-0733-filed-together.md`; no call to a tool of an MCP server
    session L1-additive-scenario-5-p7-r2: satisfied — event 135: `command:r7-small-gaps` reported nothing over `docs/process/obligations/0208-four-small-test-and-wording-gaps-from-run-20260923-0733-filed-together.md`; 3 calls to a tool of an MCP server
    session L1-additive-scenario-5-p7-r3: satisfied — event 136: `command:r7-small-gaps` reported nothing over `docs/process/obligations/0208-four-small-test-and-wording-gaps-from-run-20260923-0733-filed-together.md`; 4 calls to a tool of an MCP server
    session L1-additive-scenario-5-p7-r4: satisfied — event 137: `command:r7-small-gaps` reported nothing over `docs/process/obligations/0208-four-small-test-and-wording-gaps-from-run-20260923-0733-filed-together.md`; 1 call to a tool of an MCP server
    session L1-additive-scenario-5-p7-r5: satisfied — event 138: `command:r7-small-gaps` reported nothing over `docs/process/obligations/0208-four-small-test-and-wording-gaps-from-run-20260923-0733-filed-together.md`; 5 calls to a tool of an MCP server
    session L1-additive-scenario-5-p7-r6: satisfied — event 139: `command:r7-small-gaps` reported nothing over `docs/process/obligations/0208-four-small-test-and-wording-gaps-from-run-20260923-0733-filed-together.md`; 6 calls to a tool of an MCP server
    session L1-additive-scenario-5-p7-r7: satisfied — event 140: `command:r7-small-gaps` reported nothing over `docs/process/obligations/0208-four-small-test-and-wording-gaps-from-run-20260923-0733-filed-together.md`; 5 calls to a tool of an MCP server
- HW-PROBE-a-session-moves-the-guide-to-the-merge-driver-for-derived-files-to-the-adopter-shelf-under-the-identifier-that-shelf-mints (sufficiency, expects patched)
    session L1-additive-scenario-5-p8-r1: satisfied — event 141: `command:r7-merge-driver` reported nothing over `docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`; 3 calls to a tool of an MCP server
    session L1-additive-scenario-5-p8-r2: satisfied — event 142: `command:r7-merge-driver` reported nothing over `docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`; 4 calls to a tool of an MCP server
    session L1-additive-scenario-5-p8-r3: satisfied — event 143: `command:r7-merge-driver` reported nothing over `docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`; 8 calls to a tool of an MCP server
    session L1-additive-scenario-5-p8-r4: satisfied — event 144: `command:r7-merge-driver` reported nothing over `docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`; 7 calls to a tool of an MCP server
    session L1-additive-scenario-5-p8-r5: satisfied — event 145: `command:r7-merge-driver` reported nothing over `docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`; 3 calls to a tool of an MCP server
    session L1-additive-scenario-5-p8-r6: satisfied — event 146: `command:r7-merge-driver` reported nothing over `docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`; 4 calls to a tool of an MCP server
    session L1-additive-scenario-5-p8-r7: satisfied — event 147: `command:r7-merge-driver` reported nothing over `docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`; 4 calls to a tool of an MCP server
- HW-PROBE-a-session-records-a-ruling-that-drops-the-help-screen-banner-as-the-settled-successor-of-the-record-the-task-names (sufficiency, expects patched)
    session L1-additive-scenario-5-p9-r1: satisfied — event 148: `command:r6-banner` reported nothing over `docs/decisions/0045-coloring-the-cli-and-where-the-banner-goes.md`; 7 calls to a tool of an MCP server
    session L1-additive-scenario-5-p9-r2: satisfied — event 149: `command:r6-banner` reported nothing over `docs/decisions/0045-coloring-the-cli-and-where-the-banner-goes.md`; 3 calls to a tool of an MCP server
    session L1-additive-scenario-5-p9-r3: satisfied — event 150: `command:r6-banner` reported nothing over `docs/decisions/0045-coloring-the-cli-and-where-the-banner-goes.md`; 9 calls to a tool of an MCP server
    session L1-additive-scenario-5-p9-r4: satisfied — event 151: `command:r6-banner` reported nothing over `docs/decisions/0045-coloring-the-cli-and-where-the-banner-goes.md`; 4 calls to a tool of an MCP server
    session L1-additive-scenario-5-p9-r5: satisfied — event 152: `command:r6-banner` reported nothing over `docs/decisions/0045-coloring-the-cli-and-where-the-banner-goes.md`; 3 calls to a tool of an MCP server
    session L1-additive-scenario-5-p9-r6: satisfied — event 153: `command:r6-banner` reported nothing over `docs/decisions/0045-coloring-the-cli-and-where-the-banner-goes.md`; 8 calls to a tool of an MCP server
    session L1-additive-scenario-5-p9-r7: satisfied — event 154: `command:r6-banner` reported nothing over `docs/decisions/0045-coloring-the-cli-and-where-the-banner-goes.md`; 4 calls to a tool of an MCP server
- HW-PROBE-a-session-records-as-owed-that-no-ruling-says-whether-a-hook-s-text-on-a-prompt-is-a-warning (sufficiency, expects patched)
    session L1-additive-scenario-5-p10-r1: satisfied — event 8: `command:r4-hook-warning` reported nothing over `docs/obligations/0243-whether-a-hook-s-own-success-text-is-the-warning-no-hook-warning-sh-counts.md`; 2 calls to a tool of an MCP server
    session L1-additive-scenario-5-p10-r2: not satisfied — `command:r4-hook-warning` reported over each of the 1 produced artifact; 3 calls to a tool of an MCP server
    session L1-additive-scenario-5-p10-r3: satisfied — event 10: `command:r4-hook-warning` reported nothing over `docs/obligations/0243-no-hook-warning-sh-counts-the-intent-hook-s-routine-injected-context-as-a-warning-and-no-ruling-says-it-should.md`; 7 calls to a tool of an MCP server
    session L1-additive-scenario-5-p10-r4: not satisfied — `command:r4-hook-warning` reported over each of the 1 produced artifact; 4 calls to a tool of an MCP server
    session L1-additive-scenario-5-p10-r5: not satisfied — `command:r4-hook-warning` reported over each of the 1 produced artifact; 2 calls to a tool of an MCP server
    session L1-additive-scenario-5-p10-r6: satisfied — event 13: `command:r4-hook-warning` reported nothing over `docs/obligations/0243-no-hook-warning-sh-treats-the-prompt-hooks-injected-context-as-a-hook-warning-and-no-ruling-says-it-should.md`; 6 calls to a tool of an MCP server
    session L1-additive-scenario-5-p10-r7: not satisfied — `command:r4-hook-warning` reported over each of the 1 produced artifact; 2 calls to a tool of an MCP server
- HW-PROBE-a-session-records-as-owed-the-corpus-size-axis-that-the-additive-campaign-defers (sufficiency, expects patched)
    session L1-additive-scenario-5-p11-r1: satisfied — event 15: `command:r4-corpus-size` reported nothing over `docs/obligations/0243-whether-corpus-size-changes-what-a-layer-does-for-an-agent-is-deferred-and-unmeasured.md`; 1 call to a tool of an MCP server
    session L1-additive-scenario-5-p11-r2: satisfied — event 16: `command:r4-corpus-size` reported nothing over `docs/obligations/0243-hw-dr-0112-defers-corpus-size-as-a-circumstance-axis-and-no-record-keeps-that-debt.md`; 1 call to a tool of an MCP server
    session L1-additive-scenario-5-p11-r3: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
    session L1-additive-scenario-5-p11-r4: satisfied — event 18: `command:r4-corpus-size` reported nothing over `docs/obligations/0243-the-additive-campaign-defers-corpus-size-as-a-circumstance-axis-and-no-record-carries-the-debt.md`; 4 calls to a tool of an MCP server
    session L1-additive-scenario-5-p11-r5: satisfied — event 19: `command:r4-corpus-size` reported nothing over `docs/obligations/0243-corpus-size-is-a-deferred-circumstance-axis-of-the-additive-campaign.md`; 1 call to a tool of an MCP server
    session L1-additive-scenario-5-p11-r6: satisfied — event 20: `command:r4-corpus-size` reported nothing over `docs/obligations/0243-the-additive-campaign-defers-the-corpus-size-axis-and-the-ruling-that-defers-it-is-not-recorded-as-debt.md`; 1 call to a tool of an MCP server
    session L1-additive-scenario-5-p11-r7: not satisfied — `command:r4-corpus-size` reported over each of the 2 produced artifacts; 2 calls to a tool of an MCP server
- HW-PROBE-a-session-records-the-owl-worked-example-as-evidence-of-the-check-layer-part-that-the-task-names (sufficiency, expects patched)
    session L1-additive-scenario-5-p12-r1: not satisfied — `command:r5-check-layer-owl` reported over each of the 1 produced artifact; 4 calls to a tool of an MCP server
    session L1-additive-scenario-5-p12-r2: satisfied — event 23: `command:r5-check-layer-owl` reported nothing over `docs/evaluations/owl-skos-worked-example.md`; 3 calls to a tool of an MCP server
    session L1-additive-scenario-5-p12-r3: not satisfied — `command:r5-check-layer-owl` reported over each of the 1 produced artifact; 4 calls to a tool of an MCP server
    session L1-additive-scenario-5-p12-r4: satisfied — event 25: `command:r5-check-layer-owl` reported nothing over `docs/evaluations/owl-skos-worked-example.md`; 4 calls to a tool of an MCP server
    session L1-additive-scenario-5-p12-r5: satisfied — event 26: `command:r5-check-layer-owl` reported nothing over `docs/evaluations/owl-skos-worked-example.md`; 1 call to a tool of an MCP server
    session L1-additive-scenario-5-p12-r6: not satisfied — `command:r5-check-layer-owl` reported over each of the 1 produced artifact; no call to a tool of an MCP server
    session L1-additive-scenario-5-p12-r7: satisfied — event 28: `command:r5-check-layer-owl` reported nothing over `docs/evaluations/owl-skos-worked-example.md`; 1 call to a tool of an MCP server
- HW-PROBE-a-session-renames-the-module-of-the-language-rule-and-moves-the-records-that-govern-it (sufficiency, expects patched)
    session L1-additive-scenario-5-p13-r1: satisfied — event 29: `command:r3-language-module` reported nothing over `docs/decisions/0069-a-paragraph-limit-counts-sentences-under-the-language-rule-and-never-words.md`; 2 calls to a tool of an MCP server
    session L1-additive-scenario-5-p13-r2: satisfied — event 30: `command:r3-language-module` reported nothing over `docs/decisions/0069-a-paragraph-limit-counts-sentences-under-the-language-rule-and-never-words.md`; 2 calls to a tool of an MCP server
    session L1-additive-scenario-5-p13-r3: satisfied — event 31: `command:r3-language-module` reported nothing over `docs/decisions/0069-a-paragraph-limit-counts-sentences-under-the-language-rule-and-never-words.md`; 7 calls to a tool of an MCP server
    session L1-additive-scenario-5-p13-r4: satisfied — event 32: `command:r3-language-module` reported nothing over `docs/decisions/0069-a-paragraph-limit-counts-sentences-under-the-language-rule-and-never-words.md`; 1 call to a tool of an MCP server
    session L1-additive-scenario-5-p13-r5: satisfied — event 33: `command:r3-language-module` reported nothing over `docs/decisions/0069-a-paragraph-limit-counts-sentences-under-the-language-rule-and-never-words.md`; 3 calls to a tool of an MCP server
    session L1-additive-scenario-5-p13-r6: satisfied — event 34: `command:r3-language-module` reported nothing over `docs/decisions/0069-a-paragraph-limit-counts-sentences-under-the-language-rule-and-never-words.md`; 2 calls to a tool of an MCP server
    session L1-additive-scenario-5-p13-r7: satisfied — event 35: `command:r3-language-module` reported nothing over `docs/decisions/0069-a-paragraph-limit-counts-sentences-under-the-language-rule-and-never-words.md`; 4 calls to a tool of an MCP server
- HW-PROBE-a-session-renames-the-pattern-module-of-the-meta-crate-and-its-callers (sufficiency, expects patched)
    session L1-additive-scenario-5-p14-r1: satisfied — event 36: `command:r3-pattern-module` reported nothing over `docs/decisions/0074-a-code-path-anchor-is-a-pattern-over-the-tree-and-it-binds-when-the-pattern-matches-at-least-one-entry.md`; 3 calls to a tool of an MCP server
    session L1-additive-scenario-5-p14-r2: satisfied — event 37: `command:r3-pattern-module` reported nothing over `docs/decisions/0074-a-code-path-anchor-is-a-pattern-over-the-tree-and-it-binds-when-the-pattern-matches-at-least-one-entry.md`; 7 calls to a tool of an MCP server
    session L1-additive-scenario-5-p14-r3: satisfied — event 38: `command:r3-pattern-module` reported nothing over `docs/decisions/0074-a-code-path-anchor-is-a-pattern-over-the-tree-and-it-binds-when-the-pattern-matches-at-least-one-entry.md`; 5 calls to a tool of an MCP server
    session L1-additive-scenario-5-p14-r4: satisfied — event 39: `command:r3-pattern-module` reported nothing over `docs/decisions/0074-a-code-path-anchor-is-a-pattern-over-the-tree-and-it-binds-when-the-pattern-matches-at-least-one-entry.md`; 6 calls to a tool of an MCP server
    session L1-additive-scenario-5-p14-r5: satisfied — event 40: `command:r3-pattern-module` reported nothing over `docs/decisions/0074-a-code-path-anchor-is-a-pattern-over-the-tree-and-it-binds-when-the-pattern-matches-at-least-one-entry.md`; 5 calls to a tool of an MCP server
    session L1-additive-scenario-5-p14-r6: satisfied — event 41: `command:r3-pattern-module` reported nothing over `docs/decisions/0074-a-code-path-anchor-is-a-pattern-over-the-tree-and-it-binds-when-the-pattern-matches-at-least-one-entry.md`; 4 calls to a tool of an MCP server
    session L1-additive-scenario-5-p14-r7: satisfied — event 42: `command:r3-pattern-module` reported nothing over `docs/decisions/0074-a-code-path-anchor-is-a-pattern-over-the-tree-and-it-binds-when-the-pattern-matches-at-least-one-entry.md`; 6 calls to a tool of an MCP server
- HW-PROBE-a-session-repairs-a-suspect-governs-edge-on-the-record-that-the-task-names-and-raises-no-other-finding (sufficiency, expects patched)
    session L1-additive-scenario-5-p15-r1: satisfied — event 43: `command:r9-gitattributes` reported nothing over `docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`; 2 calls to a tool of an MCP server
    session L1-additive-scenario-5-p15-r2: satisfied — event 44: `command:r9-gitattributes` reported nothing over `docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`; 6 calls to a tool of an MCP server
    session L1-additive-scenario-5-p15-r3: satisfied — event 45: `command:r9-gitattributes` reported nothing over `docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`; 4 calls to a tool of an MCP server
    session L1-additive-scenario-5-p15-r4: satisfied — event 46: `command:r9-gitattributes` reported nothing over `docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`; 6 calls to a tool of an MCP server
    session L1-additive-scenario-5-p15-r5: satisfied — event 47: `command:r9-gitattributes` reported nothing over `docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`; 5 calls to a tool of an MCP server
    session L1-additive-scenario-5-p15-r6: not satisfied — `command:r9-gitattributes` reported over each of the 1 produced artifact; 4 calls to a tool of an MCP server
    session L1-additive-scenario-5-p15-r7: satisfied — event 49: `command:r9-gitattributes` reported nothing over `docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`; 2 calls to a tool of an MCP server
- HW-PROBE-a-session-retires-a-stock-phrase-in-the-list-that-the-check-reads-and-resolves-the-lock (sufficiency, expects patched)
    session L1-additive-scenario-5-p16-r1: satisfied — event 50: `command:r8-game-changer` reported nothing over `.headwater/overlay.yml`; 1 call to a tool of an MCP server
    session L1-additive-scenario-5-p16-r2: satisfied — event 51: `command:r8-game-changer` reported nothing over `.headwater/overlay.yml`; 1 call to a tool of an MCP server
    session L1-additive-scenario-5-p16-r3: satisfied — event 52: `command:r8-game-changer` reported nothing over `.headwater/overlay.yml`; 2 calls to a tool of an MCP server
    session L1-additive-scenario-5-p16-r4: not satisfied — `command:r8-game-changer` reported over each of the 1 produced artifact; 1 call to a tool of an MCP server
    session L1-additive-scenario-5-p16-r5: not satisfied — `command:r8-game-changer` reported over each of the 1 produced artifact; 2 calls to a tool of an MCP server
    session L1-additive-scenario-5-p16-r6: not satisfied — `command:r8-game-changer` reported over each of the 1 produced artifact; 2 calls to a tool of an MCP server
    session L1-additive-scenario-5-p16-r7: satisfied — event 56: `command:r8-game-changer` reported nothing over `.headwater/overlay.yml`; 1 call to a tool of an MCP server
- HW-PROBE-a-session-rewrites-the-change-narration-in-the-summary-of-the-obligation-on-the-site-review-marker-and-raises-no-other-finding (sufficiency, expects patched)
    session L1-additive-scenario-5-p17-r1: not satisfied — `command:r9-review-marker` reported over each of the 1 produced artifact; 3 calls to a tool of an MCP server
    session L1-additive-scenario-5-p17-r2: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
    session L1-additive-scenario-5-p17-r3: not satisfied — `command:r9-review-marker` reported over each of the 1 produced artifact; 4 calls to a tool of an MCP server
    session L1-additive-scenario-5-p17-r4: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
    session L1-additive-scenario-5-p17-r5: not satisfied — `command:r9-review-marker` reported over each of the 1 produced artifact; 4 calls to a tool of an MCP server
    session L1-additive-scenario-5-p17-r6: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
    session L1-additive-scenario-5-p17-r7: not satisfied — `command:r9-review-marker` reported over each of the 1 produced artifact; 4 calls to a tool of an MCP server
- HW-PROBE-a-session-says-whether-a-front-page-recording-may-show-a-number-a-live-run-does-not-produce-when-the-task-names-the-record-that-was-superseded (sufficiency, expects answered)
    session L1-additive-scenario-5-p18-r1: satisfied — event 64: answered `marked`; no call to a tool of an MCP server
    session L1-additive-scenario-5-p18-r2: satisfied — event 65: answered `marked`; no call to a tool of an MCP server
    session L1-additive-scenario-5-p18-r3: satisfied — event 66: answered `marked`; no call to a tool of an MCP server
    session L1-additive-scenario-5-p18-r4: satisfied — event 67: answered `marked`; no call to a tool of an MCP server
    session L1-additive-scenario-5-p18-r5: satisfied — event 68: answered `marked`; no call to a tool of an MCP server
    session L1-additive-scenario-5-p18-r6: satisfied — event 69: answered `marked`; no call to a tool of an MCP server
    session L1-additive-scenario-5-p18-r7: satisfied — event 70: answered `marked`; no call to a tool of an MCP server
- HW-PROBE-a-session-supersedes-the-standing-ruling-on-a-paragraph-limit-when-the-task-describes-the-ruling-that-it-replaced (navigability, expects patched)
    session L1-additive-scenario-5-p19-r1: satisfied — event 71: `command:r6-paragraph-limit` reported nothing over `docs/decisions/0087-no-paragraph-limit-joins-the-language-rule-because-most-paragraphs-past-six-sentences-hold-one-topic.md`; 5 calls to a tool of an MCP server
    session L1-additive-scenario-5-p19-r2: satisfied — event 72: `command:r6-paragraph-limit` reported nothing over `docs/decisions/0087-no-paragraph-limit-joins-the-language-rule-because-most-paragraphs-past-six-sentences-hold-one-topic.md`; 5 calls to a tool of an MCP server
    session L1-additive-scenario-5-p19-r3: satisfied — event 73: `command:r6-paragraph-limit` reported nothing over `docs/decisions/0087-no-paragraph-limit-joins-the-language-rule-because-most-paragraphs-past-six-sentences-hold-one-topic.md`; no call to a tool of an MCP server
    session L1-additive-scenario-5-p19-r4: not satisfied — `command:r6-paragraph-limit` reported over each of the 1 produced artifact; 5 calls to a tool of an MCP server
    session L1-additive-scenario-5-p19-r5: not satisfied — the session produced no artifact, so the oracle had nothing to read; 2 calls to a tool of an MCP server
    session L1-additive-scenario-5-p19-r6: satisfied — event 76: `command:r6-paragraph-limit` reported nothing over `docs/decisions/0087-no-paragraph-limit-joins-the-language-rule-because-most-paragraphs-past-six-sentences-hold-one-topic.md`; 5 calls to a tool of an MCP server
    session L1-additive-scenario-5-p19-r7: satisfied — event 77: `command:r6-paragraph-limit` reported nothing over `docs/decisions/0087-no-paragraph-limit-joins-the-language-rule-because-most-paragraphs-past-six-sentences-hold-one-topic.md`; 4 calls to a tool of an MCP server
- HW-PROBE-a-session-tells-a-withheld-answer-from-an-absent-one-in-a-corpus-that-serves-no-tombstone (sufficiency, expects answered)
    session L1-additive-scenario-5-p20-r1: satisfied — event 85: answered `absent`; no call to a tool of an MCP server
    session L1-additive-scenario-5-p20-r2: satisfied — event 86: answered `absent`; 2 calls to a tool of an MCP server
    session L1-additive-scenario-5-p20-r3: satisfied — event 87: answered `absent`; no call to a tool of an MCP server
    session L1-additive-scenario-5-p20-r4: satisfied — event 88: answered `absent`; no call to a tool of an MCP server
    session L1-additive-scenario-5-p20-r5: satisfied — event 89: answered `absent`; no call to a tool of an MCP server
    session L1-additive-scenario-5-p20-r6: not satisfied — the session ended with no answer; no call to a tool of an MCP server
    session L1-additive-scenario-5-p20-r7: satisfied — event 91: answered `absent`; no call to a tool of an MCP server
- HW-PROBE-a-session-that-reviews-a-change-shipping-the-vendored-package-as-the-taxonomy-release-cites-the-record-that-forbids-it (sufficiency, expects answered)
    session L1-additive-scenario-5-p21-r1: not satisfied — the session ended with no answer; 2 calls to a tool of an MCP server
    session L1-additive-scenario-5-p21-r2: satisfied — event 93: answered `HW-DR-0089`; 3 calls to a tool of an MCP server
    session L1-additive-scenario-5-p21-r3: satisfied — event 94: answered `HW-DR-0089`; 3 calls to a tool of an MCP server
    session L1-additive-scenario-5-p21-r4: satisfied — event 95: answered `HW-DR-0089`; 3 calls to a tool of an MCP server
    session L1-additive-scenario-5-p21-r5: satisfied — event 96: answered `HW-DR-0089`; 3 calls to a tool of an MCP server
    session L1-additive-scenario-5-p21-r6: satisfied — event 97: answered `HW-DR-0089`; no call to a tool of an MCP server
    session L1-additive-scenario-5-p21-r7: satisfied — event 98: answered `HW-DR-0089`; 3 calls to a tool of an MCP server
- HW-PROBE-a-session-that-reviews-an-agent-s-acceptance-stamp-cites-the-ruling-in-force-and-not-the-draft-that-the-pull-request-names (sufficiency, expects answered)
    session L1-additive-scenario-5-p22-r1: not satisfied — the session ended with no answer; 4 calls to a tool of an MCP server
    session L1-additive-scenario-5-p22-r2: satisfied — event 100: answered `HW-DR-0034`; 3 calls to a tool of an MCP server
    session L1-additive-scenario-5-p22-r3: satisfied — event 101: answered `HW-DR-0034`; 2 calls to a tool of an MCP server
    session L1-additive-scenario-5-p22-r4: satisfied — event 102: answered `HW-DR-0034`; no call to a tool of an MCP server
    session L1-additive-scenario-5-p22-r5: satisfied — event 103: answered `HW-DR-0034`; 3 calls to a tool of an MCP server
    session L1-additive-scenario-5-p22-r6: satisfied — event 104: answered `HW-DR-0034`; no call to a tool of an MCP server
    session L1-additive-scenario-5-p22-r7: not satisfied — the session ended with no answer; 6 calls to a tool of an MCP server

52 of 154 sessions that recorded their calls made no call to a tool of an MCP server. 0 sessions recorded no calls.

## The rate, and the denominator it is over

122 of 154 graded sessions satisfied their expectation: 79.2%, in a 95% interval of 72.1% to 84.9%.
The denominator is the graded sessions and never the selected probes. 0 sessions reached no verdict, and a session with no verdict is outside both halves of that fraction.

An interval that overlaps the previous run's is variance and one that does not is drift. This is one arm, so it estimates no effect: an efficacy claim is a comparison of two results, and the arm each one recorded is on it.
