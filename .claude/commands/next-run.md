---
description: Run N stacked iterations of the build order, one subagent each, merging between
argument-hint: "[iteration count, default 20]"
---

Run `$ARGUMENTS` iterations of `/next` (default 20), one Opus 5 subagent per iteration, **sequentially**. Merge between each so iteration N+1 branches from N's merge and reads a board N already changed. Parallel runs do not stack; they collide.

You are the parent. Your job is judgment: what to merge, what a stale premise means, which of the agent's surprises is a lesson and which is noise. The subagents do the volume.

## The ledger

Keep a running ledger at `~/.claude/headwater-build-order-ledger.md`, **on disk, not in your context**. Twenty iterations will compact you at least once, and everything that makes this compound lives in the ledger. Read it at the top of every iteration and append to it at the bottom. It holds three sections:

- **Lessons.** What an iteration learned that the next one should not rediscover.
- **Open findings.** The spec 13 entries and issue comments prior iterations filed, by number.
- **Log.** One line per iteration: issue, pull request, merge commit, verdict.

Seed **Lessons** with these, which cost earlier runs a cycle each:

- Fix your own wording rather than filing a finding about it. A finding is for someone else's text.
- Prefer deleting a stand-in to adding beside it. Two live paths is the defect the stand-in was meant to avoid.
- Choose a recorded fixture's grain so it survives ordinary prose edits. A fixture that changes on every commit is a fixture nobody reads.
- Never pick a milestone epic as the iteration's issue.
- Apply the `correctness-root` rule only among items whose dependencies already exist.

## Each iteration's prompt

Build it fresh. It must contain all four:

1. **The body of `.claude/commands/next.md`, pasted in full.** Subagents do not get slash commands, so an agent told to "run /next" invents its own idea of what that means.

2. **The environment traps.** `gh issue view` and `gh pr edit` fail with a `projectCards` GraphQL deprecation, so pass explicit `--json` or `--template`, and patch pull-request bodies with `gh api -X PATCH repos/headwater-ai/headwater/pulls/<N> -F body=@file.md`. Worktree isolation refuses heredocs and compound shell, so write scripts under `$CLAUDE_JOB_DIR/tmp` and run them. Use `git push -u origin <branch>`, never bare `git push`, which also tries to push a stale local `main`. Branch from `origin/main` after a fetch. The container pins Rust 1.85 while CI runs current stable, so clippy differs and `-D warnings` promotes new lints: check CI rather than trusting a local green run.

3. **The Lessons section**, verbatim.

4. **The Open findings section**, with the ones this issue touches called out by name. This is the highest-value part. An agent handed the question its predecessor filed settles the question; an agent without it builds around the question and files a second copy.

## Between iterations, before merging

**Verify independently.** Reset a scratch worktree to the branch, then run the test suite, the linter, and the CLI yourself. Do not merge on the agent's report. The difference between N stacked commits and N stacked claims is this step.

**Check the board the next iteration will read.** Close any epic whose children are all done, after confirming its Done-when bar yourself. An open epic with nothing under it captures the selection rule and misdirects the next pick.

**Harvest the report.** The fourth line of step 6 is "what you learned that is written down nowhere yet". Promote anything actionable into Lessons. Add any new spec 13 entry to Open findings.

**At a milestone boundary**, say so loudly in the log and name what the next milestone assumes. Do not stop, but do not roll past a boundary silently either: milestone order is a planning decision and the run is allowed to discover it is wrong.

## Policy, so nobody stops to ask

- Step 3 of `/next` tells the agent to halt on a stale premise. You adjudicate and redirect instead. Escalate to the human only when the redirect would change milestone order, or when a decision needs an owner rather than an answer.
- If an issue turns out to be three pieces of work, say so in the report and pick it anyway, unless it will not land in one pull request. Then split it on the board and take the first piece.
- If CI goes red after a merge, fix it in the next iteration's branch before that iteration's own work. Do not leave a red `main` behind you.
- Never force-push. Never push to `main`.
- Stop early and report if two consecutive iterations fail their own bar. Something upstream is wrong and iteration 3 will not find it.

## Report

A table of issue, pull request, and result. Then the adjustments you made between runs and why. Then anything you would not merge again without a decision from the human. Then the ledger's path.
