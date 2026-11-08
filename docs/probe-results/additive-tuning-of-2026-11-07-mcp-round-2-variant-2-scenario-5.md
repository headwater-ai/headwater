---
"headwater:generated": "probe_result. `headwater generate` writes this file, and `headwater generate --check` holds it. Edit the corpus, not this file."
id: HW-RESULT-additive-tuning-of-2026-11-07-mcp-round-2-variant-2-scenario-5
title: Probe result for additive-tuning-of-2026-11-07-mcp-round-2-variant-2-scenario-5
status: current
status_since: 2026-11-07
summary: "The grade of the transcript `additive-tuning-of-2026-11-07-mcp-round-2-variant-2-scenario-5`, taken over the probes this corpus declares and the version of the grader that evaluated them."
last_verified: 2026-11-07
---

# The result of docs/probe-runs/additive-tuning-of-2026-11-07-mcp-round-2-variant-2-scenario-5.md

A probe result is a function of committed inputs and of nothing else: the transcript at `docs/probe-runs/additive-tuning-of-2026-11-07-mcp-round-2-variant-2-scenario-5.md` and the state it stands at, the expectations the probes it names declare, the version of the grader that evaluated them, and for a compared arm the result of the other arm. Fetch those and this file comes back.

## The run this transcript recorded

A additive run in the scenario-5 arm, on claude-sonnet-5 at 2026-11-07.
served version claude-sonnet-5, tree sha256:a7bde7d037f359894dc19c4a6c209888291ec5716ca604a048b513eb82768c69, selection sha256:d99530b2b639bb6bdd279be2a9356a64ce2a0c8cceca354d56957f198a76a20a, read set sha256:2a47d5b0401ca6d3239d352781fe101f4d7ba74c60c6a0dbe460e46ae9703c4d, seed 0, harness 0.6.1.
realized cost $17.27, which the adaptive layer reads as the cost of its own instrument.

22 events over 22 probes, in 22 sessions and 660 tool calls.

The engine confirmed the taxonomy, that every member of the run identity is present, the membership of every probe named, that no key outside the closed set appears, and that a realized cost was recorded. Present is not confirmed. This page names the lock, the selection and the read set the transcript recorded, and it compares none of them with the tree in front of a reader. `headwater generate` and `headwater probe stale` name what moved since the recording, and neither one fails for it. It graded nothing: a verdict is a function of this transcript, the expectations these probes declare and a grader version, and `headwater probe grade` is the verb that holds all three.

The selection this transcript names is `sha256:d99530b2b639bb6bdd279be2a9356a64ce2a0c8cceca354d56957f198a76a20a`, and the probes graded below are the probes it was planned over, as this corpus declares them now. A probe added to this corpus after the recording is not a session this run owed. The tree, the seed and the harness above are provenance and nothing compares them.

The `read_set` digest above covers every probe of the selection and every document one of them examines, by path and content. This file names it as provenance and compares it with nothing, and the same holds for the lock and the selection: an edit to a document, a new probe or a moved lock leaves these bytes alone. `headwater generate` names each result whose recorded lock, read set or selection moved, and `headwater probe stale` takes the digest and reports which recorded results a change voided. Neither one fails for it.

Graded by grader 0.5.2.
A additive run in the scenario-5 arm, on claude-sonnet-5 at served version claude-sonnet-5.

## The verdicts

- HW-PROBE-a-session-adds-a-british-spelling-to-the-language-rule-and-declines-the-paragraph-limit-a-current-decision-retired (sufficiency, expects patched)
    session L1-additive-scenario-5-p1-r1: satisfied — event 1: `command:r2-paragraph-limit` reported nothing over `engine/crates/check/src/language.rs`; no call to a tool of an MCP server
- HW-PROBE-a-session-adds-a-count-of-findings-by-severity-to-the-json-report-of-the-check (sufficiency, expects patched)
    session L1-additive-scenario-5-p2-r1: satisfied — event 12: `command:r2-check-counts` reported nothing over `docs/interfaces/headwater-check.md`; no call to a tool of an MCP server
