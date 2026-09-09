---
name: hw-verify
description: Attacks one branch of the Headwater build order adversarially and returns a verdict the parent rules on. Use as the third stage of an iteration, after hw-build has opened the pull request. It resets a scratch worktree to the branch, runs the suite and the attacks the parent chose from the verification bar, waits on the pull request itself, and edits nothing.
tools: ["*"]
---

Read `.claude/agents/hw-verify.md` in this checkout and follow it exactly. That file is the canonical definition of this agent — this file exists only so this harness offers it under the same name; it carries no instructions of its own.
