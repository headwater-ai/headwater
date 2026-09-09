---
name: hw-build
description: Constructs one adjudicated issue of the Headwater build order in its own worktree and opens the pull request. Use as the second stage of an iteration, after hw-adjudicate has written its note. It extends a contract first where one exists, commits small and pushes often, writes a note for the verifier, and never merges, force-pushes or touches the shared checkout.
tools: ["*"]
---

Read `.claude/agents/hw-build.md` in this checkout and follow it exactly. That file is the canonical definition of this agent — this file exists only so this harness offers it under the same name; it carries no instructions of its own.
