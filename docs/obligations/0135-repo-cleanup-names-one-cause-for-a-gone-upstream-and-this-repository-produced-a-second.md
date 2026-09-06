---
id: HW-OBL-0135
status: discharged
status_since: 2026-09-06
title: "repo-cleanup names one cause for a gone upstream, and this repository produced a second"
summary: "Ancestry misreads a merged branch as unmerged wherever a squash merge or a rewritten history left it unreachable, and git branch -d then refuses on no evidence about content."
provenance:
  warrant: accepted
  agency: agent
  drafted_by: claude-sonnet-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
last_verified: 2026-09-06
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

**This record is discharged.** The `gone` paragraph of the skill names three causes. A merged pull request deletes its own head branch wherever the repository sets `delete_branch_on_merge`. A closed pull request deletes a branch it did not merge. A history rewritten after the merge leaves the branch unreachable although its work landed. Pull request state separates the first cause from the second, and content separates the third from a branch that holds real unmerged work.

**The rewrite this record named is one case of a wider fault, and squash merging is the ordinary one.** A `gone` upstream and a broken ancestry are two questions rather than one. This repository squash merges, so the forge writes a single commit onto `origin/main` and no commit of the branch is an ancestor of it. Ancestry reports every finished branch as unmerged, permanently, with no rewrite and no deleted remote in the story.

**A second run of the skill measured the size of that fault.** Of 35 local branches, `git branch --merged origin/main` reported 8. Of the 29 branches the run then deleted, `git branch -d` refused all 29. Every one of the 29 carried a pull request whose merged head equaled the local tip or descended from it.

**The refusal from `git branch -d` carries no information about content, and that is the correction both causes share.** The flag measures a branch against the upstream of that branch, and against `HEAD` where the upstream is gone. It never measures against `origin/main` unless `origin/main` is the branch checked out. The four survivors of the first run and the twenty-nine of the second have this one shape.

**The skill decides merged by content, and ancestry opens a sweep rather than closing one.** A section states the pull request as the record that survives all three causes. A branch with no pull request gets the same question asked of the commit subjects that reached `origin/main` under another identifier. The report section splits a `gone` upstream into a closed pull request and a branch that never had one.

**The fixture this record left open now holds the claim.** `.claude/skills/fixtures.sh` builds a scratch repository three times. The cases are a squash merge and a history rewritten after a fast-forward merge. The third is a branch merged into main while `HEAD` sits elsewhere. Each case pairs one sentence of the skill with the verdict git returns, so the prose and git cannot disagree without a failure.
