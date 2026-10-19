#!/bin/sh
# Is the main checkout behind origin/main? The one copy of the comparison
# that a build-order run makes before it dispatches anything (#1811).
#
#     sh tools/run/stale-main.sh <dir>
#
# <dir> is any directory of a repository: a main checkout or one of its
# worktrees. The main checkout is the directory whose `.git` is <dir>'s
# common dir, and its working tree is where the harness reads the agent
# definitions a run dispatches. So a main checkout behind origin/main
# dispatches old definitions, and in run 20261018-0144 it was 110 commits
# behind and nothing said so.
#
# When the main checkout's HEAD is behind `refs/remotes/origin/main`, it
# prints one line on standard output and exits 1:
#
#     STALE: main checkout at <short>, origin/main at <short>, <n> commits behind. Run: git -C <main checkout> merge --ff-only origin/main
#
# Otherwise it prints nothing and exits 0. That includes a checkout ahead of
# origin/main and behind it by none, and every case with nothing to judge:
# <dir> is not in a repository, the common dir is not `<checkout>/.git` (a
# bare repository with worktrees), or there is no `refs/remotes/origin/main`
# (the CI checkout of a pull request has none). A checkout that has diverged
# from origin/main is behind it, and so it is stale.
#
# It judges HEAD and not `refs/heads/main`, because the definitions the
# harness reads are the checkout's working tree, whichever branch it is on.
# The common dir's own HEAD file is the main checkout's HEAD.
#
# It never fetches. origin/main is as the common dir last fetched it, which
# every worktree of a run updates, because remote-tracking refs live in the
# common dir. A run with no fetch gives a false negative, never a false
# positive.
#
# `run-dir.sh` (start, claim, stage, next) and `supervise.sh` call it, and
# `tools/run/run-dir-fixtures.sh` and `tools/run/supervise-fixtures.sh` hold
# it.

set -u

[ $# -eq 1 ] || { echo "usage: sh tools/run/stale-main.sh <dir>" >&2; exit 2; }

common=$(git -C "$1" rev-parse --path-format=absolute --git-common-dir 2>/dev/null) || exit 0
[ -n "$common" ] || exit 0
case $common in
    */.git) ;;
    *) exit 0 ;;
esac
checkout=${common%/.git}
upstream=$(git --git-dir="$common" rev-parse -q --verify refs/remotes/origin/main 2>/dev/null) || exit 0
head=$(git --git-dir="$common" rev-parse -q --verify HEAD 2>/dev/null) || exit 0
behind=$(git --git-dir="$common" rev-list --count "$head..$upstream" 2>/dev/null) || exit 0
[ "${behind:-0}" -gt 0 ] || exit 0
printf 'STALE: main checkout at %s, origin/main at %s, %s commits behind. Run: git -C %s merge --ff-only origin/main\n' \
    "$(git --git-dir="$common" rev-parse --short "$head")" \
    "$(git --git-dir="$common" rev-parse --short "$upstream")" \
    "$behind" "$checkout"
exit 1
