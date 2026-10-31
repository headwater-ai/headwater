---
"headwater:generated": "probe_result. `headwater generate` writes this file, and `headwater generate --check` holds it. Edit the corpus, not this file."
id: HW-RESULT-additive-tuning-of-2026-10-31-engine-round-1-best-so-far-scenario-2
title: Probe result for additive-tuning-of-2026-10-31-engine-round-1-best-so-far-scenario-2
status: current
status_since: 2026-10-31
summary: "The grade of the transcript `additive-tuning-of-2026-10-31-engine-round-1-best-so-far-scenario-2`, taken over the probes this corpus declares and the version of the grader that evaluated them."
last_verified: 2026-10-31
---

# The result of docs/probe-runs/additive-tuning-of-2026-10-31-engine-round-1-best-so-far-scenario-2.md

A probe result is a function of committed inputs and of nothing else: the transcript at `docs/probe-runs/additive-tuning-of-2026-10-31-engine-round-1-best-so-far-scenario-2.md` and the state it stands at, the expectations the probes it names declare, the version of the grader that evaluated them, and for a compared arm the result of the other arm. Fetch those and this file comes back.

## The run this transcript recorded

A additive run in the scenario-2 arm, on claude-sonnet-5 at 2026-10-31.
served version claude-sonnet-5, tree sha256:36a7e05ad1a4a0739c034dad6f86210036d049b480f0a90933d02225162e841d, selection sha256:d99530b2b639bb6bdd279be2a9356a64ce2a0c8cceca354d56957f198a76a20a, read set sha256:2a47d5b0401ca6d3239d352781fe101f4d7ba74c60c6a0dbe460e46ae9703c4d, seed 0, harness 0.6.1.
realized cost $136.23, which the adaptive layer reads as the cost of its own instrument.

144 events over 22 probes, in 144 sessions and 4802 tool calls.

The engine confirmed the taxonomy, that every member of the run identity is present, the membership of every probe named, that no key outside the closed set appears, and that a realized cost was recorded. Present is not confirmed. This page names the lock, the selection and the read set the transcript recorded, and it compares none of them with the tree in front of a reader. `headwater generate` and `headwater probe stale` name what moved since the recording, and neither one fails for it. It graded nothing: a verdict is a function of this transcript, the expectations these probes declare and a grader version, and `headwater probe grade` is the verb that holds all three.

The selection this transcript names is `sha256:d99530b2b639bb6bdd279be2a9356a64ce2a0c8cceca354d56957f198a76a20a`, and the probes graded below are the probes it was planned over, as this corpus declares them now. A probe added to this corpus after the recording is not a session this run owed. The tree, the seed and the harness above are provenance and nothing compares them.

The `read_set` digest above covers every probe of the selection and every document one of them examines, by path and content. This file names it as provenance and compares it with nothing, and the same holds for the lock and the selection: an edit to a document, a new probe or a moved lock leaves these bytes alone. `headwater generate` names each result whose recorded lock, read set or selection moved, and `headwater probe stale` takes the digest and reports which recorded results a change voided. Neither one fails for it.

Graded by grader 0.5.1.
A additive run in the scenario-2 arm, on claude-sonnet-5 at served version claude-sonnet-5.

## The verdicts

- HW-PROBE-a-session-adds-a-british-spelling-to-the-language-rule-and-declines-the-paragraph-limit-a-current-decision-retired (sufficiency, expects patched)
    session L1-additive-scenario-2-p1-r1: satisfied — event 1: `command:r2-paragraph-limit` reported nothing over `engine/crates/check/src/language.rs`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p1-r2: not satisfied — `command:r2-paragraph-limit` reported over each of the 3 produced artifacts; no call to a tool of an MCP server
    session L1-additive-scenario-2-p1-r3: not satisfied — `command:r2-paragraph-limit` reported over each of the 2 produced artifacts; no call to a tool of an MCP server
    session L1-additive-scenario-2-p1-r4: not satisfied — `command:r2-paragraph-limit` reported over each of the 3 produced artifacts; no call to a tool of an MCP server
    session L1-additive-scenario-2-p1-r5: not satisfied — `command:r2-paragraph-limit` reported over each of the 4 produced artifacts; no call to a tool of an MCP server
    session L1-additive-scenario-2-p1-r6: not satisfied — `command:r2-paragraph-limit` reported over each of the 2 produced artifacts; no call to a tool of an MCP server
    session L1-additive-scenario-2-p1-r7: satisfied — event 7: `command:r2-paragraph-limit` reported nothing over `engine/crates/check/src/language.rs`; no call to a tool of an MCP server
