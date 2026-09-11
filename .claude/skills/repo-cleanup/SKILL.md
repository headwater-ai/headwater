---
name: repo-cleanup
description: Retire the worktree, the local branch and the remote branch that a finished change leaves behind, and report what is not safe to retire. Use when a pull request is merged, when asked to clean up, sweep old branches, remove a worktree or delete merged branches, and when a session opens on a repository whose branch list no longer matches its open work. It deletes what is merged and never what is not, and it decides merged by content rather than by ancestry.
---

# Repository cleanup

A finished change leaves three things: a worktree directory, a local branch, and a remote branch. Merging a pull request retires at most one of them, so the other two survive every session that does not name them. The residue is silent, and it accumulates until the branch list is no longer a picture of the open work.

Cleanup has one rule and it is not negotiable: **delete what is merged, and report what is not.** A branch that no longer exists is recoverable from the reflog for a while and from nowhere after that, so an unmerged branch is a thing to name rather than a thing to decide about.

Merged is a question about content, not about ancestry. A squash merge, a rebase merge and a rewritten history each put the work of a branch into `origin/main` while leaving the branch unreachable from it. So the ancestry commands below open a sweep and never close one.

## Read the state first

    git fetch origin --prune
    git worktree list
    git branch --merged origin/main --format='%(refname:short)'
    git branch --no-merged origin/main --format='%(refname:short)'
    git branch --format='%(refname:short) %(upstream:track)' | grep gone
    gh pr list --state all --limit 100 --json number,state,headRefName,headRefOid

`--prune` is what makes the fifth command true. Without it a remote branch that the merge deleted still looks present, `gone` marks nothing, and the sweep finds no work to do.

The lists answer different questions and a branch can be on more than one. Read `--merged origin/main` as a floor rather than as an answer. It reports the branches git can prove reachable, and a repository that squash merges puts nearly every finished branch in the `--no-merged` list instead. The last command is what makes that list readable, because the state of a pull request says what ancestry cannot.

`gone` upstream means the remote branch was deleted. It usually follows a merge, it never proves one, and it has three causes a sweep must tell apart. A merged pull request deletes its own head branch wherever the repository sets `delete_branch_on_merge`. A closed pull request deletes a branch it did not merge. A history rewritten after the merge leaves the branch unreachable from `origin/main` although its work landed. Pull request state separates the first cause from the second, and the content check below separates the third from a branch that really does hold unmerged work.

## Deciding what merged means

Under a squash merge the forge writes one new commit on `origin/main`, and no commit of the branch is an ancestor of it. Under a rebase merge the forge writes copies that carry new identifiers. Under either one, ancestry reports a finished branch as unmerged and goes on doing so forever. Treat this as the normal case. A sweep that trusts ancestry in such a repository deletes nothing and reports everything, which is the same as not running.

The pull request is the record that survives all three causes, so ask it first.

    gh pr list --head <branch> --state all --json number,state,headRefOid

A state of `MERGED` says the content of the head commit it names reached `origin/main`. Hold the local tip against that head.

    git merge-base --is-ancestor <branch> <headRefOid>

A local tip equal to the merged head, or an ancestor of it, holds nothing the merge did not carry, and that licenses the deletion. A local tip that is not an ancestor of the merged head carries commits written after the merge, and that branch belongs in the report.

A branch with no pull request needs the same question asked of the content directly.

    git log --oneline origin/main..<branch>
    git log origin/main --oneline --grep='<issue or subject>'

Read the subjects. A squashed commit usually keeps the subject of the branch it carried, so the second command finds the work under its new identifier. A branch whose work sits in `origin/main` under another name is merged, whatever ancestry says. A branch whose work is nowhere in `origin/main` is a survivor, and the report is where it goes.

Do not read a large `git diff origin/main <branch>` as unmerged work. Two tips differ because `origin/main` moved on as well, so most of that diff is main's later content missing from an old branch.

## The order that works

A branch checked out in a worktree cannot be deleted, and a worktree whose directory is removed by hand leaves an administrative entry behind. So the order is fixed.

    git worktree remove <path>          # or --force when the tree is dirty and the change is merged
    git worktree prune                  # clears entries whose directory is already gone
    git branch -d <branch>              # refuses on ancestry alone
    git branch -D <branch>              # only for a branch the content check already cleared
    git push origin --delete <branch>   # only when the remote branch survived the merge

`git branch -d` is not the safety net it appears to be. It measures a branch against that branch's upstream, and against `HEAD` when the upstream is gone. It never measures against `origin/main` unless `origin/main` is the branch checked out. So `-d` refuses every squash-merged branch, and it also refuses a branch that is merged into `origin/main` while the session sits somewhere else. A refusal from `-d` carries no information about content.

The safety net is the content check above. It is a thing the sweep performs rather than a thing git performs, so state in the report which branches needed `-D` and what cleared each one. A branch the content check did not clear belongs in the report, not in a second attempt with `-D`.

Write the tips down before deleting anything, because the reflog of a deleted branch is no longer reachable by name.

    git branch --format='%(refname:short) %(objectname)' > .git/branch-tips-before-cleanup.txt

`git worktree remove` refuses a tree with uncommitted changes. That refusal is information: read the diff before reaching for `--force`, because an uncommitted change in a stale worktree is work that no branch holds. Untracked build output is not such work, and a tree whose only change is a build directory is a clean tree for this purpose.

## In this harness

A session that entered a worktree through the harness leaves it through the harness, and `ExitWorktree` removes the tree when the branch is unchanged. A worktree created with `git worktree add` is not tracked that way and needs the commands above.

Never remove the worktree a session is running in. Leave that one to the exit, clean up the others, and say which one was skipped and why.

A worktree that `git worktree list` marks `locked` is a deliberate hold, and often a session still running inside it. Leave it, and name it in the report with its lock as the reason.

## What the report says

Cleanup ends with a report rather than a count, because the deletions are unremarkable and the survivors are the finding.

- what was deleted, as one line per kind rather than one per branch
- every branch that needed `-D`, with the pull request or the commit that cleared it
- every branch the content check did not clear, with the commits it holds that `origin/main` lacks
- every worktree left in place, with the reason: dirty, locked, in use, or unmerged
- every branch whose upstream is `gone` and whose pull request was closed rather than merged
- every branch whose upstream is `gone` and which never had a pull request at all

The last two are separate findings. A `gone` upstream has more than one cause, and a report that merges them leaves a reader no way to tell which one to act on.

A sweep that reports nothing surviving has usually not fetched with `--prune`. A sweep that reports nothing deleted has usually trusted ancestry in a repository that squash merges.

## The sweep that runs without being asked

`sh tools/repo/retire-worktree.sh` is this procedure as a tool, and `hw-integrate` runs it with `--retire` after every merge. It decides merged the way this document does, by the pull request rather than by ancestry, and it refuses a locked tree, a tree a live process is working in, a tree holding any uncommitted change, and every branch its content check does not clear. Reporting is its default, so a run with no flag is the read-the-state step above in one command.

Run it first. What it reports as kept is the list this skill exists to work through by hand, and it is a shorter list than the one a fetch and six `git` commands produce. It reaches nothing outside this repository, it never removes the tree it is running in, and it decides nothing about an unmerged branch beyond naming it.

## What this skill does not touch

`main` is never a candidate. Neither is a remote branch under a repository this session does not own, and neither is a tag. A branch that carries an open pull request stays, whatever its merge state says: check with `gh pr list --state open` before a sweep that deletes more than one branch.
