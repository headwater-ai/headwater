#!/bin/sh
# A build-order worktree: fetched, added and built, in one call.
#
# `hw-build.md` and `hw-verify.md` each spelled out `git fetch origin` then
# `git worktree add`, in the two shapes a build order needs — a new branch
# from `origin/main`, or an existing branch to attack — followed by a
# separate reminder to build the engine before any hook runs there. A fresh
# worktree has no engine, and the commit gate then fails open silently; this
# folds fetch, add and build into the one call a stage actually needs, so
# forgetting the build step stops being possible.
#
#     sh tools/repo/new-worktree.sh <path> -b <branch> <start-point>
#     sh tools/repo/new-worktree.sh <path> <existing-branch>
#
# Everything after <path> passes straight to `git worktree add`, so either
# shape works exactly as it would typed by hand. The engine then builds
# inside the new tree, through its own `tools/hw-cargo` and never a bare
# `cargo`, at --profile dev-release and never --release. HW_CARGO_SLOT, if
# already set in the environment, passes through untouched, so a caller with
# a reserved slot keeps it.
#
# How hw-verify calls it, and why, as run 20260923-0733 found:
#
#     HW_CARGO_SLOT=verify sh tools/repo/new-worktree.sh <path> --detach origin/<branch>
#
# --detach, because the build agent's own worktree still has <branch>
# checked out and git refuses a second checkout of one branch: `fatal:
# '<branch>' is already used by worktree at ...`. The verifier attacks the
# pushed tip and commits nothing, so a detached head costs it nothing.
#
# HW_CARGO_SLOT=verify, because the numbered pool is sized for builders. A
# verifier with no slot of its own queued on slot 1 behind three of them,
# gave up on its engine, and skipped `headwater check` and `generate --check`.
#
# A Bash timeout of 600000 on the call, because the build runs inside it. A
# cold slot builds for minutes, and at the default two-minute cap the harness
# moves the call to the background, where the caller then waits on a binary
# that the next `ls` does not find yet.

set -eu

usage() {
    sed -n '/^#     sh tools\/repo\/new-worktree.sh/p' "$0" | sed 's/^# *//' >&2
    exit 2
}

[ $# -ge 2 ] || usage
path=$1
shift

root=$(git rev-parse --show-toplevel)

git -C "$root" fetch origin
git -C "$root" worktree add "$path" "$@"

if [ ! -f "$path/engine/Cargo.toml" ]; then
    echo "new-worktree: no engine/Cargo.toml under $path; nothing to build." >&2
    exit 1
fi

(cd "$path" && sh tools/hw-cargo build --profile dev-release -p headwater-cli --manifest-path engine/Cargo.toml --locked)

echo "new-worktree: $path ready, engine built."