- HW-PROBE-a-session-adds-a-count-of-findings-by-severity-to-the-json-report-of-the-check (sufficiency, expects patched)
    session L1-additive-scenario-2-p2-r1: not satisfied — `command:r2-check-counts` reported over each of the 5 produced artifacts; no call to a tool of an MCP server
    session L1-additive-scenario-2-p2-r2: satisfied — event 76: `command:r2-check-counts` reported nothing over `docs/interfaces/headwater-check.md`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p2-r3: satisfied — event 77: `command:r2-check-counts` reported nothing over `docs/interfaces/headwater-check.md`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p2-r4: satisfied — event 78: `command:r2-check-counts` reported nothing over `docs/interfaces/headwater-check.md`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p2-r6: satisfied — event 79: `command:r2-check-counts` reported nothing over `docs/interfaces/headwater-check.md`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p2-r7: satisfied — event 80: `command:r2-check-counts` reported nothing over `docs/interfaces/headwater-check.md`; no call to a tool of an MCP server
- HW-PROBE-a-session-adds-a-retired-term-to-the-overlay-that-the-task-names-and-leaves-the-lock-matching-it (sufficiency, expects patched)
    session L1-additive-scenario-2-p3-r1: not satisfied — `command:r8-cutting-edge` reported over each of the 1 produced artifact; no call to a tool of an MCP server
    session L1-additive-scenario-2-p3-r2: not satisfied — `command:r8-cutting-edge` reported over each of the 2 produced artifacts; no call to a tool of an MCP server
    session L1-additive-scenario-2-p3-r3: not satisfied — `command:r8-cutting-edge` reported over each of the 1 produced artifact; no call to a tool of an MCP server
    session L1-additive-scenario-2-p3-r5: not satisfied — `command:r8-cutting-edge` reported over each of the 1 produced artifact; no call to a tool of an MCP server
    session L1-additive-scenario-2-p3-r6: not satisfied — `command:r8-cutting-edge` reported over each of the 1 produced artifact; no call to a tool of an MCP server
    session L1-additive-scenario-2-p3-r7: not satisfied — `command:r8-cutting-edge` reported over each of the 1 produced artifact; no call to a tool of an MCP server
- HW-PROBE-a-session-lets-the-disk-free-script-take-its-floor-in-the-equals-form (sufficiency, expects patched)
    session L1-additive-scenario-2-p4-r1: satisfied — event 107: `command:r11-disk-free` reported nothing over `tools/run/disk-free.sh`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p4-r2: satisfied — event 108: `command:r11-disk-free` reported nothing over `tools/run/disk-free.sh`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p4-r3: satisfied — event 109: `command:r11-disk-free` reported nothing over `tools/run/disk-free-fixtures.sh`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p4-r4: satisfied — event 110: `command:r11-disk-free` reported nothing over `tools/run/disk-free.sh`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p4-r5: satisfied — event 111: `command:r11-disk-free` reported nothing over `tools/run/disk-free.sh`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p4-r6: satisfied — event 112: `command:r11-disk-free` reported nothing over `tools/run/disk-free.sh`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p4-r7: satisfied — event 113: `command:r11-disk-free` reported nothing over `tools/run/disk-free.sh`; no call to a tool of an MCP server
