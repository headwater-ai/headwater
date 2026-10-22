---
id: HW-DR-0111
status: current
status_since: 2026-10-17
summary: "The 0.07 campaign builds 14 scenarios by adding six layers to plain code. Each scenario has one base, one registered comparison and one verdict, over 9 probes split 5 for tuning and 4 held out. Only the skills, the hook and MCP are tuned."
last_verified: 2026-10-22
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
    - tools/probe/additive-campaign.spec
    - tools/probe/additive-screening.spec
    - tools/probe/try.sh
    - tools/probe/additive-launch.env
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
| skills | `skills` | the four skills and the `headwater-maintainer` agent that `headwater init --harness` installs, under `.claude/` and `.agents/skills/`. Also the engine's source copies of that text, `engine/crates/cli/harness/skills/` and `engine/crates/cli/harness/agents/` |
| hook | `hook` | `.claude/hooks/intent.sh`, the `.claude/hooks/lib.sh` it sources, and a `settings.json` with the one `UserPromptSubmit` entry |
| MCP | `mcp` | `.mcp.json`, which declares `headwater mcp` |

Four placements need a reason:

- **The census fixture and the tutorial page go with the documents.** Each states what a document says, and the `documentation` tier removes both for that reason. Without them, plain code would hold part of `docs/`.
- **The built engine goes with the engine layer.** A session has no network and cannot build the engine. So a tree with the engine source and no binary has no `headwater` command, and plain code is that tree.
- **`.agents/skills/` goes with the skills.** `headwater init --harness` writes it, so an adopter receives it with the four skills.
- **The engine's source copies of the skills text go with the skills.** `engine/crates/cli/harness/` holds the text that `include_str!` embeds in the binary, and each file is its `.claude/` copy without the marker line that `headwater init --harness` adds. A scenario without the skills layer that keeps these files still gives a session the skills text. The campaign builds the binary from its own checkout and not from a scenario tree. So the engine source in a tree does not need them.

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

### Three layers are tuned, and three are frozen

On 2026-10-22 the owner ruled on #1777 that the tuning round tunes three layers: the skills, the hook and MCP. Each is tuned against scenario 3. So a try of the skills compares scenario 4 with scenario 3. A try of the hook compares scenario 5, and a try of MCP compares scenario 6. The documents, the engine and `CLAUDE.md` are frozen, and no tuning try changes them. `CLAUDE.md` is measured once, in the final round, on top of the layers that the tuning round keeps. Every scenario keeps its base and its registered comparison.

### One probe shape

Every probe of the campaign has one shape. It has one task in plain words, one target document, and one key. The grader checks the key from the recorded transcript, with no model as a judge. The issue says "from tool calls alone". An `answered` key reads the final answer of the session and not a call, so this record reads the clause as the recorded transcript. The grader checks all five forms in this way, so each form is admitted: `opened`, `not_opened`, `cited`, `patched` and `answered`. Each probe document states its task, its target and its key under three headings of the same names. The second slice of #1774 made the documents conform.

### The common probe set, and its split

