---
id: HW-EVAL-what-the-additive-screening-round-of-2026-10-18-measured-by-scenario-against-its-base
status: current
status_since: 2026-10-19
summary: "In screening, only the docs layer gained against its base, +38.9 points, and it adds no always-loaded byte. The other 12 comparisons show no detectable effect."
last_verified: 2026-10-19
title: "What the additive screening round of 2026-10-18 measured, by scenario against its base"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0111
    - HW-RUN-additive-screening-of-2026-10-18-scenario-0
    - HW-RUN-additive-screening-of-2026-10-18-scenario-1
    - HW-RUN-additive-screening-of-2026-10-18-scenario-2
    - HW-RUN-additive-screening-of-2026-10-18-scenario-3
    - HW-RUN-additive-screening-of-2026-10-18-scenario-4
    - HW-RUN-additive-screening-of-2026-10-18-scenario-5
    - HW-RUN-additive-screening-of-2026-10-18-scenario-6
    - HW-RUN-additive-screening-of-2026-10-18-scenario-7
    - HW-RUN-additive-screening-of-2026-10-18-scenario-8
    - HW-RUN-additive-screening-of-2026-10-18-scenario-9
    - HW-RUN-additive-screening-of-2026-10-18-scenario-10
    - HW-RUN-additive-screening-of-2026-10-18-scenario-11
    - HW-RUN-additive-screening-of-2026-10-18-scenario-12
    - HW-RUN-additive-screening-of-2026-10-18-scenario-13-three-probes
    - HW-RUN-additive-screening-of-2026-10-18-scenario-13-the-tombstone-probe
---

# What the additive screening round of 2026-10-18 measured, by scenario against its base

## What ran

