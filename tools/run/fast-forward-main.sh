#!/bin/sh
# Bring the main checkout to origin/main, by a fast-forward and nothing else.
# The one copy of the move that a build-order run makes (#1997).
#
#     sh tools/run/fast-forward-main.sh <dir>
#
# <dir> is any directory of a repository: a main checkout or one of its
# worktrees. The main checkout is the directory whose `.git` is <dir>'s
# common dir, as in `tools/run/stale-main.sh`, and its working tree is where
# the harness reads the agent definitions a run dispatches. Its absolute
# `core.hooksPath` also points there, so a stale one runs stale hooks under
# every worktree.
#
# It prints one line on standard output, and its exit status names the
# state, so that a caller can say what to do without parsing the line:
#
#     0  FAST-FORWARD: main at <short>
#     3  FAST-FORWARD: skipped, <dir> is not a worktree of a main checkout, nor a main checkout
#     4  FAST-FORWARD: skipped, <main> is on `<branch>`, not main      (or `a detached HEAD`)
#     5  FAST-FORWARD: skipped, <main> is not clean
#     6  FAST-FORWARD: skipped, fetch from origin failed
#     7  FAST-FORWARD: skipped, <main> does not fast-forward to origin/main
#
# It moves the checkout only when it is on `main` and clean and the fetch
# succeeds, and only by `merge --ff-only origin/main`. It never rebases,
# resets or stashes, so a checkout it refuses is where it was.
#
# `tools/run/supervise.sh` calls it before each session, and prints its line
# whatever the status. `tools/run/run-dir.sh start` calls it when stale-main.sh
# says the checkout is behind, prints the line on a move, and turns every
# other status into one `STALE:` line that names the state and the command
# the owner runs. `tools/run/supervise-fixtures.sh` and
# `tools/run/run-dir-fixtures.sh` hold it.

set -u

[ $# -eq 1 ] || { echo "usage: sh tools/run/fast-forward-main.sh <dir>" >&2; exit 2; }

common=$(git -C "$1" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)
main=${common%/.git}
if [ -z "$common" ] || [ "$main" = "$common" ] || [ ! -d "$main" ]; then
    echo "FAST-FORWARD: skipped, $1 is not a worktree of a main checkout, nor a main checkout"
    exit 3
fi
branch=$(git -C "$main" symbolic-ref --short -q HEAD)
if [ "$branch" != main ]; then
    echo "FAST-FORWARD: skipped, $main is on \`${branch:-a detached HEAD}\`, not main"
    exit 4
fi
if [ -n "$(git -C "$main" status --porcelain 2>/dev/null)" ]; then
    echo "FAST-FORWARD: skipped, $main is not clean"
    exit 5
fi
if ! git -C "$main" fetch -q origin 2>/dev/null; then
    echo "FAST-FORWARD: skipped, fetch from origin failed"
    exit 6
fi
if git -C "$main" merge -q --ff-only origin/main >/dev/null 2>&1; then
    echo "FAST-FORWARD: main at $(git -C "$main" rev-parse --short HEAD)"
    exit 0
fi
echo "FAST-FORWARD: skipped, $main does not fast-forward to origin/main"
exit 7
