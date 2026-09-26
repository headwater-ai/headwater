---
id: HW-OBL-0202
status: draft
status_since: 2026-09-23
summary: "tools/run/run-dir.sh claim treats the literal string none as an artifact path with no special case. As a result, two issues that both report footprint none produce a spurious WAITS-ON even when they touch disjoint files."
last_verified: 2026-09-23
title: "A bare none footprint is claimed as a literal path, so two disjoint issues collide"
waiting_on: build
---

# A bare none footprint is claimed as a literal path, so two disjoint issues collide

## Context

`hw-adjudicate.md` documents `FOOTPRINT: <path list> or none` as the literal expected value for a change with no derived artifact. `tools/run/run-dir.sh claim` reads that reported footprint and claims it with `<dir>/claims/artifacts/<artifact>`, with no case for the word `none`. Run `20260922-1121` hit this directly: issue #828 (sort/comm collation pins) claimed footprint `none` first. Issue #847's own footprint (also the bare word `none`, though its real files are `tools/repo/retire-worktree.sh` and siblings) came back `HELD: none by #828 / WAITS-ON: 828`, serializing two pieces of work that share no file.

## Obligation

**No two issues that both report no derived artifact should collide with each other in the claim store.** The bare word `none` is a sentinel meaning "nothing to claim," not a path. Treating it as one artificially serializes unrelated narrow-footprint work every time it recurs. `tools/run/run-dir-fixtures.sh` has no case for two narrow-footprint issues in flight together, so the gap is untested rather than a known and accepted limitation.

## Discharge

**The fix is in `run-dir.sh claim`, not in a run's own dispatch.** One option is to special-case the literal string `none` so it is never claimed as a path. The other is to change the contract so a caller with no artifact passes nothing rather than the word `none`. This run worked around the collision by hand, re-claiming #847 with an issue-qualified string (`none (tools/repo/retire-worktree.sh, tools/repo/retire-worktree-fixtures.sh, DEVELOPING.md)`) instead of the bare word, which is not a fix and will not recur automatically the next time two narrow-footprint issues land in the same run. Nothing discharges this until `run-dir.sh claim` and its fixture script both carry the special case.
