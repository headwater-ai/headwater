---
id: HW-OBL-0210
status: draft
status_since: 2026-09-24
summary: "The worktree script's suite stays green when pwd -P is dropped, and --name .. passes the bare-name check and leaves a branch when git then fails."
last_verified: 2026-09-24
title: "No fixture holds the symlink case of new-worktree.sh, and a bare name of two dots leaves a branch behind"
waiting_on: build
---

# No fixture holds the symlink case of new-worktree.sh, and a bare name of two dots leaves a branch behind

## Context

The verifier of PR #1061 (#848) in run `20260924-0411` found two gaps in contributor tooling. The product owner ruled both Record.

## Obligation

**The symlink case has no fixture.** The script resolves the main checkout with `pwd -P`. If that call drops to a plain `pwd`, the suite stays green.

**A name of two dots leaves a branch.** `--name ..` passes the bare-name check. Git then fails to add the worktree, and the branch the call created stays behind.

## Discharge

The first gap discharges when a fixture runs the script through a symlinked path and fails with plain `pwd`. The second discharges when `--name ..` is refused before any branch exists, and a case holds that.
