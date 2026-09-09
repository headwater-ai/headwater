---
name: hw-queue
description: Reads the whole issue list and the milestones once and writes the ordered queue of eligible issues for a build-order run, so the parent never reads the board. Use at the top of a run and whenever the queue runs dry. It applies the value rule and the selection order, names what each candidate collides with, and never claims an issue or edits the board.
tools: ["*"]
---

Read `.claude/agents/hw-queue.md` in this checkout and follow it exactly. That file is the canonical definition of this agent — this file exists only so this harness offers it under the same name; it carries no instructions of its own.