The screening round of the additive campaign ran the 4 tuning probes in each of the 14 scenarios of [HW-DR-0111](../decisions/0111-the-additive-campaign-adds-six-layers-to-plain-code-in-fourteen-scenarios-and-registers-one-comparison-each.md). It ran 18 sessions for each probe, so 72 sessions for each scenario and 1,008 in total. Every session ran on `claude-sonnet-5` under Claude Code 2.1.288, in a copy of a `git archive` of the pin `2d212a26`. The grader is version 0.5.1. The figures below come from the 15 result pages that [#1805](https://github.com/headwater-ai/headwater/pull/1805) merged at `bf15ab0b`, under `docs/probe-results/additive-screening-of-2026-10-18-scenario-*.md`. Scenario 13 has two pages, because the leak check put its tombstone probe on a line of its own. This evaluation pools the two pages into one row. The figures were derived on 2026-10-19, and this evaluation started no session and spent nothing.

**Where the raw recordings are.** The raw recordings of the campaign of 2026-10-03 are at `/home/james/.local/share/headwater/hw-1659/`, under `a/` and `b/`. They hold 5.5 GB on one host and are not committed. On 2026-10-19 the owner ruled: "Keep them, no end date: the raw recordings of the 2026-10-03 campaign stay on the machine with no end date." This evaluation reads none of them. So no figure here is evidence from a raw recording.

## The method

Each pass rate is the share of graded sessions that satisfied their expectation, with the Wilson 95% interval that the result pages print. The difference of two scenarios is the Newcombe hybrid score interval, built from the two Wilson intervals, as `Interval::minus` computes it in `engine/crates/probe/src/grade.rs`. The verdict follows HW-DR-0111. It is `gain` when the interval lies above zero, `loss` when it lies below zero, and `no detectable effect` when it holds zero.

The cost column is the realized cost of the attempt that recorded, divided by the graded sessions. A halt of the account's usage limit stopped some attempts, and those attempts spent more. The recorded attempts total 36,272 cents. The round spent 36,787 cents over every attempt, so the halts spent 515 cents that no row shows. [#1775](https://github.com/headwater-ai/headwater/issues/1775) carries the total for each scenario.

The bytes column is the raw UTF-8 bytes of the text that the layers put into every session. `sh tools/probe/seal.sh --leak --scenario --bytes <tree> <probe>...` measures it, over the tree of each scenario that `sh tools/probe/ablate.sh --present` and `--scenario <n>` build from the pin. The count holds the memory files and their imports, the listing of each skill, agent and command, and the description of each MCP tool. A skill body loads only when a session invokes the skill, so the count leaves it out, and it leaves out every document under `docs/`. Where the scenario holds the intent hook, a second figure follows the `+`. It is the mean of the text that the hook injects for the task of each of the 4 probes. That text is 3,403, 2,827, 4,036 and 3,372 bytes, and it is the same in each scenario that holds the hook. The harness's own system prompt is the same in every scenario, and the count leaves it out.

## The headline table

Only the registered comparison, a scenario against its base, earns a verdict. The screening figures are not compared across scenarios in the report ([#1779](https://github.com/headwater-ai/headwater/issues/1779)).

| scenario | layers | sessions | pass rate | difference from the base, in points, with a 95% Newcombe interval | verdict | cost per session, in cents | bytes the layers put in front of the model |
|---|---|---|---|---|---|---|---|
| [0](../probe-results/additive-screening-of-2026-10-18-scenario-0.md) | none (plain code) | 72 | 15.3% (11) | none, scenario 0 has no base | none | 86.6 | 0 |
| [1](../probe-results/additive-screening-of-2026-10-18-scenario-1.md) | docs | 72 | 54.2% (39) | +38.9 [+23.7, +51.7] against 0 | gain | 32.0 | 0 |
| [3](../probe-results/additive-screening-of-2026-10-18-scenario-3.md) | docs, engine | 72 | 55.6% (40) | +1.4 [-14.5, +17.2] against 1 | no detectable effect | 31.0 | 0 |
| [4](../probe-results/additive-screening-of-2026-10-18-scenario-4.md) | docs, engine, skills | 72 | 58.3% (42) | +2.8 [-13.1, +18.4] against 3 | no detectable effect | 32.5 | 2,085 |
| [5](../probe-results/additive-screening-of-2026-10-18-scenario-5.md) | docs, engine, hook | 72 | 51.4% (37) | -4.2 [-19.9, +11.8] against 3 | no detectable effect | 29.4 | 0 + 3,410 |
| [6](../probe-results/additive-screening-of-2026-10-18-scenario-6.md) | docs, engine, mcp | 72 | 50.0% (36) | -5.6 [-21.2, +10.5] against 3 | no detectable effect, with 29 of 72 sessions making no MCP call | 27.1 | 3,559 |
| [11](../probe-results/additive-screening-of-2026-10-18-scenario-11.md) | docs, engine, claude-md, skills, hook | 72 | 61.1% (44) | +2.8 [-13.0, +18.3] against 10 | no detectable effect | 32.9 | 11,284 + 3,410 |
| [12](../probe-results/additive-screening-of-2026-10-18-scenario-12.md) | 11 and mcp | 72 | 61.1% (44) | +0.0 [-15.6, +15.6] against 11 | no detectable effect, with 46 of 72 sessions making no MCP call | 31.7 | 14,843 + 3,410 |
| 13 ([three probes](../probe-results/additive-screening-of-2026-10-18-scenario-13-three-probes.md), [tombstone](../probe-results/additive-screening-of-2026-10-18-scenario-13-the-tombstone-probe.md)) | 11 and the rest of the harness | 72 | 63.9% (46) | +2.8 [-12.8, +18.1] against 11 | no detectable effect | 35.6 | 17,380 + 3,410 |
| [2](../probe-results/additive-screening-of-2026-10-18-scenario-2.md) | docs, claude-md | 72 | 54.2% (39) | +0.0 [-15.9, +15.9] against 1 | no detectable effect | 35.4 | 9,199 |
| [7](../probe-results/additive-screening-of-2026-10-18-scenario-7.md) | docs, engine, hook, mcp | 72 | 48.6% (35) | -2.8 [-18.6, +13.2] against 5 | no detectable effect, with 41 of 72 sessions making no MCP call | 25.2 | 3,559 + 3,410 |
| [8](../probe-results/additive-screening-of-2026-10-18-scenario-8.md) | docs, engine, claude-md, skills | 72 | 72.2% (52) | +13.9 [-1.6, +28.5] against 4 | no detectable effect | 42.3 | 11,284 |
| [9](../probe-results/additive-screening-of-2026-10-18-scenario-9.md) | docs, engine, claude-md, hook | 72 | 66.7% (48) | +15.3 [-0.8, +30.2] against 5 | no detectable effect | 33.3 | 9,199 + 3,410 |
| [10](../probe-results/additive-screening-of-2026-10-18-scenario-10.md) | docs, engine, skills, hook | 72 | 58.3% (42) | +6.9 [-9.1, +22.5] against 5 | no detectable effect | 29.0 | 2,085 + 3,410 |

The count in brackets after each rate is the satisfied sessions. The first 9 rows are the answer-once block and the last 5 are the interaction block, in the order of HW-DR-0111.

The docs layer is the one gain, and it adds no always-loaded byte, because a session reads a document only when it opens one. The two rows nearest a gain are the interaction rows 8 and 9. The lower end of each of their intervals is within 2 points of zero. The screening round separates 25 points, and the tuning and final rounds decide these rows.

**Who reached the MCP server.** HW-DR-0111 reads a `no detectable effect` with a low share of calls as a server that the sessions did not reach. In scenario 6, 43 of 72 sessions called the server. In scenario 7, 31 of 72 did, and in scenario 12, 26 of 72 did. Every other scenario holds no MCP server, and 0 of its sessions called one.

## The second table

This table gives the difference of each scenario from scenario 0 and from scenario 1. It has no verdict column, because HW-DR-0111 registers one comparison for each scenario, and this table is not that comparison.

| scenario | from scenario 0, in points [95% Newcombe] | from scenario 1, in points [95% Newcombe] |
|---|---|---|
| 1 | +38.9 [+23.7, +51.7] | none |
| 3 | +40.3 [+25.0, +53.0] | +1.4 [-14.5, +17.2] |
| 4 | +43.1 [+27.8, +55.6] | +4.2 [-11.8, +19.8] |
| 5 | +36.1 [+21.0, +49.1] | -2.8 [-18.6, +13.2] |
| 6 | +34.7 [+19.6, +47.7] | -4.2 [-19.9, +11.9] |
| 11 | +45.8 [+30.5, +58.1] | +6.9 [-9.0, +22.4] |
| 12 | +45.8 [+30.5, +58.1] | +6.9 [-9.0, +22.4] |
| 13 | +48.6 [+33.3, +60.7] | +9.7 [-6.2, +25.0] |
| 2 | +38.9 [+23.7, +51.7] | +0.0 [-15.9, +15.9] |
| 7 | +33.3 [+18.3, +46.4] | -5.6 [-21.2, +10.5] |
| 8 | +56.9 [+41.8, +68.1] | +18.1 [+2.3, +32.6] |
| 9 | +51.4 [+36.1, +63.2] | +12.5 [-3.4, +27.6] |
| 10 | +43.1 [+27.8, +55.6] | +4.2 [-11.8, +19.8] |

Every scenario differs from scenario 0 by an interval above zero. Against scenario 1, only scenario 8 has an interval that excludes zero.

## The second model in scenario 13

The harness layer of scenario 13 adds `.claude/agents/hw-explore.md`, which declares `model: claude-haiku-4-5-20251001`. Three sessions of the `cold agent` probe called that agent: `L14-additive-scenario-13-p1-r2`, `-r10` and `-r12`. The session model of each of the 72 sessions was `claude-sonnet-5`. The row above includes these three sessions. The harness layer is what scenario 13 adds, and the agent and its model are part of that layer ([the transcript](../probe-runs/additive-screening-of-2026-10-18-scenario-13-three-probes.md)). Of the three, `-r10` and `-r12` satisfied their expectation. Without the three sessions, scenario 13 is 44 of 69, 63.8%, and +2.7 [-13.1, +18.2] against scenario 11. The verdict does not change.

## Each probe, and the ceiling and the floor

Each cell is the satisfied sessions of 18 graded sessions.

| probe | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `a-cold-agent-reaches-the-governing-document-through-the-corpus-descriptor` | 0 | 8 | 2 | 4 | 3 | 5 | 1 | 7 | 8 | 3 | 3 | 3 | 1 | 3 |
| `an-agent-reaches-the-adjudication-when-the-decision-a-tombstone-links-is-superseded` | 0 | 9 | 6 | 11 | 8 | 10 | 7 | 3 | 10 | 12 | 7 | 9 | 12 | 11 |
| `a-session-follows-two-citations-from-a-record-the-task-does-not-name` | 10 | 18 | 18 | 18 | 18 | 18 | 18 | 18 | 18 | 18 | 18 | 18 | 18 | 18 |
| `a-session-records-which-obligation-an-evaluation-discharged` | 1 | 4 | 13 | 7 | 13 | 4 | 10 | 7 | 16 | 15 | 14 | 14 | 13 | 14 |

**No probe sat at the ceiling or at the floor in every scenario.** The near cases are these:

- `a-session-follows-two-citations-from-a-record-the-task-does-not-name` is at 18 of 18 in each of the 13 scenarios above scenario 0, and at 10 of 18 in scenario 0. So it can show a difference only in the comparison of scenario 1 with scenario 0. In every other comparison, and in each tuning try, it adds 18 fixed sessions to the 72 of the pooled rate.
- `a-cold-agent-reaches-the-governing-document-through-the-corpus-descriptor` and `an-agent-reaches-the-adjudication-when-the-decision-a-tombstone-links-is-superseded` are at 0 of 18 in scenario 0. That is the floor by construction, because scenario 0 holds no `docs/` (HW-DR-0111, "Scenario 0 is the floor"). The cold-agent probe is also at 1 of 18 in scenarios 6 and 12.
- `a-session-records-which-obligation-an-evaluation-discharged` is at 1 of 18 in scenario 0 and at 16 of 18 in scenario 8.

The final round runs the held-out set alone (HW-DR-0111, "The common probe set, and its split"). So it runs none of these 4 probes, and the ceiling of the two-citations probe does not reach a final verdict. The held-out probe `a-session-says-which-date-a-corrected-status-carries` has the same `answered` key. No campaign has recorded it, so no figure says whether it sits at the ceiling. HW-DR-0111 opens the set again if a held-out probe sits at the ceiling in the base of a scenario. The final round is the first round that can show it.

## Limits

- The bytes column counts the always-loaded set as the leak check assembles it. The front matter of `.claude/skills/hw-corpus/SKILL.md` does not read as YAML, because its description holds `status: current` outside quotes. So the check counts that front matter whole, 448 bytes, and the first line of its body, 38 bytes. The harness lists the skill as its name and description, 429 bytes. So the bytes of scenario 13 are 57 too high, of 17,380.
- The bytes column does not count the text that a hook injects for a tool call. Scenario 13 holds six such hooks, and what they inject depends on the calls of each session.
- The bytes column counts the hook text with the engine of this branch. That engine differs from the engine at the pin in its version number alone. No source file under `engine/crates/*/src/` changed between the two. The tree of each scenario is the pin's.
- Each cell of the probe table has 18 sessions. HW-DR-0111 says that a cell of one probe in one scenario is not a finding at that count.

## What this evaluation does not yet answer

Issue [#1776](https://github.com/headwater-ai/headwater/issues/1776) asks for more than this evaluation gives. A second change adds what rests on the raw recordings. That change makes the seven readings of the recordings. It gives one hypothesis for each layer, or says that the layer stays as it is. It marks each finding that rests on a raw recording, and it writes a tuning note on each scenario issue, #1780 to #1793. Until it lands, read this evaluation as the tables and not as the analysis of the round.