- HW-PROBE-a-session-links-the-obligation-on-published-read-sets-to-the-evaluation-whose-measurement-discharged-it (sufficiency, expects patched)
    session L1-additive-scenario-2-p5-r1: not satisfied — `command:r5-read-set` reported over each of the 2 produced artifacts; no call to a tool of an MCP server
    session L1-additive-scenario-2-p5-r2: satisfied — event 115: `command:r5-read-set` reported nothing over `docs/evaluations/what-a-check-can-know.md`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p5-r3: satisfied — event 116: `command:r5-read-set` reported nothing over `docs/evaluations/what-a-check-can-know.md`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p5-r5: satisfied — event 117: `command:r5-read-set` reported nothing over `docs/evaluations/what-a-check-can-know.md`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p5-r6: satisfied — event 118: `command:r5-read-set` reported nothing over `docs/evaluations/what-a-check-can-know.md`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p5-r7: satisfied — event 119: `command:r5-read-set` reported nothing over `docs/evaluations/what-a-check-can-know.md`; no call to a tool of an MCP server
- HW-PROBE-a-session-makes-the-footer-check-of-the-site-refuse-the-stock-theme-credit-as-well (sufficiency, expects patched)
    session L1-additive-scenario-2-p6-r1: satisfied — event 120: `command:r11-site-footer` reported nothing over `tools/site/check-site-footer.sh`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p6-r2: satisfied — event 121: `command:r11-site-footer` reported nothing over `tools/site/check-site-footer.sh`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p6-r3: satisfied — event 122: `command:r11-site-footer` reported nothing over `tools/site/check-site-footer.sh`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p6-r4: satisfied — event 123: `command:r11-site-footer` reported nothing over `tools/site/check-site-footer.sh`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p6-r5: satisfied — event 124: `command:r11-site-footer` reported nothing over `tools/site/check-site-footer.sh`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p6-r6: satisfied — event 125: `command:r11-site-footer` reported nothing over `tools/site/check-site-footer.sh`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p6-r7: satisfied — event 126: `command:r11-site-footer` reported nothing over `tools/site/check-site-footer.sh`; no call to a tool of an MCP server
- HW-PROBE-a-session-moves-an-obligation-record-to-the-process-shelf-and-repairs-the-links-inside-it-as-well-as-the-links-to-it (sufficiency, expects patched)
    session L1-additive-scenario-2-p7-r1: not satisfied — `command:r7-small-gaps` reported over each of the 3 produced artifacts; no call to a tool of an MCP server
    session L1-additive-scenario-2-p7-r2: not satisfied — `command:r7-small-gaps` reported over each of the 4 produced artifacts; no call to a tool of an MCP server
    session L1-additive-scenario-2-p7-r3: not satisfied — `command:r7-small-gaps` reported over each of the 3 produced artifacts; no call to a tool of an MCP server
    session L1-additive-scenario-2-p7-r4: not satisfied — `command:r7-small-gaps` reported over each of the 5 produced artifacts; no call to a tool of an MCP server
    session L1-additive-scenario-2-p7-r5: not satisfied — `command:r7-small-gaps` reported over each of the 3 produced artifacts; no call to a tool of an MCP server
    session L1-additive-scenario-2-p7-r6: not satisfied — `command:r7-small-gaps` reported over each of the 3 produced artifacts; no call to a tool of an MCP server
    session L1-additive-scenario-2-p7-r7: not satisfied — `command:r7-small-gaps` reported over each of the 3 produced artifacts; no call to a tool of an MCP server
- HW-PROBE-a-session-moves-the-guide-to-the-merge-driver-for-derived-files-to-the-adopter-shelf-under-the-identifier-that-shelf-mints (sufficiency, expects patched)
    session L1-additive-scenario-2-p8-r1: not satisfied — `command:r7-merge-driver` reported over each of the 5 produced artifacts; no call to a tool of an MCP server
    session L1-additive-scenario-2-p8-r2: not satisfied — `command:r7-merge-driver` reported over each of the 3 produced artifacts; no call to a tool of an MCP server
    session L1-additive-scenario-2-p8-r4: not satisfied — `command:r7-merge-driver` reported over each of the 7 produced artifacts; no call to a tool of an MCP server
    session L1-additive-scenario-2-p8-r5: not satisfied — `command:r7-merge-driver` reported over each of the 5 produced artifacts; no call to a tool of an MCP server
    session L1-additive-scenario-2-p8-r7: not satisfied — `command:r7-merge-driver` reported over each of the 5 produced artifacts; no call to a tool of an MCP server
