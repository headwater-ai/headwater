---
description: Work one iteration of the Headwater build order yourself, then stop
argument-hint: "[issue number or milestone, optional]"
---

Work one iteration of the Headwater build order (org project "Headwater build order", `headwater-ai/headwater`) **yourself**, then stop. `$ARGUMENTS`, if given, names the issue or milestone to work.

**The procedure is part 1 of `.claude/commands/next-run.md`** — read it there rather than from memory. It holds the canonical statement of how to read the board, pick an issue, check its premise, do the work, write back, and report. That file is the single definition; this one exists because a single iteration is often worth running by hand, and it states only what differs.

## What differs when you run one iteration yourself

**You are the doer and there is no parent to adjudicate for you.** In `/next-run` a subagent that meets a stale premise is told to adjudicate and keep going, because the parent will verify the result and can redirect. Here nobody will.

So: **if the issue's premise no longer holds, or its "Done when" does not parse into a checkable bar, say what changed and ask rather than guess.** That is the one instruction that reverses between the two commands, and it reverses because the second reader is a person rather than a verifier.

**Verify your own work before you claim it.** Nobody is going to break the corpus by hand on your behalf. At minimum: run the suite, run all five gates, and make the thing you built fail before you believe it works. If you added a rule, break the corpus and watch it fire with a message that names the offender. If you fixed something, revert the fix and confirm a named test goes red — a fix no test holds is a fix nobody can keep.

**Report the four lines**, and take the fourth seriously: *what you learned that is written down nowhere yet*. An empty answer usually means the write-back was rushed.

## Before you start

Read `~/.claude/headwater-build-order-ledger.md` if it exists. It carries the lessons, open findings and traps from every prior iteration, and it will save you an hour you would otherwise spend rediscovering one of them.
