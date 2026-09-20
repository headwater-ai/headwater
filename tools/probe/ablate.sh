#!/bin/sh
# Produce the absent-arm tree of a probe workspace from its present-arm copy.
#
# `.headwater/probe.yml` records what "absent" removes and why: `CLAUDE.md`,
# `.claude/`, `.githooks/` and `.headwater/`, and nothing else. This script is
# the mechanism that comment names. It takes a present-arm workspace — the
# full copy outside this repository that `tools/probe/probe-record.sh`
# already requires before it will drive a session — and deletes those four
# paths from it, in place.
#
#     sh tools/probe/ablate.sh <workspace>
#
# It refuses a workspace inside this repository's own checkout, the same
# guard `probe-record.sh` applies to the same argument and for the same
# reason: ablating a real checkout is not what an absent-arm workspace is
# for, and a refusal here is cheaper than a corpus this repository ships with
# no `CLAUDE.md`.
#
# It is idempotent. A workspace already missing one of the four paths is
# missing nothing new when this runs again, and the report at the end says
# which of the four it found.

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
# `CLAUDE.md`, `.claude/`, `.githooks/` and `.headwater/` it resolves to.
# `pwd -P` asks the kernel for the physical directory instead, which
# resolves every symlink in the path, root and workspace alike, before
# either is compared or deleted.
case $0 in
    */*) invoked_from=${0%/*} ;;
    *) invoked_from=. ;;
esac
root=$(cd "$invoked_from/../.." && pwd -P)

workspace=${1:-}
[ -n "$workspace" ] || {
    echo "usage: sh tools/probe/ablate.sh <workspace>" >&2
    exit 2
}
here=$(cd "$workspace" 2>/dev/null && pwd -P) || {
    echo "ablate: no workspace directory at $workspace" >&2
    exit 2
}

# The same guard `probe-record.sh` applies to its own `--workspace` argument:
# a path inside this repository's checkout is refused rather than ablated.
case "$here" in
    "$root"|"$root"/*)
        echo "ablate: $here is inside this repository's own checkout." >&2
        echo "ablate: an absent-arm workspace is a copy outside it. Use one." >&2
        exit 6
        ;;
esac

found=""
for path in CLAUDE.md .claude .githooks .headwater; do
    if [ -e "$here/$path" ]; then
        found="$found $path"
        rm -rf "${here:?}/$path"
    fi
done

if [ -z "$found" ]; then
    echo "ablate: $here already carried none of CLAUDE.md, .claude/, .githooks/, .headwater/"
else
    echo "ablate: removed$found from $here"
fi
