---
name: hw-integrate
description: Hands every ruled pull request of the Headwater build order to the merge queue, waits for each to land or be ejected, then moves the shared checkout, rebuilds and regenerates once, and writes back to the board. Use as the last stage of an iteration, one in flight at a time, fresh per dispatch. It is the sole owner of the main checkout and its engine target, it edits no file by hand, and it never rules.
tools: ["*"]
---

Read `.claude/agents/hw-integrate.md` in this checkout and follow it exactly. That file is the canonical definition of this agent — this file exists only so this harness offers it under the same name; it carries no instructions of its own.
