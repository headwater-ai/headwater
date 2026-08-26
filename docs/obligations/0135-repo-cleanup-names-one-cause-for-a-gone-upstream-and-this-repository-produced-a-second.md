---
id: HW-OBL-0135
status: draft
status_since: 2026-08-26
title: "repo-cleanup names one cause for a gone upstream, and this repository produced a second"
summary: "Four merged branches survived repo-cleanup's first run with a gone upstream that ancestry misread, caused by a rewritten history the skill never names."
provenance:
  warrant: accepted
  agency: agent
  drafted_by: claude-sonnet-5
  activity: draft
  accepted_by: j.baxter
  evidence_basis: evidenced
last_verified: 2026-08-26
waiting_on: adopter
---

# repo-cleanup names one cause for a gone upstream, and this repository produced a second

## Context

The `repo-cleanup` skill states that a `gone` upstream usually follows a merge and never proves one, naming a closed pull request as the alternative. The skill's first run against this repository deleted 82 merged local branches and 22 worktrees, with zero refusals. It left four branches with a `gone` upstream that ancestry reported as unmerged.

## Obligation

Every one of those four branches had a merged pull request: `issue-124-decision-documents` (PR #128), `worktree-identifier-namespace-core` (PR #36), `worktree-downstream-standards-obligation` (PR #89), and `worktree-next-command` (PR #90). For PR #90 the local branch tip matched the pull request head at `61210e8`. GitHub records the merge commit as `1776b18`, which is not reachable from `origin/main`. The cause is a rewrite of this repository's history after the merges landed, a third cause the skill's `gone` paragraph does not name.

The distinction matters at the one moment the skill exists for. `git branch -d` refuses a branch whose ancestry a rewrite has broken, permanently, regardless of its content. A reader who follows the skill as written concludes that four merged branches were closed unmerged. The reader then either keeps them forever or reaches for `-D` for the wrong reason. The skill's report section also owes this distinction, because it currently lists "upstream gone and not merged" as one finding where it is two.

No fixture in `.claude/skills/fixtures.sh` holds this case, because the claim is about git rather than about the engine.

## Discharge

What closes this is a rewrite of the `gone` paragraph in `repo-cleanup`, naming both causes and how to tell them apart. The rewritten paragraph tells a reader to check the pull request state for the branch. It also tells the reader to compare the tree the branch produces against `origin/main` at the paths it touched. Content answers the question, and ancestry is only the proxy that a rewrite breaks. The report section owes the same split, so "gone and unmerged" no longer stands as a single finding.

A fixture over a scratch repository could hold the gap that `.claude/skills/fixtures.sh` currently leaves open. That fixture would merge a branch, rewrite the history carrying the merge, and assert that `-d` still refuses it. This obligation waits on the adopter to rewrite that paragraph and add the fixture.
