---
id: HW-OBL-0209
status: discharged
status_since: 2026-09-27
summary: "hw-iterate now dispatches hw-build and hw-verify with isolation: worktree, and both agent files branch in the tree the harness gives them. Case 10 of .claude/agents/fixtures.sh fails on a loop agent that passes no isolation."
last_verified: 2026-09-27
title: "A build-order dispatch passes no isolation, so a stage can inherit the parent worktree as its working directory"
waiting_on: build
---

# A build-order dispatch passes no isolation, so a stage can inherit the parent worktree as its working directory

## Context

The intake of run `20260924-0411` held this finding from the build of #848. The product owner ruled it Record, because the only party better off is this repository's build order. The owner ruled on 2026-09-24 that a contributor to this repository is not a reader outside it.

Run `20260926-1327` met the same gap again at the build of #1151, and the product owner ruled that finding Record too. This record holds it rather than a second record.

## Obligation

No dispatch in `.claude/commands/next-run.md` passes isolation to an `Agent` call. So a build-order stage can inherit the parent's worktree as its working directory. PR #1061 made `tools/repo/new-worktree.sh` refuse a nested path, which stops the worst result. The inheritance itself remains.

Since then, the skill `hw-run-policy` states that the parent dispatches `hw-build` and `hw-verify` with `isolation: "worktree"`. It states that an agent launched this way branches in place and does not run `tools/repo/new-worktree.sh`. It also states that a tree made by hand, with `git worktree add` or `new-worktree.sh`, is not one the isolation of the agent allows. So `cd`, `git -C`, `Write` and `Edit` into that tree can be refused ([HW-OBL-0206](0206-hw-run-policy-names-a-worktree-add-workaround-that-write-edit-refuses-under-this-harness.md) records the refusal of `Write` and `Edit`). But three files still disagree with the skill. `next-run.md` names no isolation. `.claude/agents/hw-build.md` and `.claude/agents/hw-verify.md` tell each stage to make its own tree with `new-worktree.sh`. The builder of #1151 could not commit in the tree that the script made. So it started an isolated agent to commit for it. No fixture or check holds the isolation in any of these files.

## Discharge

This record discharges when two conditions hold. First, every stage dispatch in `next-run.md` names an isolated worktree, and `hw-build.md` and `hw-verify.md` agree with `hw-run-policy`. Second, a fixture or a check fails on a stage dispatch that names no isolation.

## Discharged by #1276

The parent dispatches neither `hw-build` nor `hw-verify`. `.claude/agents/hw-iterate.md` dispatches both, and each dispatch names `isolation: "worktree"` ([HW-PD-0022](../decisions/0022-the-verify-and-rework-loop-for-one-issue-runs-below-the-parent.md)). `hw-build.md` and `hw-verify.md` now branch in the tree that the harness gives them, and neither runs `new-worktree.sh`. This agrees with `hw-run-policy`. The other stages that `next-run.md` dispatches make no tree of their own, so no isolation applies to them.

Case 10 of `.claude/agents/fixtures.sh` holds the second condition. A copy of `hw-iterate.md` without `isolation: "worktree"` is reported, and a copy of `next-run.md` that dispatches `hw-verify` itself is reported.
