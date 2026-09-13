#!/bin/sh
# Retire the worktrees and branches a merge finished, and report what is not.
#
# Why a tool and not a paragraph in an agent definition: the agent that makes a
# build worktree exits when the pull request opens, and the branch is unmerged
# at that moment, so that agent can never retire its own tree. The integrator is
# the first stage that knows the branch merged, and it owns no tree of its own.
# Ownership of the directory and knowledge of the merge sit in different agents,
# and neither can act alone. This sweep is what closes that gap: it runs from
# anywhere, it retires everything that has become retirable rather than only the
# branch that was just merged, and it is safe to run at any moment.
#
# Merged is a question about content. This repository squash-merges, so no
# commit of a finished branch is an ancestor of `origin/main` and ancestry
# reports every finished branch as unmerged forever. The pull request is the
# record that survives, so a branch is retired when a MERGED pull request names
# a head that contains the branch tip, and never on ancestry alone.
#
# Five guards, each of which cost somebody work before it was written here:
#
#   locked     a lock is a deliberate hold, and often a session still inside.
#   in use     a process whose working directory is under the tree. Removing it
#              pins the directory on disk and the next `cargo test` there exits
#              101 with a `NotFound` naming the binary under test, which reads
#              as a defect in the code rather than as a deleted directory.
#   dirty      any uncommitted change, tracked or not. Build output is ignored
#              and does not appear; a new document does, and it is work that no
#              branch holds.
#   in flight  a tree holding no commit `origin/main` lacks. `git worktree add
#              <path> -b <branch> origin/main` puts a new tree exactly at that
#              tip, so ancestry cleared such a tree from the first second of its
#              life and the sweep deleted the directory, and then the branch it
#              had just freed, while the agent that made it was still working.
#              Emptiness is what a tree in flight and a tree whose work landed
#              do not share: the second is cleared by its merged pull request,
#              which is asked first and needs no clock. So a tree is never
#              retired on ancestry alone. A branch with no tree still is,
#              because the absence of a tree is what says nobody is building on
#              it, and six such branches were found in one real sweep.
#   unmerged   the content check above did not clear it.
#
# What the `in flight` guard costs: a tree that is empty and also abandoned is
# now kept rather than collected, and every sweep reports it again. That is the
# trade, and it is this way round because the other direction destroys work in
# progress. Read the `KEPT` lines, and remove such a tree by hand once you know
# nobody is in it.
#
# Run it from anywhere:
#     sh tools/repo/retire-worktree.sh              # report what it would retire
#     sh tools/repo/retire-worktree.sh --retire     # retire it
#
# Reporting is the default because the action is destructive and a forgotten
# flag must not read as a clean sweep. The report says so in its last line.
#
# It needs `git`. It uses `gh` to read pull request state and says so when there
# is none, and it reads `/proc` for the in-use guard and says so when there is
# none. `HEADWATER_RETIRE_PR_STATE` names a file of `branch<TAB>STATE<TAB>oid`
# lines that stands in for `gh`, which is how the fixtures drive the merged arm.

set -u

retire=no
for arg in "$@"; do
    case $arg in
        --retire) retire=yes ;;
        --report|--check) retire=no ;;
        *) printf 'unknown argument `%s`\n' "$arg" >&2; exit 2 ;;
    esac
done

git rev-parse --git-dir >/dev/null 2>&1 || {
    echo 'not a git repository.' >&2
    exit 2
}

common=$(git rev-parse --path-format=absolute --git-common-dir)
main_tree=$(cd "$common/.." && pwd)
here=$(pwd -P)

work=$(mktemp -d) || exit 1
trap 'rm -rf "$work"' EXIT HUP INT TERM

# `--prune` is what makes a deleted remote branch visible as one. Without it the
# sweep reads a stale remote and a merged branch keeps looking live.
git fetch origin --prune >/dev/null 2>&1 || echo 'NOTE: could not fetch; reading the remote as it stands.'

pr_state_file=${HEADWATER_RETIRE_PR_STATE:-}
if [ -z "$pr_state_file" ]; then
    if command -v gh >/dev/null 2>&1; then
        pr_state_file="$work/pr-state"
        gh pr list --state all --limit 200 \
            --json headRefName,state,headRefOid \
            --jq '.[] | [.headRefName, .state, .headRefOid] | @tsv' \
            >"$pr_state_file" 2>/dev/null || : >"$pr_state_file"
    else
        echo 'NOTE: no `gh` on the path, so only ancestry can clear a branch.'
        pr_state_file="$work/pr-state"
        : >"$pr_state_file"
    fi
