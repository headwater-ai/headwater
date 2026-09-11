---
id: HW-EVAL-the-build-order-as-a-multi-agent-system
status: current
status_since: 2026-09-07
summary: "What one 20-hour run of the build order measured about its own orchestrator, the cost model those measurements settle, the architecture that follows, what was rejected, and the numbers the next run is held against."
last_verified: 2026-09-08
title: "The build order as a multi-agent system"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-fable-5-1
  activity: measure+draft
  evidence_basis: evidenced
---

# The build order as a multi-agent system

The build order is the loop that turns a session into an orchestrator, picks issues, dispatches agents, and merges what they build. `.claude/commands/next-run.md` carried its whole design as prose for eleven runs, and the prose was read by the one agent that could not be reloaded. This evaluation records what one run measured about that shape, the cost model the measurements settle, the architecture that follows, and what was rejected on the way. Six decision records carry the rulings, and this document carries the evidence they cite.

The reader outside this repository is an adopter who runs an agentic loop over a Headwater corpus of their own. Every number here was taken on this repository, and the cost model is stated so that another corpus can take its own.

## How the numbers were taken

A session log writes one API response as several lines, one per content block, and every line carries a copy of the same usage record. A count per line inflates turns and cost by about 2.3 times, and it inflates them unevenly, because a response with four tool calls is copied more often than one with a single call. Every figure here groups by `message.id` and takes one usage record per identifier. The orchestrator of the run below reads as 712 turns and $156 per line, and as 304 turns and $67 per message.

The unit that explains the cost is carry. A tool result costs its own size times the number of turns that follow it. Seven thousand tokens read at turn 20 of a 170-turn agent carry 1.05 million token-reads, and the same seven thousand read at turn 150 carry 140 thousand. Position is worth as much as size, which is why cheap-looking calls dominate. In the single-agent shape, Bash results were 68% of all carry and file reads 29%, because an iteration made 142 Bash calls against eleven reads.

## What the single-agent shape cost, and what the four-agent split bought

Across 80 single-agent iterations, 77% of the cost was cache reads, the context re-read on every turn. Output was 12% and cache writes 11%. The median iteration ran 170 turns, peaked at 286k of context, and re-read 32 million tokens on the way.

Splitting one iteration into adjudication, construction, verification and write-back bought peak context and did not buy dollars. Adjudication at $3 and construction at $17 came to $20 against $21 for the single-agent shape, a 4% saving over a ten-hour run that merged seven pull requests. Peak context per agent fell from 286k to 211k, and adjudication alone ran at 116k and $3 because settling a premise never needs the test output. Construction grew back to 196 turns against the old 170, because the work did not shrink when it moved.

| Shape | Turns | Peak context | Cache read | Cost |
|---|---|---|---|---|
| One agent, median of 80 | 170 | 286k | 32.1M | $21 |
| Adjudication, median of 7 | 37 | 116k | 2.3M | $3 |
| Construction, median of 8 | 196 | 211k | 28.4M | $17 |

Run 22 measured the shape with verification split out. The parent was 9% of the run, against 27% before, and cache reads were 99% of every token billed. What did not move was growth: the parent climbed from 65k to 687k of context in under three hours with no compaction, and its last 84 turns cost about twice its first 84 for the same work. The built-in tool definitions were 28,987 tokens of that context, and the parent called four of the fourteen tools. Returned reports were its largest input class at 113.7k tokens, with a median of 1,475 each, and three agents notified two or three times, re-injecting the whole report on each.

## The run that measured the orchestrator as the constraint

Session `19108df3` ran for 20 hours and 29 minutes, dispatched 152 agents, and touched 45 pull requests. The human sent eight prompts in that time. The measurements below were taken from its session log after the run.

**The fleet ran at a third of its target width.** Mean in-flight concurrency was 3.03 against a target of eight. No agent at all was in flight for 20.9% of the active window, which is 3.4 hours across four windows, and the largest window was 114 minutes. In each of those windows the parent was merging, building or keeping records by itself.

**The parent was the mutex.** Of the parent's own serial shell time, 61% contended on four shared resources: the one main checkout, the one `engine/target`, `origin/main`, and the post-merge regenerate. `cargo build` alone was 49% of it, at 67 minutes over 26 calls. The merge itself was cheap, at about eleven seconds over 46 merges. Only 13% of that time was per-job work, and most of that was polling other agents' pull requests.

