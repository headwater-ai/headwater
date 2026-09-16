#!/bin/sh
# ack-site-prose-reviewed.sh — clear the pending-review marker a `site/*`
# merge conflict left behind, after you have actually carried the losing
# side's prose forward by hand.
#
# `.githooks/merge-regenerate` refuses every conflicting merge on a `site/*`
# page (they hold hand-written prose beside measured `data-figure` spans, and
# a driver cannot tell which side's prose was meant to survive). When the two
# sides also differ outside their figure spans, it drops a marker under
# `<git-common-dir>/headwater-pending-site-review/<path, with / as _>` holding
# that diff, because the warning it also prints is easy to miss in a rebase
# running unattended and nothing else records that the conflict ever needed a
# human decision. `.githooks/post-rewrite` reports a marker the moment a
# rebase finishes, and `.githooks/pre-push` refuses to push while one exists.
#
# This is the one command that clears it. It does not run automatically and
# it does not try to judge whether the prose was carried forward correctly —
# that is a reading a script cannot do, and this file's whole reason to exist
# is that the driver already declined to fake that judgment once. Run it only
# after you have compared the diff below against the page and are satisfied
# every hand-written change either side made is still on it.
#
# With no path, it lists every pending marker and prints nothing else, so
# `.githooks/post-rewrite` and `.githooks/pre-push` can both call it that way
# to report what is outstanding.

set -eu

common_dir=$(git rev-parse --git-common-dir 2>/dev/null) || {
    echo "ack-site-prose-reviewed.sh: not inside a git repository" >&2
    exit 1
}
marker_dir="$common_dir/headwater-pending-site-review"

if [ $# -eq 0 ]; then
    found=
    if [ -d "$marker_dir" ]; then
        for marker in "$marker_dir"/*; do
            [ -e "$marker" ] || continue
            if [ -z "$found" ]; then
                echo "Pending site/* prose review:"
                found=1
            fi
            echo "  $(basename "$marker" | tr '_' '/')"
        done
    fi
    exit 0
fi

path=$1
marker_name=$(printf '%s' "$path" | tr '/' '_')
marker="$marker_dir/$marker_name"

if [ ! -f "$marker" ]; then
    echo "ack-site-prose-reviewed.sh: no pending review for $path" >&2
    echo "  (nothing under $marker_dir matches it)" >&2
    exit 1
fi

echo "Non-figure diff that was pending review for $path:"
echo
cat "$marker"
echo
rm -f "$marker"
echo "Cleared. $path can be pushed once its figures are current:"
echo "    sh tools/site/refresh-figures.sh"
