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
#     sh tools/probe/ablate.sh <tier> <workspace>
#     sh tools/probe/ablate.sh --present <workspace>
#     sh tools/probe/ablate.sh --instrument
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

usage="usage: sh tools/probe/ablate.sh <tier> <workspace>, --present <workspace>, or --instrument"
tier=${1:-}
workspace=${2:-}
arm=absent
case "$tier" in
    --present) arm=present ;;
    --instrument) arm=list ;;
esac
if [ "$arm" = list ]; then
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
listing=$(awk -v want="$tier" -v arm="$arm" '
    function trim(s) { sub(/^[ \t]+/, "", s); sub(/[ \t]+$/, "", s); return s }
    function emit(e, kind,    n, c, i) {
        bad = (e == "" || e ~ /^\//)
        n = split(e, c, "/")
        for (i = 1; i <= n; i++) if (c[i] == ".." || c[i] == "." || c[i] == "") bad = 1
        print (bad ? "unsafe " : (kind == "instrument" ? "instrument " : "entry ")) e
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
        cur = trim(substr($0, 3)); sub(/:$/, "", cur); block = 0
        if (cur == want && arm == "absent") print "tier"
        next
    }
    cur != want { next }
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

if [ "$arm" = absent ]; then
    case "$listing" in
        tier*|*"
tier"*) ;;
        *)
            echo "ablate: \`$tier\` is not a tier $declaration declares" >&2
            exit 2
            ;;
    esac
fi
case "$listing" in
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
if [ "$arm" = absent ] && [ -z "$entries" ]; then
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