**The batches had a barrier by accident.** Subagent runtime had a median of 13.5 minutes, a ninetieth percentile of 46, and a longest of 91. Across the batches of three agents or more, 9.3 hours separated the first completion from the last. No code wrote that barrier. The parent advanced a batch when its slowest member returned.

**Merging was not the constraint.** Of the 45 pull requests, 44 merged at a median of 40 minutes from opening to merge, and the runner queued nothing that showed as idle workers.

**Width eight measured worse than width five, and the comparison is confounded.** The width was raised from five to eight in the middle of the run. Mean concurrency fell from 4.00 to 3.22, and the share of the window with an idle fleet rose from none to 32.1%. The owner of that run attributes the largest single cause to merging in ready order rather than in artifact-footprint order, so that one branch touching every corpus-wide derived artifact was overtaken four times and re-derived four times. The comparison stands as a caution and not as a finding, and the next measurement takes it again under footprint order.

**The parent compacted five times.** Its single largest gap, 82.8 minutes, ended at a compaction. The doctrine it ran under was 67 KB of prose in its own context, and a compaction summarizes prose of that length rather than preserving it.

**Of 67 messages the parent sent, two kinds want opposite treatment.** Most were authority: a branch sent back with a defect named, or a delta sent for re-verification. A few were coordination: a warning to one worker that another was about to touch the same derived artifact. The first kind is the parent exercising a veto only it may exercise. The second kind is a shared-state problem the parent was relaying by hand.

## The finding that set the unit of cost

Pull request #685 measured the same parent's polling. It made 76 `gh pr view` and `gh pr list` calls whose combined output was 25 KB. Those calls occupied 77 turns and 21.3 million cache reads, which is 10.3% of the parent's entire 207 million for the run, to carry about 6.3 thousand tokens of payload. That is roughly 3,400 times the payload in carrier cost.

The unit of cost is therefore a parent turn at the parent's full context, and not a token. The same finding rules out the obvious remedy: a fresh agent dispatched to make one poll is worse than the poll, because the dispatch is itself a parent turn at the same context, with a prompt on top. A dispatch pays only when it retires more parent turns than it costs. [HW-PD-0003](../process/decisions/0003-a-dispatch-pays-when-it-retires-more-parent-turns-than-it-costs.md) is that rule.

Counting repeated calls does not find this class. The 76 calls covered 43 pull requests at about one and a half calls each, and no shape repeated more than 40 times. An earlier incident, 232 identical calls in twelve minutes, was visible from any angle. This one was visible only by grouping shell calls by purpose and reading the total.

## What the following run added

Run 24 ran under the integrator slot and with polling moved out of the parent. Its owner reported four things this evaluation carries forward.

**Refusal was the highest-value output.** Six of eight slots returned something other than "build as specified". Three rejected part of their own Done-when, and one found a pre-existing defect that permanently bricks an output path, reproducible at one run in three with its own change stashed. The owner's reading is that this happens only when adjudication is separate from construction and refusal is licensed in the prompt, because an agent handed a prescribed remedy implements it and cannot find the error in it. [HW-PD-0002](../process/decisions/0002-adjudication-is-a-separate-stage-and-refusal-is-licensed.md) rules on it.

**Widest artifact footprint merges first.** Under that rule no branch was overtaken. Its corollary is to never add rebase work to a branch that is still running, because a finished branch waiting costs nothing and a working branch redoing its derives costs a full pass.

**A blocking wait outlives the agent that started it.** The run ended with 30 orphaned `until … sleep 30` loops, the oldest still polling after eleven hours, emitting phantom notifications and pinning deleted worktrees on disk. One of them was still alive nine hours after the run stopped.

**A stale instrument reports a correct tree as defective.** A binary four hours behind `main` reported twelve site figures stale, and the gate's own printed remedy would have written the wrong values into three pages and staged them. Two sessions read that report and both concluded the pages were stale. Rebuilt at the same commit, the check reported zero stale. The direction of every delta was the tell: a measurement missing something the page knows about is an old binary, not a stale page. Issue #679 holds it as a correctness root.

## The cost model

Three quantities move independently, and a change serves at most two of them.

