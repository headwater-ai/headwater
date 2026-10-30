#!/bin/sh
# Generic oracle: two documents link each other (#1874). Row 5 of the grid in
# HW-DR-0112, "edit a document others depend on".
#
#     sh tools/probe/oracles/reciprocal-link.sh [--check-clean] <document-a> <document-b>
#
# Each document is an identifier or a path under the root. It passes when
# <document-a> declares an edge, of any relation, to <document-b>, and
# <document-b> declares one to <document-a>. Each edge names the other end
# by its identifier or by its path from the root, exactly, so that
# `draft-<name>.md` or a bare file name does not stand for the document. A
# document that the engine does not read fails.
#
# It reads each edge from `headwater explain --json`, through
# `hw_oracle_cites` in `lib.sh`, and never from lines of the file, so a
# commented relation line, a summary or a body sentence that names the
# other document is not a link (#1874.3b). That report also lists an edge
# that the other end declares, with `inbound` true, and the script reads
# only the entries with `inbound` false, so a link written at one end alone
# does not read as reciprocal. `targets` holds an edge as it is written, and
# the engine resolves no path written under some relations, such as
# `superseded_by`: then the edge names the document here and is a strict
# error, which `--check-clean` fails. `tools/probe/oracle-fixtures.sh` holds
# both.

set -u
. "${0%/*}/lib.sh"
[ $# -eq 2 ] || hw_oracle_usage "reciprocal-link.sh [--check-clean] <document-a> <document-b>"
hw_oracle_tmp
hw_oracle_explain "$1" "$hw_oracle_scratch/a.json" || hw_oracle_fail "the engine reads no document $1"
hw_oracle_explain "$2" "$hw_oracle_scratch/b.json" || hw_oracle_fail "the engine reads no document $2"
path_a=$(jq -r '.path // ""' "$hw_oracle_scratch/a.json")
path_b=$(jq -r '.path // ""' "$hw_oracle_scratch/b.json")
id_a=$(jq -r '.id // ""' "$hw_oracle_scratch/a.json")
id_b=$(jq -r '.id // ""' "$hw_oracle_scratch/b.json")
[ -n "$path_a" ] && [ -f "$path_a" ] || hw_oracle_fail "the engine reads no document $1"
[ -n "$path_b" ] && [ -f "$path_b" ] || hw_oracle_fail "the engine reads no document $2"

# Whether the explained document <json> declares an edge to the document of
# identifier <id> (which may be empty) at <path>.
names() {
    if [ -n "$2" ]; then
        hw_oracle_cites "$1" "$2" "$3"
    else
        hw_oracle_cites "$1" "$3"
    fi
}

names "$hw_oracle_scratch/a.json" "$id_b" "$path_b" || hw_oracle_fail "$path_a declares no edge to $path_b as the engine parses it"
names "$hw_oracle_scratch/b.json" "$id_a" "$path_a" || hw_oracle_fail "$path_b declares no edge to $path_a as the engine parses it"
hw_oracle_subjects="$path_a $path_b"
hw_oracle_pass_clean "$path_a and $path_b each declare an edge to the other"
