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
# - every file outside `docs/` and the instrument that names a probe by its
#   identifier or slug, unless `folds:` in `.headwater/probe.yml` declares it
#   (#1384). A fold names a probe by path or title and states no answer: the
#   derived `.headwater/nav.yml`, `.headwater/corpus.json` and
#   `.headwater/capture-cost.jsonl`, the census and graph fixtures under
#   `engine/`, the hand-written overlay and probe declaration, and the route
#   fixtures of the IDE clients. The present arm is meant to test
#   `.headwater/`, so each fold stays as a file, less the lines that name a
#   deleted document. A file that is not a fold states an answer, as
#   `.claude/skills/fixtures.sh` does, and it goes whole.
#
# No probe's `examines` target names a probe, so the documents a probe tests
# survive the seal. `.headwater/probe.yml` records what the seal removes and
# why, and `probe-record.sh` refuses a workspace that still holds the
# instrument (exit 8) or a file that names the probe and is not a fold
# (exit 9). A file that states an answer and names neither the probe, nor a
# declared answer key, nor a document that names the probe, is found by no
# search: `.headwater/probe.yml` states that limit beside `folds:`.
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

# The folds `.headwater/probe.yml` declares, one path per line: the files
# outside `docs/` that name a probe and state no answer, which the seal keeps
# and the guard passes (#1384). A declaration with no `folds:` block lists
# none, so every file outside `docs/` that names a sealed probe goes. `awk`
# alone, because the guard runs it on the smallest `PATH` the fixtures give.
folds() {
    declaration=${HW_PROBE_YML:-$root/.headwater/probe.yml}
    awk '
        /^folds:/ { on = 1; next }
        on && /^[^ #]/ { on = 0 }
        on && /^  - / {
            entry = substr($0, 5)
            sub(/[ ]+#.*$/, "", entry)
            gsub(/^[ "'\'']+|[ "'\'']+$/, "", entry)
            if (entry != "") print entry
        }
    ' "$declaration"
}

# Every file of a workspace that names a probe by its identifier or its slug,
# less the declared folds, one absolute path per line (#1384). The seal
# removes each one and the guard refuses a workspace that holds one, so the
# two read one list. The slug is the file name the probe has on this
# checkout's shelf. A file under `docs/` is a record about the probe, and a
# file outside it states an answer unless it is a fold. It reads binary files
# too, so a file that `grep -I` would pass over is removed and not kept.
#
#     naming <workspace> <probe-id>
naming() {
    naming_here=$1
    naming_probe=$2
    naming_slug=""
    # `grep` exits 1 on no match and 2 when it cannot read a file. A 2 is a
    # search that did not finish, and its matches are not the whole list, so
    # it fails closed: the seal stops at 8 and the guard refuses with 9.
    naming_status=0
    naming_shelf=$(grep -rlx -- "id: $naming_probe" "$root/docs/probes" 2>/dev/null) || naming_status=$?
    [ "$naming_status" -le 1 ] || return 8
    case "$naming_shelf" in
        *.md)
            naming_slug=${naming_shelf##*/}
            naming_slug=${naming_slug%.md}
            ;;
    esac
    if [ -n "$naming_slug" ]; then
        naming_status=0
        naming_files=$(grep -rlF -e "$naming_probe" -e "$naming_slug" -- "$naming_here" 2>/dev/null) || naming_status=$?
    else
        naming_status=0
        naming_files=$(grep -rlF -e "$naming_probe" -- "$naming_here" 2>/dev/null) || naming_status=$?
    fi
    [ "$naming_status" -le 1 ] || return 8
    [ -n "$naming_files" ] || return 0
    naming_folds=$(folds) || return 8
    printf '%s\n' "$naming_files" | HW_SEAL_FOLDS="$naming_folds" HW_SEAL_HERE="$naming_here/" awk '
        BEGIN {
            n = split(ENVIRON["HW_SEAL_FOLDS"], f, "\n")
            for (i = 1; i <= n; i++) if (f[i] != "") fold[f[i]] = 1
            here = ENVIRON["HW_SEAL_HERE"]
        }
        $0 != "" {
            rel = $0
            if (index(rel, here) == 1) rel = substr(rel, length(here) + 1)
            if (rel in fold) next
            print
        }'
}

# `--folds` prints the declared folds and touches nothing.
if [ "${1:-}" = --folds ]; then
    folds
    exit $?
fi

# `--naming <workspace> <probe>` prints each file of the workspace that the
# seal would remove for naming the probe, and touches nothing.
# `probe-record.sh` reads it to refuse a workspace that was not sealed.
if [ "${1:-}" = --naming ]; then
    [ -n "${2:-}" ] && [ -n "${3:-}" ] || { echo "usage: sh tools/probe/seal.sh --naming <workspace> <probe-id>" >&2; exit 2; }
    naming_dir=$(cd "$2" 2>/dev/null && pwd -P) || { echo "seal: no workspace directory at $2" >&2; exit 2; }
    naming "$naming_dir" "$3"
    exit $?
fi

# `--keys <probe>` prints the answer keys of one probe and touches nothing.
# `probe-record.sh` reads it to refuse a workspace that still names one.
if [ "${1:-}" = --keys ]; then
    [ -n "${2:-}" ] || { echo "usage: sh tools/probe/seal.sh --keys <probe-id>" >&2; exit 2; }
    answer_keys "$2"
    exit 0
fi

# `--named <probe>` prints each name by which the seal strips a document that
# names the probe, one per line as `<name> <path>`, and touches nothing. The
# seal deletes such a document and every line that names it (#1293), so a
# workspace that still holds one of those lines was not sealed. `probe-record.sh`
# reads this list to refuse that workspace with exit 9 (#1384). Before, the
# guard checked the probe itself and the answer keys, and a register line that
# still linked a deleted document passed it.
#
# It reads this checkout's `docs/`, never the workspace, because the workspace
# is the tree the seal already cut. It passes over the instrument, which the
# seal removes whole. The name is the identifier when the document carries an
# `id:`, and `<slug>.md` when its slug is not generic. That is the two names
# `strip_document` matches. A generic slug gives no name, as in the seal: it is
# one word, or another tracked file outside the instrument has the same name.
if [ "${1:-}" = --named ]; then
    [ -n "${2:-}" ] || { echo "usage: sh tools/probe/seal.sh --named <probe-id>" >&2; exit 2; }
    named_probe=$2
    named_instrument=$(sh "$root/tools/probe/ablate.sh" --instrument) || {
        echo "seal: the instrument of the probe declaration could not be read." >&2
        exit 8
    }
    named_slug=""
    named_shelf=$(grep -rlx -- "id: $named_probe" "$root/docs/probes" 2>/dev/null) || named_shelf=""
    case "$named_shelf" in
        *.md)
            named_slug=${named_shelf##*/}
            named_slug=${named_slug%.md}
            ;;
    esac
    if [ -n "$named_slug" ]; then
        named_files=$(grep -rlF -e "$named_probe" -e "$named_slug" -- "$root/docs" 2>/dev/null) || named_files=""
    else
        named_files=$(grep -rlF -e "$named_probe" -- "$root/docs" 2>/dev/null) || named_files=""
    fi
    named_ifs=$IFS
    IFS='
'
    for named_file in $named_files; do
        IFS=$named_ifs
        named_rel=${named_file#"$root"/}
        named_skip=0
        for named_path in $named_instrument; do
            case "$named_rel" in
                "$named_path"|"$named_path"/*) named_skip=1 ;;
            esac
        done
        if [ "$named_skip" = 0 ] && [ -f "$named_file" ]; then
            named_id=$(awk '/^id: */ { sub(/^id: */, ""); print; exit }' "$named_file" 2>/dev/null) || named_id=""
            [ -z "$named_id" ] || printf '%s %s\n' "$named_id" "$named_rel"
            doc_slug=${named_rel##*/}
            doc_slug=${doc_slug%.md}
            case $doc_slug in
                *-*|*_*)
                    # Another tracked file of this name, outside the
                    # instrument, makes the slug generic in the sealed tree.
                    others=0
                    for other in $(git -C "$root" ls-files -- "$doc_slug.md" "*/$doc_slug.md" 2>/dev/null); do
                        [ "$other" != "$named_rel" ] || continue
                        for named_path in $named_instrument; do
                            case "$other" in
                                "$named_path"|"$named_path"/*) other="" ;;
                            esac
                        done
                        [ -z "$other" ] || others=$((others + 1))
                    done
                    [ "$others" -gt 0 ] || printf '%s.md %s\n' "$doc_slug" "$named_rel"
                    ;;
            esac
        fi
        IFS='
'
    done
    IFS=$named_ifs
    exit 0
fi

# `--leak <workspace> <probe>...` prints each cue of a named probe that the
# text a harness loads into every session states, and touches nothing (#1472).
#
# The seal finds an answer by a name, and the status probe's leak names none:
# the description of the authoring skill states the ruling the probe expects
# and never the probe. So each probe declares its **cues** under `cues:` in
# `.headwater/probe.yml`: the strings whose presence in always-loaded text
# gives the answer away. A cue is never an `expected:` value, because an
# expected value such as `merge` is a common word that every file holds.
#
# The always-loaded set is `CLAUDE.md`, the `description:` of each
# `.claude/skills/*/SKILL.md` and of each `.claude/agents/*.md`, and, where
# the workspace declares a project MCP server in `.mcp.json`, the description
# of each tool that `headwater mcp` lists. One line per hit:
#
#     leak <probe> <where> <cue>      a cue the declaration does not keep
#     kept <probe> <where> <cue>      a cue kept on purpose, under `cued:`
#     uncued <probe>                  a probe that declares no cue
#
# `<where>` is the path relative to the workspace, or `mcp:<tool>`. A probe
# listed under `cued:` keeps its cue on purpose: its own document says it
# measures the cue, and a campaign reports it on its own line and never in a
# rate of its category. It exits 1 when any `leak` line printed, 0 when none
# did, 2 on a usage error, and 3 when the workspace declares an MCP server and
# no engine or no `jq` can list its tools. An `uncued` probe is a probe this
# check cannot see, and it does not fail the check.
leak_cues() {
    declaration=${HW_PROBE_YML:-$root/.headwater/probe.yml}
    awk -v want="$1" '
        function trim(s) { sub(/^[ \t]+/, "", s); sub(/[ \t]+$/, "", s); return s }
        /^cues:/ { on = 1; next }
        on && /^[^ #]/ { on = 0 }
        on {
            line = $0
            sub(/^[ ]+/, "", line)
            if (index(line, want ":") != 1) next
            line = substr(line, length(want) + 2)
            sub(/^[^[]*\[/, "", line)
            sub(/\][ \t]*(#.*)?$/, "", line)
            n = split(line, parts, ",")
            for (i = 1; i <= n; i++) {
                cue = trim(parts[i])
                if (cue ~ /^".*"$/ || cue ~ /^\047.*\047$/) cue = substr(cue, 2, length(cue) - 2)
                if (cue != "") print cue
            }
        }
    ' "$declaration"
}

leak_kept() {
    declaration=${HW_PROBE_YML:-$root/.headwater/probe.yml}
    awk -v want="$1" '
        /^cued:/ { on = 1; next }
        on && /^[^ #]/ { on = 0 }
        on && /^  - / {
            entry = substr($0, 5)
            sub(/[ ]+#.*$/, "", entry)
            gsub(/^[ "\047]+|[ "\047]+$/, "", entry)
            if (entry == want) found = 1
        }
        END { exit found ? 0 : 1 }
    ' "$declaration"
}

# The always-loaded text of a workspace, one record per line as
# `<where><TAB><text>`, with every line of `CLAUDE.md` its own record.
leak_loaded() {
    leak_here=$1
    if [ -f "$leak_here/CLAUDE.md" ]; then
        awk '{ printf "CLAUDE.md\t%s\n", $0 }' "$leak_here/CLAUDE.md"
    fi
    for leak_file in "$leak_here"/.claude/skills/*/SKILL.md "$leak_here"/.claude/agents/*.md; do
        [ -f "$leak_file" ] || continue
        # The first `description:` key of the front matter, which is the one a
        # harness loads, with a folded or literal block read to its end.
        awk -v where="${leak_file#"$leak_here"/}" '
            NR == 1 && $0 == "---" { on = 1; next }
            on && $0 == "---" { exit }
            on && folded && /^[ \t]/ { sub(/^[ \t]+/, ""); text = text " " $0; next }
            on && folded { exit }
            on && /^description:/ {
                sub(/^description:[ \t]*/, "")
                if ($0 ~ /^[>|][-+]?$/) { folded = 1; text = ""; next }
                text = $0
                exit
            }
            END { if (text != "") printf "%s\t%s\n", where, text }
        ' "$leak_file"
    done
    if [ -f "$leak_here/.mcp.json" ]; then
        leak_engine=$root/engine/target/dev-release/headwater
        [ -x "$leak_engine" ] || leak_engine=$root/engine/target/release/headwater
        [ -x "$leak_engine" ] || return 3
        command -v jq >/dev/null 2>&1 || return 3
        leak_listed=$(printf '%s\n' \
            '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05"}}' \
            '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
            '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' \
            | "$leak_engine" mcp --root "$leak_here" 2>/dev/null \
            | jq -r 'select(.id == 2) | .result.tools[] | "mcp:" + .name + "\t" + (.description | gsub("[\n\t]"; " "))' 2>/dev/null) || return 3
        [ -n "$leak_listed" ] || return 3
        printf '%s\n' "$leak_listed"
    fi
    return 0
}

