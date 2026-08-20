---
description: Read the whole board — milestone order, what is finished and unclosed, what is misfiled, what blocks an adopter
argument-hint: "[window, e.g. 7d or 2026-08-12, default 14d]"
---

Run the product owner over the whole board, with a window of `$ARGUMENTS` (default the last 14 days), and report what it says.

**Launch it as a subagent, not in your own context.** `Agent` with `subagent_type: headwater-product-owner`. The agent carries its own instructions, its own write boundary and its own report shape; do not paste them here and do not summarize them into the prompt. Give it the window and nothing else it can derive.

The reason it runs in its own context is the reason it exists. An agent that has been building has every reason to find the next thing to build, and an agent that has been reading a diff has no idea whether the diff belonged to the plan. This one reads the board and never the branch.

## What to hand it

- The window, resolved to an ISO date. `14d` from today is the date it uses in the `closedAt` comparisons.
- Whether this is a standalone run, the top of a build-order run, or the fifth-iteration check inside one. It reports the same five parts either way; the difference is that inside a run its part 1 has a run to attribute the off-plan share to, and standalone it has a stretch of history.
- Nothing else. In particular do not tell it what you think the answer is, and do not hand it a milestone to look at. Naming one is how a whole-board read becomes the read you already had.

## What comes back

Five parts: **Order**, **Completion**, **Misfiled**, **Blocked**, **Undecided**. It will have written to the board — closed a finished milestone, moved a misfiled issue, applied `adopter-blocking` — and every write is named in the part that motivated it, with the reason.

**Read part 5 first.** It is the part that needs you rather than the agent, and it is the part a reader skips. A missing parallel-track reason, a bar the agent believes is wrong, a milestone at zero open whose Done-when it could not verify: all of these wait on a ruling and none of them will resolve itself.

**Check that the writes took.** An agent can state a write-back and not land it, which is a recorded failure of this repository's build order. Re-read the milestone list and the labels it says it changed.

**Reverse anything you disagree with.** Every write it makes is structural — where an issue sits, whether a milestone is open, which of two labels an issue carries — and every one of them is one command to undo. It closes no issues and edits no scope, so there is nothing it can do that costs you an argument about what the work was.

## When to run it

Standalone, whenever the board has not been read as a whole recently — which after a build-order run is always, because a run reads the board an iteration at a time and never once end to end. Also at the top of every run and every fifth iteration inside one, which `.claude/commands/next-run.md` already requires under *Policy*.

Running it more often than a run does costs one agent and finds the bookkeeping before it compounds. Four finished milestones sat open on this board for a week because nothing read the list.
