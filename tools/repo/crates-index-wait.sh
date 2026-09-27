#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
#
# Waits until the crates.io index lists a crate's workspace dependencies, or
# the crate itself, at the release version (#1197).
#
#     sh tools/repo/crates-index-wait.sh <crate> <version>
#     sh tools/repo/crates-index-wait.sh --published <crate> <version>
#
# The first form reads <crate>'s dependencies on other members of the engine
# workspace from `cargo metadata --no-deps`, every kind, because each one
# carries a version and `cargo publish` keeps it. It waits until the index
# lists each of them at <version>. `.github/workflows/publish-crates.yml` runs
# it before it publishes a crate. The second form waits until the index lists
# <crate> itself at <version>, and the workflow runs it after `cargo publish`,
# so a cargo "timed out waiting for availability" warning is not a success.
#
# `publish-crates.yml` waited a fixed 30 seconds between crates. At v0.3.0
# and again at v0.4.0 that was not enough, and the job stopped with crates
# unpublished until somebody ran it again (HW-PD-0011).
#
# The index is the sparse one cargo reads: one file per crate at
# <base>/<prefix>/<name>, one JSON line per version with a `vers` field.
# Two overrides exist for `engine/crates/cli/tests/publish_order.rs`, which
# holds this script against an index on disk:
#
#     HW_CRATES_INDEX            the index base, default https://index.crates.io
#                                (a file:// URL reads a directory)
#     HW_CRATES_INDEX_DEADLINE   seconds to wait for each name, default 600
#
# A name the index does not list by the deadline prints one `::error::` line
# on standard output, which the Actions runner shows as an annotation, and the
# script exits 1. Progress goes to standard error. The deadline is not a
# retry ceiling for a 429, which HW-OBL-0182 records.
set -eu

mode=deps
if [ "${1:-}" = "--published" ]; then
    mode=published
    shift
fi
if [ "$#" -ne 2 ]; then
    echo "usage: sh tools/repo/crates-index-wait.sh [--published] <crate> <version>" >&2
    exit 2
fi
crate=$1
version=$2
base=${HW_CRATES_INDEX:-https://index.crates.io}
deadline=${HW_CRATES_INDEX_DEADLINE:-600}
interval=10

here=$(cd "$(dirname "$0")" && pwd)
manifest="$here/../../engine/Cargo.toml"

if [ "$mode" = published ]; then
    names=$crate
else
    names=$(cargo metadata --no-deps --offline --format-version 1 --manifest-path "$manifest" |
        python3 -c '
import json, sys
crate = sys.argv[1]
packages = json.load(sys.stdin)["packages"]
members = {p["name"] for p in packages}
mine = [p for p in packages if p["name"] == crate]
if not mine:
    sys.exit(f"{crate} is not a member of the engine workspace")
print(" ".join(sorted({d["name"] for d in mine[0]["dependencies"]} & members - {crate})))
' "$crate")
fi

# The sparse index path of a crate name, which cargo derives from its length.
index_path() {
    name=$(printf '%s' "$1" | tr '[:upper:]' '[:lower:]')
    case ${#name} in
        1) echo "1/$name" ;;
        2) echo "2/$name" ;;
        3) echo "3/$(printf '%s' "$name" | cut -c1)/$name" ;;
        *) echo "$(printf '%s' "$name" | cut -c1-2)/$(printf '%s' "$name" | cut -c3-4)/$name" ;;
    esac
}

# crates.io refuses curl's default User-Agent, and the CDN in front of the
# index serves a cached file unless asked not to.
listed() {
    curl -fsS -H 'User-Agent: headwater-publish-workflow (https://github.com/headwater-ai/headwater)' \
        -H 'Cache-Control: no-cache' "$base/$(index_path "$1")" 2>/dev/null |
        grep -qF "\"vers\":\"$version\""
}

for name in $names; do
    start=$(date +%s)
    until listed "$name"; do
        waited=$(($(date +%s) - start))
        if [ "$waited" -ge "$deadline" ]; then
            if [ "$mode" = published ]; then
                echo "::error::$crate: the crates.io index does not list $crate $version after ${deadline}s, though cargo publish returned"
            else
                echo "::error::$crate: the crates.io index does not list $name $version after ${deadline}s"
            fi
            exit 1
        fi
        pause=$((deadline - waited))
        if [ "$pause" -gt "$interval" ]; then
            pause=$interval
        fi
        echo "waiting for the crates.io index to list $name $version (${waited}s of ${deadline}s)" >&2
        sleep "$pause"
    done
    echo "the crates.io index lists $name $version" >&2
done
