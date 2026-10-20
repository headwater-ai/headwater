---
id: HW-EVAL-what-the-tuning-round-of-the-additive-campaign-measured-layer-by-layer
status: current
status_since: 2026-10-20
summary: "Try 1 of the documents layer passed 32 of 72 sessions in scenario 1, below the 40 it needed. The loop stopped, and the seven sentences went out again."
last_verified: 2026-10-20
title: "What the tuning round of the additive campaign measured, layer by layer"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0111
    - HW-EVAL-what-the-additive-screening-round-of-2026-10-18-measured-by-scenario-against-its-base
    - HW-RUN-additive-screening-of-2026-10-18-scenario-0
    - HW-RUN-additive-screening-of-2026-10-18-scenario-1
    - HW-RUN-additive-tuning-of-2026-10-20-documents-try-1-scenario-1
---

# What the tuning round of the additive campaign measured, layer by layer

## What this record is

This is the tuning record that [#1777](https://github.com/headwater-ai/headwater/issues/1777) asks for. The tuning round changes one layer at a time, in the one scenario where that layer is the only difference from its base. For each try, this record states the hypothesis, the change and its commit. It states the rate of the tuned scenario against its base, with an interval. It also states the running total against the approval, and whether the try was kept. Each layer gets its own section. The documents layer is the first, and it is finished. The other five layers have no section yet.

## The method

The rate is the share of graded sessions that satisfied their expectation, with the Wilson 95% interval that the result page prints. The difference from the base is the Newcombe hybrid score interval, as `Interval::minus` computes it in `engine/crates/probe/src/grade.rs`. The screening evaluation uses the same method. The script that computed the figures below also gives two rows that the screening evaluation prints. They are +38.9 [+23.7, +51.7] for scenario 1 against 0, and +1.4 [-14.5, +17.2] for scenario 3 against 1.

**The keep rule** is the rule of #1777. A try is kept when the pooled point estimate of the tuned scenario against its base is above the best one so far. A loop stops at the first try that is not kept, or after three tries. A try that is not kept is reverted. On 2026-10-20 the owner replaced this rule. A try is kept only when the whole interval of its difference against the best so far is above zero. `tools/probe/try.sh result` applies the new rule (#1822). Try 1, 32 of 72 against 39 of 72, is -9.7 [-25.2, +6.5], so neither rule keeps it.

**The approval.** The owner approved $1,512.00 for the whole tuning round, over every attempt and every layer (OWNER lines of 2026-10-19 and 2026-10-20 in the run `20261018-0144`). The total below is the sum of `sessions/*/cost` and `sessions/*/spent-before` under every batch directory of the round.

## Documents, scenario 1 against scenario 0

**Where it started.** In the screening round, scenario 1 passed 39 of 72 sessions (54.2%) and scenario 0 passed 11 of 72 (15.3%). By probe, scenario 1 passed 8 of 18 on the cold agent and 9 of 18 on adjudication. It passed 18 of 18 on two citations and 4 of 18 on discharge. The best pooled estimate before try 1 was 39 of 72, so try 1 needed at least 40 of 72 to be kept.

**The base.** Base 0 is the screening measurement of scenario 0, 11 of 72, at the pin `2d212a26`. It was not measured again for try 1, which ran at the pin `2e3d71a7`. The scenario-0 tree differs between the two pins in 30 paths. The version moved from 0.6.0 to 0.6.1, `headwater/standard` moved to 4.16.2, and fixtures, site and README text, `tools/probe` and `tools/run` changed. No path under `docs/` or `engine/crates/*/src/` differs. The owner accepted this difference on 2026-10-20 and ruled that base 0 stands. The build note of the try, `build-paid1.md` in the scratch directory of #1777, lists each of the 30 paths with what changed in it.

### Try 1: one sentence beside each relation that requires its inverse

**Hypothesis.** In the screening round, 14 of 18 discharge sessions failed. Each one edited the evaluation and left the obligation without its half of the relation. Before the edit, 9 of the 14 opened a taxonomy doctrine page, where `reciprocal: required` is only a line of YAML. A plain sentence beside each such relation can tell a session to write the inverse half in the same change. The expected effect was up on discharge, by 3 to 5 sessions, or about +4 to +7 points pooled.

**The change.** Seven sentences, one for each relation that requires its inverse. The decision-record doctrine got one for `discharges`. The design-spec doctrine got one for `cites_evidence` and one for `applied_in`. The standards-spec doctrine got one for `regulates` and one for `realizes`. The brd-prd doctrine got one for `elaborates`. Spec 2 got one general sentence for `supersedes`, under "Behavior at the limits". [#1816](https://github.com/headwater-ai/headwater/pull/1816) added them at `2e3d71a7`, with `headwater/standard` 4.16.2. No sentence edited the target document of a probe.

**The run.** 72 sessions of the scenario-1 arm, 18 on each of the 4 tuning probes, at the pin `2e3d71a7`, on `claude-sonnet-5` under Claude Code 2.1.288. The 8 canary sessions count toward the 72. The driver wrote `done` at 20:20 UTC+10 on 2026-10-20, with 72 of 72 sessions recorded and 0 halts. The transcript is [the try 1 transcript](../probe-runs/additive-tuning-of-2026-10-20-documents-try-1-scenario-1.md), and the rate comes from [its result](../probe-results/additive-tuning-of-2026-10-20-documents-try-1-scenario-1.md).

**The rate.** Scenario 1 passed 32 of 72 sessions: 44.4%, with a Wilson interval of 33.5% to 55.9%. Against base 0 at 11 of 72, the difference is +29.2 points [+14.3, +42.4]. Against the screening rate of scenario 1, 39 of 72, it is -9.7 points [-25.2, +6.5].

| probe | try 1 | against base 0, in points | screening, scenario 1 | try 1 against screening, in points |
|---|---|---|---|---|
| discharge, the probe that try 1 aims at | 5 of 18 | +22.2 [-3.1, +45.8] against 1 of 18 | 4 of 18 | +5.6 [-22.1, +32.2] |
| cold agent | 6 of 18 | +33.3 [+8.8, +56.3] against 0 of 18 | 8 of 18 | -11.1 [-38.8, +19.2] |
| two citations | 18 of 18 | +44.4 [+17.9, +66.3] against 10 of 18 | 18 of 18 | +0.0 [-17.6, +17.6] |
| adjudication | 3 of 18 | +16.7 [-4.0, +39.2] against 0 of 18 | 9 of 18 | -33.3 [-56.9, -2.5] |
| pooled | 32 of 72 | +29.2 [+14.3, +42.4] against 11 of 72 | 39 of 72 | -9.7 [-25.2, +6.5] |

**Raw recording.** On discharge, a reading of each `raw.jsonl` agrees with the grader in 18 of 18 sessions. A session passed exactly when it edited the obligation. 9 of 18 sessions opened a doctrine page. All 5 that passed opened one, and 4 of the 13 that failed opened one and still left the obligation alone.

**Raw recording.** On adjudication, 0 of 18 sessions named spec 2 in any call, so no session read the general sentence on `supersedes`. In the screening round, 0 of 18 named spec 2 too. The fall from 9 to 3 did not pass through a page that try 1 changed. One probe of 18 sessions, against a base on another pin, does not separate that fall from chance.

**The ruling: not kept, and reverted.** 32 of 72 is below the 40 of 72 that the keep rule asks for, and the point estimate moved down from 39. So try 1 is not kept. The documents layer puts 0 bytes in front of the model in every session, so the rule's branch for text that shrank does not apply. The same pull request that records this try reverts the seven sentences. Each of the four doctrine pages, its vendored copy and spec 2 is byte-identical again to its text at `2551a794`. The doctrine ships in `headwater/standard`, so the revert publishes it as 4.16.3. The owner pushes any tag.

**The loop stops.** The loop stops because the first try was not kept, and not because it reached three tries. Try 2 of the plan put a link from each superseded decision record to its successor. It was to run only if try 1 was kept, so it does not run. The documents layer is at its final version, which is its text before try 1.

**Spend.** Try 1 spent 2,452 cents over 72 sessions, with 0 cents spent by halted attempts, or 34.1 cents a session. The dry run priced it at $36.00 against a cap of $60.00. The tuning round has spent $24.52 of $1,512.00, and $1,487.48 is left for the five other layers.

**Leak.** The seven sentences named no held-out key, and the revert takes out text only. The leak check of the dry run reads the always-loaded set. No document under `docs/` is in that set, so the check prints nothing for this layer.

## Limits

- Base 0 was measured at another pin. The owner accepted the 30 paths of difference. None of them is under `docs/` or engine source, but the base is not a measurement on the tree that try 1 ran on.
- Take 72 sessions and a true rate of 39 in 72. By the binomial distribution, a change with no effect reaches 40 of 72 about 45% of the time. It falls to 32 of 72 or lower about 6% of the time. "Not kept" is the rule of #1777, and it is not evidence that the sentences did harm. Only the discharge probe was the target, and it moved from 4 to 5 of 18.
- The raw recordings of try 1 are outside git, under `/mnt/build/hw-1777/docs-try1/a/sessions/`. Nobody deletes them until the report of [#1779](https://github.com/headwater-ai/headwater/issues/1779) is published.
