---
id: HW-OBL-0209
status: draft
status_since: 2026-09-24
summary: "No dispatch in next-run.md asks for an isolated worktree, so a stage can start inside the parent's worktree. new-worktree.sh refuses the nested path but the inheritance remains."
last_verified: 2026-09-24
title: "A build-order dispatch passes no isolation, so a stage can inherit the parent worktree as its working directory"
waiting_on: build
---

# A build-order dispatch passes no isolation, so a stage can inherit the parent worktree as its working directory

## Context

The intake of run `20260924-0411` held this finding from the build of #848. The product owner ruled it Record, because the only party better off is this repository's build order. The owner ruled on 2026-09-24 that a contributor to this repository is not a reader outside it.

## Obligation

No dispatch in `.claude/commands/next-run.md` passes isolation to an `Agent` call. So a build-order stage can inherit the parent's worktree as its working directory. PR #1061 made `tools/repo/new-worktree.sh` refuse a nested path, which stops the worst result. The inheritance itself remains.

## Discharge

This record discharges when every stage dispatch in `next-run.md` names an isolated worktree, or when the record states why the refusal in `new-worktree.sh` is enough.