fi

# The first MERGED row for a branch, if any. A branch can carry several rows
# when it was reused, and a merged one anywhere in that history is what clears
# the content it carried.
merged_head() {
    awk -F'\t' -v b="$1" '$1 == b && $2 == "MERGED" { print $3; exit }' "$pr_state_file"
}

open_pr() {
    awk -F'\t' -v b="$1" '$1 == b && $2 == "OPEN" { found = 1 } END { print (found ? "yes" : "no") }' "$pr_state_file"
}

# Merged by content: the tip is inside a merged pull request head, or inside
# `origin/main` itself. Prints the reason that cleared it, or nothing.
#
# The third argument says whether ancestry may clear it. It is `no` for a
# worktree, where sitting at the tip of `origin/main` says the tree holds
# nothing rather than that its work landed, and `yes` for a branch with no tree.
cleared_by() {
    tip=$1 branch=$2 ancestry=${3:-yes}
    if [ -n "$branch" ]; then
        head=$(merged_head "$branch")
        if [ -n "$head" ] && git cat-file -e "$head" 2>/dev/null \
            && git merge-base --is-ancestor "$tip" "$head" 2>/dev/null; then
            printf 'inside the merged head of its pull request'
            return 0
        fi
    fi
    if [ "$ancestry" = yes ] && git merge-base --is-ancestor "$tip" origin/main 2>/dev/null; then
        printf 'an ancestor of origin/main'
        return 0
    fi
    # A detached tree names no branch, so there is no pull request to look up.
    # Ask every merged head instead: a verifier's scratch tree sits on a commit
    # that some other branch carried into a merge, and only containment says so.
    if [ -z "$branch" ]; then
        while IFS="$(printf '\t')" read -r row_branch row_state row_oid; do
            [ "$row_state" = MERGED ] || continue
            git cat-file -e "$row_oid" 2>/dev/null || continue
            if git merge-base --is-ancestor "$tip" "$row_oid" 2>/dev/null; then
                printf 'inside the merged head of #%s' "$row_branch"
                return 0
            fi
        done <"$pr_state_file"
    fi
    return 1
}

# A process whose working directory is at or under the path. Without `/proc`
# this cannot be asked, and the guard says so rather than passing quietly.
proc_available=yes
[ -d /proc/1 ] || proc_available=no

# Read every working directory once. Asking per tree re-walks every process for
# every candidate, which on a host running a fleet of agents is the whole cost
# of the sweep.
: >"$work/cwds"
if [ "$proc_available" = yes ]; then
    for link in /proc/[0-9]*/cwd; do
        readlink "$link" 2>/dev/null
    done | sort -u >"$work/cwds"
fi

in_use() {
    path=$1
    [ "$proc_available" = yes ] || return 1
    awk -v p="$path" '$0 == p || index($0, p "/") == 1 { found = 1; exit } END { exit !found }' "$work/cwds"
}

retired_trees=0
retired_branches=0
retired_remotes=0
kept=0

say_keep() {
    kept=$((kept + 1))
    printf 'KEPT     %s\n' "$1"
}

printf '# worktrees\n'

# One record per worktree: path, branch (empty when detached), tip, locked.
git worktree list --porcelain | awk '
    /^worktree /  { if (path != "") emit(); path = substr($0, 10); branch = ""; tip = ""; locked = "no" }
    /^HEAD /      { tip = substr($0, 6) }
    /^branch /    { branch = substr($0, 8); sub("^refs/heads/", "", branch) }
    /^locked/     { locked = "yes" }
    END           { if (path != "") emit() }
    function emit() { printf "%s\t%s\t%s\t%s\n", path, branch, tip, locked }
' >"$work/worktrees"

