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
# It refuses a tier the declaration does not carry, a tier that declares no
# ablation (the `regression` tier runs no absent arm), and an entry that is
# empty, absolute or has a `..` or `.` component, because every entry goes to
# `rm -rf`. The engine refuses the same entries when it reads the file, so a
# refusal here means the file was edited past `headwater probe plan`.
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
# read from `components` under the tier, and it is applied to the present
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

# Print `tier` when the tier is declared, then one `entry <path>` line per
# ablation entry, in declared order. The file is a flat two-level mapping
# with two-space indentation, which is what the engine reads too; `ablation`
# is either a one-line flow sequence or a block sequence under the key.
listing=$(awk -v want="$tier" -v arm="$arm" -v component="$component" '
    function trim(s) { sub(/^[ \t]+/, "", s); sub(/[ \t]+$/, "", s); return s }
    function emit(e, kind,    n, c, i) {
        bad = (e == "" || e ~ /^\//)
        n = split(e, c, "/")
        for (i = 1; i <= n; i++) if (c[i] == ".." || c[i] == "." || c[i] == "") bad = 1
        print (bad ? "unsafe " : (kind == "instrument" ? "instrument " : (kind == "component" ? "delta " : "entry "))) e
    }
    # A YAML comment starts at a `#` after whitespace, and the engine reads
    # past it. So does this, or a comment the plan accepts would make the
    # sequence malformed here and nowhere else.
    function uncomment(s) { sub(/(^|[ \t])#.*$/, "", s); return trim(s) }
    function unquote(s) {
        s = trim(s)
        if (s ~ /^".*"$/ || s ~ /^\047.*\047$/) s = substr(s, 2, length(s) - 2)
        return s
    }
    /^instrument:/ {
        intiers = 0; cur = ""; block = 0
        rest = uncomment(substr($0, index($0, ":") + 1))
        if (rest == "") { iblock = 1; next }
        if (rest !~ /^\[.*\]$/) { print "malformed"; next }
        rest = substr(rest, 2, length(rest) - 2)
        n = split(rest, parts, ",")
        if (trim(rest) == "") n = 0
        for (i = 1; i <= n; i++) emit(unquote(parts[i]), "instrument")
        next
    }
    iblock && /^  - / { emit(unquote(uncomment(substr($0, 5))), "instrument"); next }
    /^[^ #]/ { iblock = 0; intiers = ($0 ~ /^tiers:/); cur = ""; block = 0; next }
    !intiers { next }
    /^  [^ #][^:]*:[ \t]*$/ {
        cur = trim(substr($0, 3)); sub(/:$/, "", cur); block = 0; comp = 0
        if (cur == want && (arm == "absent" || arm == "component")) print "tier"
        next
    }
    cur != want { next }
    # `components`, a mapping of arm to a sequence of paths (#1472): a flow
    # sequence on the key line, or a block sequence under it, which the
    # engine reads as the same value (verify round 1).
    /^    components:[ \t]*$/ { comp = 1; block = 0; cblock = 0; next }
    comp && cblock && /^(      |        )-[ \t]/ {
        entry = $0; sub(/^[ \t]*-[ \t]*/, "", entry)
        emit(unquote(uncomment(entry)), "component")
        next
    }
    comp && /^      [^ #-][^:]*:/ {
        cblock = 0
        name = trim(substr($0, 7)); sub(/:.*$/, "", name)
        if (name != component) next
        print "component"
        rest = uncomment(substr($0, index($0, ":") + 1))
        if (rest == "") { cblock = 1; next }
        if (rest !~ /^\[.*\]$/) { print "malformed-component"; next }
        rest = substr(rest, 2, length(rest) - 2)
        n = split(rest, parts, ",")
        if (trim(rest) == "") n = 0
        for (i = 1; i <= n; i++) emit(unquote(parts[i]), "component")
        next
    }
    /^    [^ ]/ { comp = 0; cblock = 0 }
    /^    ablation:/ {
        rest = uncomment(substr($0, index($0, ":") + 1))
        if (rest == "") { block = 1; print "declared"; next }
        if (rest !~ /^\[.*\]$/) { print "malformed"; next }
        print "declared"
        rest = substr(rest, 2, length(rest) - 2)
        n = split(rest, parts, ",")
        if (trim(rest) == "") n = 0
        for (i = 1; i <= n; i++) emit(unquote(parts[i]))
        next
    }
    block && /^      - / { emit(unquote(uncomment(substr($0, 9)))); next }
    block && /^      -$/ { emit(""); next }
    /^    [^ ]/ { block = 0 }
' "$declaration")

if [ "$arm" = absent ] || [ "$arm" = component ]; then
    case "$listing" in
        tier*|*"
tier"*) ;;
        *)
            echo "ablate: \`$tier\` is not a tier $declaration declares" >&2
            exit 2
            ;;
    esac
fi
if [ "$arm" = component ]; then
    case "$listing" in
        *component*) ;;
        *)
            echo "ablate: the \`$tier\` tier declares no delta for the \`$component\` arm under \`components\`" >&2
            exit 2
            ;;
    esac
fi
case "$listing" in
    *malformed-component*)
        echo "ablate: the \`$component\` delta of the \`$tier\` tier is not a sequence of paths" >&2
        exit 2
        ;;
    *malformed*)
        echo "ablate: the \`$tier\` tier's ablation is not a sequence of paths" >&2
        exit 2
        ;;