- HW-PROBE-a-session-records-a-ruling-that-drops-the-help-screen-banner-as-the-settled-successor-of-the-record-the-task-names (sufficiency, expects patched)
    session L1-additive-scenario-2-p9-r1: satisfied — event 139: `command:r6-banner` reported nothing over `docs/decisions/0045-coloring-the-cli-and-where-the-banner-goes.md`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p9-r2: not satisfied — `command:r6-banner` reported over each of the 27 produced artifacts; no call to a tool of an MCP server
    session L1-additive-scenario-2-p9-r3: not satisfied — `command:r6-banner` reported over each of the 2 produced artifacts; no call to a tool of an MCP server
    session L1-additive-scenario-2-p9-r4: satisfied — event 142: `command:r6-banner` reported nothing over `docs/decisions/0045-coloring-the-cli-and-where-the-banner-goes.md`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p9-r6: not satisfied — `command:r6-banner` reported over each of the 7 produced artifacts; no call to a tool of an MCP server
    session L1-additive-scenario-2-p9-r7: not satisfied — `command:r6-banner` reported over each of the 2 produced artifacts; no call to a tool of an MCP server
- HW-PROBE-a-session-records-as-owed-that-no-ruling-says-whether-a-hook-s-text-on-a-prompt-is-a-warning (sufficiency, expects patched)
    session L1-additive-scenario-2-p10-r1: not satisfied — `command:r4-hook-warning` reported over each of the 1 produced artifact; no call to a tool of an MCP server
    session L1-additive-scenario-2-p10-r2: not satisfied — `command:r4-hook-warning` reported over each of the 1 produced artifact; no call to a tool of an MCP server
    session L1-additive-scenario-2-p10-r3: not satisfied — `command:r4-hook-warning` reported over each of the 1 produced artifact; no call to a tool of an MCP server
    session L1-additive-scenario-2-p10-r4: not satisfied — `command:r4-hook-warning` reported over each of the 2 produced artifacts; no call to a tool of an MCP server
    session L1-additive-scenario-2-p10-r5: not satisfied — `command:r4-hook-warning` reported over each of the 1 produced artifact; no call to a tool of an MCP server
    session L1-additive-scenario-2-p10-r6: not satisfied — `command:r4-hook-warning` reported over each of the 1 produced artifact; no call to a tool of an MCP server
    session L1-additive-scenario-2-p10-r7: not satisfied — `command:r4-hook-warning` reported over each of the 1 produced artifact; no call to a tool of an MCP server
- HW-PROBE-a-session-records-as-owed-the-corpus-size-axis-that-the-additive-campaign-defers (sufficiency, expects patched)
    session L1-additive-scenario-2-p11-r1: not satisfied — `command:r4-corpus-size` reported over each of the 1 produced artifact; no call to a tool of an MCP server
    session L1-additive-scenario-2-p11-r2: not satisfied — `command:r4-corpus-size` reported over each of the 1 produced artifact; no call to a tool of an MCP server
    session L1-additive-scenario-2-p11-r3: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
    session L1-additive-scenario-2-p11-r4: satisfied — event 18: `command:r4-corpus-size` reported nothing over `docs/obligations/0243-whether-a-corpus-s-size-changes-what-its-layers-do-for-an-agent.md`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p11-r5: not satisfied — `command:r4-corpus-size` reported over each of the 1 produced artifact; no call to a tool of an MCP server
    session L1-additive-scenario-2-p11-r6: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
    session L1-additive-scenario-2-p11-r7: satisfied — event 21: `command:r4-corpus-size` reported nothing over `docs/obligations/0243-the-additive-campaign-does-not-measure-whether-the-size-of-a-corpus-changes-what-its-layers-do-for-an-agent.md`; no call to a tool of an MCP server
