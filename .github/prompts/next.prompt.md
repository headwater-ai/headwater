---
description: Work one iteration of the Headwater build order yourself, then stop
agent: agent
argument-hint: "[issue number or milestone, optional]"
---

Read `.claude/commands/next.md` in this checkout and follow it exactly, with `$ARGUMENTS` (if given) as the issue or milestone it names. That file is the canonical definition of this procedure — this file exists only so this harness offers it under the same name; it carries no instructions of its own.

`.claude/commands/next.md` was written against Claude Code's tools (an `Agent` subagent, in particular). Where a step names a tool this harness does not have, use the nearest equivalent this harness offers, or stop and say which step you could not carry out.
