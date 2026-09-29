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
#   `.headwater/ids/`, and every line anywhere in the workspace that holds
#   the document's identifier as a whole name or its file name `<slug>.md`
#   (#1293). A shelf index, a register or a paragraph that cites the document
#   loses that line and keeps the rest. A JSON fold loses the array element
#   that names it, through `jq`, and still parses. A generic slug, one word
#   or a file name that another file shares such as `README`, removes no
#   line, and the seal prints that it kept them.
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
# stops at exit 3 when a JSON file names a deleted document and no `jq` is on
# the path, or when `jq` cannot read that file. It is idempotent: the slug is read from this checkout's shelf, so a second run
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
# workspace that names it, from the files that hold such a line. Most files
# that cite a record are shelf indexes, registers and paragraphs, each one
# line per item, so each file keeps every other line and stays a file. The
# removal also reaches scripts and fixtures, such as the census fixture and
# `probe-record-fixtures.sh`, so the workspace copy of such a file can no
# longer run. A session answers from the corpus and does not run them.
#
#     strip_document <identifier> <slug>
#
# A line names the document when it holds the identifier as a whole name, or
# the file name `<slug>.md` after a separator. A name that another name
# contains, such as `a-b` inside `a-b-c` or `x-a-b`, does not match the longer
# one. Either argument may be empty. A JSON file is not cut by lines, because
# a record in it spans several: it loses each array element whose own values
# name the document, through `jq`, and still parses (verify round 1 of #1293).
# It sets `stripped` to the number of lines or elements removed.
strip_document() {
    strip_id=$1
    strip_slug=$2
    strip_edge='[^A-Za-z0-9_-]'
    strip_re=""
    if [ -n "$strip_id" ]; then
        for strip_claim in "$here"/.headwater/ids/*/"$strip_id"; do
            if [ -e "$strip_claim" ]; then
                rm -f -- "$strip_claim"
            fi
        done
        strip_re="(^|$strip_edge)$(printf '%s' "$strip_id" | sed 's/[].[\\*^$()+?{}|]/\\&/g')($strip_edge|\$)"
    fi
    if [ -n "$strip_slug" ]; then
        strip_re="${strip_re:+$strip_re|}(^|$strip_edge)$(printf '%s' "$strip_slug" | sed 's/[].[\\*^$()+?{}|]/\\&/g')\\.md"
    fi
    stripped=0
    [ -n "$strip_re" ] || return 0
    strip_naming=$(grep -rlIE -e "$strip_re" -- "$here" 2>/dev/null) || strip_naming=""
    strip_ifs=$IFS
    IFS='
'
    for strip_file in $strip_naming; do
        case $strip_file in
            *.json)
                command -v jq >/dev/null 2>&1 || {
                    echo "seal: $strip_file names a deleted document, and no \`jq\` is on the path to remove it" >&2
                    exit 3
                }
                strip_jq='
                    def names: (type == "string" and test($re)) or ((type == "number" or type == "boolean") and (tostring | test($re)));
                    def own: if type == "object" then any(.[]; names) elif type == "array" then false else names end;
                    def strip: if type == "array" then map(select(own | not) | strip)
                        elif type == "object" then map_values(strip) else . end;'
                jq --arg re "$strip_re" "$strip_jq strip" "$strip_file" > "$strip_file.seal" || exit 3
                stripped=$((stripped + $(jq --arg re "$strip_re" "$strip_jq"' [.. | arrays | .[] | select(own)] | length' "$strip_file")))
                ;;
            *)
                grep -vE -e "$strip_re" -- "$strip_file" > "$strip_file.seal" || true
                stripped=$((stripped + $(grep -cE -e "$strip_re" -- "$strip_file")))
                ;;
        esac
        cat -- "$strip_file.seal" > "$strip_file"
        rm -f -- "$strip_file.seal"
    done
    IFS=$strip_ifs
}

# A slug that names a role or a topic rather than one record, so it never
# drives line removal. It is generic when another file in the workspace has
# the same name, as every shelf's README.md and every skill's SKILL.md do,
# because a line that links that name may mean the other file. It is generic
# when it is one word with no `-` or `_`, such as `glossary`, because many
# lines hold such a word for another reason. Call it after the document is
# deleted, so that the count sees only the other files.
generic_slug() {
    case $1 in
        *-*|*_*) ;;
        *) return 0 ;;
    esac
    [ -n "$(find "$here" -name "$1.md" -print 2>/dev/null | head -1)" ]
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
        # by its file name alone, and a generic slug matches nothing.
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
                    echo "seal: kept every line naming $doc_slug, a generic slug, after removing ${file#"$here"/}"
                else
                    strip_document "$doc_id" "$doc_slug"
                    echo "seal: removed the named document ${doc_id:-$doc_slug} of $probe, and $stripped lines naming it"
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