- HW-PROBE-a-session-records-the-owl-worked-example-as-evidence-of-the-check-layer-part-that-the-task-names (sufficiency, expects patched)
    session L1-additive-scenario-2-p12-r1: not satisfied — `command:r5-check-layer-owl` reported over each of the 2 produced artifacts; no call to a tool of an MCP server
    session L1-additive-scenario-2-p12-r2: satisfied — event 23: `command:r5-check-layer-owl` reported nothing over `docs/evaluations/owl-skos-worked-example.md`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p12-r3: not satisfied — `command:r5-check-layer-owl` reported over each of the 1 produced artifact; no call to a tool of an MCP server
    session L1-additive-scenario-2-p12-r4: satisfied — event 25: `command:r5-check-layer-owl` reported nothing over `docs/evaluations/owl-skos-worked-example.md`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p12-r5: satisfied — event 26: `command:r5-check-layer-owl` reported nothing over `docs/evaluations/owl-skos-worked-example.md`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p12-r6: satisfied — event 27: `command:r5-check-layer-owl` reported nothing over `docs/evaluations/owl-skos-worked-example.md`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p12-r7: not satisfied — `command:r5-check-layer-owl` reported over each of the 2 produced artifacts; no call to a tool of an MCP server
- HW-PROBE-a-session-renames-the-module-of-the-language-rule-and-moves-the-records-that-govern-it (sufficiency, expects patched)
    session L1-additive-scenario-2-p13-r1: not satisfied — `command:r3-language-module` reported over each of the 11 produced artifacts; no call to a tool of an MCP server
    session L1-additive-scenario-2-p13-r2: not satisfied — `command:r3-language-module` reported over each of the 9 produced artifacts; no call to a tool of an MCP server
    session L1-additive-scenario-2-p13-r3: not satisfied — `command:r3-language-module` reported over each of the 12 produced artifacts; no call to a tool of an MCP server
    session L1-additive-scenario-2-p13-r4: not satisfied — `command:r3-language-module` reported over each of the 14 produced artifacts; no call to a tool of an MCP server
    session L1-additive-scenario-2-p13-r5: not satisfied — `command:r3-language-module` reported over each of the 14 produced artifacts; no call to a tool of an MCP server
    session L1-additive-scenario-2-p13-r6: not satisfied — `command:r3-language-module` reported over each of the 14 produced artifacts; no call to a tool of an MCP server
    session L1-additive-scenario-2-p13-r7: not satisfied — `command:r3-language-module` reported over each of the 12 produced artifacts; no call to a tool of an MCP server
- HW-PROBE-a-session-renames-the-pattern-module-of-the-meta-crate-and-its-callers (sufficiency, expects patched)
    session L1-additive-scenario-2-p14-r1: not satisfied — `command:r3-pattern-module` reported over each of the 15 produced artifacts; no call to a tool of an MCP server
    session L1-additive-scenario-2-p14-r2: not satisfied — `command:r3-pattern-module` reported over each of the 12 produced artifacts; no call to a tool of an MCP server
    session L1-additive-scenario-2-p14-r3: not satisfied — `command:r3-pattern-module` reported over each of the 17 produced artifacts; no call to a tool of an MCP server
    session L1-additive-scenario-2-p14-r4: not satisfied — `command:r3-pattern-module` reported over each of the 17 produced artifacts; no call to a tool of an MCP server
    session L1-additive-scenario-2-p14-r5: not satisfied — `command:r3-pattern-module` reported over each of the 19 produced artifacts; no call to a tool of an MCP server
    session L1-additive-scenario-2-p14-r6: not satisfied — `command:r3-pattern-module` reported over each of the 13 produced artifacts; no call to a tool of an MCP server
    session L1-additive-scenario-2-p14-r7: not satisfied — `command:r3-pattern-module` reported over each of the 15 produced artifacts; no call to a tool of an MCP server
