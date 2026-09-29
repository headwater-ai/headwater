---
id: HW-OBL-0221
status: current
status_since: 2026-09-27
summary: "Every agent of a run reads its skill text from the shared checkout. The integrator is told to fast-forward that checkout after each merge, but under worktree isolation nothing does, so later agents read stale skills."
last_verified: 2026-09-27
title: "Subagents read skills from the shared checkout, and a worktree-isolated integrator cannot fast-forward it after a merge"
waiting_on: build
---

# Subagents read skills from the shared checkout, and a worktree-isolated integrator cannot fast-forward it after a merge

## Context

The harness loads each skill and each agent definition from the shared checkout of this repository. It does not load them from the worktree of the agent that reads them. `.claude/agents/hw-integrate.md` tells the integrator to run `git fetch`, `git checkout main` and `git merge --ff-only origin/main` in the shared checkout after each merge. `hw-run-policy` states that worktree isolation refuses `git -C` into another worktree.

Run `20260926-1327` met this gap. The product owner ruled it Record, because the run tooling does not ship to an adopter.

## Obligation

In run `20260926-1327`, the session was worktree-isolated, and no stage moved the shared checkout after a merge. Every agent of the run read its skill text from commit `ecceb0fd` while `main` moved on. One result: [#1174](https://github.com/headwater-ai/headwater/pull/1174) corrected a sentence of `headwater-authoring` about the far half of a relation, and agents after that merge still read the old sentence. On 2026-09-27, the local `main` branch still pointed at `ecceb0fd`, which was 10 commits behind `origin/main`.

So an agent can act on a rule that `main` has already changed. Nothing tells the agent, and nothing tells the parent.

## Discharge

This record discharges in one of two ways. A stage that can reach the shared checkout fast-forwards it after each merge, and the integrator or `tools/run/run-dir.sh` reports the commit it moved to. Or a run refuses to start when no stage can fast-forward the shared checkout, and a case shows the refusal.
