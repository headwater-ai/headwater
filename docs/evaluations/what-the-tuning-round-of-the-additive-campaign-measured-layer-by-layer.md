---
id: HW-EVAL-what-the-tuning-round-of-the-additive-campaign-measured-layer-by-layer
status: current
status_since: 2026-10-20
summary: "Two kept rounds took the skills layer to 134 of 145 sessions, against 76 of 145 with no skills. Documents try 1 was not kept. The round stopped before the hook and MCP layers."
last_verified: 2026-10-28
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
    - HW-RUN-additive-tuning-of-2026-10-24-base-re-measure-of-scenario-3
    - HW-RUN-additive-tuning-of-2026-10-24-skills-round-1-best-so-far-scenario-4
    - HW-RUN-additive-tuning-of-2026-10-24-skills-round-1-variant-1-scenario-4
    - HW-RUN-additive-tuning-of-2026-10-24-skills-round-1-variant-2-scenario-4
    - HW-RUN-additive-tuning-of-2026-10-24-skills-round-1-variant-3-scenario-4
    - HW-RUN-additive-tuning-of-2026-10-24-skills-round-1-variant-4-scenario-4
    - HW-RUN-additive-tuning-of-2026-10-24-skills-round-1-confirmation-scenario-4
    - HW-RUN-additive-tuning-of-2026-10-24-skills-round-2-variant-1-scenario-4
    - HW-RUN-additive-tuning-of-2026-10-24-skills-round-2-variant-2-scenario-4
    - HW-RUN-additive-tuning-of-2026-10-24-skills-round-2-variant-3-scenario-4
    - HW-RUN-additive-tuning-of-2026-10-24-skills-round-2-variant-4-scenario-4
    - HW-RUN-additive-tuning-of-2026-10-25-skills-round-2-confirmation-scenario-4
---

# What the tuning round of the additive campaign measured, layer by layer

## What this record is