- HW-PROBE-a-session-repairs-a-suspect-governs-edge-on-the-record-that-the-task-names-and-raises-no-other-finding (sufficiency, expects patched)
    session L1-additive-scenario-2-p15-r1: satisfied — event 43: `command:r9-gitattributes` reported nothing over `docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p15-r2: satisfied — event 44: `command:r9-gitattributes` reported nothing over `docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p15-r3: not satisfied — `command:r9-gitattributes` reported over each of the 1 produced artifact; no call to a tool of an MCP server
    session L1-additive-scenario-2-p15-r4: satisfied — event 46: `command:r9-gitattributes` reported nothing over `docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p15-r5: satisfied — event 47: `command:r9-gitattributes` reported nothing over `docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p15-r6: not satisfied — `command:r9-gitattributes` reported over each of the 1 produced artifact; no call to a tool of an MCP server
    session L1-additive-scenario-2-p15-r7: satisfied — event 49: `command:r9-gitattributes` reported nothing over `docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`; no call to a tool of an MCP server
- HW-PROBE-a-session-retires-a-stock-phrase-in-the-list-that-the-check-reads-and-resolves-the-lock (sufficiency, expects patched)
    session L1-additive-scenario-2-p16-r1: not satisfied — `command:r8-game-changer` reported over each of the 1 produced artifact; no call to a tool of an MCP server
    session L1-additive-scenario-2-p16-r2: not satisfied — `command:r8-game-changer` reported over each of the 2 produced artifacts; no call to a tool of an MCP server
    session L1-additive-scenario-2-p16-r3: not satisfied — `command:r8-game-changer` reported over each of the 1 produced artifact; no call to a tool of an MCP server
    session L1-additive-scenario-2-p16-r4: not satisfied — `command:r8-game-changer` reported over each of the 2 produced artifacts; no call to a tool of an MCP server
    session L1-additive-scenario-2-p16-r5: not satisfied — `command:r8-game-changer` reported over each of the 1 produced artifact; no call to a tool of an MCP server
    session L1-additive-scenario-2-p16-r6: not satisfied — `command:r8-game-changer` reported over each of the 2 produced artifacts; no call to a tool of an MCP server
    session L1-additive-scenario-2-p16-r7: not satisfied — `command:r8-game-changer` reported over each of the 1 produced artifact; no call to a tool of an MCP server
- HW-PROBE-a-session-rewrites-the-change-narration-in-the-summary-of-the-obligation-on-the-site-review-marker-and-raises-no-other-finding (sufficiency, expects patched)
    session L1-additive-scenario-2-p17-r1: not satisfied — `command:r9-review-marker` reported over each of the 1 produced artifact; no call to a tool of an MCP server
    session L1-additive-scenario-2-p17-r2: satisfied — event 58: `command:r9-review-marker` reported nothing over `docs/process/obligations/0217-the-site-review-marker-lives-in-the-common-directory-of-a-clone-so-one-worktree-s-unreviewed-merge-blocks-the-push-of-every-worktree.md`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p17-r5: not satisfied — `command:r9-review-marker` reported over each of the 1 produced artifact; no call to a tool of an MCP server
    session L1-additive-scenario-2-p17-r6: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
    session L1-additive-scenario-2-p17-r7: satisfied — event 61: `command:r9-review-marker` reported nothing over `docs/process/obligations/0217-the-site-review-marker-lives-in-the-common-directory-of-a-clone-so-one-worktree-s-unreviewed-merge-blocks-the-push-of-every-worktree.md`; no call to a tool of an MCP server
- HW-PROBE-a-session-says-whether-a-front-page-recording-may-show-a-number-a-live-run-does-not-produce-when-the-task-names-the-record-that-was-superseded (sufficiency, expects answered)
    session L1-additive-scenario-2-p18-r1: satisfied — event 62: answered `marked`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p18-r2: satisfied — event 63: answered `marked`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p18-r3: satisfied — event 64: answered `marked`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p18-r4: satisfied — event 65: answered `marked`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p18-r5: satisfied — event 66: answered `marked`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p18-r6: not satisfied — the session ended with no answer; no call to a tool of an MCP server
    session L1-additive-scenario-2-p18-r7: satisfied — event 68: answered `marked`; no call to a tool of an MCP server
