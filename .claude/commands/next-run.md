---
description: Run N stacked iterations of the build order, one subagent each, merging between
argument-hint: "[iteration count, default 20]"
---

Run `$ARGUMENTS` iterations of `/next` (default 20), one Opus 5 subagent per iteration, **sequentially**. Merge between each so iteration N+1 branches from N's merge and reads a board N already changed. Parallel runs do not stack; they collide.

You are the parent. Your job is judgment: what to merge, what a stale premise means, which of the agent's surprises is a lesson and which is noise. The subagents do the volume. Almost everything you control sits in two places: what you put in the next prompt, and what you refuse to take on trust.

## The ledger

Keep a running ledger at `~/.claude/headwater-build-order-ledger.md`, **on disk, not in your context**. Twenty iterations will compact you at least once, and everything that makes this compound lives in the ledger. Read it at the top of every iteration and append to it at the bottom. It holds five sections:

- **Lessons.** What an iteration learned that the next one should not rediscover.
- **Open findings.** The [13 — Open obligations](docs/spec/13-open-obligations.md) entries and issue comments prior iterations filed, by number, plus any defect you found in verification that you are handing forward.
- **Decisions needing an owner.** What you merged but would not merge again without a human ruling. Write it when it happens, not at the end, because the reason is legible then and paraphrased later.
- **Milestone status.** Which milestones closed, whose bar you verified, and what the next one assumes.
- **Log.** One line per iteration: issue, pull request, merge commit, verdict, and what your own verification proved rather than what the agent claimed.

Seed **Lessons** with these, which cost earlier runs a cycle each:

- Fix your own wording rather than filing a finding about it. A finding is for someone else's text.
- Prefer deleting a stand-in to adding beside it. Two live paths is the defect the stand-in was meant to avoid.
- Choose a recorded fixture's grain so it survives ordinary prose edits. A fixture that changes on every commit is a fixture nobody reads.
- Never pick a milestone epic as the iteration's issue.
- Apply the `correctness-root` rule only among items whose dependencies already exist.
- When you close a finding, grep for every sentence that asserts the old state, and grep instruction files as well as prose. One missed entry becomes false in three places, and the sweep that follows finds a dozen more.
- A derived state must be derived from what runs, not from what is declared.
- A green run is no evidence that a constraint is enforced. The strongest finding can come from a fixture that passes.
- A projection is a good place to put pressure on a schema, and the pressure arrives as "I would have to print something false here".
- Never pipe the gate. `<gate> | <filter> && <next>` reports the filter's exit status, so a failure passes silently.
- Commit and push in small steps. A transport error costs you everything unbanked.

## Each iteration's prompt

Build it fresh. It must contain all six:

1. **The body of `.claude/commands/next.md`, pasted in full.** Subagents do not get slash commands, so an agent told to "run /next" invents its own idea of what that means.

2. **The environment traps.** `gh issue view` and `gh pr edit` fail with a `projectCards` GraphQL deprecation, so pass explicit `--json` or `--template`, and patch bodies with `gh api -X PATCH repos/headwater-ai/headwater/{pulls,issues}/<N> -F body=@file.md`. Worktree isolation refuses heredocs and compound shell, so write scripts under `$CLAUDE_JOB_DIR/tmp` and run them. Use `git push -u origin <branch>`, never bare `git push`, which also tries to push a stale local `main`. Branch from `origin/main` after a fetch. The container pins Rust 1.85 while CI runs current stable, so clippy differs and `-D warnings` promotes new lints: run the pinned container with `--user` and a container-internal `CARGO_TARGET_DIR`, capture each step's status separately, and check CI rather than trusting a local green run. Carry the traps a prior iteration hit forward — that list grows and it is cheap to paste.

3. **The Lessons section**, verbatim.

4. **The Open findings section**, with the ones this issue touches called out by name. An agent handed the question its predecessor filed settles the question; an agent without it builds around the question and files a second copy. Say plainly which entries are the agent's *to close* rather than to record.

5. **The state of the system as of this merge.** The invariants that must stay true, the blocking CI steps, the crates or modules the work will touch, the verbs that exist, and **the current numbers** — instance counts, finding counts, coverage, test suites. Then: *state exactly why each number moves, and re-bless recorded fixtures deliberately.* Numbers are the cheapest tripwire you have. An agent that must explain a delta notices the delta it did not intend.

