---
id: HW-OBL-0209
status: current
status_since: 2026-09-26
summary: "hw-run-policy names worktree isolation for each stage, but next-run.md passes none and hw-build and hw-verify still make their own tree with new-worktree.sh. No fixture holds the isolation."
last_verified: 2026-09-26
title: "A build-order dispatch passes no isolation, so a stage can inherit the parent worktree as its working directory"
waiting_on: build
---

# A build-order dispatch passes no isolation, so a stage can inherit the parent worktree as its working directory

## Context

The intake of run `20260924-0411` held this finding from the build of #848. The product owner ruled it Record, because the only party better off is this repository's build order. The owner ruled on 2026-09-24 that a contributor to this repository is not a reader outside it.

Run `20260926-1327` met the same gap again at the build of #1151, and the product owner ruled that finding Record too. This record holds it rather than a second record.

## Obligation

No dispatch in `.claude/commands/next-run.md` passes isolation to an `Agent` call. So a build-order stage can inherit the parent's worktree as its working directory. PR #1061 made `tools/repo/new-worktree.sh` refuse a nested path, which stops the worst result. The inheritance itself remains.

Since then, the skill `hw-run-policy` states that the parent dispatches `hw-build` and `hw-verify` with `isolation: "worktree"`. It also states that an agent launched without that isolation can be refused `Write`, `Edit` and git in a tree that `tools/repo/new-worktree.sh` made ([HW-OBL-0206](0206-hw-run-policy-names-a-worktree-add-workaround-that-write-edit-refuses-under-this-harness.md) records the refusal of `Write` and `Edit`). But three files still disagree with the skill. `next-run.md` names no isolation. `.claude/agents/hw-build.md` and `.claude/agents/hw-verify.md` tell each stage to make its own tree with `new-worktree.sh`. The builder of #1151 could not commit in the tree that the script made. So it started an isolated agent to commit for it. No fixture or check holds the isolation in any of these files.

## Discharge

This record discharges when two conditions hold. First, every stage dispatch in `next-run.md` names an isolated worktree, and `hw-build.md` and `hw-verify.md` agree with `hw-run-policy`. Second, a fixture or a check fails on a stage dispatch that names no isolation.
