#!/bin/sh
# Fetch the embedding model files `.headwater/embedding.yml` pins, and check
# each one against its digest before it is kept.
#
# HW-DR-0064 keeps a socket out of the engine, so the fetch is this script's,
# the way `taxonomy vendor` leaves the fetch to its caller. `headwater
# neighbors` checks the digests again on every run, so a file this script did
# not write is refused there too.
#
# Usage: sh tools/embed/fetch-model.sh [--into <dir>]
#
# The default directory is `<git common dir>/headwater-models`, which every
# worktree of one clone shares, and which `.claude/hooks/intent.sh` passes to
# `headwater neighbors`. One fetch serves all of them.
set -eu

root=$(cd "$(dirname "$0")/../.." && pwd)
pin="$root/.headwater/embedding.yml"

into=
case ${1:-} in
    --into) into=${2:?--into takes a directory} ;;
    '') ;;
    *) echo "fetch-model: unknown argument $1" >&2; exit 2 ;;
esac
if [ -z "$into" ]; then
    common=$(git -C "$root" rev-parse --path-format=absolute --git-common-dir)
    into="$common/headwater-models"
fi
mkdir -p "$into"

command -v curl >/dev/null || { echo "fetch-model: needs curl" >&2; exit 1; }
command -v sha256sum >/dev/null || { echo "fetch-model: needs sha256sum" >&2; exit 1; }

# The pin has one shape: a `files:` mapping of `name:` then `url:` and
# `digest:` lines. Anything else is refused by the engine, not guessed at here.
awk '
    /^files:/ { infiles = 1; next }
    infiles && /^  [^ ].*:$/ { name = $1; sub(/:$/, "", name); next }
    infiles && /^    url:/ { url = $2; next }
    infiles && /^    digest:/ { print name, url, $2 }
' "$pin" | while read -r name url digest; do
    want=${digest#sha256:}
    target="$into/$name"
    if [ -f "$target" ] && [ "$(sha256sum "$target" | cut -d' ' -f1)" = "$want" ]; then
        echo "kept    $name, already the pinned bytes"
        continue
    fi
    staged="$target.part.$$"
    curl -fsSL --retry 3 -o "$staged" "$url"
    got=$(sha256sum "$staged" | cut -d' ' -f1)
    if [ "$got" != "$want" ]; then
        rm -f "$staged"
        echo "fetch-model: $name is not the pinned bytes: pinned sha256:$want, fetched sha256:$got" >&2
        exit 1
    fi
    mv "$staged" "$target"
    echo "fetched $name"
done

echo "model files in $into"
