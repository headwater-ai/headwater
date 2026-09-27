#!/bin/sh
# Remove every answer key from a probe workspace, in both arms.
#
# A probe's own document states its expectation, a recorded run states what a
# session answered, and a derived fold names the probe beside its target. A
# session that reads any of them reads its answer key. The tombstone session
# of 2026-09-17 did exactly that: it listed `docs/probes/` in its copy of this
# corpus, read its own probe file, and then answered (#1229). `ablate.sh`
# removes the four paths that deliver governance and leaves `docs/` alone, so
# before this script both arms held every probe.
#
#     sh tools/probe/seal.sh <workspace> <probe-id>...
#
# It deletes, in place:
#
# - `docs/probes/`, `docs/probe-runs/` and `docs/probe-results/`;
# - every file under the workspace whose bytes contain a named probe's
#   identifier or its slug, the file name the probe has on the shelf. That
#   reaches the derived folds under `.headwater/`, the census and graph
#   fixtures under `engine/`, and every obligation and evaluation that names
#   the probe.
#
# No probe's `examines` target names a probe, so the documents a probe tests
# survive the seal. `.headwater/probe.yml` records what the seal removes, and
# `probe-record.sh` refuses a workspace that still names the probe (exit 9).
#
# It refuses a workspace inside this checkout (exit 6), the guard `ablate.sh`
# applies for the same reason. It is idempotent: the slug is read from this
# checkout's shelf, so a second run over a sealed tree finds the same names and
# nothing left to delete.

set -eu

# `pwd -P` for the reason `ablate.sh` gives: a symlinked workspace must not
# reach this checkout through the prefix check below.
case $0 in
    */*) invoked_from=${0%/*} ;;
    *) invoked_from=. ;;
esac
root=$(cd "$invoked_from/../.." && pwd -P)

workspace=${1:-}
[ -n "$workspace" ] && [ "$#" -ge 2 ] || {
    echo "usage: sh tools/probe/seal.sh <workspace> <probe-id>..." >&2
    exit 2
}
shift
here=$(cd "$workspace" 2>/dev/null && pwd -P) || {
    echo "seal: no workspace directory at $workspace" >&2
    exit 2
}
case "$here" in
    "$root"|"$root"/*)
        echo "seal: $here is inside this repository's own checkout." >&2
        echo "seal: a probe workspace is a copy outside it. Use one." >&2
        exit 6
        ;;
esac

removed=0
for shelf in docs/probes docs/probe-runs docs/probe-results; do
    if [ -e "$here/$shelf" ]; then
        rm -rf "${here:?}/$shelf"
        removed=$((removed + 1))
    fi
done
echo "seal: removed $removed of 3 probe shelves from $here"

for probe in "$@"; do
    slug=""
    shelf_file=$(grep -rlx -- "id: $probe" "$root/docs/probes" 2>/dev/null) || shelf_file=""
    case "$shelf_file" in
        *.md)
            slug=${shelf_file##*/}
            slug=${slug%.md}
            ;;
    esac
    if [ -n "$slug" ]; then
        named=$(grep -rlF -e "$probe" -e "$slug" -- "$here" 2>/dev/null) || named=""
    else
        named=$(grep -rlF -e "$probe" -- "$here" 2>/dev/null) || named=""
    fi
    count=0
    if [ -n "$named" ]; then
        # One path per line. A path holding a newline is split here and the
        # parts are not files, so `rm -f` passes over them and the guard in
        # `probe-record.sh` still finds the file.
        old_ifs=$IFS
        IFS='
'
        for file in $named; do
            rm -f -- "$file"
            count=$((count + 1))
        done
        IFS=$old_ifs
    fi
    echo "seal: removed $count files naming $probe${slug:+ or $slug}"
done
