#!/bin/sh
# Print the free space on the volume each build-order job writes to: the
# cargo pool, where every verifier and builder makes a 10-13 GB target, and
# the worktrees directory, where every agent's tree lives.
#
#     sh tools/run/disk-free.sh [--floor <G>]
#
# It prints two lines, one per role, each naming the resolved path it
# measured, the mount point that path is on, and the free space in G
# (1024^3 bytes, rounded down):
#
#     pool       /mnt/build/headwater/cargo-pool  mount=/mnt/build  free=86G
#     worktrees  /home/james/projects/headwater/.claude/worktrees  mount=/home/james/projects  free=47G
#
# Why it exists: hw-verify and hw-iterate guarded a verify with `df -h /`,
# but on 2026-10-07 `~/.cache/headwater/cargo-pool` resolved to
# `/mnt/build/headwater/cargo-pool`, on its own volume, so the guard read a
# volume no build writes to (#1694). The host has three mounts, and a reading
# is only evidence about the volume it was taken on.
#
# The pool is `${HW_CARGO_POOL:-${XDG_CACHE_HOME:-$HOME/.cache}/headwater/cargo-pool}`,
# the same expression as `tools/hw-cargo`. The worktrees directory is
# `<main>/.claude/worktrees`, with <main> computed as
# `tools/repo/new-worktree.sh` computes it; `HW_WORKTREES` overrides it. Each
# path is resolved through its symlinks before `df` reads it, and a path that
# does not exist yet is measured at its deepest existing ancestor. Nothing is
# created.
#
# Exit status: 0; 1 when `--floor <G>` is given and the pool's volume has
# less than <G> free (the worktrees volume never fails the floor); 2 on a bad
# argument or when a path cannot be measured.
#
# Fixtures: `sh tools/run/disk-free-fixtures.sh`.

set -u

usage() {
    echo "usage: sh tools/run/disk-free.sh [--floor <G>]" >&2
    exit 2
}

floor=
while [ $# -gt 0 ]; do
    case "$1" in
        --floor)
            [ $# -ge 2 ] || usage
            case "$2" in
                ''|*[!0-9]*) echo "disk-free: --floor takes a whole number of G, not \`$2\`." >&2; exit 2 ;;
            esac
            floor=$2
            shift 2
            ;;
        *) usage ;;
    esac
done

pool=${HW_CARGO_POOL:-${XDG_CACHE_HOME:-$HOME/.cache}/headwater/cargo-pool}

if [ -n "${HW_WORKTREES:-}" ]; then
    worktrees=$HW_WORKTREES
else
    common=$(git rev-parse --path-format=absolute --git-common-dir 2>/dev/null) || {
        echo "disk-free: not in a git repository, so the worktrees directory is unknown; set HW_WORKTREES." >&2
        exit 2
    }
    main=$(dirname "$common")
    main=$(cd "$main" && pwd -P)
    worktrees="$main/.claude/worktrees"
fi

# The symlink-free form of the deepest part of $1 that exists.
resolve() {
    p=$1
    while [ ! -e "$p" ]; do
        parent=$(dirname "$p")
        [ "$parent" != "$p" ] || return 1
        p=$parent
    done
    [ -d "$p" ] || p=$(dirname "$p")
    echo "$p"
}

# Sets $at, $mount and $free_g for the path $1, or exits 2.
measure() {
    at=$(resolve "$1") || { echo "disk-free: no part of \`$1\` exists." >&2; exit 2; }
    line=$(df -P -k "$at" | tail -n 1) || { echo "disk-free: df could not read \`$at\`." >&2; exit 2; }
    set -- $line
    [ $# -ge 6 ] || { echo "disk-free: df printed \`$line\` for \`$at\`." >&2; exit 2; }
    avail_k=$4
    shift 5
    mount=$*
    case "$avail_k" in
        ''|*[!0-9]*) echo "disk-free: df printed \`$line\` for \`$at\`." >&2; exit 2 ;;
    esac
    free_g=$((avail_k / 1048576))
}

measure "$pool"
pool_free=$free_g
echo "pool       $at  mount=$mount  free=${free_g}G"

measure "$worktrees"
echo "worktrees  $at  mount=$mount  free=${free_g}G"

if [ -n "$floor" ] && [ "$pool_free" -lt "$floor" ]; then
    echo "disk-free: the pool's volume has ${pool_free}G free, under the floor of ${floor}G." >&2
    exit 1
fi
exit 0