The set has nine probes: five for tuning and four held out. The owner fixed the split on 2026-10-17, and on 2026-10-22 he changed the tuning set (#1833). The screening round and every tuning try run the tuning set alone. The final round runs the held-out set alone, so no held-out result exists until then.

| probe | key | 2026-10-03 present arm | set |
|---|---|---|---|
| a-session-records-which-obligation-an-evaluation-discharged | patched | 24 of 30 | tuning |
| an-agent-reaches-the-adjudication-when-the-decision-a-tombstone-links-is-superseded | opened | not recorded (#1718) | tuning |
| a-session-that-changes-the-recorder-reaches-the-recorder-contract | opened | not recorded (#1833) | tuning |
| a-session-that-moves-a-shelf-reaches-the-relocation-guide | opened | not recorded (#1833) | tuning |
| a-session-that-changes-a-verb-s-default-reaches-the-verb-s-interface-contract | opened | not recorded (#1833) | tuning |
| what-a-session-writes-points-back-at-the-ruling-it-rests-on | cited | 5 of 30 | held out |
| the-pointer-this-corpus-offers-for-a-task-is-the-document-a-session-opens | opened | 7 of 118 | held out |
| a-session-says-which-date-a-corrected-status-carries | answered | not recorded (#1718) | held out |
| a-session-says-whether-a-command-or-a-hand-writes-a-governs-line | answered | not recorded (#1709) | held out |

The tuning set holds one probe that the campaign of 2026-10-03 recorded, and the held-out set holds two. No probe in the set was at the ceiling of the `present` arm.

Two probes left the tuning set on 2026-10-22, and each stays on the shelf as a record:

- **The two-citations probe**, `a-session-follows-two-citations-from-a-record-the-task-does-not-name`. It sat at 18 of 18 in each of the 13 scenarios that hold `docs/`. So it added the same passes to both sides of each tuning comparison. Plain code also passed it: in scenario 0, 11 of 18 sessions answered from comments in the code.
- **The cold-agent probe**, `a-cold-agent-reaches-the-governing-document-through-the-corpus-descriptor`. Its task asks for a rule that already exists, `facet.value.blank`, so the task is defective. Each scenario-0 session that answered said so.

The three probes that replace them key on `opened`. Each task asks for a change to the code or to the taxonomy. The key reads whether the session opened the document that governs that change. The targets are the recorder contract, the relocation guide and the interface contract of `headwater route`. Proposal 1, the first comment on #1833, is the design, and it states the rate that each probe expects in scenario 3.

#1835 adds a third set, `code`, with a round of its own. It is an addition, and it changes neither the tuning set nor the held-out set.

The other probes on the shelf are left out:

- **At the ceiling.** Eight probes sat at 30 of 30 in the `present` arm. They are the tombstone, the citation, the unmeasured claim and the accepted event. They are also the status in a pull request, the register, the figure on the site and the adjudication. No scenario above its base can show a gain on a probe at the ceiling. #1720 added a harder probe beside four of them, and two of those four are in the set.
- **Near the ceiling.** `a-session-finds-a-ruling-from-a-task-in-plain-words` was 113 of 118. A gain over its base could not pass five points. Its key also accepts two documents, the ruling and the evaluation that the ruling cites.
- **A second document in the key.** `a-session-answers-from-the-register-when-the-task-names-the-list-it-replaced` keys on a document that the session must not open, and that document is not its target. So it has two documents and not one. The adjudication probe, which is out for the ceiling, also keys on two documents: the decision register and the record that its Q8 row links.
- **A target that some scenarios cannot hold.** `the-authoring-skill-reaches-an-agent-that-is-about-to-write-a-governed-document` opens a file under `.claude/`, which the eight scenarios without the skills layer do not hold. Its rate there is zero by construction.
- **Superseded.** `a-session-says-how-a-governs-line-reaches-a-document` is the record of the 2026-10-03 campaign, and its successor is in the set.

### The counts, from one calculation

Every count comes from the formula that `tools/probe/campaign-dry-run.sh` states. It is the sessions per arm for two rates at a two-sided level of 0.05 and a power of 0.80, with the Fleiss continuity correction. A round pools the probes of its set into one rate per scenario, so it runs the count over the probes and rounds up.

| round | probes | sessions per scenario | repetitions per probe | the difference it separates |
|---|---|---|---|---|
| screening | the 5 tuning probes | 70 | 14 | 25 points, anywhere: 0.375 against 0.625 needs 70 |
| each tuning try, at most 3 | the 5 tuning probes | 70 | 14 | 25 points, as the screening round |
| final | the 4 held-out probes | 220 | 55 | 15 points anywhere, 0.425 against 0.575 needs 187, and 10 points from 0.80 to 0.90, which needs 219 |

So the final round separates any scenario from its base by 15 points of the pooled rate. Where the rates sit near 0.85, it separates them by 10 points. Every verdict rests on the pooled rate. A cell of one probe in one scenario has 14 or 55 sessions, and at those counts it is not a finding. Over the 14 scenarios, the plan is 980 screening sessions, 2,940 tuning sessions at most, and 3,080 final sessions. The dry run prices that total. The owner approves each round before it spends, and the Consequences state what he approved.

The bases of the tuned comparisons are measured again on the new tuning set. On 2026-10-22 the owner ruled that a base and a confirmation each run 145 sessions, 29 repetitions of each of the five tuning probes. The dry run does not compute that count yet, and #1838 is the issue for it.

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

This record does not run anything. The probe documents state their task, target and key under the headings `Task`, `Target` and `Key`, and the `probe` kind requires all three. The engine reads the closed set of an `answered` key under `Key` and under no other heading. A test of the engine holds each probe to one file under `Target`. A file under `docs/` is a link. A file outside `docs/` is a path in one code span, because the site serves no file there. The test counts each link and each path toward that one file, and it resolves a path before it judges it. For `opened` and `cited`, that file is one of the documents that the probe examines. Each generated result states the calls that each graded session made to an MCP server, and the share of sessions that made none.

`.headwater/probe.yml` declares the `additive` tier, its arms `scenario-0` to `scenario-13`, the two sets and the three rounds. `headwater probe plan --arm scenario-<n> --set <name>` plans one scenario over one set. `tools/probe/additive-campaign.spec` holds one line for each scenario and round. `sh tools/probe/campaign.sh --dry-run --spec tools/probe/additive-campaign.spec` computes each round's count from the rates of the round. It prints this total:

    round screening: 980 sessions, $490.00
    round tuning: 2940 sessions, $1470.00 (3 tries at most)
    round final: 3080 sessions, $1540.00
    total: 7000 sessions, $3500.00

That is at the declared 50 cents a session. At the 35.0 cents a session of 2026-10-03, the total is $2,450.00.

**The owner approved the screening round at $700.** On 2026-10-18 he ruled on #1775: "restart doesn't reset count, but raise ceiling to $700". Over the first tuning set of four probes, the plan of the round was 1,008 sessions and $504.00. So $196.00 was room for a restart, which is 392 sessions at 50 cents. A restart does not reset the count: the money that every attempt of the screening round spends counts toward the one $700.

**The owner approved the tuning round at $1,512.00.** On 2026-10-19 he ruled on #1777: "Approve the full $1,512: tuning may spend up to $1,512." The sum covers every attempt and every layer. He also ruled that every layer gets at least one paid tuning try. On 2026-10-22 he narrowed that ruling: only the skills, the hook and MCP are tuned, and the documents try 1 is withdrawn (#1833). The tuning runs one layer at a time. The owner accepts the plan of each layer before any spend on that layer. Since #1819, a mechanism holds this rule, and the fourth mechanism below states it. Since #1821, `tools/probe/try.sh spent <root>` derives the total across tuning batches from the files, and the fifth mechanism below states it. The final round, $1,540.00, is not approved, and it waits for a ruling of its own.

Five mechanisms hold these rulings. The first three hold the spend of one batch or one root, and the fifth holds the total of the tuning round. The additive tier's ceiling in `.headwater/probe.yml` is 70000 cents, and it bounds one batch directory. Each round under `rounds:` declares `approved_cents`: 70000 for the screening round, 151200 for the tuning round and 0 for the final round. `tools/probe/campaign.sh` refuses a line of a round whose approval is 0, with exit 5, before it builds a tree or asks the harness anything. The ceiling alone cannot do this, because the plan prices one line at a time, and a final line of 220 sessions costs $110.00. `tools/probe/additive-screening.spec` holds the 15 screening lines alone, so the paid run names a spec that holds no refused line. `tools/probe/campaign-drive.sh` gives each batch the cap less what every other batch directory under its root spent. So a restart moves the old batch directory aside under the same root. The new batch then gets $700 less what every earlier attempt spent, less the overshoot margin.

The fourth mechanism holds the rule of one layer at a time (#1819). The tuning round under `rounds:` declares `open`. Its value is `none`, or the one layer whose plan the owner accepted. An open layer also names its scenario, the try, the cap of the try, the URL of the acceptance and the commit of the change. `python3 tools/probe/declared.py .headwater/probe.yml open` prints the value. It refuses a scenario that does not add exactly the named layer over its base. It also refuses a cap above the round's `approved_cents` and a try past the round's `tries`. `tools/probe/campaign.sh` refuses a tuning line with exit 5 when `open` is `none`. It also refuses a tuning line of a scenario other than the open one, and it names the open layer and scenario. Both refusals occur at the same point as the round approval, before the batch builds a tree or asks the harness anything. Try 1 of the documents layer scored 32 of 72 sessions, and the owner did not keep it. The ruling of 2026-10-22 withdrew that try with the layer. No other layer has an accepted plan, so `open` is `none`.

`.headwater/probe.yml` also declares `base_invalidators`, the paths whose change can make a measured base stale. Its six entries are the engine source, `engine/crates/*/src/**`, and five scripts under `tools/probe/`: `probe-record.sh`, `probe-transform.sh`, `egress-proxy.py`, `campaign.sh` and `ablate.sh`. The owner ruled on 2026-10-20 that `tools/probe/seal.sh` and `tools/probe/leak.py` are not on the list. By the same ruling, his acceptance of the 30 paths by which the scenario-0 trees at 2d212a26 and 2e3d71a7 differ stands. A base is stale when a later change to one of these paths alters what a session measures. A refusal that `campaign.sh` makes before it builds a tree alters no session, so it does not make a base stale.

On 2026-10-20 the owner also ruled on #1821, in two parts. An edit to `campaign.sh` makes a base stale only when it touches the code that builds, runs or grades a session. The gate and refusal checks are exempt. So `campaign.sh` marks each refusal above its check of the tools with the comment lines `# base-neutral: begin <reason>` and `# base-neutral: end`. A changed line is neutral when it is a comment, a blank, or inside a marked region of its own copy. An edit to `campaign.sh` makes a base stale only when one changed line is not neutral. The base pin 2d212a26 has no markers. So a changed line of code at that pin is never neutral, and the reader errs toward stale.

The fifth mechanism holds the total, the base and the record of each try (#1821, #1822). `tools/probe/try.sh` has ten verbs, and `plan` is the gate. It sums every `cost` and `spent-before` under every try root of the round, and it stores no total. It refuses with exit 5 when that sum plus the open layer's `cap_cents` is above `approved_cents`. It then builds the base scenario's tree at the base's pin and at `HEAD`, with `campaign.sh --scenario-tree`. It exits 4 when a path by which the two trees differ matches `base_invalidators`, under the rule for `campaign.sh` above. An acceptance of that path, below, prevents the exit. `launch` runs `plan` first and starts the driver only when `plan` passes. On 2026-10-21 the total read 2452 of 151200 cents, and the base check against 2d212a26 named four engine files and `campaign.sh`.

`accept` records an acceptance of the owner in `ledger.jsonl` under the root of the round. The record holds the URL of the ruling and the digest of each path at `HEAD`. `plan` reads that record. A path is covered when an acceptance at the same base and pin lists the path with the digest that it has now. A path that changes again after the acceptance is not covered, so the base is stale again. The ledger is outside git, so the URL of the ruling is the record that lasts, and a transcript cites it. The 2026-10-20 acceptance is not in a ledger, and it covers only the paths that differ between 2d212a26 and 2e3d71a7.

`result` makes the record of a finished try. It writes the transcript, compares it with the base and with the best try so far, and appends the verdict to the ledger. A try with no `done` gets a `halt` record and no verdict. `ledger` prints the records. The owner ruled the keep rule on 2026-10-20 (#1777). Keep a tuning change only when the whole uncertainty range of its difference against the best so far is above zero. So `result` keeps a try only when the low bound of that range is above zero. Try 1 of the documents layer, 32 of 72 against 39 of 72, was -9.7 points [-25.2, +6.5], so the rule does not keep it. Each transcript states its round in the `round` facet, so the committed spend of each round is the sum of `cost_cents` over its transcripts. A test of `headwater-probe` holds that sum at or below the approval of the round.

These rulings open again if the owner approves the final round, or moves the $700 or the $1,512.00.

The dry run builds the tree of each scenario and runs the leak check over it. The check reads the always-loaded text, the body of each skill, and the text the intent hook injects for the task of each probe. A leak stops the dry run with 8. A batch of the spec runs the same check before each invocation, so the check runs again before each round.

**The harness block of scenario 13.** The clause names the internal skill `hw-corpus`. The measured text of scenario 13 also states answers in `ste-editor`, `headwater-engine`, `hw-run-policy` and `hw-verification-bar`. This record reads the clause as the whole `harness` layer. The reason is the same for each file: the seal cannot remove internal harness text. The check prints a hit there as `harness`, and the dry run refuses a line that pools such a probe with another probe. So scenario 13 runs three probes on lines of their own. This reading opens again if an outside reader can see the body of a harness skill, because the text is then not internal.

**Scenario 0 is the floor.** Scenario 0 holds no `docs/`. So a probe whose key needs a document under `docs/` is at zero there by construction. That is the floor that scenario 1 is compared with, and it is not a defect of the set.

A probe at the ceiling of 2026-10-03 stays out of the set even where it sits lower in plain code. Its gain from the bottom is real and unmeasured here. The set opens again if a held-out probe sits at the ceiling in the base of a scenario. That probe can then show no gain there. The tuning set also opens again if a tuning probe sits at the ceiling in the base of every tuned scenario. That probe can then show no gain in any tuning try.
