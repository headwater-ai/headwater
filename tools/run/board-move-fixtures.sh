#!/bin/sh
# What holds `tools/run/board-move.sh`: it resolves an issue to its project
# item id through `gh project item-add` before it edits anything, an issue the
# project does not have yet is added and then moved rather than refused, it
# sends the right option id for each of the three statuses, and it never reads
# the item list whose `--limit` once hid every card past the two-hundredth.
#
# Run it from anywhere:
#     sh tools/run/board-move-fixtures.sh
#
# It needs no network and no token: `gh` is a shell script on `PATH` for the
# length of this suite that logs its own arguments and answers `item-add` the
# way the real one does, with the existing item's id for an issue already on
# the project and a new id for one that is not.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
tool="$root/tools/run/board-move.sh"

if [ ! -f "$tool" ]; then
    echo "no script at \`tools/run/board-move.sh\`." >&2
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
    *"item-add 1 --owner headwater-ai --url https://github.com/headwater-ai/headwater/issues/42 "*) echo PVTI_abc123 ;;
    *"item-add 1 --owner headwater-ai --url https://github.com/headwater-ai/headwater/issues/7 "*) echo PVTI_new007 ;;
    *"item-add"*) echo "GraphQL: Could not resolve to an issue" >&2; exit 1 ;;
    *"item-edit"*) : ;;
esac
EOF
chmod +x "$fakebin/gh"

passed=0
failed=0

run() {
    : > "$log"
    PATH="$fakebin:$PATH" LOG="$log" sh "$tool" "$@"
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

echo "an issue with no card yet is added and then moved"
run 7 in-progress > "$scratch/out" 2>"$scratch/err"
status=$?
if [ "$status" -eq 0 ] && grep -q -- '--id PVTI_new007 ' "$log" && grep -q 'moved to in-progress' "$scratch/out"; then
    passed=$((passed + 1))
    echo "  ok    a missing card is added by item-add and moved with the id it returned"
else
    failed=$((failed + 1))
    echo "  FAIL  a missing card is added and moved (exit $status)"
    echo "        log: $(cat "$log")"
fi

echo "the item list and its limit are never read"
if ! grep -q 'item-list' "$log"; then
    passed=$((passed + 1))
    echo "  ok    no call reads item-list, so no limit can hide a card"
else
    failed=$((failed + 1))
    echo "  FAIL  a call read item-list: $(grep item-list "$log")"
fi

echo "an issue gh cannot add is refused, and nothing is edited"
run 404 in-progress > "$scratch/out" 2>"$scratch/err"
status=$?
if [ "$status" -ne 0 ] && ! grep -q item-edit "$log" && grep -q 'could not be put on the project board' "$scratch/err"; then
    passed=$((passed + 1))
    echo "  ok    a failed add is reported and item-edit is never called"
else
    failed=$((failed + 1))
    echo "  FAIL  a failed add is reported (exit $status)"
    echo "        log: $(cat "$log")"
fi

echo "an unknown status is refused before any gh call"
: > "$log"
PATH="$fakebin:$PATH" LOG="$log" sh "$tool" 42 blocked > "$scratch/out" 2>"$scratch/err"
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
