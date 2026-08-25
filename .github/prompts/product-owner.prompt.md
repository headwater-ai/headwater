---
description: Read the whole board — milestone order, what is finished and unclosed, what is misfiled, what blocks an adopter
agent: agent
argument-hint: "[window, e.g. 7d or 2026-08-12, default 14d]"
---

Read `.claude/commands/product-owner.md` in this checkout and follow it exactly, with `$ARGUMENTS` (default the last 14 days) as the window. That file is the canonical definition of this procedure — this file exists only so this harness offers it under the same name; it carries no instructions of its own.

`.claude/commands/product-owner.md` launches the product-owner judgment as an isolated subagent rather than in the calling session's own context, and `.github/agents/headwater-product-owner.agent.md` is this harness's binding of that agent. Launch it there rather than answering in this context, for the same reason the canonical file gives: an agent that has been building has every reason to find the next thing to build, and this judgment has to come from somewhere that did not.
