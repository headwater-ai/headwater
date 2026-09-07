---
description: Work one iteration of the Headwater build order in this session, over the same agents the run uses, then stop
argument-hint: "[issue number, optional]"
---

Work one iteration of the Headwater build order (org project "Headwater build order", `headwater-ai/headwater`), then stop. `$ARGUMENTS`, if given, names the issue; without it, dispatch `hw-queue` and take the top of what it writes.

**The value rule binds here too.** `.claude/commands/next-run.md` states it once, under *The value rule*, and this file does not restate it. Before you start, name the reader who is not this repository.

## What is the same

The stages are the same five agent definitions the run uses, dispatched by `subagent_type` and in the same order: `hw-adjudicate`, then `hw-build`, then `hw-verify`, and `hw-integrate` once the merge is ruled. Each carries its own procedure, its own write boundary and its own fixed report block, and the `hw-run-policy` and `hw-verification-bar` skills carry the rest. Use the dispatch template from `.claude/commands/next-run.md` and pass paths, never pasted prose.

## What differs

**Width is one and the parent is this session.** There is no run directory unless you make one; the scratch directory under `$CLAUDE_JOB_DIR/tmp/issue-<N>/` is enough for one iteration's notes.

**The merge is the human's unless they said otherwise.** Report the verifier's verdict and your ruling, and dispatch `hw-integrate` only when the person in the session says merge. A run has a standing licence to merge; a single iteration by hand does not.

**A refusal stops here.** When `hw-adjudicate` returns `VERDICT: REFUSE`, say what changed and ask, because the second reader is a person rather than a run that can redirect.

**Report the four lines** the build agent returns, and take the fourth seriously: *what you learned that is written down nowhere yet*.
