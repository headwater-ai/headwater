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

# The answer keys `.headwater/probe.yml` declares for one probe, one per line
# as `<identifier> <path>`. The declaration and the key's identifier are read
# from this checkout and never from the workspace, which is the tree being
# sealed. A key whose document carries no `id:` is printed with its slug as
# the identifier, so the line removal below still has a name to match.
answer_keys() {
    declaration=${HW_PROBE_YML:-$root/.headwater/probe.yml}
    awk -v want="$1" '
        /^answer_keys:/ { on = 1; next }
        on && /^[^ #]/ { on = 0 }
        on {
            line = $0
            sub(/^[ ]+/, "", line)
            if (index(line, want ":") != 1) next
            sub(/^[^[]*\[/, "", line)
            sub(/\].*$/, "", line)
            n = split(line, paths, ",")
            for (i = 1; i <= n; i++) {
                gsub(/^[ ]+|[ ]+$/, "", paths[i])
                if (paths[i] != "") print paths[i]
            }
        }
    ' "$declaration" | while IFS= read -r path; do
        id=$(sed -n 's/^id: *//p' "$root/$path" 2>/dev/null | head -1)
        if [ -z "$id" ]; then
            id=${path##*/}
            id=${id%.md}
        fi
        printf '%s %s\n' "$id" "$path"
    done
}

# `--keys <probe>` prints the answer keys of one probe and touches nothing.
# `probe-record.sh` reads it to refuse a workspace that still names one.
if [ "${1:-}" = --keys ]; then
    [ -n "${2:-}" ] || { echo "usage: sh tools/probe/seal.sh --keys <probe-id>" >&2; exit 2; }
    answer_keys "$2"
    exit 0
fi

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

    # The answer keys of the probe (#980). The key is removed whole, with its
    # identifier claim. Every other file loses the lines that name the key and
    # keeps the rest, because the files that cite a key are shelf indexes,
    # registers and paragraphs, and each one is one line per item.
    keys=$(answer_keys "$probe")
    old_ifs=$IFS
    IFS='
'
    for pair in $keys; do
        IFS=$old_ifs
        key_id=${pair%% *}
        key_path=${pair#* }
        key_slug=${key_path##*/}
        key_slug=${key_slug%.md}
        rm -f -- "$here/$key_path"
        for claim in "$here"/.headwater/ids/*/"$key_id"; do
            if [ -e "$claim" ]; then
                rm -f -- "$claim"
            fi
        done
        lines=0
        naming=$(grep -rlIF -e "$key_id" -e "$key_slug" -- "$here" 2>/dev/null) || naming=""
        IFS='
'
        for file in $naming; do
            grep -vF -e "$key_id" -e "$key_slug" -- "$file" > "$file.seal" || true
            lines=$((lines + $(grep -cF -e "$key_id" -e "$key_slug" -- "$file")))
            cat -- "$file.seal" > "$file"
            rm -f -- "$file.seal"
        done
        IFS=$old_ifs
        echo "seal: removed the answer key $key_id of $probe, and $lines lines naming it"
        IFS='
'
    done
    IFS=$old_ifs
done
