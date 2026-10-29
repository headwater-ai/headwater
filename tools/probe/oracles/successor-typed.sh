#!/bin/sh
# Generic oracle: a document is superseded by a typed successor, and both ends
# of the link are written (#1874). Row 6 of the grid in HW-DR-0112,
# "supersede or retire a document".
#
#     sh tools/probe/oracles/successor-typed.sh [--check-clean] <old-document>
#
# <old-document> is an identifier or a path under the root. The successor is
# new, so its identifier is not known in advance, and the script finds it. It
# passes when all of these hold:
#
# 1. the front matter of <old-document> says `status: superseded`;
# 2. some document under `docs/` names <old-document> under `supersedes` in
#    its own front matter, and the engine types it;
# 3. the `relations:` block of <old-document> names that successor, which is
#    the `superseded_by` link the other end writes. A summary that says
#    "superseded by" is not a link and is not read. The successor counts only
#    as a whole name, so `HW-DR-01120` is not `HW-DR-0112` (#1874.3).
#
# With `--check-clean`, the strict errors counted are those on <old-document>
# and on the successor, because a sealed tree holds errors elsewhere.
#
# Retirement with no successor is not graded here: a probe of that kind
# declares its own oracle.

set -u
. "${0%/*}/lib.sh"
[ $# -eq 1 ] || hw_oracle_usage "successor-typed.sh [--check-clean] <old-document>"
old_path=$(hw_oracle_path_of "$1")
[ -n "$old_path" ] && [ -f "$old_path" ] || hw_oracle_fail "the engine reads no document $1"
hw_oracle_tmp
old_id=$(hw_oracle_id_of "$old_path")
[ -n "$old_id" ] || hw_oracle_fail "$old_path declares no identifier"
hw_oracle_front_matter "$old_path" | grep -E -q '^status: *"?superseded"? *$' || hw_oracle_fail "$old_path is not \`status: superseded\`"
hw_oracle_relations "$old_path" > "$hw_oracle_scratch/old"

# A successor names the old identifier inside a `supersedes:` list of its own
# front matter: the list item lines that follow the key, or a flow list.
grep -r -l -F -- "$old_id" docs > "$hw_oracle_scratch/candidates" 2> /dev/null || true
found=""
while IFS= read -r candidate; do
    [ "$candidate" = "$old_path" ] && continue
    hw_oracle_relations "$candidate" | awk -v id="$old_id" '
        # Whether the line holds id as a whole name, as hw_oracle_names_whole
        # reads one: no letter, digit, `_`, `-` or `.` on either side.
        function whole(line, rest, at, before, after) {
            rest = line
            while ((at = index(rest, id)) > 0) {
                before = at > 1 ? substr(rest, at - 1, 1) : ""
                after = substr(rest, at + length(id), 1)
                if (before !~ /[A-Za-z0-9_.-]/ && after !~ /[A-Za-z0-9_.-]/) return 1
                rest = substr(rest, at + 1)
            }
            return 0
        }
        /^[^ ]/ || /^  [a-z_]+:/ { in_list = ($0 ~ /^ *supersedes:/) }
        /^ *supersedes: *\[/ && whole($0) { hit = 1 }
        in_list && /^ *- / && whole($0) { hit = 1 }
        END { exit hit ? 0 : 1 }
    ' || continue
    found=$candidate
    break
done < "$hw_oracle_scratch/candidates"
[ -n "$found" ] || hw_oracle_fail "no document under docs/ declares that it supersedes $old_id"

hw_oracle_engine
typed=$("$hw_engine" explain --root . "$found" 2> /dev/null | sed -n 's/^  kind \([^ ]*\).*/\1/p' | sed -n '1p')
[ -n "$typed" ] || hw_oracle_fail "$found supersedes $old_id, and the engine types it as nothing"
successor_id=$(hw_oracle_id_of "$found")
[ -n "$successor_id" ] || hw_oracle_fail "$found supersedes $old_id and declares no identifier"
hw_oracle_names_whole "$successor_id" "$hw_oracle_scratch/old" || hw_oracle_fail "$found supersedes $old_id, and $old_path does not name $successor_id"
hw_oracle_subjects="$old_path $found"
hw_oracle_pass_clean "$successor_id, typed $typed, supersedes $old_id, and $old_path names it"
