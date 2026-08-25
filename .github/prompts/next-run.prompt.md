---
description: Run N stacked iterations of the Headwater build order, merging between
agent: agent
argument-hint: "[iteration count, default 20]"
---

Read `.claude/commands/next-run.md` in this checkout and follow it exactly, with `$ARGUMENTS` (default 20) as the iteration count. That file is the canonical definition of this procedure — this file exists only so this harness offers it under the same name; it carries no instructions of its own.

`.claude/commands/next-run.md` was written against Claude Code's tools: it stacks iterations by launching subagents (`Agent`, `subagent_type` per iteration) and merging between them. Where this harness has no equivalent to a launched subagent, run each iteration in sequence yourself instead, and say so in your report rather than silently skipping the parallel-agent step.
