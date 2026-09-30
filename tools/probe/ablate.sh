#!/bin/sh
# Produce the absent-arm tree of a probe workspace from its present-arm copy.
#
# `.headwater/probe.yml` declares, per tier, what "absent" removes: the
# `ablation` sequence of that tier. This script is the mechanism that field
# names. It takes a present-arm workspace — the full copy outside this
# repository that `tools/probe/probe-record.sh` already requires before it
# will drive a session — and deletes the tier's ablation entries from it, in
# place.
#
#     sh tools/probe/ablate.sh <tier> <workspace> [<arm>]
#     sh tools/probe/ablate.sh --present <workspace>
#     sh tools/probe/ablate.sh --diff <tier> <arm> <present-tree>
#     sh tools/probe/ablate.sh --instrument
#
# `<arm>` is `absent` when it is not given. A component arm of the tier
# (#1472), such as `no-hook` or `mcp`, applies the delta the tier declares for
# it under `components` to the present tree the workspace already is: `mcp`
# adds its paths from `tools/probe/arms/mcp/`, and every other component arm
# removes its paths. `--diff` is described where it is parsed below.
#
# `--instrument` prints the instrument entries, one per line, and touches
# nothing. `probe-record.sh` reads it to refuse a workspace that still holds
# one.
#
# Every arm also loses the `instrument`, the top-level sequence of the same
# file: the probe shelves, whose documents state the answer each probe
# expects. `--present` produces a present-arm tree, which removes the
# instrument alone, and it is how every present arm, the regression tier's
# included, is prepared.
#
# `campaign` removes `CLAUDE.md`, `.claude/`, `.githooks/` and `.headwater/`.
# `documentation` removes those four and `docs/`. The list is read from the
# checkout's own `.headwater/probe.yml` and never from the workspace, because
# the workspace is the tree being ablated and its copy is one of the paths
# removed. `HW_PROBE_YML` names another declaration, for the fixtures alone.
#
# The ablation, each component delta and the instrument are read from the
# engine, `headwater probe plan --delta` and `--instrument`, and never parsed
# here (#1472). So it needs a built engine, and without one it refuses. It
# refuses a tier the declaration does not carry, a tier that does not run the
# arm (the `regression` tier runs no absent arm), and an entry that is empty,
# absolute or has a `..` or `.` component, because every entry goes to
# `rm -rf`. The engine makes each of those refusals when it reads the file.
#
# It refuses a workspace inside this repository's own checkout, the same
# guard `probe-record.sh` applies to the same argument and for the same
# reason: ablating a real checkout is not what an absent-arm workspace is
# for, and a refusal here is cheaper than a corpus this repository ships with
# no `CLAUDE.md`.
#
# It is idempotent. A workspace already missing an entry is missing nothing
# new when this runs again, and the report at the end says which entries it
# found.

set -eu