- HW-PROBE-a-session-adds-a-retired-term-to-the-overlay-that-the-task-names-and-leaves-the-lock-matching-it (sufficiency, expects patched)
    session L1-additive-scenario-5-p3-r1: satisfied — event 16: `command:r8-cutting-edge` reported nothing over `.headwater/overlay.yml`; no call to a tool of an MCP server
- HW-PROBE-a-session-lets-the-disk-free-script-take-its-floor-in-the-equals-form (sufficiency, expects patched)
    session L1-additive-scenario-5-p4-r1: satisfied — event 17: `command:r11-disk-free` reported nothing over `tools/run/disk-free.sh`; no call to a tool of an MCP server
- HW-PROBE-a-session-links-the-obligation-on-published-read-sets-to-the-evaluation-whose-measurement-discharged-it (sufficiency, expects patched)
    session L1-additive-scenario-5-p5-r1: satisfied — event 18: `command:r5-read-set` reported nothing over `docs/evaluations/what-a-check-can-know.md`; 13 calls to a tool of an MCP server
- HW-PROBE-a-session-makes-the-footer-check-of-the-site-refuse-the-stock-theme-credit-as-well (sufficiency, expects patched)
    session L1-additive-scenario-5-p6-r1: satisfied — event 19: `command:r11-site-footer` reported nothing over `tools/site/check-site-footer.sh`; no call to a tool of an MCP server
- HW-PROBE-a-session-moves-an-obligation-record-to-the-process-shelf-and-repairs-the-links-inside-it-as-well-as-the-links-to-it (sufficiency, expects patched)
    session L1-additive-scenario-5-p7-r1: satisfied — event 20: `command:r7-small-gaps` reported nothing over `docs/process/obligations/0208-four-small-test-and-wording-gaps-from-run-20260923-0733-filed-together.md`; 4 calls to a tool of an MCP server
- HW-PROBE-a-session-moves-the-guide-to-the-merge-driver-for-derived-files-to-the-adopter-shelf-under-the-identifier-that-shelf-mints (sufficiency, expects patched)
    session L1-additive-scenario-5-p8-r1: satisfied — event 21: `command:r7-merge-driver` reported nothing over `docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`; 3 calls to a tool of an MCP server
- HW-PROBE-a-session-records-a-ruling-that-drops-the-help-screen-banner-as-the-settled-successor-of-the-record-the-task-names (sufficiency, expects patched)
    session L1-additive-scenario-5-p9-r1: satisfied — event 22: `command:r6-banner` reported nothing over `docs/decisions/0045-coloring-the-cli-and-where-the-banner-goes.md`; 1 call to a tool of an MCP server
- HW-PROBE-a-session-records-as-owed-that-no-ruling-says-whether-a-hook-s-text-on-a-prompt-is-a-warning (sufficiency, expects patched)
    session L1-additive-scenario-5-p10-r1: not satisfied — `command:r4-hook-warning` reported over each of the 2 produced artifacts; 1 call to a tool of an MCP server
- HW-PROBE-a-session-records-as-owed-the-corpus-size-axis-that-the-additive-campaign-defers (sufficiency, expects patched)
    session L1-additive-scenario-5-p11-r1: satisfied — event 3: `command:r4-corpus-size` reported nothing over `docs/obligations/0243-the-additive-campaign-defers-corpus-size-as-a-circumstance-axis-and-no-obligation-record-held-it.md`; 1 call to a tool of an MCP server
- HW-PROBE-a-session-records-the-owl-worked-example-as-evidence-of-the-check-layer-part-that-the-task-names (sufficiency, expects patched)
    session L1-additive-scenario-5-p12-r1: satisfied — event 4: `command:r5-check-layer-owl` reported nothing over `docs/evaluations/owl-skos-worked-example.md`; 6 calls to a tool of an MCP server