while IFS="$(printf '\t')" read -r path branch tip locked; do
    [ -n "$path" ] || continue
    label=${branch:-"detached at $(git rev-parse --short=8 "$tip" 2>/dev/null)"}

    if [ "$path" = "$main_tree" ]; then
        continue
    fi
    case "$here" in
        "$path"|"$path"/*)
            say_keep "$path — the sweep is running inside it"
            continue ;;
    esac
    if [ "$locked" = yes ]; then
        say_keep "$path — locked, which is a deliberate hold"
        continue
    fi
    if in_use "$path"; then
        say_keep "$path — a live process has its working directory there"
        continue
    fi
    if [ ! -d "$path" ]; then
        # The directory is gone and only the administrative entry survives.
        if [ "$retire" = yes ]; then
            git worktree prune >/dev/null 2>&1
            printf 'PRUNED   %s — the directory was already gone\n' "$path"
        else
            printf 'WOULD    prune %s — the directory is already gone\n' "$path"
        fi
        continue
    fi
    dirt=$(git -C "$path" status --porcelain 2>/dev/null | wc -l | tr -d ' ')
    if [ "${dirt:-0}" -gt 0 ]; then
        say_keep "$path ($label) — $dirt uncommitted change(s)"
        continue
    fi
    if [ -n "$branch" ] && [ "$(open_pr "$branch")" = yes ]; then
        say_keep "$path ($label) — its pull request is open"
        continue
    fi
    if reason=$(cleared_by "$tip" "$branch" no); then
        if [ "$retire" = yes ]; then
            if git worktree remove "$path" 2>"$work/err"; then
                retired_trees=$((retired_trees + 1))
                printf 'RETIRED  %s (%s) — %s\n' "$path" "$label" "$reason"
            else
                say_keep "$path ($label) — git refused to remove it: $(cat "$work/err")"
            fi
        else
            printf 'WOULD    retire %s (%s) — %s\n' "$path" "$label" "$reason"
        fi
    else
        ahead=$(git rev-list --count origin/main.."$tip" 2>/dev/null)
        if [ "${ahead:-}" = 0 ]; then
            say_keep "$path ($label) — holds no commit origin/main lacks, so it is a tree in flight rather than a tree whose work landed"
        else
            say_keep "$path ($label) — unmerged, holding ${ahead:-?} commit(s) origin/main lacks"
        fi
    fi
done <"$work/worktrees"

printf '\n# branches\n'

# Re-read the worktrees after the removals above, so a branch freed by this
# sweep is deletable in the same run.
git worktree list --porcelain | sed -n 's|^branch refs/heads/||p' | sort -u >"$work/checked-out"

current=$(git symbolic-ref --quiet --short HEAD 2>/dev/null || echo '')

git for-each-ref --format='%(refname:short)' refs/heads >"$work/branches"

while IFS= read -r branch; do
    [ -n "$branch" ] || continue
    [ "$branch" = main ] && continue
    [ "$branch" = "$current" ] && continue
    if grep -qxF "$branch" "$work/checked-out"; then
        continue
    fi
    if [ "$(open_pr "$branch")" = yes ]; then
        say_keep "$branch — its pull request is open"
        continue
    fi
    if reason=$(cleared_by "$branch" "$branch" yes); then
        if [ "$retire" = yes ]; then
            # `-d` measures a branch against its own upstream and never against
            # `origin/main`, so it refuses every squash-merged branch. The
            # content check above is the safety net; `-D` is how it is spent.
            if git branch -D "$branch" >/dev/null 2>&1; then
                printf 'RETIRED  %s — %s\n' "$branch" "$reason"
                if git ls-remote --exit-code --heads origin "$branch" >/dev/null 2>&1; then
                    if git push origin --delete "$branch" >/dev/null 2>&1; then
                        printf 'RETIRED  origin/%s — the remote branch survived the merge\n' "$branch"
                    fi
                fi
            fi
        else
            printf 'WOULD    retire %s — %s\n' "$branch" "$reason"
        fi
    else
        ahead=$(git rev-list --count "origin/main..$branch" 2>/dev/null)
        say_keep "$branch — unmerged, holding ${ahead:-?} commit(s) origin/main lacks"
    fi
done <"$work/branches"

printf '\n'
if [ "$retire" = yes ]; then
    printf 'RETIRED: %s worktree(s), and the branches named above\n' "$retired_trees"
else
    printf 'RETIRED: nothing. This was a report; re-run with --retire to act.\n'
fi
[ "$proc_available" = yes ] || printf 'NOTE: no /proc, so the in-use guard could not be asked.\n'
