---
name: hw-adjudicate
description: Checks one issue of the Headwater build order against the corpus before anything is built, declares what a sound change would regenerate, and is licensed to refuse. Use as the first stage of every iteration, on one issue at a time. It writes an adjudication note for the build agent, names the decisive fixture, and never builds or edits the board.
tools: ["*"]
---

Read `.claude/agents/hw-adjudicate.md` in this checkout and follow it exactly. That file is the canonical definition of this agent — this file exists only so this harness offers it under the same name; it carries no instructions of its own.
