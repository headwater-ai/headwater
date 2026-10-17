---
id: HW-DR-0111
status: current
status_since: 2026-10-17
summary: "The 0.07 campaign builds 14 scenarios by adding six layers to plain code. Each scenario has one base, one registered comparison and one verdict, over 8 probes split 4 for tuning and 4 held out."
last_verified: 2026-10-17
title: "111 — The additive campaign adds six layers to plain code in fourteen scenarios and registers one comparison each"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  governs:
    - tools/probe/ablate.sh
    - tools/probe/arms/hook/.claude/settings.json
---

# 111 — The additive campaign adds six layers to plain code in fourteen scenarios and registers one comparison each

## Context

The campaign of 2026-10-03 (#1659) removed one part at a time from the full layer. So each figure it has for a part is measured at the top of the stack. Its `absent` arm also removed four paths together, which no single-part arm removed (#1712). Eight of its eleven batch A probes sat at 30 of 30 in the `present` arm. So most of its probes could not show a gain at all.

Milestone 0.07 turns the design around. It starts from plain code and adds layers to it, one combination at a time, so each layer is also measured from the bottom. The owner asked for this on 2026-10-17. He asked in #1774 that "the output be simple to understand". He also asked that the scenarios and tests be "similarly structured across them". This record fixes the design before any session runs: the layers, the scenarios, the probes, the counts, the tables and the verdict rule.

The data half of this record is in `.headwater/probe.yml`, under `layers` and `scenarios`. The engine reads it, and `headwater probe plan --scenario <n> --delta` prints what a scenario holds. `sh tools/probe/ablate.sh --scenario <n> <workspace>` builds the tree of a scenario from that output.

The campaign is the next instrument for four open claims of spec 13: HW-OBL-0012, HW-OBL-0013, HW-OBL-0014 and HW-OBL-0016. This record reads those claims and closes none of them.

## Decision

### Six layers and the rest of the harness

A layer is a set of paths that it adds to plain code. Plain code is the present tree with the instrument removed and the path of every layer removed. So plain code holds the source of this repository, the engine source, `tools/` and the CI workflows. It holds no part of what delivers Headwater to a session.

| layer | name in the declaration | paths it adds |
|---|---|---|
| documents | `docs` | `docs/`, `engine/crates/census/fixtures/corpus.census`, `site/tutorial/index.html` |
| engine | `engine` | `.headwater/` and the built `headwater` command, `engine/target/dev-release/headwater` |
| standing instructions | `claude-md` | `CLAUDE.md` |
| skills | `skills` | the four skills and the `headwater-maintainer` agent that `headwater init --harness` installs, under `.claude/` and `.agents/skills/` |
| hook | `hook` | `.claude/hooks/intent.sh`, the `.claude/hooks/lib.sh` it sources, and a `settings.json` with the one `UserPromptSubmit` entry |
| MCP | `mcp` | `.mcp.json`, which declares `headwater mcp` |

Three placements need a reason:

- **The census fixture and the tutorial page go with the documents.** Each states what a document says, and the `documentation` tier removes both for that reason. Without them, plain code would hold part of `docs/`.
- **The built engine goes with the engine layer.** A session has no network and cannot build the engine. So a tree with the engine source and no binary has no `headwater` command, and plain code is that tree.
- **`.agents/skills/` goes with the skills.** `headwater init --harness` writes it, so an adopter receives it with the four skills.

The rest of this repository's harness is one block, `harness`, and only scenario 13 holds it. It is the six internal skills, the other eight agents, `.claude/commands`, the other six hooks with their fixtures, `.claude/run`, `.claude/tutorial`, the whole `.claude/settings.json` and `.githooks/`. It also holds the Codex copies, `AGENTS.md` and `.codex/`. It also holds the Copilot copies under `.github/`: `copilot-instructions.md`, `hooks/`, `agents/` and `prompts/`. `copilot-instructions.md` is a link to `CLAUDE.md`. So scenario 2 differs from scenario 1 by one path to the standing instructions and not three. No `headwater init --harness` writes a Codex or Copilot copy, the binding of `headwater-maintainer` in `.github/agents/` included, so none of them goes with the skills. The rest of `.github/` serves GitHub and not a session, so plain code keeps it. That is the workflows, the images, the issue template and the two community files.

The hook layer writes its own `settings.json` from `tools/probe/arms/hook/`, because the whole file wires seven hooks. Where a scenario holds the hook and the rest of the harness, as scenario 13 does, the whole file is kept. The engine admits this one pair of equal paths, and it refuses every other path that two layers claim.

### Fourteen scenarios, each with one base

Each scenario except scenario 0 names one base. A scenario differs from its base by exactly one added layer, and the engine refuses a declaration where it does not.

| scenario | issue | block | layers | base |
|---|---|---|---|---|
| 0 | #1780 | answer-once | none (plain code) | none |
| 1 | #1781 | answer-once | docs | 0 |
| 3 | #1783 | answer-once | docs, engine | 1 |
| 4 | #1784 | answer-once | docs, engine, skills | 3 |
| 5 | #1785 | answer-once | docs, engine, hook | 3 |
| 6 | #1786 | answer-once | docs, engine, mcp | 3 |
| 11 | #1791 | answer-once | docs, engine, claude-md, skills, hook | 10 |
| 12 | #1792 | answer-once | 11 and mcp | 11 |
| 13 | #1793 | answer-once | 11 and the rest of the harness | 11 |
| 2 | #1782 | interaction | docs, claude-md | 1 |
| 7 | #1787 | interaction | docs, engine, hook, mcp | 5 |
| 8 | #1788 | interaction | docs, engine, claude-md, skills | 4 |
| 9 | #1789 | interaction | docs, engine, claude-md, hook | 5 |
| 10 | #1790 | interaction | docs, engine, skills, hook | 5 |

The answer-once block answers each layer once. The interaction block measures a layer over a base that already holds another layer.

### One probe shape

Every probe of the campaign has one shape. It has one task in plain words, one target document, and one key. The grader checks the key from the recorded transcript, with no model as a judge. The issue says "from tool calls alone". An `answered` key reads the final answer of the session and not a call, so this record reads the clause as the recorded transcript. The grader checks all five forms in this way, so each form is admitted: `opened`, `not_opened`, `cited`, `patched` and `answered`. Each probe document states its task, its target and its key under three headings of the same names. The second slice of #1774 made the documents conform.

### The common probe set, and its split

The set has eight probes, and the split is fixed now. The screening round and every tuning try run the tuning set alone. The final round runs the held-out set alone, so no held-out result exists until then.

| probe | key | 2026-10-03 present arm | set |
|---|---|---|---|
| a-session-records-which-obligation-an-evaluation-discharged | patched | 24 of 30 | tuning |
| a-cold-agent-reaches-the-governing-document-through-the-corpus-descriptor | opened | 17 of 118 | tuning |
| a-session-follows-two-citations-from-a-record-the-task-does-not-name | answered | not recorded (#1718) | tuning |
| an-agent-reaches-the-adjudication-when-the-decision-a-tombstone-links-is-superseded | opened | not recorded (#1718) | tuning |
| what-a-session-writes-points-back-at-the-ruling-it-rests-on | cited | 5 of 30 | held out |
| the-pointer-this-corpus-offers-for-a-task-is-the-document-a-session-opens | opened | 7 of 118 | held out |
| a-session-says-which-date-a-corrected-status-carries | answered | not recorded (#1718) | held out |
| a-session-says-whether-a-command-or-a-hand-writes-a-governs-line | answered | not recorded (#1709) | held out |

Each set holds two recorded probes and two that no campaign has run. No probe in the set was at the ceiling of the `present` arm.

The other probes on the shelf are left out:

- **At the ceiling.** Eight probes sat at 30 of 30 in the `present` arm. They are the tombstone, the citation, the unmeasured claim and the accepted event. They are also the status in a pull request, the register, the figure on the site and the adjudication. No scenario above its base can show a gain on a probe at the ceiling. #1720 added a harder probe beside four of them, and three of those four are in the set.
- **Near the ceiling.** `a-session-finds-a-ruling-from-a-task-in-plain-words` was 113 of 118. A gain over its base could not pass five points. Its key also accepts two documents, the ruling and the evaluation that the ruling cites.
- **A second document in the key.** `a-session-answers-from-the-register-when-the-task-names-the-list-it-replaced` keys on a document that the session must not open, and that document is not its target. So it has two documents and not one. The adjudication probe, which is out for the ceiling, also keys on two documents: the decision register and the record that its Q8 row links.
- **A target that some scenarios cannot hold.** `the-authoring-skill-reaches-an-agent-that-is-about-to-write-a-governed-document` opens a file under `.claude/`, which the eight scenarios without the skills layer do not hold. Its rate there is zero by construction.
- **Superseded.** `a-session-says-how-a-governs-line-reaches-a-document` is the record of the 2026-10-03 campaign, and its successor is in the set.

### The counts, from one calculation

Every count comes from the formula that `tools/probe/campaign-dry-run.sh` states. It is the sessions per arm for two rates at a two-sided level of 0.05 and a power of 0.80, with the Fleiss continuity correction. A round pools the probes of its set into one rate per scenario, so it runs the count over the probes and rounds up.

| round | probes | sessions per scenario | repetitions per probe | the difference it separates |
|---|---|---|---|---|
| screening | the 4 tuning probes | 72 | 18 | 25 points, anywhere: 0.375 against 0.625 needs 70 |
| each tuning try, at most 3 | the 4 tuning probes | 72 | 18 | 25 points, as the screening round |
| final | the 4 held-out probes | 220 | 55 | 15 points anywhere, 0.425 against 0.575 needs 187, and 10 points from 0.80 to 0.90, which needs 219 |

So the final round separates any scenario from its base by 15 points of the pooled rate. Where the rates sit near 0.85, it separates them by 10 points. Every verdict rests on the pooled rate. A cell of one probe in one scenario has 18 or 55 sessions, and at those counts it is not a finding. Over the 14 scenarios, the plan is 1,008 screening sessions, 3,024 tuning sessions at most, and 3,080 final sessions. The dry run of the next slice of #1774 prices that total, and the owner rules on it before anything spends.

### Two tables, one verdict

The headline table has the same columns for every scenario and every round:

| scenario | layers | sessions | pass rate | difference from the base, in points, with a 95% Newcombe interval | verdict | cost per session, in cents | bytes the layers put in front of the model |
|---|---|---|---|---|---|---|---|

This record registers one comparison before the run, the difference from the base. It is the only comparison that earns a verdict. The verdict is one word. It is `gain` when the interval lies above zero, `loss` when it lies below zero, and `no detectable effect` when it holds zero.

A second table gives the difference of each scenario from scenario 0 and from scenario 1, each with its interval, and with no verdict.

### What counts as a call to the server

A call is a `tool_use` event whose tool name starts with `mcp__`. The harness gives that name to each tool of an MCP server. `tools/probe/probe-record.sh` counts calls this way, and each recorded session states the count. In 2026-10-03, 224 of 330 batch A sessions and 140 of 354 batch B sessions of the `mcp` arm made no call.

The task of a probe stays the same in every scenario, and the plain task is the fair comparison. A task that names the server would make scenario 6 differ from scenario 3 by its task and by its layer. The registered comparison would then measure two things. So each scenario that holds MCP reports the share of sessions that never called the server. A verdict of `no detectable effect` with a low share reads as a server that was not reached. It does not read as a server that did not help.

### One model, one harness

The campaign runs on `claude-sonnet-5`, the model of the 2026-10-03 records, and on Claude Code 2.1.288, the version those records name. The Headwater engine is built from the commit that the screening round starts at, and the transcript of each session names its digest. Other models are #1700's, and other harnesses are #503's, each after the final round.

## Consequences

`headwater probe plan --scenario <n> --delta` prints the layers of a scenario from `.headwater/probe.yml`, and no script parses the declaration. The engine refuses four declarations: a path that two layers claim, and a scenario that names an undeclared layer. It also refuses a scenario 0 that holds a layer, and a scenario that does not add exactly one layer to its base. `sh tools/probe/ablate.sh --diff --scenario <n> <present-tree> [<base>]` builds a scenario and its base. It prints the paths by which they differ, and it fails on a difference outside the added layer. The fixtures hold each scenario of the table against its base. Their present tree has the Codex copies, a built engine and the whole `settings.json`.

A file added to the harness later is in no layer until somebody declares it, so plain code would hold it. A test of the engine walks `.claude/`, `.agents/`, `.codex/`, `.githooks/` and `.github/`, and it fails on a file that no layer holds. In `.github/`, it skips only the five entries that serve GitHub.

This record does not run anything. The probe documents state their task, target and key under the headings `Task`, `Target` and `Key`, and the `probe` kind requires all three. The engine reads the closed set of an `answered` key under `Key` and under no other heading. A test of the engine holds each probe to one link under `Target`. For `opened` and `cited`, that link is one of the documents that the probe examines. The leak check over the text of each scenario and the one ceiling are later slices of #1774. So is the count of server calls in each generated result.

A probe at the ceiling of 2026-10-03 stays out of the set even where it sits lower in plain code. Its gain from the bottom is real and unmeasured here. The set opens again if a held-out probe sits at the ceiling in the base of a scenario. That probe can then show no gain there.
