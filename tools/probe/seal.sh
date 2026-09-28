#!/bin/sh
# Remove every answer key from a probe workspace, in every arm.
#
# A probe's own document states its expectation, a recorded run states what a
# session answered, and a record under `docs/` that names a probe states its
# expected value, a recorded answer or its target. A session that reads any of
# them reads its answer key. The tombstone session of 2026-09-17 did exactly
# that: it listed `docs/probes/` in its copy of this corpus, read its own probe
# file, and then answered (#1229).
#
#     sh tools/probe/seal.sh <workspace> <probe-id>...
#
# It deletes, in place:
#
# - the instrument that `.headwater/probe.yml` declares and `ablate.sh
#   --instrument` prints: the three probe shelves and `.headwater/export.json`,
#   which restates each probe's expectation and target;
# - every document under the workspace's `docs/` whose bytes contain a named
#   probe's identifier or its slug, the file name the probe has on the shelf.
#
# It removes nothing outside `docs/` and the instrument. A file there names a
# probe by path or title and states no answer: the derived folds
# `.headwater/nav.yml`, `.headwater/corpus.json` and
# `.headwater/capture-cost.jsonl`, the census and graph fixtures under
# `engine/`, and a comment in the hand-written `.headwater/overlay.yml`. The
# present arm is meant to test `.headwater/`, so it keeps every one of them.
#
# No probe's `examines` target names a probe, so the documents a probe tests
# survive the seal. `.headwater/probe.yml` records what the seal removes and
# why, and `probe-record.sh` refuses a workspace that still holds the
# instrument (exit 8) or a record under `docs/` that names the probe (exit 9).
#
# It refuses a workspace inside this checkout (exit 6), the guard `ablate.sh`
# applies for the same reason, and an instrument it cannot read (exit 8). It
# is idempotent: the slug is read from this checkout's shelf, so a second run
# over a sealed tree finds the same names and nothing left to delete.

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

# The instrument, read from the checkout's declaration the way the driver's
# guard reads it, so the two can never disagree about a path.
instrument=$(sh "$root/tools/probe/ablate.sh" --instrument) || {
    echo "seal: the instrument of the probe declaration could not be read." >&2
    exit 8
}
removed=0
declared=0
for path in $instrument; do
    declared=$((declared + 1))
    if [ -e "$here/$path" ]; then
        rm -rf "${here:?}/$path"
        removed=$((removed + 1))
    fi
done
echo "seal: removed $removed of $declared instrument paths from $here"

for probe in "$@"; do
    slug=""
    shelf_file=$(grep -rlx -- "id: $probe" "$root/docs/probes" 2>/dev/null) || shelf_file=""
    case "$shelf_file" in
        *.md)
            slug=${shelf_file##*/}
            slug=${slug%.md}
            ;;
    esac
    named=""
    if [ -d "$here/docs" ]; then
        if [ -n "$slug" ]; then
            named=$(grep -rlF -e "$probe" -e "$slug" -- "$here/docs" 2>/dev/null) || named=""
        else
            named=$(grep -rlF -e "$probe" -- "$here/docs" 2>/dev/null) || named=""
        fi
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
    echo "seal: removed $count documents under docs/ naming $probe${slug:+ or $slug}"
done
