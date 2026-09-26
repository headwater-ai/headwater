---
id: HW-OBL-0206
status: current
status_since: 2026-09-26
summary: "hw-run-policy tells a subagent needing its own workspace to make one by hand with git worktree add. Edit and Write on a file under that hand-made worktree are refused, because the write sandbox stays pinned to the worktree the subagent was launched into."
last_verified: 2026-09-23
title: "hw-run-policy names a worktree-add workaround that Write/Edit refuses under this harness"
waiting_on: build
---

# hw-run-policy names a worktree-add workaround that Write/Edit refuses under this harness

## Context

`.claude/skills/hw-run-policy/SKILL.md` line 37 reads: "`Write` is refused in the shared checkout and `EnterWorktree` is refused from a subagent. Make a worktree by hand with `git worktree add`, from `origin/main` after a fetch, and remove it at the end or say that it is still there." Under this harness, `Edit`/`Write` on a file under a worktree a session makes by hand is refused. The write sandbox is pinned to the one worktree the subagent was launched into, not to any it creates afterward, and the refusal names the launch worktree as the one to edit instead. Two independent build agents in run `20260922-1121` (working #971 and #574) each spent real time discovering this on their own. Both fell back to `git checkout -b <branch> origin/main` inside their own launch worktree, which is what every build agent in that run actually did, per `git worktree list` at the time.

## Obligation

The documented workaround describes a pattern that fails under this harness's own sandbox rule. A future subagent reading this line will hit the same refusal and lose the same time two agents already lost in run `20260922-1121`. The line needs to name the pattern that works — a branch checked out inside the subagent's own launch worktree — rather than the one that fails.

## Discharge

This discharges when `.claude/skills/hw-run-policy/SKILL.md`'s line for a subagent's own workspace is rewritten to name `git checkout -b <branch> origin/main`, run inside the subagent's own launch worktree, as the working pattern. The `git worktree add` line is removed, or it stays and states why it does not work here.
