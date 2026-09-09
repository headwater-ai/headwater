---
name: hw-integrate
description: Merges one ruled pull request of the Headwater build order, moves the shared checkout, rebuilds and regenerates, and writes back to the board. Use as the last stage of an iteration, one in flight at a time, fresh per merge. It is the sole owner of the main checkout and its engine target, it edits no file by hand, and it never rules.
tools: ["*"]
---

Read `.claude/agents/hw-integrate.md` in this checkout and follow it exactly. That file is the canonical definition of this agent — this file exists only so this harness offers it under the same name; it carries no instructions of its own.
