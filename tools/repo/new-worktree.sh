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