6. **How you verify, and any pre-work.** Tell the agent what you actually do before merging — that you break the corpus by hand, re-derive claims, regress implementations, and validate artifacts with tools that are not this system. Quality rises when the agent knows the report is not the deliverable. If your verification of the previous branch found a defect, hand it over as **pre-work**: its own commit on this branch, before the issue's own work. That mechanism is the highest-yield thing in this command — one pre-work item early in a run triggered a sweep that found fifteen more stale claims.

Norms transfer too. "The last four iterations each landed prose with zero new advisory findings — match that" is worth a line, and it holds.

## Between iterations, before merging

**Verify independently, and adversarially.** Reset a scratch worktree to the branch and run the suite, the linter and the CLI. That is necessary and it is nowhere near sufficient — it catches almost nothing an agent has not already caught. What actually finds things:

- **Make the new thing fail.** Break the corpus by hand and confirm the new rule fires with a message that names the offender. A check that fires on no input is indistinguishable from a check that does not work. When a change reports *fewer* findings, that is the case to attack hardest.
- **Test a gate in both directions.** A real drift must fail it; an ordinary edit must not. A gate that fires on every prose change is a gate someone will disable.
- **Re-derive one claimed number by hand.** Count the words yourself. Parse the artifact with a different tool. Diff the digest against the parent commit.
- **Regress the implementation to prove a new test can fail.** A differential nobody has seen fail is a differential that proves nothing.
- **Validate artifacts with something that is not this system** — an external parser, a published schema, a stock validator.
- **Re-run the experiment that found a prior defect**, against the fix.
- **Reproduce a performance claim in the build configuration the claim implies.** Debug and release differ by an order of magnitude, and an argument resting on "30 ms" needs the build where that is true.

**When your own check contradicts the agent, suspect your check first.** Twice in one run I was the one who was wrong — once reading a stale lock the engine deliberately never reads from source, once deriving an anchor slug by the wrong rule. Confirm your method before you write up a defect, and if the agent was right, say so plainly and move on.

**Check the board the next iteration will read.** Close any epic whose children are all done, after confirming its Done-when bar yourself, clause by clause, against the merged tree. An open epic with nothing under it captures the selection rule and misdirects the next pick. **If the bar names work that no open issue carries, file that issue yourself** — twice in one run this was the only thing standing between the run and a closed milestone.

**Harvest the report.** The fourth line of step 6 is "what you learned that is written down nowhere yet". Promote anything actionable into Lessons. Add any new finding to Open findings. Watch the net: a findings register with an inflow and no sink is itself a finding, and once you see it, tell every later prompt to settle where it honestly can rather than file.

**At a milestone boundary**, say so loudly in the log and name what the next milestone assumes. Do not stop, but do not roll past a boundary silently either: milestone order is a planning decision and the run is allowed to discover it is wrong.

## Policy, so nobody stops to ask

- Step 3 of `/next` tells the agent to halt on a stale premise. You adjudicate and redirect instead. Escalate to the human only when the redirect would change milestone order, or when a decision needs an owner rather than an answer.
- If an issue turns out to be three pieces of work, say so in the report and pick it anyway, unless it will not land in one pull request. Then split it on the board and take the first piece. A sound first piece plus honestly-filed follow-ups beats a rushed whole, and late in a run it is usually the right call.
- **When an agent refuses to build something the issue names, adjudicate the kind of refusal.** "Nothing states what this is, so I shipped a verb that reports the gap" is sound engineering and merges. "This would be better done another way" is a product decision: merge it only if it is reversible and recorded with the condition that reopens it, and put it under *Decisions needing an owner* either way.
- If CI goes red after a merge, fix it in the next iteration's branch before that iteration's own work. Do not leave a red `main` behind you.
- **If an agent dies mid-iteration**, check its worktree before restarting. A transport error can cost twenty-five minutes of settled design and leave nothing on disk. `SendMessage` to its agent id resumes it with its context intact; a fresh agent throws that away.
- You may open your own pull request when a merge leaves something stale that no issue owns — a claim in an instruction file, a count nobody re-derives. Keep it small and separate from the iteration's work.
- Never force-push. Never push to `main`. Squash-merge when a branch's history carries a garbled commit message.
- Stop early and report if two consecutive iterations fail their own bar. Something upstream is wrong and iteration 3 will not find it.
- Verify `main` itself once at the end. Every merge was verified on its branch; nothing yet has verified their composition.

## Report

A table of issue, pull request, and result. Then the adjustments you made between runs and why — including what your verification caught that a report did not. Then anything you would not merge again without a decision from the human. Then the ledger's path.
