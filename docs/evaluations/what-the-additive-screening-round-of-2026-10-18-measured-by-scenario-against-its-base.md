---
id: HW-EVAL-what-the-additive-screening-round-of-2026-10-18-measured-by-scenario-against-its-base
status: current
status_since: 2026-10-19
summary: "In screening, only the docs layer gained against its base, +38.9 points. The raw recordings of 2026-10-03 name one change to try, in the skills layer, and leave the other five layers as they are."
last_verified: 2026-10-20
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
    - HW-EVAL-what-the-paid-layer-campaign-of-2026-10-03-measured-by-arm-and-by-probe-line
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

**Where the raw recordings are.** The raw recordings of the campaign of 2026-10-03 are at `/home/james/.local/share/headwater/hw-1659/`, under `a/` and `b/`. They hold 5.5 GB on one host and are not committed. On 2026-10-19 the owner ruled: "Keep them, no end date: the raw recordings of the 2026-10-03 campaign stay on the machine with no end date." The section "What the raw recordings of 2026-10-03 show" rests on them. So does each entry of "One hypothesis for each layer" that cites a reading. **The marking rule:** each paragraph or list item that states a finding from a raw recording opens with **Raw recording.** A statement without that mark rests on the committed pages alone.

## The method

Each pass rate is the share of graded sessions that satisfied their expectation, with the Wilson 95% interval that the result pages print. The difference of two scenarios is the Newcombe hybrid score interval, built from the two Wilson intervals, as `Interval::minus` computes it in `engine/crates/probe/src/grade.rs`. The verdict follows HW-DR-0111. It is `gain` when the interval lies above zero, `loss` when it lies below zero, and `no detectable effect` when it holds zero.

The cost column is the realized cost of the attempt that recorded, divided by the graded sessions. A halt of the account's usage limit stopped some attempts, and those attempts spent more. The recorded attempts total 36,272 cents. The round spent 36,787 cents over every attempt, so the halts spent 515 cents that no row shows. [#1775](https://github.com/headwater-ai/headwater/issues/1775) carries the total for each scenario.

The bytes column is the raw UTF-8 bytes of the text that the layers put into every session. `sh tools/probe/seal.sh --leak --scenario --bytes <tree> <probe>...` measures it, over the tree of each scenario that `sh tools/probe/ablate.sh --present` and `--scenario <n>` build from the pin. The count holds the memory files and their imports, the listing of each skill, agent and command, and the definition of each MCP tool. A tool's definition is its name, description and input schema in compact JSON, as the server's `tools/list` result gives it. A skill body loads only when a session invokes the skill, so the count leaves it out, and it leaves out every document under `docs/`. Where the scenario holds the intent hook, a second figure follows the `+`. It is the mean of the text that the hook injects for the task of each of the 4 probes. That text is 3,403, 2,827, 4,036 and 3,372 bytes, and it is the same in each scenario that holds the hook. The harness's own system prompt is the same in every scenario, and the count leaves it out.

**The readings of the raw recordings.** `tools/probe/recordings.py` reads one batch of the recordings and prints one line for each event: a tool call from the stream, a `Skill` call, or an attachment from the harness's own log. Its header states the two sources, the event form and the root `/var/tmp/hw-1659/` that every path in them names. It never prints a tool result, so a file that quotes a message is not evidence of that message. The pass or fail of each session comes from the committed pages `docs/probe-results/campaign-of-2026-10-03-*.md` (batch `a`) and `campaign-of-2026-10-05-*.md` (batch `b`). Each page lists a session under the probe it graded, so the join takes the probe from the page and not from the `p` number. One session id can name a session in each batch, so the join is by batch and id. Each reading below is the reader and one filter, run from the root of this repository with `R=/home/james/.local/share/headwater/hw-1659`:

