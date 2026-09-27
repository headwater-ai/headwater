#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
#
# Collect what the engine can say about one change without a model, and hand
# it to `upkeep-report.py`, which writes the four-part report. The action runs
# this script, and so does `fixtures/governed-change.sh`, so the fixture holds
# the same code path a consumer's workflow runs.
#
# Usage: HEADWATER_BIN=<engine> sh upkeep.sh <root> <base-rev> <work-dir> <report-path>
#
# <root> is the corpus, inside a git checkout. <base-rev> is the revision the
# change is measured from: the merge base of a pull request, per spec 12.
# <work-dir> must be outside the checkout. This script writes only there and
# at <report-path>, and it runs every engine verb with `--no-cache`, because
# `headwater check` otherwise writes `.headwater/cache/` inside the checkout.
# The report proposes. Nothing here commits, and nothing writes `accepted_by`.
#
# The engine verbs, one run each except `route`:
#   headwater change <base> <work>/change      the manifest of the change
#   headwater check --change <manifest> --format json
#   headwater check --format json over the base tree, unpacked under <work>,
#     so the report can tell a finding the change introduced from one that
#     stood before it
#   headwater route <path> <ancestors...> --json, once per path the change
#     leaves on the tree, and once over the base tree per path it deleted or
#     renamed away, because the edge that named such a path is on the base
# `route` is given each ancestor directory of the path as a further word, so
# a pointer that anchors on an ancestor and not on the path itself is a
# literal directory edge, which the engine matches by equality and which
# therefore never reaches the file under it. The report names that case
# rather than counting the file as governed or as ungoverned.
set -eu

root=${1:?usage: upkeep.sh <root> <base-rev> <work-dir> <report-path>}
base=${2:?usage: upkeep.sh <root> <base-rev> <work-dir> <report-path>}
work=${3:?usage: upkeep.sh <root> <base-rev> <work-dir> <report-path>}
report=${4:?usage: upkeep.sh <root> <base-rev> <work-dir> <report-path>}
bin=${HEADWATER_BIN:-headwater}
here=$(cd "$(dirname "$0")" && pwd)

rm -rf "$work"
mkdir -p "$work/change" "$work/route" "$work/route-base" "$work/base"

# Paths relative to <root>, which is how every engine verb below names them.
# `--name-status -M` names a rename as one line with both paths, so the old
# path is kept: the edge that named it is the one the rename breaks.
#   changed.txt  every path the change leaves on the tree
#   gone.tsv     D<tab>path for a deletion, R<tab>old<tab>new for a rename
( cd "$root" && git diff --name-status -M --relative "$base" ) >"$work/name-status.txt"
: >"$work/changed.txt"
: >"$work/gone.tsv"
while IFS="$(printf '\t')" read -r status first second; do
    case "$status" in
        D) printf 'D\t%s\n' "$first" >>"$work/gone.tsv" ;;
        R*) printf 'R\t%s\t%s\n' "$first" "$second" >>"$work/gone.tsv"
            printf '%s\n' "$second" >>"$work/changed.txt" ;;
        *) printf '%s\n' "$first" >>"$work/changed.txt" ;;
    esac
done <"$work/name-status.txt"
( cd "$root" && git ls-files --others --exclude-standard ) >>"$work/changed.txt"

# The base tree, unpacked outside the checkout, so a finding can be told new
# from old and a path the change removed can still be routed.
prefix=$(cd "$root" && git rev-parse --show-prefix)
( cd "$root" && git archive --format=tar "$base:$prefix" ) | tar -x -C "$work/base"

"$bin" change "$base" "$work/change" --root "$root" >"$work/change.out"
manifest=$(head -n 1 "$work/change.out")

# `check` exits non-zero on an error finding. That is a result to report, not
# a reason to stop, so its status is kept and the run goes on.
set +e
"$bin" check --root "$root" --no-cache --change "$manifest" --format json \
    >"$work/check.json" 2>"$work/check.err"
code=$?
"$bin" check --root "$work/base" --no-cache --format json \
    >"$work/base-check.json" 2>"$work/base-check.err"
set -e
echo "$code" >"$work/check.exit"

# route_all <list> <tree> <out-dir> <index>: route each path of <list>, one
# per line, against <tree>, with each ancestor directory as a further word.
# The words go to `route` unquoted, so no pattern in a path may expand.
set -f
route_all() {
    n=0
    : >"$4"
    while IFS= read -r path; do
        [ -n "$path" ] || continue
        n=$((n + 1))
        words=$path
        dir=$path
        while :; do
            case "$dir" in
                */*) dir=${dir%/*}; words="$words $dir" ;;
                *) break ;;
            esac
        done
        # shellcheck disable=SC2086 # each ancestor is its own word on purpose
        "$bin" route $words --json --root "$2" >"$3/$n.json"
        printf '%s\t%s\n' "$n" "$path" >>"$4"
    done <"$1"
}
route_all "$work/changed.txt" "$root" "$work/route" "$work/routes.tsv"
cut -f2 "$work/gone.tsv" >"$work/gone.txt"
route_all "$work/gone.txt" "$work/base" "$work/route-base" "$work/routes-base.tsv"

python3 "$here/upkeep-report.py" "$work" "$report"
