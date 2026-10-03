#!/bin/sh
# What holds `tools/ci/governed-scope.jq`, the governed-scope table of the CI
# job summary (#1649).
#
# The CI job runs on a hosted runner for a pull request and on the
# self-hosted runner for a push and a merge group, and the two can carry
# different jq versions. jq 1.6 prints the JSON literal `100.0` as `100`, and
# jq 1.7 and later print it as written, so a filter that interpolated the
# share would draw `100%` on one runner and `100.0%` on the other. Each case
# below renders a recorded document and asserts the share with one decimal
# place, as the text report prints it, whatever jq this is.
#
# With a built engine as its one argument, it also runs `taxonomy audit` over
# this repository twice, as text and as JSON, and asserts that the table's
# total states the share the text report states.
#
# Run it from anywhere:
#     sh tools/ci/governed-scope-fixtures.sh [path/to/headwater]
#
# It needs `jq`, and it exits 3 without it. It writes only under a temporary
# directory.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
# `GOVERNED_SCOPE_FILTER` names another filter to hold, which is how a mutant
# of the shipped one is run against these cases.
filter=${GOVERNED_SCOPE_FILTER:-"$root/tools/ci/governed-scope.jq"}
engine=${1:-}

if ! command -v jq >/dev/null 2>&1; then
    echo "no \`jq\` on the path, and the filter under test is a jq program." >&2
    exit 3
fi

scratch=$(mktemp -d) || exit 1
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

passed=0
failed=0

pass() {
    printf 'ok   %s\n' "$1"
    passed=$((passed + 1))
}

fail() {
    printf 'FAIL %s\n' "$1"
    failed=$((failed + 1))
}

echo "jq: $(jq --version 2>&1)"

# document <total share> <row share> writes one recorded document, with the
# share members written as the engine writes them, as raw numbers to one
# decimal place, or `null`.
document() {
    printf '{"version":1,"subject":{"package":"audit/fixture","version":"1.0.0","lock":"sha256:0","now":"2026-01-01"},"scope":[{"anchor_kind":"code-path","pattern":"src/**","in_scope":4,"governed":1,"share":%s,"ungoverned":["src/b.rs"]}],"scope_total":{"in_scope":4,"governed":1,"share":%s}}\n' "$2" "$1"
}

# expect <share as the document holds it> <cell the table must draw>
expect() {
    document "$1" "$1" > "$scratch/doc.json"
    if ! jq -r -f "$filter" "$scratch/doc.json" > "$scratch/table.md" 2> "$scratch/err"; then
        fail "the filter refuses a share of $1: $(cat "$scratch/err")"
        return
    fi
    row=$(grep -c "^| \`src/\*\*\` | code-path | 1 | 4 | $2 |\$" "$scratch/table.md")
    total=$(grep -c "^| \*\*total, over the union\*\* | | 1 | 4 | $2 |\$" "$scratch/table.md")
    if [ "$row" = 1 ] && [ "$total" = 1 ]; then
        pass "a share of $1 draws | $2 | in the row and the total"
    else
        fail "a share of $1 does not draw | $2 | in both lines:"
        cat "$scratch/table.md"
    fi
}

expect 100.0 '100.0%'
expect 25.0 '25.0%'
expect 33.3 '33.3%'
expect 66.7 '66.7%'
expect 99.9 '99.9%'
expect 12.5 '12.5%'
expect 0.0 '0.0%'
expect null 'no entry'

if [ -n "$engine" ]; then
    "$engine" taxonomy audit --root "$root" --now 2026-01-01 > "$scratch/audit.txt" 2> "$scratch/audit.err"
    "$engine" taxonomy audit --root "$root" --now 2026-01-01 --json > "$scratch/audit.json" 2>> "$scratch/audit.err"
    text=$(sed -n 's/.*in total [0-9]* of [0-9]* entries in scope are governed, \([0-9.]*%\)$/\1/p' "$scratch/audit.txt")
    jq -r -f "$filter" "$scratch/audit.json" > "$scratch/audit.md"
    table=$(sed -n 's/^| \*\*total, over the union\*\* | | [0-9]* | [0-9]* | \(.*\) |$/\1/p' "$scratch/audit.md")
    if [ -n "$text" ] && [ "$text" = "$table" ]; then
        pass "the table's total over this repository states the text report's share, $text"
    else
        fail "the text report states '$text' and the table states '$table'"
    fi
fi

printf '%s passed, %s failed\n' "$passed" "$failed"
[ "$failed" = 0 ]