- **Fewer parent turns.** Served by work that never returns to the parent, by state the parent never reads, and by making every unavoidable turn do everything it can. A dispatch pays only when it retires more turns than it costs.
- **Smaller parent context.** Served by moving prose out of the command, because every turn re-reads the whole of it.
- **Smaller subagent context.** Served by trimming what every dispatched agent loads. `CLAUDE.md` at 6,500 tokens loads into each of 150 agents per run, and that is a larger lever than any per-agent model choice.

Decomposing the command into agent definitions serves the second quantity only. It does nothing for the first, and a design that claimed otherwise was corrected by the #685 measurement.

## The architecture that follows

[HW-PD-0001](../process/decisions/0001-orchestration-prose-has-one-owner-per-sentence.md) states the ownership rule, which decides where any sentence of orchestration prose lives. Under it the command shrinks to a doctrine block of ten numbered lines, a loop, a veto and a dispatch template. Each stage becomes an agent definition with its model in frontmatter. Rules two or more stages obey become skills. Measurement and rationale become this evaluation and the records it cites.

The stages are adjudication, construction, verification and integration, and integration fuses the merge, the regenerate and the write-back because all three are mechanical and all three touch the one shared checkout. A fifth definition writes the queue. The parent takes four turns per issue, which is one more than the fused shape would take, and [HW-PD-0002](../process/decisions/0002-adjudication-is-a-separate-stage-and-refusal-is-licensed.md) records why that turn is paid.

[HW-PD-0004](../process/decisions/0004-coordination-is-a-create-only-claim-and-authority-stays-on-the-tree.md) moves coordination to create-only claims in a run directory and keeps the veto with the parent. [HW-PD-0005](../process/decisions/0005-the-ledger-is-split-its-tabular-parts-are-jsonl-and-its-totals-are-derived.md) splits the ledger so that its doctrine reads alone and its log reads by the line. [HW-PD-0006](../process/decisions/0006-the-entrypoint-keeps-its-name-and-becomes-a-resumable-run.md) keeps `/next-run` and makes a run resumable from that directory.

## What was rejected

**Fusing adjudication into construction** would retire one parent turn per issue. Run 24's refusals are the evidence against it, and the turn is paid.

**A fresh agent per poll** fails the dispatch rule. The poll's output is one line, and the dispatch costs a full turn plus a prompt.

**Peer-to-peer authority** was rejected on a live demonstration. One session declined to edit `CLAUDE.md` on another session's reasoning, because a peer's message carries no owner authority, and that refusal was correct. In a mesh, every agent adjudicates provenance on every message. On a tree, authority flows down and provenance is never in question.

**Renaming the entrypoint** was rejected because the name was not what was wrong. The command was the orchestrator's whole brain and its only memory, and a resumable run fixes that where a rename does not.

**Re-reading the doctrine every iteration** was rejected because a re-read is itself a parent turn and it grows every later turn's context. The doctrine is the header of the ledger instead, and the parent reads it on a turn it already pays.

**Per-worktree engine builds** were rejected. A fresh worktree has no engine, so the commit gate fails open there, and `sccache` bakes a dead worktree's path into a test binary. One warm target directory behind the integrator beats several cold ones.

**SQLite for the ledger** is deferred and not refused. The problem it solves, reading a header without the body, is solved by splitting the file and writing the tabular parts as JSONL. A database earns its place when a cross-run question is asked, and JSONL imports cleanly then.

**A headless loop**, where a script drives each stage and the session spends turns only on verdicts, is the end state this design points at and does not take. Of the 45 pull requests in the measured run, 44 merged under the veto, and nothing has measured what merges without one.

## The numbers the next run is held against

| Measure | Baseline by hand | Baseline by the tool | Target | New shape, by the tool |
|---|---|---|---|---|
| Parent turns per issue | not measured as such | 19.6, as 880 turns over 45 pull requests | 4 | 24.5, as 98 turns over 4 pull requests |
| Share of the window with no agent in flight | 20.9% | 0%, largest window 0 min | near zero | 0.3%, largest window 0.4 min |
| Mean in-flight concurrency at width 5 | 4.00 | 9.54 over the whole run | at or above 4.00 | 3.91 over the whole run |
| Largest gap after a compaction | 82.8 min | 22.3 min after, 87.6 min before | under the dispatch cost | no compaction ran |
| Parent `gh` calls | 92 | 88 by leading verb | 0 | 2 by leading verb |
| `cargo build` in the parent | 26 calls, 67 min | 8 by leading verb, 15 mentioned | 0 | 0 |
| First-to-last completion spread per batch | 9.3 h total | not taken | not applicable, no batches | 5 min over five adjudications, 1.95 h over five constructions |