- HW-PROBE-a-session-renames-the-module-of-the-language-rule-and-moves-the-records-that-govern-it (sufficiency, expects patched)
    session L1-additive-scenario-5-p13-r1: not satisfied — `command:r3-language-module` reported over each of the 7 produced artifacts; 2 calls to a tool of an MCP server
- HW-PROBE-a-session-renames-the-pattern-module-of-the-meta-crate-and-its-callers (sufficiency, expects patched)
    session L1-additive-scenario-5-p14-r1: satisfied — event 6: `command:r3-pattern-module` reported nothing over `docs/decisions/0074-a-code-path-anchor-is-a-pattern-over-the-tree-and-it-binds-when-the-pattern-matches-at-least-one-entry.md`; 9 calls to a tool of an MCP server
- HW-PROBE-a-session-repairs-a-suspect-governs-edge-on-the-record-that-the-task-names-and-raises-no-other-finding (sufficiency, expects patched)
    session L1-additive-scenario-5-p15-r1: satisfied — event 7: `command:r9-gitattributes` reported nothing over `docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`; 5 calls to a tool of an MCP server
- HW-PROBE-a-session-retires-a-stock-phrase-in-the-list-that-the-check-reads-and-resolves-the-lock (sufficiency, expects patched)
    session L1-additive-scenario-5-p16-r1: satisfied — event 8: `command:r8-game-changer` reported nothing over `.headwater/overlay.yml`; no call to a tool of an MCP server
- HW-PROBE-a-session-rewrites-the-change-narration-in-the-summary-of-the-obligation-on-the-site-review-marker-and-raises-no-other-finding (sufficiency, expects patched)
    session L1-additive-scenario-5-p17-r1: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
- HW-PROBE-a-session-says-whether-a-front-page-recording-may-show-a-number-a-live-run-does-not-produce-when-the-task-names-the-record-that-was-superseded (sufficiency, expects answered)
    session L1-additive-scenario-5-p18-r1: satisfied — event 10: answered `marked`; no call to a tool of an MCP server
- HW-PROBE-a-session-supersedes-the-standing-ruling-on-a-paragraph-limit-when-the-task-describes-the-ruling-that-it-replaced (navigability, expects patched)
    session L1-additive-scenario-5-p19-r1: satisfied — event 11: `command:r6-paragraph-limit` reported nothing over `docs/decisions/0087-no-paragraph-limit-joins-the-language-rule-because-most-paragraphs-past-six-sentences-hold-one-topic.md`; 9 calls to a tool of an MCP server
- HW-PROBE-a-session-tells-a-withheld-answer-from-an-absent-one-in-a-corpus-that-serves-no-tombstone (sufficiency, expects answered)
    session L1-additive-scenario-5-p20-r1: satisfied — event 13: answered `absent`; no call to a tool of an MCP server
- HW-PROBE-a-session-that-reviews-a-change-shipping-the-vendored-package-as-the-taxonomy-release-cites-the-record-that-forbids-it (sufficiency, expects answered)
    session L1-additive-scenario-5-p21-r1: satisfied — event 14: answered `HW-DR-0089`; 3 calls to a tool of an MCP server
- HW-PROBE-a-session-that-reviews-an-agent-s-acceptance-stamp-cites-the-ruling-in-force-and-not-the-draft-that-the-pull-request-names (sufficiency, expects answered)
    session L1-additive-scenario-5-p22-r1: not satisfied — the session ended with no answer; no call to a tool of an MCP server

10 of 22 sessions that recorded their calls made no call to a tool of an MCP server. 0 sessions recorded no calls.

## The rate, and the denominator it is over

18 of 22 graded sessions satisfied their expectation: 81.8%, in a 95% interval of 61.5% to 92.7%.
The denominator is the graded sessions and never the selected probes. 0 sessions reached no verdict, and a session with no verdict is outside both halves of that fraction.

An interval that overlaps the previous run's is variance and one that does not is drift. This is one arm, so it estimates no effect: an efficacy claim is a comparison of two results, and the arm each one recorded is on it.