esac

# Refuse every unsafe entry before removing anything, so a refusal leaves the
# workspace as it was.
unsafe=$(printf '%s\n' "$listing" | awk '/^unsafe/ { printf "%s`%s`", sep, substr($0, 8); sep = ", " }')
if [ -n "$unsafe" ]; then
    echo "ablate: the \`$tier\` tier's ablation names $unsafe, and an entry is a path inside the tree with no \`..\`, \`.\` or empty component" >&2
    exit 2
fi
instrument=$(printf '%s\n' "$listing" | awk '/^instrument / { print substr($0, 12) }')
if [ "$arm" = list ]; then
    [ -z "$instrument" ] || printf '%s\n' "$instrument"
    exit 0
fi
entries=$(printf '%s\n' "$listing" | awk '/^entry / { print substr($0, 7) }')
additions=
if [ "$arm" = component ]; then
    delta=$(printf '%s\n' "$listing" | awk '/^delta / { print substr($0, 7) }')
    if [ -z "$delta" ]; then
        echo "ablate: the \`$component\` arm of the \`$tier\` tier declares an empty delta" >&2
        exit 2
    fi
    # The direction of a delta is the arm's, as the engine's `Arm::adds`
    # states it: `mcp` puts its paths into the present tree, and every other
    # component arm takes them out. An added path is copied from
    # `tools/probe/arms/<arm>/` in this checkout.
    case "$component" in
        mcp)
            additions=$delta
            entries=
            for path in $additions; do
                [ -e "$root/tools/probe/arms/$component/$path" ] || {
                    echo "ablate: the \`$component\` arm adds \`$path\`, and tools/probe/arms/$component/$path is not there to copy" >&2
                    exit 2
                }
            done
            ;;
        *) entries=$delta ;;
    esac
    printf '%s\n' "$entries" | awk 'NF { print "declared - " $0 }'
    printf '%s\n' "$additions" | awk 'NF { print "declared + " $0 }'
    arm=absent
    tier="$tier $component"
elif [ "$arm" = absent ]; then
    printf '%s\n' "$entries" | awk 'NF { print "declared - " $0 }'
fi
[ "$dry" = 0 ] || exit 0
if [ "$arm" = absent ] && [ -z "$entries" ] && [ -z "$additions" ]; then
    echo "ablate: the \`$tier\` tier declares no ablation, so it runs no absent arm to produce" >&2
    exit 2
fi
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
