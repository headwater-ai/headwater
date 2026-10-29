#!/bin/sh
# Generic oracle: two documents link each other (#1874). Row 5 of the grid in
# HW-DR-0112, "edit a document others depend on".
#
#     sh tools/probe/oracles/reciprocal-link.sh [--check-clean] <document-a> <document-b>
#
# Each document is an identifier or a path under the root. It passes when the
# `relations:` block of <document-a> names <document-b> and the `relations:`
# block of <document-b> names <document-a>, each by its identifier, its path
# or its file name, each as a whole name, so that `draft-<name>.md` does not
# stand for `<name>.md`. The engine resolves each argument to its path, so a
# document that the engine does not read fails.
#
# It reads the front matter of each file, not the edges `headwater explain`
# prints, because that report also prints an edge the other end declares, and
# then a link written at one end alone reads as reciprocal. It reads the
# `relations:` block alone, because a summary that mentions the other
# document is not a link.

set -u
. "${0%/*}/lib.sh"
[ $# -eq 2 ] || hw_oracle_usage "reciprocal-link.sh [--check-clean] <document-a> <document-b>"
path_a=$(hw_oracle_path_of "$1")
path_b=$(hw_oracle_path_of "$2")
[ -n "$path_a" ] && [ -f "$path_a" ] || hw_oracle_fail "the engine reads no document $1"
[ -n "$path_b" ] && [ -f "$path_b" ] || hw_oracle_fail "the engine reads no document $2"

# Whether the relations of $1 name the document at path $2 by identifier, by
# path from the root, or by its file name.
names() {
    named_id=$(hw_oracle_id_of "$2")
    hw_oracle_tmp
    hw_oracle_relations "$1" > "$hw_oracle_scratch/front"
    if [ -n "$named_id" ] && grep -E -q "(^|[^A-Za-z0-9_-])$named_id([^A-Za-z0-9_-]|\$)" "$hw_oracle_scratch/front"; then
        return 0
    fi
    hw_oracle_names_whole "$2" "$hw_oracle_scratch/front" && return 0
    hw_oracle_names_whole "${2##*/}" "$hw_oracle_scratch/front"
}

names "$path_a" "$path_b" || hw_oracle_fail "the relations of $path_a do not name $path_b"
names "$path_b" "$path_a" || hw_oracle_fail "the relations of $path_b do not name $path_a"
hw_oracle_pass_clean "$path_a and $path_b name each other in their relations"
