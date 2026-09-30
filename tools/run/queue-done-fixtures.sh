#!/bin/sh
# What holds `tools/run/queue-done.sh`: a pull request in the queue, or one
# with auto-merge set, is not finished; a merged, closed, ejected or never
# queued one is, and each says which; a failed call is never read as an
# ejection; and a number that is not a number is refused before `gh` runs.
#
# Run it from anywhere:
#     sh tools/run/queue-done-fixtures.sh
#
# It needs no network and no token: `gh` is a fake on `PATH` for the length
# of this suite, which logs its arguments and prints the TSV row the real
# `gh` would print through the script's own `--jq` filter.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
tool="$root/tools/run/queue-done.sh"

if [ ! -f "$tool" ]; then
    echo "no script at \`tools/run/queue-done.sh\`." >&2
    exit 1
fi

scratch=$(mktemp -d) || exit 1
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

fakebin="$scratch/fakebin"
mkdir -p "$fakebin"
log="$scratch/gh.log"
answer="$scratch/answer.tsv"

cat > "$fakebin/gh" <<'EOF'
#!/bin/sh
printf '%s\n' "$*" >> "$LOG"
[ -f "$ANSWER" ] || exit 1
cat "$ANSWER"
EOF
chmod +x "$fakebin/gh"

t=$(printf '\t')
passed=0
failed=0

run() {
    : > "$log"
    PATH="$fakebin:$PATH" LOG="$log" ANSWER="$answer" sh "$tool" "$@" > "$scratch/out" 2> "$scratch/err"
}

ok() { passed=$((passed + 1)); echo "  ok    $1"; }
bad() { failed=$((failed + 1)); echo "  FAIL  $1"; echo "        out: $(cat "$scratch/out")"; echo "        err: $(cat "$scratch/err")"; }

echo "a pull request in the queue is not finished"
printf 'OPEN%s-%sAWAITING_CHECKS%s-%s-\n' "$t" "$t" "$t" "$t" > "$answer"
run 42; status=$?
if [ "$status" -eq 1 ] && [ ! -s "$scratch/out" ] && grep -q 'in the queue, AWAITING_CHECKS' "$scratch/err"; then ok "queued: exit 1, nothing on stdout"; else bad "queued (exit $status)"; fi

echo "a pull request with auto-merge set is not finished"
printf 'OPEN%s-%s-%sauto%s-\n' "$t" "$t" "$t" "$t" > "$answer"
run 42; status=$?
if [ "$status" -eq 1 ] && grep -q 'waits on its own checks' "$scratch/err"; then ok "auto-merge: exit 1"; else bad "auto-merge (exit $status)"; fi

echo "a merged pull request is finished and names its commit"
printf 'MERGED%sabc123%s-%s-%s-\n' "$t" "$t" "$t" "$t" > "$answer"
run 42; status=$?
if [ "$status" -eq 0 ] && grep -qx 'queue-done: #42 merged abc123' "$scratch/out"; then ok "merged: exit 0 with the sha"; else bad "merged (exit $status)"; fi

echo "an ejected pull request is finished and says why"
printf 'OPEN%s-%s-%s-%sremoved:The merge queue group failed: Engine tests\n' "$t" "$t" "$t" "$t" > "$answer"
run 42; status=$?
if [ "$status" -eq 0 ] && grep -qx 'queue-done: #42 ejected: The merge queue group failed: Engine tests' "$scratch/out"; then ok "ejected: exit 0 with the reason"; else bad "ejected (exit $status)"; fi

echo "an open pull request never queued is finished and is not called ejected"
printf 'OPEN%s-%s-%s-%s-\n' "$t" "$t" "$t" "$t" > "$answer"
run 42; status=$?
if [ "$status" -eq 0 ] && grep -q 'not queued' "$scratch/out" && ! grep -q ejected "$scratch/out"; then ok "never queued: exit 0, not queued"; else bad "never queued (exit $status)"; fi

echo "a removal the queue made because it merged is not an ejection (#1353)"
printf 'OPEN%s-%s-%s-%sremoved:merged\n' "$t" "$t" "$t" "$t" > "$answer"
run 42; status=$?
if [ "$status" -eq 1 ] && [ ! -s "$scratch/out" ] && grep -q 'merge is not recorded yet' "$scratch/err"; then ok "#1353: exit 1, nothing on stdout, never ejected"; else bad "#1353 removed as merged (exit $status)"; fi

