#!/bin/sh
# What a verifier does before its first build, in one call: fetch, detach at
# the branch, read the disk, and name the slot and the head.
#
#     sh tools/run/verify-setup.sh <issue|none> <branch>   in the verifier's own worktree
#     sh tools/run/verify-setup.sh --clean <issue|none>    drop the slot's target directory from the pool
#
# `.claude/agents/hw-verify.md` stated these as four commands a verifier
# typed in order, and worktree isolation refuses the one shape that would
# have joined them: a `git` call inside a `;` sequence or a command
# substitution. So every verifier paid a turn for each, in its own order,
# and the disk reading was the one most often skipped. The sequence never
# varies, so it is a script, and the reading cannot be skipped.
#
# It prints, on exit 0:
#
#     HEAD: <sha>                       the commit it detached at, the one CI tested
#     SLOT: HW_CARGO_SLOT=verify-<N>    the slot every tools/hw-cargo call of this verify names
#     pool ... / worktrees ...          what disk-free.sh printed
#
# Exit 1, with `disk: <the reading>` as the first line, when the pool's
# volume has less free space than the floor: the verifier reports `FAIL`
# with that line and builds nothing, because its build would fill the volume
# that holds the pool. The floor is 40G, and `HW_VERIFY_FLOOR` overrides it
# for a fixture. Exit 2 on a refused argument, on the main checkout, and on
# a fetch or a detach that failed, each with one line on stderr.
#
# It refuses the main checkout: a worktree's git dir is not its common dir,
# and the shared checkout is never detached by a stage (the #978 adjudicator
# did, and left `HEAD` detached for everything after it).
#
# `--clean` removes `target-verify-<N>` and its `.root` under the pool, which
# the run policy has every verifier free when it reports, because each one
# is 10-13 GB. `HW_CARGO_POOL` names the pool, as it does for tools/hw-cargo.
# `HW_DISK_FREE` names the disk reader, for a fixture.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
floor=${HW_VERIFY_FLOOR:-40}
disk_free=${HW_DISK_FREE:-$root/tools/run/disk-free.sh}
pool=${HW_CARGO_POOL:-${XDG_CACHE_HOME:-$HOME/.cache}/headwater/cargo-pool}

usage() {
    sed -n '/^#     sh tools\/run\/verify-setup.sh/p' "$0" | sed 's/^# *//' >&2
    exit 2
}

# The slot of an issue, or of the run-lessons verify, which has none.
slot_of() {
    case $1 in
        none) printf 'verify-lessons' ;;
        '' | *[!0-9]*) echo "verify-setup: \`$1\` is not an issue number or \`none\`." >&2; exit 2 ;;
        *) printf 'verify-%s' "$1" ;;
    esac
}

if [ "${1:-}" = --clean ]; then
    [ $# -eq 2 ] || usage
    slot=$(slot_of "$2") || exit 2
    rm -rf "$pool/target-$slot" "$pool/target-$slot.root"
    echo "CLEANED: $pool/target-$slot and its .root"
    exit 0
fi

[ $# -eq 2 ] || usage
slot=$(slot_of "$1") || exit 2
branch=$2
case $branch in
    '' | -* | *[[:space:]]*) echo "verify-setup: \`$branch\` is not a branch name." >&2; exit 2 ;;
esac

gitdir=$(git rev-parse --absolute-git-dir 2>/dev/null) || { echo "verify-setup: not inside a git worktree." >&2; exit 2; }
common=$(cd "$(git rev-parse --git-common-dir)" 2>/dev/null && pwd -P)
gitdir=$(cd "$gitdir" && pwd -P)
if [ "$gitdir" = "$common" ]; then
    echo "verify-setup: this is the main checkout, which a verifier never detaches. Run it in the worktree the harness made for you." >&2
    exit 2
fi

git fetch -q origin || { echo "verify-setup: git fetch origin failed." >&2; exit 2; }
git switch -q --detach "origin/$branch" || { echo "verify-setup: no \`origin/$branch\` to detach at." >&2; exit 2; }
head=$(git rev-parse HEAD)

reading=$(sh "$disk_free" --floor "$floor" 2>&1)
status=$?
if [ "$status" -eq 1 ]; then
    printf 'disk: %s\n' "$(printf '%s\n' "$reading" | sed -n 's/^disk-free: //p' | head -n 1)"
fi
printf 'HEAD: %s\nSLOT: HW_CARGO_SLOT=%s\n' "$head" "$slot"
printf '%s\n' "$reading" | grep -v '^disk-free:'
[ "$status" -eq 0 ] || exit 1
