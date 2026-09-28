#!/bin/sh
# Fail when `headwater check --fix` writes anything to this tree.
#
#     sh tools/repo/fix-fixed-point.sh <headwater binary> [--now <date>]
#
# CI runs it in the `headwater` job so that `main` stays a fixed point of
# `--fix`. Before it existed, a contributor who ran `--fix` before a commit got
# edits to documents their change never touched, and reverted them by hand.
# Most of those edits were `verified_revision` stamps. A stamp says that a
# person read the document against the file it governs. The engine can write
# the digest, and it cannot do the reading. Since #1259 `--fix` offers a stamp
# only in a run with `--change`, on a document the change re-verified, and
# this passes no change. So a stamp never moves this tree, and the answer is
# the same on every date. What can still move it is a patch that needs no
# reading, such as a spelling or a reciprocal half. The remedy this prints is
# not to commit what `--fix` wrote blind: run it, read the diff, and commit
# what is still true.
#
# # WHAT IS COMPARED, AND WHY NOT `git status`
#
# The steps before this one in CI leave report files in the tree, so a clean
# tree is not a precondition this can require. The first version compared
# `git status --porcelain` before and after, and that misses a write into a
# tracked file that was already modified: the file reads ` M` on both sides.
# So this compares content. It stages the whole tree, untracked files
# included, into a scratch copy of the index and writes a tree object, before
# the run and after it. Two equal tree identifiers mean no byte moved in any
# file git does not ignore. The real index and the working tree are not
# touched by the snapshot. A file that `.gitignore` names is outside the
# comparison, and `--fix` writes none: it writes documents and claim files,
# and both are tracked.
#
# Any other argument is passed to `headwater check` after `--fix --no-cache`.
# A refused patch makes `check --fix` exit 1, and this exits 1 with it.
#
# `tools/repo/fix-fixed-point-fixtures.sh` holds each arm of this with a stub
# in place of the engine.

set -u

[ $# -ge 1 ] || {
    printf 'usage: sh tools/repo/fix-fixed-point.sh <headwater binary> [check arguments]\n' >&2
    exit 2
}
headwater=$1
shift

top=$(git rev-parse --show-toplevel) || exit 1
cd "$top" || exit 1

scratch=$(mktemp -d) || exit 1
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

# The tree object of everything git does not ignore, as it stands now.
snapshot() {
    index=$(git rev-parse --git-path index) || return 1
    rm -f "$scratch/index"
    if [ -f "$index" ]; then
        cp "$index" "$scratch/index" || return 1
    fi
    GIT_INDEX_FILE="$scratch/index" git add -A -- . || return 1
    GIT_INDEX_FILE="$scratch/index" git write-tree
}

before=$(snapshot) || {
    printf 'fix-fixed-point: could not read the tree before the run\n' >&2
    exit 1
}

"$headwater" check --fix --no-cache "$@" > /dev/null 2> "$scratch/fix.err"
status=$?

after=$(snapshot) || {
    printf 'fix-fixed-point: could not read the tree after the run\n' >&2
    exit 1
}

if [ "$status" -ne 0 ]; then
    printf 'headwater check --fix exited %s:\n' "$status"
    cat "$scratch/fix.err"
    exit 1
fi

if [ "$before" != "$after" ]; then
    printf 'headwater check --fix wrote to this tree, which should be a fixed point of it:\n'
    grep '^headwater: fixed ' "$scratch/fix.err"
    git diff --stat "$before" "$after"
    git diff "$before" "$after"
    printf "Run 'headwater check --fix' locally, read each file it writes, and commit only what is still true.\n"
    exit 1
fi

printf 'headwater check --fix wrote nothing\n'
