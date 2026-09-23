#!/bin/sh
# What holds `tools/run/wait-for.sh`: a condition already true returns at
# once, a condition that never becomes true is reported as RE-ISSUE once the
# cap is reached and not retried by the script itself, and a bad argument is
# refused before any sleep.
#
# Run it from anywhere:
#     sh tools/run/wait-for-fixtures.sh
#
# Every case below caps at one or two seconds, so the suite runs fast; the
# 240/30 production defaults are exercised only as argument parsing, never as
# a real wait.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
tool="$root/tools/run/wait-for.sh"

if [ ! -f "$tool" ]; then
    echo "no script at \`tools/run/wait-for.sh\`." >&2
    exit 1
fi

scratch=$(mktemp -d) || exit 1
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

passed=0
failed=0

echo "a condition already true returns 0 without sleeping"
: > "$scratch/flag"
start=$(date +%s)
sh "$tool" --cap 5 --poll 1 "[ -f $scratch/flag ]" > "$scratch/out" 2>"$scratch/err"
status=$?
end=$(date +%s)
if [ "$status" -eq 0 ] && [ $((end - start)) -le 1 ] && grep -q 'condition met after 0s' "$scratch/out"; then
    passed=$((passed + 1))
    echo "  ok    returns immediately, exit 0"
else
    failed=$((failed + 1))
    echo "  FAIL  returns immediately, exit 0 (status $status, $((end - start))s, out: $(cat "$scratch/out"))"
fi

echo "a condition that never becomes true is RE-ISSUE at the cap, not retried"
sh "$tool" --cap 1 --poll 1 "[ -f $scratch/never ]" > "$scratch/out" 2>"$scratch/err"
status=$?
if [ "$status" -eq 2 ] && grep -q 'RE-ISSUE' "$scratch/err"; then
    passed=$((passed + 1))
    echo "  ok    exits 2 and says RE-ISSUE"
else
    failed=$((failed + 1))
    echo "  FAIL  exits 2 and says RE-ISSUE (status $status, err: $(cat "$scratch/err"))"
fi

echo "no condition is refused before any sleep"
start=$(date +%s)
sh "$tool" > "$scratch/out" 2>"$scratch/err"
status=$?
end=$(date +%s)
if [ "$status" -eq 2 ] && [ $((end - start)) -le 1 ]; then
    passed=$((passed + 1))
    echo "  ok    a missing condition is refused at once"
else
    failed=$((failed + 1))
    echo "  FAIL  a missing condition is refused at once (status $status)"
fi

echo "a non-numeric cap is refused before any sleep"
start=$(date +%s)
sh "$tool" --cap soon "true" > "$scratch/out" 2>"$scratch/err"
status=$?
end=$(date +%s)
if [ "$status" -eq 2 ] && [ $((end - start)) -le 1 ]; then
    passed=$((passed + 1))
    echo "  ok    a non-numeric --cap is refused at once"
else
    failed=$((failed + 1))
    echo "  FAIL  a non-numeric --cap is refused at once (status $status)"
fi

echo "a poll under ten seconds is a warning, not a refusal"
sh "$tool" --cap 1 --poll 1 "true" > "$scratch/out" 2>"$scratch/err"
status=$?
if [ "$status" -eq 0 ] && grep -q 'under the floor' "$scratch/err"; then
    passed=$((passed + 1))
    echo "  ok    a short poll still runs, and warns"
else
    failed=$((failed + 1))
    echo "  FAIL  a short poll still runs, and warns (status $status)"
fi

echo "$passed passed; $failed failed"
[ "$failed" -eq 0 ]
