---
id: HW-OBL-0217
status: current
status_since: 2026-09-26
summary: "The pre-push hook reads a site-review marker that every worktree of a clone shares, so one unreviewed site merge refuses the push of every worktree."
last_verified: 2026-09-26
title: "The site-review marker lives in the common directory of a clone, so one worktree's unreviewed merge blocks the push of every worktree"
waiting_on: build
---

# The site-review marker lives in the common directory of a clone, so one worktree's unreviewed merge blocks the push of every worktree

## Context

Run `20260926-0718` met this defect when the push of #615 was refused. The product owner ruled it Record, because the hooks belong to this repository and do not ship to an adopter.

## Obligation

`.githooks/merge-regenerate` writes a marker when it refuses a conflicting merge on a `site/*` page and the two sides differ outside their figure spans. The marker goes under `headwater-pending-site-review/` in the directory that `git rev-parse --git-common-dir` names. `tools/site/ack-site-prose-reviewed.sh` lists and clears the markers in the same directory. `.githooks/pre-push` refuses every push while that script lists a marker.

Every worktree of a clone shares the common directory. So a marker that one worktree writes also refuses the push of each other worktree, which has no `site/*` merge to review. The header of `.githooks/pre-push` says that the refusal applies to "this clone". It does not say that a clone with several worktrees has several branches that the refusal blocks.

## Discharge

This record discharges when two conditions hold. First, the marker is keyed to the worktree or to the commit that holds the unreviewed resolution. Second, `.githooks/pre-push` reads only the markers of the worktree that pushes. A case writes a marker in one worktree of a clone. The case then requires that a push from a second worktree succeeds, and that a push from the first worktree is refused.
