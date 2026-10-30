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
# 1. the front matter of <old-document> says `status: superseded`, as a
#    top-level key, so a comment or the same words under another key is no
#    status;
# 2. a document that the engine types declares a succession edge to
#    <old-document>, and its own status is not `superseded`;
# 3. <old-document> declares a succession edge to that successor, which is
#    the `superseded_by` link the other end writes.
#
# Both edges are read through `headwater explain --json`, as
# `hw_oracle_explain` and `hw_oracle_edge` in `lib.sh` read them, and never
# as lines of the front matter (round 4 of #1874.3). So an edge that only a
# comment holds, or that stands under another relation such as `traces_to`,
# is no edge. The engine names each edge as the document explained stands to
# the far end, whichever end wrote it (#1901): the successor reads
# `supersedes` and the old record reads `superseded_by`, outbound for the key
# that document wrote and inbound for the key the other end wrote. So the
# relation says which end is old. Status still guards one case the relation
# does not: a successor that was superseded in turn is not the successor
# this grade asks for. With `--check-clean`, the strict check reads the keys
# themselves, and a key written on the wrong end fails its reciprocity rule.
#
# With `--check-clean`, the strict errors counted are those on <old-document>
# and on the successor, because a sealed tree holds errors elsewhere.
#
# Retirement with no successor is not graded here: a probe of that kind
# declares its own oracle.

set -u
. "${0%/*}/lib.sh"
[ $# -eq 1 ] || hw_oracle_usage "successor-typed.sh [--check-clean] <old-document>"
hw_oracle_tmp
hw_oracle_explain "$1" "$hw_oracle_scratch/old.json" || hw_oracle_fail "the engine reads no document $1"
old_path=$(jq -r '.path // ""' "$hw_oracle_scratch/old.json")
old_id=$(jq -r '.id // ""' "$hw_oracle_scratch/old.json")
[ -n "$old_path" ] && [ -f "$old_path" ] || hw_oracle_fail "the engine reads no document $1"
[ -n "$old_id" ] || hw_oracle_fail "$old_path declares no identifier"

# The status of a file, as a top-level key of its front matter.
superseded() {
    hw_oracle_front_matter "$1" | grep -E -q '^status: *"?superseded"? *$'
}
superseded "$old_path" || hw_oracle_fail "$old_path is not \`status: superseded\`"

# Each document that declares `supersedes` to the old one is printed on it as
# `superseded_by`, inbound, with that document's path. A document that wrote
# `superseded_by` to it is a record the old one superseded, and is printed as
# `supersedes`, inbound, so it is no candidate.
jq -r '[.related[]? | select(.inbound == true and .relation == "superseded_by") | .targets[]?] | unique | .[]' "$hw_oracle_scratch/old.json" > "$hw_oracle_scratch/candidates"
found="" found_id="" typed="" refused=""
while IFS= read -r candidate; do
    [ "$candidate" = "$old_path" ] && continue
    hw_oracle_explain "$candidate" "$hw_oracle_scratch/candidate.json" || continue
    candidate_id=$(jq -r '.id // ""' "$hw_oracle_scratch/candidate.json")
    hw_oracle_edge "$hw_oracle_scratch/candidate.json" supersedes "$old_id" "$old_path" || continue
    if superseded "$candidate"; then
        refused="$candidate"
        continue
    fi
    found=$candidate found_id=$candidate_id
    typed=$(jq -r '.kind // ""' "$hw_oracle_scratch/candidate.json")
    break
done < "$hw_oracle_scratch/candidates"
if [ -z "$found" ]; then
    [ -n "$refused" ] && hw_oracle_fail "the only document that supersedes $old_id is $refused, which is superseded itself, so it is not the successor that stands"
    hw_oracle_fail "no document declares that it supersedes $old_id, as the engine parses the front matter"
fi
[ -n "$typed" ] && [ "$typed" != null ] || hw_oracle_fail "$found supersedes $old_id, and the engine types it as nothing"
[ -n "$found_id" ] || hw_oracle_fail "$found supersedes $old_id and declares no identifier"
hw_oracle_edge "$hw_oracle_scratch/old.json" superseded_by "$found_id" "$found" || hw_oracle_fail "$found supersedes $old_id, and $old_path declares no superseded_by edge to $found_id, as the engine parses the front matter"
hw_oracle_subjects="$old_path $found"
hw_oracle_pass_clean "$found_id, typed $typed, supersedes $old_id, and $old_path names it"