- R1: the `jq` program of `debe0c70:tools/probe/probe-record.sh`, lines 1020 to 1035, with `| length` replaced by `| .[]`, over each `$R/a/sessions/L7-*/raw.jsonl`, with `here` set to `/var/tmp/hw-1659/a/ws/<id>`, `config` to `/var/tmp/hw-1659/a/config/<id>` and `harness_dir` to `/opt/headwater-harness`.
- R2: `python3 -I tools/probe/recordings.py --batch $R/a 'L2-*-p1-*' | awk -F'\t' '$4=="Read" && index($5, "/docs/")'`, and the same for each cell, joined to the sessions that the page marks `satisfied`. A `Bash` call that runs `cat`, `sed`, `head`, `tail`, `less` or `awk` on a `docs/` path also counts as an open. A `grep` and a `Grep` do not.
- R3: `python3 -I tools/probe/recordings.py --batch $R/b 'L1-campaign-present-p1-*' | awk -F'\t' '$2=="stream"'`, read from the first `skill` line of each session to the three lines after it.
- R4: `python3 -I tools/probe/recordings.py --batch $R/a 'L3-*' 'L10-*' 'L17-*' | awk -F'\t' '$3=="hook_non_blocking_error" {print $1, $6}'`, and the same over `$R/b 'L3-*'`.
- R5: `python3 -I tools/probe/recordings.py --batch $R/a --text 'L8-campaign-present-p1-*' | awk -F'\t' '$3=="hook_success" && $6=="yes" && index($8, "Acceptance is the merge onto main") {print $1}' | sort -u`.
- R6: `python3 -I tools/probe/recordings.py --batch $R/a 'L1-campaign-present-p4-*' | awk -F'\t' '$4=="Bash" && $5 ~ /headwater[ ]+check/ {print $1}' | sort -u`, and the same over `'L2-campaign-absent-p4-*'`.
- R7: `python3 -I tools/probe/recordings.py --batch $R/b 'L1-campaign-present-p2-*' 'L5-campaign-no-claude-md-p2-*' | awk -F'\t' '$4=="EnterWorktree" {print $1}' | sort -u`, and the same for each action that `CLAUDE.md` names.

## The headline table