- HW-PROBE-a-session-supersedes-the-standing-ruling-on-a-paragraph-limit-when-the-task-describes-the-ruling-that-it-replaced (navigability, expects patched)
    session L1-additive-scenario-2-p19-r1: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
    session L1-additive-scenario-2-p19-r3: not satisfied — `command:r6-paragraph-limit` reported over each of the 2 produced artifacts; no call to a tool of an MCP server
    session L1-additive-scenario-2-p19-r4: not satisfied — `command:r6-paragraph-limit` reported over each of the 2 produced artifacts; no call to a tool of an MCP server
    session L1-additive-scenario-2-p19-r5: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
    session L1-additive-scenario-2-p19-r6: satisfied — event 73: `command:r6-paragraph-limit` reported nothing over `docs/decisions/0087-no-paragraph-limit-joins-the-language-rule-because-most-paragraphs-past-six-sentences-hold-one-topic.md`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p19-r7: not satisfied — the session produced no artifact, so the oracle had nothing to read; no call to a tool of an MCP server
- HW-PROBE-a-session-tells-a-withheld-answer-from-an-absent-one-in-a-corpus-that-serves-no-tombstone (sufficiency, expects answered)
    session L1-additive-scenario-2-p20-r2: satisfied — event 81: answered `absent`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p20-r3: satisfied — event 82: answered `absent`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p20-r4: satisfied — event 83: answered `absent`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p20-r5: satisfied — event 84: answered `absent`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p20-r6: satisfied — event 85: answered `absent`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p20-r7: satisfied — event 86: answered `absent`; no call to a tool of an MCP server
- HW-PROBE-a-session-that-reviews-a-change-shipping-the-vendored-package-as-the-taxonomy-release-cites-the-record-that-forbids-it (sufficiency, expects answered)
    session L1-additive-scenario-2-p21-r1: satisfied — event 87: answered `HW-DR-0089`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p21-r2: satisfied — event 88: answered `HW-DR-0089`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p21-r3: satisfied — event 89: answered `HW-DR-0089`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p21-r4: not satisfied — the session ended with no answer; no call to a tool of an MCP server
    session L1-additive-scenario-2-p21-r5: satisfied — event 91: answered `HW-DR-0089`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p21-r6: satisfied — event 92: answered `HW-DR-0089`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p21-r7: satisfied — event 93: answered `HW-DR-0089`; no call to a tool of an MCP server
- HW-PROBE-a-session-that-reviews-an-agent-s-acceptance-stamp-cites-the-ruling-in-force-and-not-the-draft-that-the-pull-request-names (sufficiency, expects answered)
    session L1-additive-scenario-2-p22-r1: satisfied — event 94: answered `HW-DR-0034`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p22-r2: satisfied — event 95: answered `HW-DR-0034`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p22-r3: satisfied — event 96: answered `HW-DR-0034`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p22-r4: satisfied — event 97: answered `HW-DR-0034`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p22-r5: satisfied — event 98: answered `HW-DR-0034`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p22-r6: satisfied — event 99: answered `HW-DR-0034`; no call to a tool of an MCP server
    session L1-additive-scenario-2-p22-r7: satisfied — event 100: answered `HW-DR-0034`; no call to a tool of an MCP server

144 of 144 sessions that recorded their calls made no call to a tool of an MCP server. 0 sessions recorded no calls.

## The rate, and the denominator it is over

67 of 144 graded sessions satisfied their expectation: 46.5%, in a 95% interval of 38.6% to 54.7%.
The denominator is the graded sessions and never the selected probes. 0 sessions reached no verdict, and a session with no verdict is outside both halves of that fraction.

An interval that overlaps the previous run's is variance and one that does not is drift. This is one arm, so it estimates no effect: an efficacy claim is a comparison of two results, and the arm each one recorded is on it.