echo "the merged reason is read in any case"
printf 'OPEN%s-%s-%s-%sremoved:MERGED\n' "$t" "$t" "$t" "$t" > "$answer"
run 42; status=$?
if [ "$status" -eq 1 ] && [ ! -s "$scratch/out" ]; then ok "MERGED reason: exit 1, never ejected"; else bad "MERGED reason (exit $status)"; fi

echo "an add with no removal visible yet is not finished, and never not queued (#1412)"
printf 'OPEN%s-%s-%s-%sadded\n' "$t" "$t" "$t" "$t" > "$answer"
run 42; status=$?
if [ "$status" -eq 1 ] && [ ! -s "$scratch/out" ] && grep -q 'removal is not visible yet' "$scratch/err"; then ok "#1412: exit 1, nothing on stdout"; else bad "#1412 added, no entry (exit $status)"; fi

echo "a failed_checks removal after an earlier add is an ejection"
printf 'OPEN%s-%s-%s-%sremoved:failed_checks\n' "$t" "$t" "$t" "$t" > "$answer"
run 42; status=$?
if [ "$status" -eq 0 ] && grep -qx 'queue-done: #42 ejected: failed_checks' "$scratch/out"; then ok "ejected: failed_checks"; else bad "failed_checks (exit $status)"; fi

echo "a reason that holds a colon is kept whole"
printf 'OPEN%s-%s-%s-%sremoved:group failed: Lint\n' "$t" "$t" "$t" "$t" > "$answer"
run 42; status=$?
if [ "$status" -eq 0 ] && grep -qx 'queue-done: #42 ejected: group failed: Lint' "$scratch/out"; then ok "a colon in the reason survives"; else bad "colon reason (exit $status)"; fi

echo "an unmergeable entry is finished at once, before the queue ejects it (#1328)"
printf 'OPEN%s-%sUNMERGEABLE%s-%sadded\n' "$t" "$t" "$t" "$t" > "$answer"
run 42; status=$?
if [ "$status" -eq 0 ] && grep -qx 'queue-done: #42 unmergeable: the queue will eject it with merge_conflict' "$scratch/out"; then ok "#1328: exit 0 with the unmergeable line"; else bad "#1328 unmergeable (exit $status)"; fi

echo "a queue entry in another state still holds the wait, whatever the timeline says"
printf 'OPEN%s-%sMERGEABLE%s-%sremoved:failed_checks\n' "$t" "$t" "$t" "$t" > "$answer"
run 42; status=$?
if [ "$status" -eq 1 ] && [ ! -s "$scratch/out" ] && grep -q 'in the queue, MERGEABLE' "$scratch/err"; then ok "requeued after an ejection: exit 1"; else bad "requeued (exit $status)"; fi

echo "an event this script does not know is asked again"
printf 'OPEN%s-%s-%s-%sreopened\n' "$t" "$t" "$t" "$t" > "$answer"
run 42; status=$?
if [ "$status" -eq 1 ] && [ ! -s "$scratch/out" ]; then ok "unknown event: exit 1"; else bad "unknown event (exit $status)"; fi

echo "a closed pull request is finished"
printf 'CLOSED%s-%s-%s-%s-\n' "$t" "$t" "$t" "$t" > "$answer"
run 42; status=$?
if [ "$status" -eq 0 ] && grep -qx 'queue-done: #42 closed' "$scratch/out"; then ok "closed: exit 0"; else bad "closed (exit $status)"; fi

echo "a failed call is asked again, and never read as an ejection"
rm -f "$answer"
run 42; status=$?
if [ "$status" -eq 1 ] && [ ! -s "$scratch/out" ] && grep -q 'query failed' "$scratch/err"; then ok "gh error: exit 1, nothing on stdout"; else bad "gh error (exit $status)"; fi

echo "a short answer is asked again"
printf 'OPEN%s-\n' "$t" > "$answer"
run 42; status=$?
if [ "$status" -eq 1 ] && [ ! -s "$scratch/out" ]; then ok "short row: exit 1"; else bad "short row (exit $status)"; fi

echo "the number is the pull request's, passed as an integer"
printf 'MERGED%sabc123%s-%s-%s-\n' "$t" "$t" "$t" "$t" > "$answer"
run 42
if grep -q 'graphql' "$log" && grep -q 'n=42' "$log"; then ok "gh api graphql with n=42"; else bad "arguments: $(cat "$log")"; fi

echo "a number that is not a number is refused before gh runs"
run '42; rm -rf /'; status=$?
if [ "$status" -eq 64 ] && [ ! -s "$log" ]; then ok "refused: exit 64, gh never ran"; else bad "non-number (exit $status)"; fi

echo
echo "queue-done: $passed passed, $failed failed"
[ "$failed" -eq 0 ]