Only the registered comparison, a scenario against its base, earns a verdict. The screening figures are not compared across scenarios in the report ([#1779](https://github.com/headwater-ai/headwater/issues/1779)).

| scenario | layers | sessions | pass rate | difference from the base, in points, with a 95% Newcombe interval | verdict | cost per session, in cents | bytes the layers put in front of the model |
|---|---|---|---|---|---|---|---|
| [0](../probe-results/additive-screening-of-2026-10-18-scenario-0.md) | none (plain code) | 72 | 15.3% (11) | none, scenario 0 has no base | none | 86.6 | 0 |
| [1](../probe-results/additive-screening-of-2026-10-18-scenario-1.md) | docs | 72 | 54.2% (39) | +38.9 [+23.7, +51.7] against 0 | gain | 32.0 | 0 |
| [3](../probe-results/additive-screening-of-2026-10-18-scenario-3.md) | docs, engine | 72 | 55.6% (40) | +1.4 [-14.5, +17.2] against 1 | no detectable effect | 31.0 | 0 |
| [4](../probe-results/additive-screening-of-2026-10-18-scenario-4.md) | docs, engine, skills | 72 | 58.3% (42) | +2.8 [-13.1, +18.4] against 3 | no detectable effect | 32.5 | 2,085 |
| [5](../probe-results/additive-screening-of-2026-10-18-scenario-5.md) | docs, engine, hook | 72 | 51.4% (37) | -4.2 [-19.9, +11.8] against 3 | no detectable effect | 29.4 | 0 + 3,410 |
| [6](../probe-results/additive-screening-of-2026-10-18-scenario-6.md) | docs, engine, mcp | 72 | 50.0% (36) | -5.6 [-21.2, +10.5] against 3 | no detectable effect, with 29 of 72 sessions making no MCP call | 27.1 | 5,013 |
| [11](../probe-results/additive-screening-of-2026-10-18-scenario-11.md) | docs, engine, claude-md, skills, hook | 72 | 61.1% (44) | +2.8 [-13.0, +18.3] against 10 | no detectable effect | 32.9 | 11,284 + 3,410 |
| [12](../probe-results/additive-screening-of-2026-10-18-scenario-12.md) | 11 and mcp | 72 | 61.1% (44) | +0.0 [-15.6, +15.6] against 11 | no detectable effect, with 46 of 72 sessions making no MCP call | 31.7 | 16,297 + 3,410 |
| 13 ([three probes](../probe-results/additive-screening-of-2026-10-18-scenario-13-three-probes.md), [tombstone](../probe-results/additive-screening-of-2026-10-18-scenario-13-the-tombstone-probe.md)) | 11 and the rest of the harness | 72 | 63.9% (46) | +2.8 [-12.8, +18.1] against 11 | no detectable effect | 35.6 | 17,380 + 3,410 |
| [2](../probe-results/additive-screening-of-2026-10-18-scenario-2.md) | docs, claude-md | 72 | 54.2% (39) | +0.0 [-15.9, +15.9] against 1 | no detectable effect | 35.4 | 9,199 |
| [7](../probe-results/additive-screening-of-2026-10-18-scenario-7.md) | docs, engine, hook, mcp | 72 | 48.6% (35) | -2.8 [-18.6, +13.2] against 5 | no detectable effect, with 41 of 72 sessions making no MCP call | 25.2 | 5,013 + 3,410 |
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

## What the raw recordings of 2026-10-03 show

These readings come from the raw recordings of the paid-layer campaign of 2026-10-03, which [its evaluation](what-the-paid-layer-campaign-of-2026-10-03-measured-by-arm-and-by-probe-line.md) reads by arm and by probe. The files are not committed. They hold 5.5 GB on one host, and the owner ruled on 2026-10-19 that they stay there with no end date. A reader of this repository cannot derive them again. That campaign took one component away from the whole harness in each arm, so its arms are not the scenarios of this round. A reading shows what the sessions of 2026-10-03 did, and it is a lead for a hypothesis, not a measured effect of a layer.

Each batch names its sessions `L<line>-<arm>-p<probe>-r<repeat>`. Each denominator below is the graded sessions of one cell, from the committed pages.

### R1. The 137 outside-workspace paths of line 7

Cell: `a/L7`, the `documentation-absent` arm of the four sufficiency probes, 120 sessions.

**Raw recording.** The recorder's classifier at `debe0c70`, run again with each path kept, gives 137 paths over 120 sessions. In 120 of 120 sessions, the count is equal to the count that the session's `record.md` states. 56 sessions name 0 paths, 36 name 1, 10 name 2, 3 name 3, 9 name 4, 3 name 5 and 3 name 7.

**Raw recording.** The 137 paths fall into five groups:

| group | paths | sessions | examples |
|---|---|---|---|
| the tree of the batch above the workspace | 32 | 24 | `/var/tmp/hw-1659`, `/var/tmp/hw-1659/a/config` |
| a path that the confinement binds and the classifier leaves out | 11 | 10 | `/opt/headwater-egress`, `/opt` |
| a directory of the host | 6 | 3 | `/root`, `/home` |
| a fragment of a path in a filter | 59 | 44 | `grep -v "/target/"`, `find . -path ./site -prune` |
| the text of a pattern or a program | 29 | 19 | `awk '/^kinds:/{f=1}'`, `grep -v "/\.git/"` |

**Raw recording.** The classifier missed three patterns:

- **Raw recording.** It counts `/opt/headwater-egress` as a path that no file of the host was readable at. The confinement binds that directory read-only, because the socket of the egress proxy is in it. So for these 10 paths the sentence of the transcript is false.
- **Raw recording.** It counts text that is not a path. 88 of the 137 paths, in the last two groups, are a fragment in a filter or the text of a pattern. No session opened them.
- **Raw recording.** It does not count a search from the root. 68 of the 120 sessions ran `find /` at least once, in 137 commands. The classifier reads `/` as bound, so none of these commands adds to the count. 25 of the 68 sessions have a count of 0 in their transcripts. A path that opens with `~` is not counted either, and 5 sessions name one.

These are defects of the recorder that [HW-OBL-0237](../obligations/0237-eighteen-findings-that-the-paid-layer-campaign-of-run-20261003-1026-left-in-the-probe-recorder-and-the-campaign-driver.md) does not list. This evaluation does not fix the recorder.

### R2. What the passing sessions of the absent arm opened

Cells: the `absent` arm, which held the documents and none of the harness. Passing sessions, of 30 in batch `a` and of 118 in batch `b`: tombstone 30, citation 30, unmeasured claim 29, discharge 7 (`a/L2`). Accepted event 23, status in a pull request 30 (`a/L9`). Adjudication 30, register 30, governs line 26, figure on the site 29, ruling cited 6 (`a/L16`). Cold agent 36, finds a ruling 115, pointer 1 (`b/L2`). That is 422 passing sessions over 14 probes.

**Raw recording.** No document was opened by the passing sessions of more than 4 of the 14 probes. In 13 of the 14 probes, the document that the most passing sessions opened is a document of that probe's own task. The documents that 3 or 4 probes share are indexes and parts of the specification:

| document | shelf | probes | passing sessions that opened it |
|---|---|---|---|
| `docs/decisions/README.md` (no identifier) | decisions | 4 | 12 |
| HW-SPEC-taxonomy-model | spec | 4 | 11 |
| `docs/taxonomies/decision-record/bundle.yml` (no identifier) | taxonomies | 4 | 9 |
| HW-EVAL-why-corpus-counts-are-derived | evaluations | 3 | 109 |
| HW-DR-0049 | decisions | 3 | 86 |
| HW-DR-0039 | decisions | 3 | 39 |
| HW-DR-0097 | decisions | 3 | 31 |
| HW-REG-open-obligations | spec | 3 | 15 |
| HW-DR-0104 | decisions | 3 | 13 |
| HW-LIB-decision-record | taxonomies | 3 | 7 |
| HW-SPEC-authoring-and-lifecycle | spec | 3 | 6 |
| HW-OBL-0142 | obligations | 3 | 5 |
| `docs/spec/README.md` (no identifier) | spec | 3 | 3 |

**Raw recording.** So the gain of the documents does not pass through one document. It passes through the governing document of each task.

### R3. What a failing cold agent loaded when it held the skills

Cells: the `cold agent` probe in batch `b`. The `present` arm passed 17 of 118, the `no-skills` arm 33 of 118 and the `absent` arm 36 of 118. The probe passes when a session opens `docs/spec/12-check-layer.md`, and its task is to add a rule to the engine.

**Raw recording.** 86 of the 101 failing `present` sessions called the `Skill` tool, and 72 of them called `headwater-taxonomy`. The first skill was `headwater-taxonomy` in 45 sessions, `headwater-orient` in 32, `headwater-engine` in 7 and `hw-run-policy` in 2. In 61 of the 86, one of the next 3 calls named `.headwater/` or `taxonomy-source`. In 48 of the 86, one of the next 3 calls was another skill. In 3 of the 86, one of the next 3 calls named a path under `docs/`. Only 1 of the 101 failing sessions named spec 12 in a call, and all 17 passing sessions did.

**Raw recording.** The other two arms fail in the same place. All 85 failing `no-skills` sessions and 77 of the 82 failing `absent` sessions named `.headwater/packages` or `taxonomy-source`, and none of them named spec 12. So a failing cold agent goes to the declaration of the rules and not to the document that governs them, with the skills or without them. The skills add one thing: the `headwater-taxonomy` description, "when a check must start or stop reading something", matches the task, and 72 of 101 failing `present` sessions followed it.

**Raw recording.** The `no-skills` arm did not hold the skills, and its listing did not name them. Yet 55 of its 118 cold-agent sessions called `Skill` with the name of a skill of this repository, which `CLAUDE.md` names in its table. The recordings do not show whether the harness loaded a skill for such a call, because the reader prints no tool result.

### R4. Whether the error of a missing hook reached the no-hook sessions

Cells: the `no-hook` arm, `a/L3` 120, `a/L10` 60, `a/L17` 150 and `b/L3` 354 sessions, 684 in all.

**Raw recording.** The harness log of each of the 684 sessions holds at least one `hook_non_blocking_error` attachment for the `UserPromptSubmit` hook, 728 attachments in all. None of the 728 has the `rendered` key, so the harness showed the model none of them. A search of the 120 `a/L3` streams for the text "hook script not found" finds it in 20 streams. Each of the 20 is the text of a file that the session read, not the error.

**Raw recording.** The `no-hook` arm removed the script of the intent hook and nothing else. The hooks before a `Read` or an `Edit` still ran. They put text in front of the model in 186 of the 684 sessions. That is 38 in `a/L3`, 7 in `a/L17`, 0 in `a/L10` and 141 in `b/L3`.

### R5. The accepted event and the figure on the site, with the hook and without it

Cells: `accepted event`, `present` 30 of 30 (`a/L8`) and `no-hook` 7 of 30 (`a/L10`). `figure on the site`, `present` 30 of 30 (`a/L15`) and `no-hook` 22 of 30 (`a/L17`).

**Raw recording.** Each of the 60 `present` sessions made no tool call. In each of them, the text that the intent hook showed the model held the summary of the governing document, and that summary states the answer. For `accepted event` it is [HW-DR-0034](../decisions/0034-q34-whether-acceptance-means-merged-to-main-and-what-an-agent-may-write-before-that.md), "Acceptance is the merge onto main", in 30 of 30 sessions. For `figure on the site` it is [HW-DR-0097](../decisions/0097-a-figure-on-a-hand-built-page-is-measured-when-the-site-is-published-and-the-committed-page-carries-none.md), in 30 of 30 sessions.

**Raw recording.** Without the hook, the passing and failing sessions did the same thing. 30 of 30 `accepted event` sessions and 27 of 30 `figure on the site` sessions made no tool call. They differ only in the word that they answered: `merge` or `stamp`, and `published` or `committed`, as the committed pages state. So the gain of the hook on these two probes was the answer in a summary line, not a route to a document.

### R6. Discharge, with the harness and without it

Cells: `discharge`, `present` 24 of 30 (`a/L1`) and `absent` 7 of 30 (`a/L2`).

**Raw recording.** 22 of the 24 passing `present` sessions ran `headwater check`, and 0 of the 6 failing ones did. 8 of the 24 named `reciprocity` in a later call. The reader prints no tool result, so it cannot show whether a session read `relation.reciprocity.missing` in the output of the check.

**Raw recording.** No `absent` session ran the engine, the 7 that passed included. Passing and failing `absent` sessions opened the same documents: the evaluation on governs edges and HW-OBL-0104, in each of the 30. 22 of the 23 failing sessions made an edit, so they wrote an answer without the check that would have named the missing half.

### R7. Which sentence of CLAUDE.md a session acted on

Cells: `finds a ruling`, `present` 113 of 118 (`b/L1`) and `no-claude-md` 100 of 118 (`b/L5`). `ruling cited`, `present` 5 of 30 (`a/L15`) and `no-claude-md` 13 of 30 (`a/L19`). `discharge`, `present` 24 of 30 (`a/L1`) and `no-claude-md` 25 of 30 (`a/L5`).

The method matches each sentence of `CLAUDE.md` that names an action to the calls that perform it.

**Raw recording.** Every `present` session has the `instructions` attachment, 118 of 118 in `b/L1` and 30 of 30 in `a/L15`. No `no-claude-md` session has it, 0 of 118 in `b/L5` and 0 of 30 in `a/L19`.

**Raw recording.** Only one sentence has an action that `present` sessions performed and `no-claude-md` sessions never did. It is line 7, "Call `EnterWorktree` at the start of every session". In `finds a ruling`, 25 of 113 passing and 0 of 5 failing `present` sessions called it. In `ruling cited`, 29 of 30 `present` sessions called it, and 24 of those 29 failed.

**Raw recording.** Each other action that `CLAUDE.md` names was taken in the `no-claude-md` arm at about the same share or a higher one. `headwater-orient` was called by 16 of 118 `present` and 64 of 118 `no-claude-md` sessions in `finds a ruling`. In `ruling cited`, the passing `no-claude-md` sessions ran `headwater check` (9 of 13), `headwater new` (9 of 13) and `headwater-authoring` (10 of 13). The failing ones did so in 1, 1 and 2 of 17. In `discharge`, 16 of 25 passing and 0 of 5 failing `no-claude-md` sessions ran `headwater check`. So the recordings do not show a sentence of `CLAUDE.md` that carries a pass.

## One hypothesis for each layer

Each entry reads one registered comparison. The cells are the satisfied sessions of 18, base then scenario, for the probes `cold agent`, `adjudication`, `two citations` and `discharge`. HW-DR-0111 sets a tuning try at 72 sessions, which separates 25 points. One probe is 18 of the 72 sessions, so a change that moves one probe from 0 to 18 moves the pooled rate by 25 points. An expected effect well under that is a reason to leave a layer.

### Documents, scenario 1 against 0

+38.9 [+23.7, +51.7], gain. The cells moved 0 to 8, 0 to 9, 10 to 18 and 1 to 4.

**Left as it is.** This is the one gain of the round, and it adds no always-loaded byte.

**Raw recording.** R2 shows that the passing sessions open the governing document of each task, and that no document serves more than 4 of 14 probes. So no edit to one document is a change that this round supports.

### Engine, scenario 3 against 1

+1.4 [-14.5, +17.2], no detectable effect. The cells moved 8 to 4, 9 to 11, 18 to 18 and 4 to 7.

**Left as it is.** The cells move in both directions. The largest move is 4 sessions of 18, about 6 points on the pooled rate, against the 25 points that a tuning try separates.

**Raw recording.** R6 shows that the engine helps `discharge` when a session runs it: 22 of 24 passing sessions ran `headwater check`. What makes a session run it is a skill or the standing instructions, not the engine. No reading names a change to the engine itself.

### Skills, scenario 4 against 3

+2.8 [-13.1, +18.4], no detectable effect. The cells moved 4 to 3, 11 to 8, 18 to 18 and 7 to 13.

**The hypothesis.** Add one sentence to the description of `headwater-taxonomy` that sends a session which adds a rule to the check layer, spec 12, before the package source. The expected direction is up, on the `cold agent` probe.

**Raw recording.** R3 shows that 72 of 101 failing `cold agent` sessions with the skills called `headwater-taxonomy`. Its description matches the task, and after it the sessions went to the package source and not to spec 12.

The expected effect is small. The best `cold agent` cell of the round is 8 of 18, so the change can add about 5 sessions to scenario 4, about 7 points on the pooled rate. That is well under the 25 points a tuning try separates. So a try of this change alone can rule out a large effect, and it cannot confirm this one. It is the only change to a layer that a reading names.

### Hook, scenario 5 against 3

-4.2 [-19.9, +11.8], no detectable effect. The cells moved 4 to 5, 11 to 10, 18 to 18 and 7 to 4.

**Left as it is.** No cell moves by more than 3 sessions of 18, about 4 points on the pooled rate, against the 25 points that a tuning try separates. The readings below name no change to the hook that is not a leak.

**Raw recording.** R5 shows that the gain of the hook in 2026-10-03 came from the answer in the summary of the governing document, which the hook printed. An answer in the text that a layer loads is a leak, and the leak check of this round exists to refuse it. So that gain is not one to tune toward, and no reading names another change to the hook.

**Raw recording.** R4 shows that the error of a missing hook did not reach the model in any `no-hook` session. So that arm of 2026-10-03 was not marked by the error.

### MCP, scenario 6 against 3

-5.6 [-21.2, +10.5], no detectable effect, with 29 of 72 sessions making no MCP call. The cells moved 4 to 1, 11 to 7, 18 to 18 and 7 to 10.

**Left as it is.** HW-DR-0111 reads this row as a server that many sessions did not reach. No reading of this evaluation reads the MCP arm, so nothing measured says in which direction the rate moves when more sessions reach the server. The two cells that moved most moved down, by 3 and 4 sessions of 18, about 4 and 6 points on the pooled rate, against the 25 points that a tuning try separates.

### Standing instructions, scenario 11 against 10

+2.8 [-13.0, +18.3], no detectable effect. The cells moved 3 to 3, 7 to 9, 18 to 18 and 14 to 14.

**Left as it is.** At this comparison the base already holds `discharge` at 14 of 18, and no cell moves by more than 2 sessions, about 3 points on the pooled rate, against the 25 points that a tuning try separates. The two interaction rows that add the same layer are the nearest rows to a gain: scenario 8 against 4, +13.9, and scenario 9 against 5, +15.3. `discharge` rises each time the layer is added: 4 to 13 (2 against 1), 13 to 16 (8 against 4), 4 to 15 (9 against 5) and 14 to 14 (11 against 10). The tuning and final rounds decide these rows, and a cell of 18 is not a finding.

**Raw recording.** R7 shows no sentence of `CLAUDE.md` that carries a pass. The one sentence that only its sessions acted on, line 7, was acted on by passing and failing sessions alike. R7 also shows that 16 of 25 passing `discharge` sessions without `CLAUDE.md` ran `headwater check`. So the file is not the only route to the engine, and no one sentence to change is named.

## The discharge ladder

`discharge` is at 14 of 18 in scenario 11. Scenarios 8 (16), 9 (15), 10 (14) and 13 (14) hold it at that rate or above. Of these, scenarios 8, 9 and 10 have the fewest layers, four each. Scenario 10 (docs, engine, skills, hook) puts the fewest bytes in front of the model, 2,085 and 3,410 from the hook. Scenarios 2 (docs, claude-md), 4 (docs, engine, skills) and 12 are at 13, one session below. Scenario 4 has three layers and 2,085 bytes. HW-DR-0111 says that a cell of 18 is not a finding, so a difference of one session between these scenarios is not a result. The smallest scenario that holds the rate of scenario 11 is scenario 10, and scenario 4 is one session short of it with one layer fewer.

## The tuning note on each scenario issue

Each sentence below is the tuning note on the issue of its scenario. It agrees with the entry above for the layer that the scenario adds.

| scenario | issue | the one sentence |
|---|---|---|
| 0 | [#1780](https://github.com/headwater-ai/headwater/issues/1780) | Scenario 0 holds no layer, so nothing is tuned here, and it stays the floor of the round. |
| 1 | [#1781](https://github.com/headwater-ai/headwater/issues/1781) | Leave the documents layer as it is, because it is the one gain of the round and no single document carries it (R2). |
| 3 | [#1783](https://github.com/headwater-ai/headwater/issues/1783) | Leave the engine layer as it is, because no reading names a change to the engine and no cell moved by more than 4 sessions of 18. |
| 4 | [#1784](https://github.com/headwater-ai/headwater/issues/1784) | Try one sentence in the description of `headwater-taxonomy` that sends a session which adds a rule to spec 12 first, expected to raise the `cold agent` probe by about 7 points pooled (R3). |
| 5 | [#1785](https://github.com/headwater-ai/headwater/issues/1785) | Leave the hook layer as it is, because its gain in 2026-10-03 was the answer in a summary line, which is a leak and not a route (R5). |
| 6 | [#1786](https://github.com/headwater-ai/headwater/issues/1786) | Leave the MCP layer as it is, because 29 of 72 sessions made no MCP call and no reading says which way the rate moves when they do. |
| 11 | [#1791](https://github.com/headwater-ai/headwater/issues/1791) | Leave the standing-instructions layer as it is, because no sentence of `CLAUDE.md` carries a pass in the recordings (R7) and no cell moved by more than 2 sessions. |
| 12 | [#1792](https://github.com/headwater-ai/headwater/issues/1792) | Leave the MCP layer as it is, because 46 of 72 sessions made no MCP call and the row is +0.0 against scenario 11. |
| 13 | [#1793](https://github.com/headwater-ai/headwater/issues/1793) | Nothing is to be tuned here, because the rest of the harness is not one of the six layers and no reading names a change to it. |
| 2 | [#1782](https://github.com/headwater-ai/headwater/issues/1782) | Leave the standing-instructions layer as it is, because the row is +0.0 against scenario 1 and no sentence of `CLAUDE.md` carries a pass (R7). |
| 7 | [#1787](https://github.com/headwater-ai/headwater/issues/1787) | Leave the MCP layer as it is, because 41 of 72 sessions made no MCP call and the row is -2.8 against scenario 5. |
| 8 | [#1788](https://github.com/headwater-ai/headwater/issues/1788) | Leave the standing-instructions layer as it is, because +13.9 over scenario 4 is an interaction that the tuning and final rounds decide, not a change. |
| 9 | [#1789](https://github.com/headwater-ai/headwater/issues/1789) | Leave the standing-instructions layer as it is, because +15.3 over scenario 5 is an interaction that the tuning and final rounds decide, not a change. |
| 10 | [#1790](https://github.com/headwater-ai/headwater/issues/1790) | Try the same sentence in `headwater-taxonomy` as scenario 4, because this row adds the skills to the hook and its `cold agent` cell fell from 5 to 3 (R3). |

## Limits

- The bytes column counts the always-loaded set as the leak check assembles it. The front matter of `.claude/skills/hw-corpus/SKILL.md` does not read as YAML, because its description holds `status: current` outside quotes. So the check counts that front matter whole, 448 bytes, and the first line of its body, 38 bytes. The harness lists the skill as its name and description, 429 bytes. So the bytes of scenario 13 are 57 too high, of 17,380.
- The MCP figure counts each tool definition as compact JSON. The harness can frame the definitions in a different encoding, so the bytes that reach the model can differ from 5,013 by that framing. The 7 definitions are 447, 828, 795, 364, 745, 789 and 1,045 bytes.
- The bytes column does not count the text that a hook injects for a tool call. Scenario 13 holds six such hooks, and what they inject depends on the calls of each session.
- The bytes column counts the hook text with the engine of this branch. That engine differs from the engine at the pin in its version number alone. No source file under `engine/crates/*/src/` changed between the two. The tree of each scenario is the pin's.
- Each cell of the probe table has 18 sessions. HW-DR-0111 says that a cell of one probe in one scenario is not a finding at that count.
- The seven readings read the campaign of 2026-10-03, whose arms take one component away from the whole harness. The scenarios of this round add one layer to a smaller base. So a reading names a lead for a hypothesis, and it does not measure the effect of a layer in this round.
- A reading counts sessions and tests no difference. The counts are of one cell each, 30 sessions in batch `a` and 118 in batch `b`.
- The reader prints calls and attachments, and never a tool result or the text of the model. So R6 cannot show what a session read in the output of `headwater check`, and R3 cannot show whether the harness loaded a skill that its listing did not name.
- The workspaces of the recordings are gone. So a reading shows which path a session named, and not what the file at that path held.
- R2 counts an open as a `Read`, or as a `Bash` call that runs `cat`, `sed`, `head`, `tail`, `less` or `awk` on a path under `docs/`. A `grep` that names a document is not an open in R2, although the grader counts a call that names the document. R3 counts any call that names spec 12, as the grader does.
- **Raw recording.** R1 runs the classifier with the path of the run's own log directory unknown. Each path under `/tmp` is bound, so a log directory there changes no count, and each of the 120 counts equals its transcript.