if [ "${1:-}" = --leak ]; then
    [ -n "${2:-}" ] && [ -n "${3:-}" ] || { echo "usage: sh tools/probe/seal.sh --leak <workspace> <probe-id>..." >&2; exit 2; }
    leak_dir=$(cd "$2" 2>/dev/null && pwd -P) || { echo "seal: no workspace directory at $2" >&2; exit 2; }
    shift 2
    leak_text=$(leak_loaded "$leak_dir") || {
        echo "seal: $leak_dir declares an MCP server, and no engine or no \`jq\` listed its tools, so the always-loaded text is not whole." >&2
        exit 3
    }
    leak_found=0
    for leak_probe in "$@"; do
        leak_list=$(leak_cues "$leak_probe")
        if [ -z "$leak_list" ]; then
            printf 'uncued %s\n' "$leak_probe"
            continue
        fi
        leak_verdict=leak
        leak_kept "$leak_probe" && leak_verdict=kept
        leak_hits=$(printf '%s\n' "$leak_list" | HW_LEAK_TEXT="$leak_text" awk -v probe="$leak_probe" -v verdict="$leak_verdict" '
            BEGIN {
                n = split(ENVIRON["HW_LEAK_TEXT"], rows, "\n")
                for (i = 1; i <= n; i++) {
                    tab = index(rows[i], "\t")
                    if (tab == 0) continue
                    where[i] = substr(rows[i], 1, tab - 1)
                    text[i] = substr(rows[i], tab + 1)
                }
            }
            $0 != "" {
                for (i = 1; i <= n; i++) {
                    if (!(i in where)) continue
                    key = where[i] SUBSEP $0
                    if (index(text[i], $0) > 0 && !(key in seen)) {
                        seen[key] = 1
                        printf "%s %s %s %s\n", verdict, probe, where[i], $0
                    }
                }
            }')
        if [ -n "$leak_hits" ]; then
            printf '%s\n' "$leak_hits"
            [ "$leak_verdict" = kept ] || leak_found=1
        fi
    done
    exit "$leak_found"
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
    # A path is one name: `set -f` keeps a `*`, `?` or `[` in it from
    # expanding to other files. The body and the code after the loop turn
    # globbing back on, because the claim store is read through a glob.
    set -f
    for strip_file in $strip_naming; do
        set +f
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
    set +f
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
    # One search over the workspace, less the folds, split by where each file
    # is: a record under `docs/`, or a file outside it that states an answer.
    naming_all=$(naming "$here" "$probe") || {
        echo "seal: the files that name $probe could not be read: a file of the workspace is unreadable, or the folds of the probe declaration are." >&2
        exit 8
    }
    named=$(printf '%s\n' "$naming_all" | awk -v docs="$here/docs/" 'index($0, docs) == 1')
    outside=$(printf '%s\n' "$naming_all" | awk -v docs="$here/docs/" '$0 != "" && index($0, docs) != 1')
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
        # A path is one name: `set -f` keeps a `*`, `?` or `[` in it from
        # expanding to other files. The body and the code after the loop turn
        # globbing back on, because the claim store is read through a glob.
        set -f
        for file in $named; do
            set +f
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
        set +f
    fi
    echo "seal: removed $count documents under docs/ naming $probe${slug:+ or $slug}"

    # The files outside `docs/` that name the probe and are not folds
    # (#1384). Each states an answer, so each goes whole. It is not a record,
    # so no line elsewhere is stripped for it. The list was read before the
    # line removal above, so a file that the removal cleared of the probe
    # still goes.
    count=0
    old_ifs=$IFS
    IFS='
'
    # A path is one name: `set -f` keeps a `*`, `?` or `[` in it from
    # expanding to other files. The body and the code after the loop turn
    # globbing back on, because the claim store is read through a glob.
    set -f
    for file in $outside; do
        set +f
        IFS=$old_ifs
        if [ -f "$file" ]; then
            rm -f -- "$file"
            count=$((count + 1))
            echo "seal: removed ${file#"$here"/}, which names $probe outside docs/ and is not a declared fold"
        fi
        IFS='
'
    done
    IFS=$old_ifs
    set +f
    echo "seal: removed $count files outside docs/ naming $probe${slug:+ or $slug}"

    # The answer keys of the probe (#980). The key is removed whole, and
    # `strip_document` removes its claim and the lines that name it.
    keys=$(answer_keys "$probe")
    old_ifs=$IFS
    IFS='
'
    # A path is one name: `set -f` keeps a `*`, `?` or `[` in it from
    # expanding to other files. The body and the code after the loop turn
    # globbing back on, because the claim store is read through a glob.
    set -f
    for pair in $keys; do
        set +f
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
    set +f
done
