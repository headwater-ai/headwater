#!/bin/sh
# What holds `tools/run/board-move.sh`: it resolves an issue to its project
# item id before it edits anything, it sends the right option id for each of
# the three statuses, and a card the project does not have is reported rather
# than edited into existence.
#
# Run it from anywhere:
#     sh tools/run/board-move-fixtures.sh
#
# It needs no network and no token: `gh` and `jq` are on `PATH` as fakes for
# the length of this suite. `jq` is the real one — it only ever filters the
# fake `gh project item-list` output below — and `gh` is a shell script that
# logs its own arguments and prints a canned item list.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
tool="$root/tools/run/board-move.sh"

if [ ! -f "$tool" ]; then
    echo "no script at \`tools/run/board-move.sh\`." >&2
    exit 1
fi

if ! command -v jq >/dev/null 2>&1; then
    echo "board-move-fixtures: no \`jq\` on PATH; this suite filters a fake item list with the real one." >&2
    exit 1
fi

scratch=$(mktemp -d) || exit 1
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

fakebin="$scratch/fakebin"
mkdir -p "$fakebin"
log="$scratch/gh.log"
: > "$log"

cat > "$fakebin/gh" <<'EOF'
#!/bin/sh
printf '%s\n' "$*" >> "$LOG"
case "$*" in
    *"item-list 1 --owner headwater-ai --format json --limit 200"*)
        cat "$ITEMS"
        ;;
    *"item-edit"*) : ;;
esac
EOF
chmod +x "$fakebin/gh"

items="$scratch/items.json"
cat > "$items" <<'EOF'
{"items":[{"id":"PVTI_abc123","content":{"number":42}},{"id":"PVTI_def456","content":{"number":99}}]}
EOF

passed=0
failed=0

run() {
    : > "$log"
    PATH="$fakebin:$PATH" LOG="$log" ITEMS="$items" sh "$tool" "$@"
}

echo "in-progress resolves the item id and sends its option id"
out=$(run 42 in-progress)
status=$?
if [ "$status" -eq 0 ] && grep -q -- '--id PVTI_abc123 --project-id PVT_kwDOEsVrw84BgGeU --field-id PVTSSF_lADOEsVrw84BgGeUzhaUkwg --single-select-option-id 47fc9ee4' "$log"; then
    passed=$((passed + 1))
    echo "  ok    #42 resolved to its item id, moved with the in-progress option"
else
    failed=$((failed + 1))
    echo "  FAIL  in-progress move (exit $status)"
    echo "        log: $(cat "$log")"
fi

echo "done and todo send their own option ids"
run 42 done > /dev/null
done_ok=$(grep -c -- '--single-select-option-id 98236657' "$log")
run 42 todo > /dev/null
todo_ok=$(grep -c -- '--single-select-option-id f75ad846' "$log")
if [ "$done_ok" -eq 1 ] && [ "$todo_ok" -eq 1 ]; then
    passed=$((passed + 1))
    echo "  ok    done and todo each send their own option id"
else
    failed=$((failed + 1))
    echo "  FAIL  done and todo each send their own option id"
fi

echo "an issue with no card on the project is reported, not edited"
run 7 in-progress > "$scratch/out" 2>"$scratch/err"
status=$?
if [ "$status" -ne 0 ] && ! grep -q item-edit "$log" && grep -q 'no card on the project board' "$scratch/err"; then
    passed=$((passed + 1))
    echo "  ok    a missing card is reported and item-edit is never called"
else
    failed=$((failed + 1))
    echo "  FAIL  a missing card is reported (exit $status)"
    echo "        log: $(cat "$log")"
fi

echo "an unknown status is refused before any gh call"
: > "$log"
PATH="$fakebin:$PATH" LOG="$log" ITEMS="$items" sh "$tool" 42 blocked > "$scratch/out" 2>"$scratch/err"
status=$?
if [ "$status" -ne 0 ] && [ ! -s "$log" ]; then
    passed=$((passed + 1))
    echo "  ok    an unknown status is refused, and gh is never called"
else
    failed=$((failed + 1))
    echo "  FAIL  an unknown status is refused before any gh call"
fi

echo "$passed passed; $failed failed"
[ "$failed" -eq 0 ]
