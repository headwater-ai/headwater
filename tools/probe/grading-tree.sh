#!/bin/sh
# The grading tree of a campaign (#1834): the sealed oracle tree, plus each
# file a command oracle `holds`.
#
#     sh tools/probe/grading-tree.sh <sealed-tree> <source> <grading-tree>
#     sh tools/probe/grading-tree.sh --holds <sealed-tree>
#
# A `cargo-test` oracle can run a hidden test. The test's header names its
# probe, so `tools/probe/seal.sh` removes it from the sealed tree, and so from
# every arm tree, which `tools/probe/campaign.sh` copies from the sealed tree.
# A session never sees it. But the recorder grades in a copy of the tree that
# `--oracle-tree` names, and a test removed from that tree cannot run: every
# session of the probe would read as a failed build or a case that never ran.
#
# So this copies <sealed-tree> to <grading-tree> and restores each `holds`
# path of each declared oracle from <source>, a tree of the same commit that
# no seal ran over. Nothing else is restored. Unsealing the whole tree would
# change what `headwater check` reports over a document artifact, and so
# change what a document probe measures. The grading tree sits beside the
# sealed tree under `<out>/trees/`, which `ci-confine.sh` keeps from a session.
#
# `--holds` prints each `holds` path, one per line, and touches nothing.
# `campaign.sh` reads it to know which paths to take out of `git archive`.
#
# The holds are read from <sealed-tree>'s own `.headwater/probe.yml`, which
# the seal keeps as a fold, through `headwater probe plan --oracles`. It is
# the declaration the transform reads from the same tree, so the files this
# restores are the files the transform's commands expect. This script parses
# no YAML.
#
# Exit 2 on a usage error, 3 when a `holds` path is absent from <source> or a
# copy fails, and 8 when the engine is not built or refuses the declaration.
# A failure leaves no <grading-tree>, because the tree is built under another
# name and moved.

set -u

case $0 in
    */*) invoked_from=${0%/*} ;;
    *) invoked_from=. ;;
esac
checkout=$(cd "$invoked_from/../.." && pwd -P)

holds() {
    engine=$checkout/engine/target/dev-release/headwater
    [ -x "$engine" ] || engine=$checkout/engine/target/release/headwater
    [ -x "$engine" ] || {
        echo "grading-tree: no engine under $checkout/engine/target to read the probe declaration. Build it first: cargo build --profile dev-release -p headwater-cli --manifest-path engine/Cargo.toml --locked" >&2
        return 8
    }
    [ -f "$1/.headwater/probe.yml" ] || return 0
    printed=$("$engine" probe plan --root "$1" --oracles) || {
        echo "grading-tree: the engine refused the probe declaration of $1." >&2
        return 8
    }
    printf '%s\n' "$printed" | awk -F '\t' '$2 == "hold" && !seen[$3]++ { print $3 }'
}

if [ "${1:-}" = --holds ]; then
    [ $# -eq 2 ] || { echo "usage: grading-tree.sh --holds <sealed-tree>" >&2; exit 2; }
    holds "$2"
    exit $?
fi

[ $# -eq 3 ] || {
    echo "usage: grading-tree.sh <sealed-tree> <source> <grading-tree> | --holds <sealed-tree>" >&2
    exit 2
}
sealed=$1 source=$2 grading=$3
[ -d "$sealed" ] && [ -d "$source" ] || {
    echo "grading-tree: $sealed and $source must both be directories." >&2
    exit 2
}

held=$(holds "$sealed") || exit $?
rm -rf "$grading.building"
cp -a "$sealed" "$grading.building" || exit 3
if [ -n "$held" ]; then
    # One path per line. `set -f` keeps a `*` in a path from expanding.
    old_ifs=$IFS
    IFS='
'
    set -f
    for path in $held; do
        if [ ! -f "$source/$path" ]; then
            echo "grading-tree: $path is declared under \`holds\` and $source does not hold it." >&2
            rm -rf "$grading.building"
            exit 3
        fi
        mkdir -p "$grading.building/$(dirname "$path")" && cp "$source/$path" "$grading.building/$path" || {
            rm -rf "$grading.building"
            exit 3
        }
        echo "grading-tree: restored $path"
    done
    set +f
    IFS=$old_ifs
fi
rm -rf "$grading"
mv "$grading.building" "$grading" || exit 3