# `cd` and `pwd` are builtins and `dirname` is not. Resolving the root with a
# parameter expansion means the refusal below reaches its message on a host
# with nothing on `PATH` at all.
#
# `pwd -P` and never bare `pwd`. Bare `pwd` on this host's `/bin/sh` (dash)
# prints the logical path, symlinks intact, so a workspace argument that is
# itself a symlink into this checkout — or that reaches one through an
# ancestor directory — would never match the prefix check below, and the
# `rm -rf` after it would then follow that symlink and delete the real
# paths it resolves to. `pwd -P` asks the kernel for the physical directory
# instead, which resolves every symlink in the path, root and workspace
# alike, before either is compared or deleted.
case $0 in
    */*) invoked_from=${0%/*} ;;
    *) invoked_from=. ;;
esac
root=$(cd "$invoked_from/../.." && pwd -P)

usage="usage: sh tools/probe/ablate.sh <tier> <workspace> [<arm>], --present <workspace>, --diff <tier> <arm> <present-tree>, or --instrument"

# `--diff <tier> <arm> <present-tree>` builds the arm's tree in a copy of a
# present tree outside this checkout, prints how it differs from the present
# tree, and deletes the copy (#1472). One line per difference: `- <path>` for
# a path the arm lacks and `+ <path>` for a path it adds, where a whole
# directory is one line. It exits 1 when the arm's tree does not differ, or
# differs by a path its declared delta does not name, which it prints as
# `unexpected <line>`. A present tree has no instrument, so a tree that still
# holds one is refused before anything is copied.
if [ "${1:-}" = --diff ]; then
    [ -n "${2:-}" ] && [ -n "${3:-}" ] && [ -n "${4:-}" ] || { echo "$usage" >&2; exit 2; }
    diff_tier=$2
    diff_arm=$3
    diff_present=$(cd "$4" 2>/dev/null && pwd -P) || { echo "ablate: no present tree at $4" >&2; exit 2; }
    for diff_path in $(sh "$0" --instrument); do
        if [ -e "$diff_present/$diff_path" ]; then
            echo "ablate: $diff_present still holds \`$diff_path\`, which every arm removes, so it is not a present tree" >&2
            exit 2
        fi
    done
    diff_scratch=$(mktemp -d "${TMPDIR:-/tmp}/headwater-ablate-diff.XXXXXX") || exit 2
    cp -a "$diff_present" "$diff_scratch/arm" || { rm -rf "$diff_scratch"; exit 2; }
    sh "$0" "$diff_tier" "$diff_scratch/arm" "$diff_arm" > "$diff_scratch/ablate.out" 2> "$diff_scratch/ablate.err" || {
        cat "$diff_scratch/ablate.err" >&2
        rm -rf "$diff_scratch"
        exit 2
    }
    # The delta the declaration names, as `- <path>` and `+ <path>` lines.
    sed -n 's/^declared \([-+]\) /\1 /p' "$diff_scratch/ablate.out" > "$diff_scratch/declared"
    # `diff` exits 1 when the trees differ, which is the expected case, and 2
    # when it could not compare them, which fails closed.
    diff_status=0
    diff -rq --no-dereference "$diff_present" "$diff_scratch/arm" > "$diff_scratch/diff" 2>&1 || diff_status=$?
    if [ "$diff_status" -gt 1 ]; then
        echo "ablate: diff could not compare the present tree with the \`$diff_arm\` arm:" >&2
        cat "$diff_scratch/diff" >&2
        rm -rf "$diff_scratch"
        exit 2
    fi
    diff_status=0
    awk -v present="$diff_present" -v arm="$diff_scratch/arm" '
        /^Only in / {
            rest = substr($0, 9)
            colon = index(rest, ": ")
            dir = substr(rest, 1, colon - 1)
            name = substr(rest, colon + 2)
            if (index(dir, arm) == 1) { sign = "+"; dir = substr(dir, length(arm) + 1) }
            else if (index(dir, present) == 1) { sign = "-"; dir = substr(dir, length(present) + 1) }
            else { print "? " $0; next }
            sub(/^\//, "", dir)
            print sign " " (dir == "" ? name : dir "/" name)
            next
        }
        { print "? " $0 }
    ' "$diff_scratch/diff" | LC_ALL=C sort > "$diff_scratch/found"
    if [ ! -s "$diff_scratch/found" ]; then
        echo "ablate: the \`$diff_arm\` arm of the \`$diff_tier\` tier builds a tree no different from the present tree" >&2
        rm -rf "$diff_scratch"
        exit 1
    fi
    while IFS= read -r diff_line; do
        diff_expected=0
        while IFS= read -r diff_entry; do
            diff_sign=${diff_entry%% *}
            diff_named=${diff_entry#* }
            case "$diff_line" in
                "$diff_sign $diff_named"|"$diff_sign $diff_named"/*) diff_expected=1 ;;
            esac
        done < "$diff_scratch/declared"
        if [ "$diff_expected" = 1 ]; then
            printf '%s\n' "$diff_line"
        else
            printf 'unexpected %s\n' "$diff_line"
            diff_status=1
        fi
    done < "$diff_scratch/found"
    rm -rf "$diff_scratch"
    exit "$diff_status"
fi

# `--delta <tier> <arm>` prints the arm's declared delta as `declared - <path>`
# and `declared + <path>` lines and touches nothing. `probe-record.sh` reads it
# to state in a transcript what the arm holds (#1472).
dry=0
if [ "${1:-}" = --delta ]; then
    [ -n "${2:-}" ] && [ -n "${3:-}" ] || { echo "$usage" >&2; exit 2; }
    dry=1
    set -- "$2" "" "$3"
    [ "$3" != present ] || exit 0
fi

tier=${1:-}
workspace=${2:-}
arm=absent
case "$tier" in
    --present) arm=present ;;
    --instrument) arm=list ;;
esac
# The third argument names a component arm of the tier (#1472): its delta is
# the one `headwater probe plan --delta` prints, and it is applied to the present
# tree the workspace already is. `absent` is the default, and `present` is
# `--present`.
component=
case "${3:-}" in
    ""|absent) ;;
    present) arm=present ;;
    *) arm=component; component=$3 ;;
esac
if [ "$arm" = list ] || [ "$dry" = 1 ]; then
    here=""
else
    [ -n "$tier" ] && [ -n "$workspace" ] || {
        echo "$usage" >&2
        exit 2
    }
    here=$(cd "$workspace" 2>/dev/null && pwd -P) || {
        echo "ablate: no workspace directory at $workspace" >&2
        exit 2
    }
fi

# The same guard `probe-record.sh` applies to its own `--workspace` argument:
# a path inside this repository's checkout is refused rather than ablated.
case "$here" in
    "") ;;
    "$root"|"$root"/*)
        echo "ablate: $here is inside this repository's own checkout." >&2
        echo "ablate: an absent-arm workspace is a copy outside it. Use one." >&2
        exit 6
        ;;
esac

declaration=${HW_PROBE_YML:-$root/.headwater/probe.yml}
[ -f "$declaration" ] || {
    echo "ablate: no probe declaration at $declaration" >&2
    exit 2
}

# The ablation, the delta of each component arm and the instrument are all
# the engine's parse of the declaration (#1472): `headwater probe plan
# --delta` prints an arm's delta as `- <path>` for a path the arm removes and
# `+ <path>` for a path it adds, and `--instrument` prints the instrument one
# path per line. This script parses no YAML, so every form the engine accepts
# builds the same tree here. The engine refuses a tier it does not know, a
# tier that does not run the arm, and an entry that is empty, absolute or has
# a `..` or `.` component, and each refusal comes before anything is removed.
#
# It fails closed. With no built engine it refuses, and it never prints an
# empty instrument in place of one it could not read.
engine=$root/engine/target/dev-release/headwater
[ -x "$engine" ] || engine=$root/engine/target/release/headwater
[ -x "$engine" ] || {
    echo "ablate: no engine under $root/engine/target to read $declaration. Build it first: cargo build --profile dev-release -p headwater-cli --manifest-path engine/Cargo.toml --locked" >&2
    exit 2
}

# Run `headwater probe plan` with the arguments given, over the declaration.
# The engine reads `.headwater/probe.yml` under its `--root`: the checkout
# itself when `HW_PROBE_YML` is unset, which needs nothing on `PATH`, and a
# scratch root that holds a copy of the named declaration otherwise.
plan_print() {
    if [ -z "${HW_PROBE_YML:-}" ]; then
        "$engine" probe plan --root "$root" "$@"
        return
    fi
    plan_scratch=$(mktemp -d "${TMPDIR:-/tmp}/headwater-ablate-plan.XXXXXX") || return 2
    if ! mkdir -p "$plan_scratch/root/.headwater" ||
        ! cp "$declaration" "$plan_scratch/root/.headwater/probe.yml"; then
        rm -rf "$plan_scratch"
        return 2
    fi
    plan_status=0
    "$engine" probe plan --root "$plan_scratch/root" "$@" || plan_status=$?
    rm -rf "$plan_scratch"
    return "$plan_status"
}

instrument=$(plan_print --instrument) || {
    echo "ablate: the engine did not read the instrument of $declaration, so no arm can be cleared of it" >&2
    exit 2
}
if [ "$arm" = list ]; then
    [ -z "$instrument" ] || printf '%s\n' "$instrument"
    exit 0
fi
entries=
additions=
if [ "$arm" = component ] || [ "$arm" = absent ]; then
    delta_arm=${component:-absent}
    delta=$(plan_print --tier "$tier" --arm "$delta_arm" --delta) || {
        echo "ablate: the engine did not print the \`$delta_arm\` delta of the \`$tier\` tier in $declaration" >&2
        exit 2
    }
    entries=$(printf '%s\n' "$delta" | sed -n 's/^- //p')
    additions=$(printf '%s\n' "$delta" | sed -n 's/^+ //p')
    if [ -z "$entries" ] && [ -z "$additions" ]; then
        echo "ablate: the engine printed an empty delta for the \`$delta_arm\` arm of the \`$tier\` tier" >&2
        exit 2
    fi
fi
if [ "$arm" = component ]; then
    # An added path is copied from `tools/probe/arms/<arm>/` in this checkout.
    for path in $additions; do
        [ -e "$root/tools/probe/arms/$component/$path" ] || {
            echo "ablate: the \`$component\` arm adds \`$path\`, and tools/probe/arms/$component/$path is not there to copy" >&2
            exit 2
        }
    done
    printf '%s\n' "$entries" | awk 'NF { print "declared - " $0 }'
    printf '%s\n' "$additions" | awk 'NF { print "declared + " $0 }'
    arm=absent
    tier="$tier $component"
elif [ "$arm" = absent ]; then
    printf '%s\n' "$entries" | awk 'NF { print "declared - " $0 }'
fi
[ "$dry" = 0 ] || exit 0
if [ "$arm" = present ]; then
    tier=present
    entries=$instrument
    if [ -z "$entries" ]; then
        echo "ablate: $declaration declares no instrument, so a present arm removes nothing"
        exit 0
    fi
else
    entries=$(printf '%s\n%s\n' "$instrument" "$entries" | awk 'NF')
fi

found=""
missing=""
while IFS= read -r path; do
    # Never an empty path: "$here/" is the workspace itself.
    [ -n "$path" ] || continue
    if [ -e "$here/$path" ] || [ -L "$here/$path" ]; then
        found="$found $path"
        rm -rf "${here:?}/$path"
    else
        missing="$missing $path"
    fi
done <<EOF
$entries
EOF

if [ -z "$found" ]; then
    echo "ablate: $here already carried none of the \`$tier\` ablation:$missing"
else
    echo "ablate: removed$found from $here for the \`$tier\` tier"
fi

# The paths an adding arm puts in, after the instrument is gone.
for path in $additions; do
    mkdir -p "$here/$(dirname "$path")"
    cp "$root/tools/probe/arms/${tier#* }/$path" "$here/$path" || exit 2
    echo "ablate: added $path to $here for the \`$tier\` arm"
done