This is the tuning record that [#1777](https://github.com/headwater-ai/headwater/issues/1777) asks for. The tuning round changes one layer at a time, in the one scenario where that layer is the only difference from its base. For each try, this record states the hypothesis, the change and its commit. It states the rate of the tuned scenario against its base, with an interval. It also states the running total against the approval, and whether the try was kept. Each layer gets its own section. The documents layer and the skills layer are finished. The round stopped before it tuned any other layer, and the last section states why.

## The method

The rate is the share of graded sessions that satisfied their expectation, with the Wilson 95% interval that the result page prints. The difference from the base is the Newcombe hybrid score interval, as `Interval::minus` computes it in `engine/crates/probe/src/grade.rs`. The screening evaluation uses the same method. The script that computed the figures below also gives two rows that the screening evaluation prints. They are +38.9 [+23.7, +51.7] for scenario 1 against 0, and +1.4 [-14.5, +17.2] for scenario 3 against 1.

**The keep rule** is the rule of #1777. A try is kept when the pooled point estimate of the tuned scenario against its base is above the best one so far. A loop stops at the first try that is not kept, or after three tries. A try that is not kept is reverted. On 2026-10-20 the owner replaced this rule. A try is kept only when the whole interval of its difference against the best so far is above zero. `tools/probe/try.sh result` applies the new rule (#1822). Try 1, 32 of 72 against 39 of 72, is -9.7 [-25.2, +6.5], so neither rule keeps it.

**The shape of a round.** The run prompt of 2026-10-22 gave each later round one shape, and the skills layer used it. A round screens four variants at 25 sessions each, which is 5 sessions on each of the 5 tuning probes. It ranks the screens on the use of the layer first and on the pass rate second. It confirms the first variant at 145 sessions, which is 29 on each probe, against the best so far at the same count. A screen and a best so far are never kept, and only a confirmation is. A session uses the skills layer when its raw log holds a call to the `Skill` tool.

**The keep rule near the ceiling.** On 2026-10-25 the owner added a second rule ([#1777 comment 6394137349](https://github.com/headwater-ai/headwater/issues/1777#issuecomment-6394137349)). The rule applies when the best so far passes 85% or more. Then a change is kept when its point estimate is up and its 80% interval is above zero. The 80% interval is a one-sided 90% bound. Below 85%, the rule of 2026-10-20 stays. `try.sh result` still applies only the rule of 2026-10-20.

**The approval.** The owner approved $1,512.00 for the whole tuning round, over every attempt and every layer (OWNER lines of 2026-10-19 and 2026-10-20 in the run `20261018-0144`). The total below is the sum of `sessions/*/cost` and `sessions/*/spent-before` under every batch directory of the round. From the skills layer on, `.headwater/probe.yml` splits that spend into two ceilings. The base ceiling of $220.00 pays for the first base and the first best so far of a layer ([#1833](https://github.com/headwater-ai/headwater/issues/1833), [#1854](https://github.com/headwater-ai/headwater/pull/1854)). The tuning ceiling of $1,290.00 pays for every screen and every confirmation, and it holds the $24.52 of try 1. The code-set pilot of [#1835](https://github.com/headwater-ai/headwater/issues/1835) has its own ceiling of $130.00.

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

## Skills, scenario 4 against scenario 3

**Where it started.** The skills layer is the only difference between scenario 4 and scenario 3. It ran on a tuning set of 5 probes: tombstone, obligation, verb default, recorder contract and shelf move. Every batch ran on `claude-sonnet-5` under Claude Code 2.1.288. The shipped skill bodies call `headwater`, and `headwater` is not on the `PATH` of a session. So a session that loaded a skill could not run the command that the skill names.

**The base.** [#1838](https://github.com/headwater-ai/headwater/issues/1838) added a `base` stage, which measures the base scenario again at the count of a confirmation. Scenario 3 passed 76 of 145 sessions at `bea46653`, the commit of [#1855](https://github.com/headwater-ai/headwater/pull/1855) that opened the layer. Against the screening measurement of scenario 3, 40 of 72, the difference is -3.1 points [-16.7, +10.8]. So the base shows no change from the screening round. The transcript is [the base transcript](../probe-runs/additive-tuning-of-2026-10-24-base-re-measure-of-scenario-3.md).

**The best so far.** The skills as they ship passed 81 of 142 graded sessions at `bea46653`, and 37 of 145 sessions used the skills layer. Against the base, the difference is +4.6 points [-6.8, +15.9], so the shipped skills show no proven gain. The recorder could not grade three sessions of the shelf-move probe. Each one edited the taxonomy and left its lock stale. They were accepted as counted non-records under [#1659](https://github.com/headwater-ai/headwater/issues/1659) and not run again. If the three count as failures, the line passed 81 of 145. The transcript is [the best-so-far transcript](../probe-runs/additive-tuning-of-2026-10-24-skills-round-1-best-so-far-scenario-4.md).

### Round 1: four ways to get a skill loaded

**Hypothesis.** A session loads a skill at its first or second tool call, or never. So the description of a skill decides its use, and a better trigger raises use. Each of the four variants also adds a "Run the engine" section to the skills. With it, a session that loads a skill can run the engine.

**The screens.** Each variant is one commit on `bea46653`, screened from a pushed branch under `origin/tune/` ([#1857](https://github.com/headwater-ai/headwater/pull/1857)).

| variant | what it changes | pin | used a skill | passed | against the base, in points |
|---|---|---|---|---|---|
| 1 | `headwater-orient` is the gate at the start of every task | `e303021c` | 23 of 25 | 21 of 25 | +31.6 [+11.3, +44.1] |
| 4 | each skill names the shapes of task it serves | `3bd71df6` | 21 of 25 | 20 of 25 | +27.6 [+6.9, +41.4] |
| 2 | `headwater-orient` is the way to run the engine | `6ceccacd` | 16 of 25 | 19 of 25 | +23.6 [+2.6, +38.5] |
| 3 | `headwater-authoring` is the gate before a task is complete | `88bea5de` | 10 of 25 | 15 of 25 | +7.6 [-13.3, +26.0] |

The transcripts are the screens of [variant 1](../probe-runs/additive-tuning-of-2026-10-24-skills-round-1-variant-1-scenario-4.md), [variant 2](../probe-runs/additive-tuning-of-2026-10-24-skills-round-1-variant-2-scenario-4.md), [variant 3](../probe-runs/additive-tuning-of-2026-10-24-skills-round-1-variant-3-scenario-4.md) and [variant 4](../probe-runs/additive-tuning-of-2026-10-24-skills-round-1-variant-4-scenario-4.md). Variant 1 ranks first on use.

**The confirmation.** Variant 1 passed 127 of 145 sessions at the pin `e303021c`, and 131 of 145 sessions used the skills layer. Against the best so far, the difference is +30.5 points [+20.4, +39.9]. Against the base, it is +35.2 points [+25.0, +44.4]. The transcript is [the round-1 confirmation](../probe-runs/additive-tuning-of-2026-10-24-skills-round-1-confirmation-scenario-4.md).

**The ruling: kept.** The whole 95% interval is above zero, so the rule of 2026-10-20 keeps variant 1. [#1860](https://github.com/headwater-ai/headwater/pull/1860) brought it to `main`. The confirmation became the best so far of round 2.

### Round 2: a second moment to load a skill

**Hypothesis.** In the round-1 confirmation, the recorder-contract probe held 14 of the 18 failures. It also held all 14 sessions that used no skill. On that probe, each session searched for the script that the task names before it loaded any skill. A trigger at another moment of the task can reach those sessions. Each of the four variants also carries the same three fixes to the body of `headwater-orient`. After the last edit, run `check --fix` and then `check`. Open the record in force before the answer. List the how-to shelf before routing.

**The screens.** Each variant is one commit on `e303021c`, the pin of round 1, with the round-2 opening of [#1861](https://github.com/headwater-ai/headwater/pull/1861) on top of it.

| variant | what it changes | pin | used a skill | passed | against the base, in points |
|---|---|---|---|---|---|
| 4 | `headwater-orient` loads before the final answer, and the start gate stays | `22c9ed57` | 25 of 25 | 25 of 25 | +47.6 [+32.1, +55.7] |
| 2 | `headwater-orient` loads before the session locates or opens a file | `1640cf81` | 24 of 25 | 24 of 25 | +43.6 [+26.1, +52.3] |
| 3 | `headwater-authoring` claims a change to code, a script or configuration, as a second pull | `ee9c29c4` | 24 of 25 | 24 of 25 | +43.6 [+26.1, +52.3] |
| 1 | `headwater-orient` loads when a file cites a document that the session has not opened | `2c2ec8f0` | 22 of 25 | 22 of 25 | +35.6 [+15.9, +46.8] |

The transcripts are the screens of [variant 1](../probe-runs/additive-tuning-of-2026-10-24-skills-round-2-variant-1-scenario-4.md), [variant 2](../probe-runs/additive-tuning-of-2026-10-24-skills-round-2-variant-2-scenario-4.md), [variant 3](../probe-runs/additive-tuning-of-2026-10-24-skills-round-2-variant-3-scenario-4.md) and [variant 4](../probe-runs/additive-tuning-of-2026-10-24-skills-round-2-variant-4-scenario-4.md). Variant 4 ranks first on use.

**The confirmation.** Variant 4 passed 134 of 145 sessions at the pin `22c9ed57`, and 136 of 145 sessions used the skills layer. Against the best so far, 127 of 145, the difference is +4.8 points [-2.2, +12.0]. Against the base, it is +40.0 points [+30.3, +48.7]. The transcript is [the round-2 confirmation](../probe-runs/additive-tuning-of-2026-10-25-skills-round-2-confirmation-scenario-4.md).

**The ruling: kept under the near-ceiling rule.** The 95% interval includes zero, so `try.sh result` wrote "not kept", and the transcript keeps that verdict. The best so far passed 127 of 145, which is above 85%. So the owner's rule of 2026-10-25 applies. The point estimate is up, and the 80% interval is [+0.3, +9.4], which is above zero. So variant 4 is kept. [#1865](https://github.com/headwater-ai/headwater/pull/1865) brought its commit `ede124a7` to `main` as `b5eea371`.

### The layer closes

The same ruling of 2026-10-25 skipped round 3. The skills layer closes at 134 of 145 sessions (92.4%). Scenario 3, with no skills, passed 76 of 145. The difference is +40.0 points [+30.3, +48.7]. The skills as they shipped before the round passed 81 of 142.

**Spend.** The tuning ceiling holds $374.61 of $1,290.00, and that sum includes the $24.52 of documents try 1. The base ceiling holds $214.66 of $220.00. Each figure is what the driver charged, so each one includes the cost of the halted attempts.

## How the round ended

**The hook and MCP layers were never tuned.** On 2026-10-25 the round paused with the skills layer closed and the hook layer next. Before the hook layer opened, the owner reviewed its coverage with an outside review of the probe design. The review found that the tuning probes covered the hook moments unevenly. It also found that the held-out set could not confirm any hook other than the intent hook.

**The rulings of 2026-10-28.** The owner ruled in [#1777 comment 6465567947](https://github.com/headwater-ai/headwater/issues/1777#issuecomment-6465567947) and [#1777 comment 6466863444](https://github.com/headwater-ai/headwater/issues/1777#issuecomment-6466863444):

- The round tunes only a layer that `headwater init` ships. `CLAUDE.md` is out of every tuning effort, because `headwater init` does not ship it.
- `headwater init` will ship the hooks and the MCP server. The read, write and review hooks move into the hook layer, beside the intent hook. A `.mcp.json` that declares `headwater mcp` ships beside them.
- Tuning moves to a new probe set, organized by canonical action. A canonical action is what a user or an agent sets out to do in a codebase. Each action has tuning probes and held-out probes, and the new set gets new baselines.

**What follows from the rulings.** The original tuning set and its baselines are retired. The kept skills changes stay on `main`. The final round of [#1778](https://github.com/headwater-ai/headwater/issues/1778) does not run in its planned form. A new epic holds the plan of 2026-10-28, and it replaces the rest of the work of #1777.

**Spend at the end.** The tuning ceiling holds $374.61 of $1,290.00, and the base ceiling holds $214.66 of $220.00. The code-set pilot of #1835 spent $125.01 of its $130.00.

## Limits

- Base 0 was measured at another pin. The owner accepted the 30 paths of difference. None of them is under `docs/` or engine source, but the base is not a measurement on the tree that try 1 ran on.
- Take 72 sessions and a true rate of 39 in 72. By the binomial distribution, a change with no effect reaches 40 of 72 about 45% of the time. It falls to 32 of 72 or lower about 6% of the time. "Not kept" is the rule of #1777, and it is not evidence that the sentences did harm. Only the discharge probe was the target, and it moved from 4 to 5 of 18.
- The raw recordings of try 1 are outside git, under `/mnt/build/hw-1777/docs-try1/a/sessions/`. Nobody deletes them until the report of [#1779](https://github.com/headwater-ai/headwater/issues/1779) is published.
- The skills result is a tuned result. The round chose each variant on the same 5 probes that measured it, and no held-out round ran. So 134 of 145 is the rate on the tuning set, and it is not a rate for tasks outside it.
- The near-ceiling rule reads an 80% interval, which is a one-sided 90% bound. So it accepts about a 1-in-10 chance that a kept change is neutral. Round 2 of the skills layer is kept only under that rule.
- The driver halted on the usage limit of the account several times. It halted six times in the best so far, five times in the round-1 confirmation and six times in the round-2 confirmation. Each time it continued in the same try root at the same pin, so each batch ran on one tree, but not in one sitting.
- The raw recordings of the skills layer are outside git, under `/mnt/build/hw-1777/` on the host that ran the batches. Nobody deletes them until the report of #1779 is published.
