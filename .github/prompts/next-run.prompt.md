---
description: Run N stacked iterations of the Headwater build order, merging between
agent: agent
argument-hint: "[iteration count, default 20]"
---

Read `.claude/commands/next-run.md` in this checkout and follow it exactly, with `$ARGUMENTS` (default 20) as the iteration count. That file is the canonical definition of this procedure — this file exists only so this harness offers it under the same name; it carries no instructions of its own.

`.claude/commands/next-run.md` was written against Claude Code's tools: it stacks iterations by launching subagents (`Agent`, `subagent_type` per iteration) and merging between them. Dispatch each stage as its own `copilot -p` session with `tools/run/copilot-dispatch.sh`, not inline in your own turn — `tools/run/copilot-next-run.md` states the mechanism and what has been verified live about it, and the veto (`sh tools/run/copilot-dispatch.sh veto <run> <tag> <finding-file>`) is what resumes a build's own session with a verifier's finding rather than starting it over. You also have a native `task` tool that can dispatch a named agent in-process; this driver does not use it yet, so prefer the external dispatch above unless you have verified the in-process path yourself for the stage in question.
