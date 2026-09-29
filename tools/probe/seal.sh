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
#   probe's identifier or its slug, the file name the probe has on the shelf;
# - every answer key `.headwater/probe.yml` declares for a named probe;
# - for each document it deletes, named or key, the identifier claim under
#   `.headwater/ids/` and every line anywhere in the workspace that names the
#   document's identifier or slug (#1293). A shelf index, a register, a fold
#   or a paragraph that cites the document loses that line and keeps the rest.
#   A document with no `id:` is matched by its slug. A generic slug such as
#   `README` removes no line, and the seal prints that it kept them.
#
# It deletes no other file outside `docs/` and the instrument. A file there
# names a probe by path or title and states no answer: the derived folds
# `.headwater/nav.yml`, `.headwater/corpus.json` and
# `.headwater/capture-cost.jsonl`, the census and graph fixtures under
# `engine/`, and a comment in the hand-written `.headwater/overlay.yml`. The
# present arm is meant to test `.headwater/`, so each one stays as a file,
# less the lines that name a deleted document.
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

# Remove a deleted document's identifier claim, and every line in the
# workspace that names it, from the files that hold such a line. The files
# that cite a record are shelf indexes, registers, folds and paragraphs, each
# one line per item, so each file keeps every other line and stays a file.
#
#     strip_document <identifier> <slug>
#
# An empty slug matches by the identifier alone. It sets `stripped` to the
# number of lines removed.
strip_document() {
    strip_id=$1
    strip_slug=$2
    for strip_claim in "$here"/.headwater/ids/*/"$strip_id"; do
        if [ -e "$strip_claim" ]; then
            rm -f -- "$strip_claim"
        fi
    done
    set -- -e "$strip_id"
    if [ -n "$strip_slug" ]; then
        set -- "$@" -e "$strip_slug"
    fi
    stripped=0
    strip_naming=$(grep -rlIF "$@" -- "$here" 2>/dev/null) || strip_naming=""
    strip_ifs=$IFS
    IFS='
'
    for strip_file in $strip_naming; do
        grep -vF "$@" -- "$strip_file" > "$strip_file.seal" || true
        stripped=$((stripped + $(grep -cF "$@" -- "$strip_file")))
        cat -- "$strip_file.seal" > "$strip_file"
        rm -f -- "$strip_file.seal"
    done
    IFS=$strip_ifs
}

# A slug that names a role rather than a record. Every shelf has a README, so
# a line that holds the word names no answer, and the slug of such a file
# never drives line removal.
generic_slug() {
    case $1 in
        README|readme|Readme|index|INDEX|_index) return 0 ;;
    esac
    return 1
}

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
        # parts are not files, so they are passed over and the guard in
        # `probe-record.sh` still finds the file.
        #
        # Each document is sealed as an answer key is (#1293): its identifier
        # is read from the workspace copy before it goes, then its claim and
        # every line that names it go too. A document with no `id:` is matched
        # by its slug, and a generic slug matches nothing.
        old_ifs=$IFS
        IFS='
'
        for file in $named; do
            IFS=$old_ifs
            if [ -f "$file" ]; then
                doc_slug=${file##*/}
                doc_slug=${doc_slug%.md}
                doc_id=$(sed -n 's/^id: *//p' "$file" 2>/dev/null | head -1)
                rm -f -- "$file"
                count=$((count + 1))
                if generic_slug "$doc_slug"; then
                    if [ -n "$doc_id" ]; then
                        strip_document "$doc_id" ""
                        echo "seal: removed the named document $doc_id of $probe, and $stripped lines naming it"
                    fi
                    echo "seal: kept every line naming $doc_slug, a name every shelf uses, after removing ${file#"$here"/}"
                else
                    [ -n "$doc_id" ] || doc_id=$doc_slug
                    strip_document "$doc_id" "$doc_slug"
                    echo "seal: removed the named document $doc_id of $probe, and $stripped lines naming it"
                fi
            fi
            IFS='
'
        done
        IFS=$old_ifs
    fi
    echo "seal: removed $count documents under docs/ naming $probe${slug:+ or $slug}"

    # The answer keys of the probe (#980). The key is removed whole, and
    # `strip_document` removes its claim and the lines that name it.
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
        strip_document "$key_id" "$key_slug"
        echo "seal: removed the answer key $key_id of $probe, and $stripped lines naming it"
        IFS='
'
    done
    IFS=$old_ifs
done
