#!/bin/sh
# What holds `tools/site/check-site-asset-size.sh`.
#
# A size check over no files is not a pass, and a limit that cannot be exceeded
# is not a check. The fixtures use a limit of 1000 bytes so that no large file is written.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
tool="$root/tools/site/check-site-asset-size.sh"

if [ ! -f "$tool" ]; then
    echo "no checker at \`tools/site/check-site-asset-size.sh\`." >&2
    exit 1
fi

scratch=$(mktemp -d) || exit 1
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

passed=0
failed=0

# report NAME EXPECTED-STATUS ACTUAL-STATUS REQUIRED-TEXT OUTPUT-FILE
report() {
    name=$1
    want=$2
    got=$3
    text=$4
    output=$5
    if [ "$want" = "$got" ] && grep -qF -- "$text" "$output"; then
        passed=$((passed + 1))
        echo "  ok    $name"
    else
        failed=$((failed + 1))
        echo "  FAIL  $name (wanted $want, got $got)"
    fi
}

run() {
    HEADWATER_SITE_ASSET_LIMIT_BYTES=1000 "$tool" "$1" >"$scratch/out" 2>"$scratch/err"
    echo $?
}

echo "a site under the limit"
mkdir -p "$scratch/small/search"
head -c 999 /dev/zero >"$scratch/small/search/search_index.json"
status=$(run "$scratch/small")
report "passes and names the largest file" 0 "$status" "largest served file" "$scratch/out"

echo "a file above the limit"
mkdir -p "$scratch/big/search"
head -c 999 /dev/zero >"$scratch/big/index.html"
head -c 1001 /dev/zero >"$scratch/big/search/search_index.json"
status=$(run "$scratch/big")
report "fails and names the file" 1 "$status" "search_index.json" "$scratch/err"

echo "a site with no file"
mkdir -p "$scratch/empty"
status=$(run "$scratch/empty")
report "is refused rather than passed" 2 "$status" "no asset size was checked" "$scratch/err"

echo "a directory that is not there"
status=$(run "$scratch/absent")
report "is refused" 2 "$status" "no served site" "$scratch/err"

echo "$passed passed, $failed failed."
[ "$failed" -eq 0 ]
