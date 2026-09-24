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
#     sh tools/repo/new-worktree.sh --name <name> <git-worktree-add-args>
#
# Everything after <path> passes straight to `git worktree add`, so either
# shape works exactly as it would typed by hand. `--name <name>` stands for
# the path `<main>/.claude/worktrees/<name>`, so a caller never computes a
# root of its own.
#
# The main checkout, and why it is not `--show-toplevel` (#848): an agent
# whose session inherited a linked worktree asked `git rev-parse
# --show-toplevel` for its root, got that linked tree, and built its new tree
# nested inside it, so builds took over each other's worktrees. The main
# checkout is the parent of `--git-common-dir`, the same derivation
# `tools/repo/retire-worktree.sh` and `tools/run/run-dir.sh` use, and fetch
# and add run there. A <path> inside any linked worktree is refused, and the
# refusal names the path under the main checkout; the tool never moves a path
# silently. A relative <path> is read against the caller's directory. The engine then builds
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

name=
if [ "${1:-}" = --name ]; then
    [ $# -ge 3 ] || usage
    case $2 in
        '' | */*) echo "new-worktree: --name takes a bare name, not a path: '$2'." >&2; exit 2 ;;
    esac
    name=$2
    shift 2
else
    [ $# -ge 2 ] || usage
    path=$1
    shift
fi

main=$(dirname "$(git rev-parse --path-format=absolute --git-common-dir)")
main=$(cd "$main" && pwd -P)
[ -z "$name" ] || path="$main/.claude/worktrees/$name"

# The absolute, symlink-free form of <path>: the deepest part that exists,
# resolved by `pwd -P`, then the rest appended. The path does not exist yet.
case $path in
    /*) abs=$path ;;
    *) abs="$(pwd)/$path" ;;
esac
head=$abs
tail=
while [ ! -d "$head" ]; do
    tail="/$(basename "$head")$tail"
    head=$(dirname "$head")
done
abs="$(cd "$head" && pwd -P)$tail"

trees=$(git -C "$main" worktree list --porcelain | sed -n 's/^worktree //p')
set -f
old_ifs=$IFS
IFS='
'
for tree in $trees; do
    [ "$tree" = "$main" ] && continue
    case "$abs/" in
        "$tree"/*)
            echo "new-worktree: $abs is inside the linked worktree $tree." >&2
            echo "  A root from \`git rev-parse --show-toplevel\` there answers that tree, not the main checkout." >&2
            echo "  Use $main/.claude/worktrees/$(basename "$abs"), or --name $(basename "$abs")." >&2
            exit 1
            ;;
    esac
done
IFS=$old_ifs
set +f
path=$abs

git -C "$main" fetch origin
git -C "$main" worktree add "$path" "$@"

if [ ! -f "$path/engine/Cargo.toml" ]; then
    echo "new-worktree: no engine/Cargo.toml under $path; nothing to build." >&2
    exit 1
fi

(cd "$path" && sh tools/hw-cargo build --profile dev-release -p headwater-cli --manifest-path engine/Cargo.toml --locked)

echo "new-worktree: $path ready, engine built."
