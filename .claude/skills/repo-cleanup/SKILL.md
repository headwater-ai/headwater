---
name: repo-cleanup
description: Retire the worktree, the local branch and the remote branch that a finished change leaves behind, and report what is not safe to retire. Use when a pull request is merged, when asked to clean up, sweep old branches, remove a worktree or delete merged branches, and when a session opens on a repository whose branch list no longer matches its open work. It deletes what is merged and never what is not.
---

# Repository cleanup

A finished change leaves three things: a worktree directory, a local branch, and a remote branch. Merging a pull request retires at most one of them, so the other two survive every session that does not name them. The residue is silent, and it accumulates until the branch list is no longer a picture of the open work.

Cleanup has one rule and it is not negotiable: **delete what is merged, and report what is not.** A branch that no longer exists is recoverable from the reflog for a while and from nowhere after that, so an unmerged branch is a thing to name rather than a thing to decide about.

## Read the state first

    git fetch origin --prune
    git worktree list
    git branch --merged origin/main --format='%(refname:short)'
    git branch --no-merged origin/main --format='%(refname:short)'
    git branch --format='%(refname:short) %(upstream:track)' | grep gone

`--prune` is what makes the fourth and fifth commands true. Without it a remote branch that the merge deleted still looks present, `gone` marks nothing, and the sweep finds no work to do.

The three lists answer different questions and a branch can be on more than one. Merged into `origin/main` is the question that licenses a deletion. `gone` upstream means the remote branch was deleted, which usually follows a merge and never proves one: a closed pull request deletes a branch it did not merge.

## The order that works

A branch checked out in a worktree cannot be deleted, and a worktree whose directory is removed by hand leaves an administrative entry behind. So the order is fixed.

    git worktree remove <path>          # or --force when the tree is dirty and the change is merged
    git worktree prune                  # clears entries whose directory is already gone
    git branch -d <branch>              # -d refuses an unmerged branch, which is the point
    git push origin --delete <branch>   # only when the remote branch survived the merge

Use `git branch -d` and never `-D`. The lower-case flag refuses to delete a branch that is not merged, so the safety rule is enforced by git rather than by attention. When `-d` refuses, that branch belongs in the report, not in a second attempt with `-D`.

`git worktree remove` refuses a tree with uncommitted changes. That refusal is information: read the diff before reaching for `--force`, because an uncommitted change in a stale worktree is work that no branch holds.

## In this harness

A session that entered a worktree through the harness leaves it through the harness, and `ExitWorktree` removes the tree when the branch is unchanged. A worktree created with `git worktree add` is not tracked that way and needs the commands above.

Never remove the worktree a session is running in. Leave that one to the exit, clean up the others, and say which one was skipped and why.

## What the report says

Cleanup ends with a report rather than a count, because the deletions are unremarkable and the survivors are the finding.

- what was deleted, as one line per kind rather than one per branch
- every branch that `-d` refused, with the commit it holds that `origin/main` does not
- every worktree left in place, with the reason: dirty, in use, or unmerged
- every branch whose upstream is `gone` but which is not merged, because that pair means a pull request was closed rather than merged

A sweep that reports nothing surviving has usually not fetched with `--prune`.

## What this skill does not touch

`main` is never a candidate. Neither is a remote branch under a repository this session does not own, and neither is a tag. A branch that carries an open pull request stays, whatever its merge state says: check with `gh pr list --state open` before a sweep that deletes more than one branch.