`tools/run/run-census.sh` takes these from a session log and the agent transcripts beside it. The run that follows this design writes its numbers into this table.

The figures in the table were taken by hand, part-way through the run, and the tool was written after them. Over the whole transcript of that run the tool reports 880 turns and 227.3 million cache reads. Mentions of `gh pr view` cost 66 calls, 66 turns and 18.1 million cache reads, which is 8.0% of the run. Mentions of `gh pr list` cost 24 calls, 24 turns and 6.7 million, which is 2.9%. The tool counts a call once per turn and a turn once per message, and it reads a verb past a leading `cd` or `set -e`, because that run wrote nearly every command in that shape. The next run is compared with numbers the same tool takes, and not with the hand count.

The fleet section of the same tool reads the agent transcripts that the harness writes beside the session file. Over the whole run it does not reproduce the hand count. It finds 165 agents at depth one over a span of 20 hours. No window had nothing in flight, and 9.54 agents were in flight on average. The hand count found 20.9% idle and a mean between 3.03 and 4.00. The tool counts an agent as in flight from its first line to its last turn, so every minute inside a blocking wait counts. The hand count was taken over a part of the run, by a method this evaluation does not record. The tool's reading stands, because the next run's reading is taken the same way. The largest gap before a compaction was 87.6 minutes from the parent's last turn. The largest gap after one was 22.3 minutes to its next dispatch. The parent took 5.5 turns per agent over 161 agents of one type, and 19.6 turns per pull request over 45.

## What the first run of the new shape measured

Session `8e38c6e8` ran the ledger's RUN 20260907-2208 under the five-agent shape, for two hours and thirty-six minutes, and merged four pull requests. The parent context grew from 55k to 269k tokens over 98 turns, with no compaction. It made two `gh` calls and no `cargo build` call, against 88 and 8 for the baseline run. Idle share fell to 0.3%, with a largest gap of 0.4 minutes, against 20.9% before.

Mean concurrency read 3.91 over the whole span, just under the 4.00 target. The reading understates the busy middle of the run. Five construction agents started within eight minutes of each other and finished across a span of 117 minutes, from 28 to 145. That spread thins the fleet after minute 110, once the rest of the run's work had already finished. Over minutes 20 to 100, the tool reads a mean of 4.9 agents in flight and a peak of 6. The whole-span figure is the one this table tracks, and it sits close to the target. The busy-core figure says the design sustains more parallel work than the whole-span number alone would suggest. A staggered batch of long, uneven construction runs narrows the gap between the two figures.

Refusal held on one issue across three rounds. Adjudication refused issue #648 outright, on the ground that two already-merged pull requests had answered its Done-when clause. Verification failed issue #485 twice, on two distinct construction defects. The doctrine's two-fail stop condition held the branch rather than dispatch a third round. A second-opinion agent, dispatched outside the normal construction path, reproduced the second defect for real. It proposed the fix that the third construction round then applied and verification confirmed. A fourth round did not run, for a different reason. The only regression test for that fix does not run in the continuous-integration job. A merge on a green check would repeat the pattern this evaluation names for the corpus tree in general, at line 78. The run left the pull request open for the owner's ruling instead.

One risk surfaced outside the measured numbers. A construction agent for issue #603 proposed a force push, against the standing rule that forbids one. The parent allowed it, after checking by hand that the branch and `main` were both intact. The rule held on the parent's judgment this round, and not on a check inside the construction agent's own prompt. The construction agent's prompt needs that rule stated, before the next occurrence depends on the same judgment again.

## What this evaluation cannot show

One run measured the orchestrator, and one run reported the refusals. The width comparison is confounded and is recorded as such. The claim that a compaction preserves a short numbered list and not a long narrative is plausible and untested, and the doctrine sizing rests on it. The integrator's net saving of about five parent turns per merge is arithmetic from the measured turn classes and not a measurement of the new shape. Each of these is a thing the next measurement can settle.
