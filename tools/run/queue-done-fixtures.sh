#!/bin/sh
# What holds `tools/run/queue-done.sh`: a pull request in the queue, or one
# with auto-merge set, is not finished; a merged, closed, ejected,
# unmergeable, stalled or never queued one is, and each says which; a
# removal as merged (#1353) and an add whose removal is not visible yet
# (#1412) are not finished and never read as ejected or not queued; an
# unmergeable entry (#1328) is final at once; an entry at the head of the
# queue whose merge group is green and has not merged for the stall interval
# (#1548) is final, and a fresh green group, a group behind another, or a
# group with a check still pending is not; a failed call is never read as an
# ejection; and a number that is not a number is refused before `gh` runs.
#
# Run it from anywhere:
#     sh tools/run/queue-done-fixtures.sh
#
# It needs `jq` and no network and no token. `gh` is a fake on `PATH` for
# the length of this suite. It logs its arguments, takes the program after
# `--jq`, and runs it with `jq -r` over the raw GraphQL response body that a
# case writes. So every case that reaches `gh` runs the script's own filter,
# and a change to that filter fails a case here. A case writes the body with
# `answer`, and an entry in the queue with `entry`. Each writes JSON, with
# `null` for each value a case gives as `-`.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
tool="$root/tools/run/queue-done.sh"

if [ ! -f "$tool" ]; then
    echo "no script at \`tools/run/queue-done.sh\`." >&2
    exit 1
fi
if ! command -v jq >/dev/null 2>&1; then
    echo "no \`jq\` on the path, and the fake \`gh\` of this suite runs the script's own \`--jq\` filter with it." >&2
    exit 1
fi

scratch=$(mktemp -d) || exit 1
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

fakebin="$scratch/fakebin"
mkdir -p "$fakebin"
log="$scratch/gh.log"
body="$scratch/body.json"

cat > "$fakebin/gh" <<'EOF'
#!/bin/sh
printf '%s\n' "$*" >> "$LOG"
[ -f "$ANSWER" ] || exit 1
filter=
prev=
for arg in "$@"; do
    [ "$prev" = "--jq" ] && filter=$arg
    prev=$arg
done
[ -n "$filter" ] || exit 1
jq -r "$filter" "$ANSWER"
EOF
chmod +x "$fakebin/gh"

# The clock of every case: 2026-10-01T12:00:00Z.
now=1790856000

passed=0
failed=0

# answer <state> <merge commit> <queue entry> <auto-merge> [<event>...]
#
# Writes the GraphQL response body for one pull request. `-` is `null`. The
# queue entry is `-` or the JSON that `entry` prints. Auto-merge is `auto`
# or `-`. Each event is `added` or `removed:<reason>`, oldest first; none at
# all is an empty `nodes`.
answer() {
    jq -n --arg state "$1" --arg oid "$2" --argjson entry "$(if [ "$3" = - ]; then echo null; else printf '%s' "$3"; fi)" --arg auto "$4" \
        --args '{data: {repository: {pullRequest: {
            state: $state,
            mergeCommit: (if $oid == "-" then null else {oid: $oid} end),
            mergeQueueEntry: $entry,
            autoMergeRequest: (if $auto == "-" then null else {enabledAt: "2026-10-01T11:00:00Z"} end),
            timelineItems: {nodes: [$ARGS.positional[4:][] | if . == "added" then {__typename: "AddedToMergeQueueEvent"} else {__typename: "RemovedFromMergeQueueEvent", reason: ltrimstr("removed:")} end]}
        }}}}' "$@" > "$body"
}

# entry <state> [<position> <group rollup> [<context>...]]
#
# Prints one queue entry. A context is `check:<completedAt>` for a check
# run, `status:<createdAt>` for a commit status, and `check:-` for a check
# run that has not completed. With no position the entry carries none, and
# with no rollup its head commit carries none.
entry() {
    jq -cn --arg state "$1" --arg pos "${2:--}" --arg rollup "${3:--}" \
        --args '{
            state: $state,
            position: (if $pos == "-" then null else ($pos | tonumber) end),
            enqueuedAt: "2026-10-01T09:00:00Z",
            headCommit: {statusCheckRollup: (if $rollup == "-" then null else {state: $rollup, contexts: {nodes: [$ARGS.positional[3:][] |
                if startswith("check:") then {__typename: "CheckRun", completedAt: (ltrimstr("check:") | if . == "-" then null else . end)}
                else {__typename: "StatusContext", createdAt: ltrimstr("status:")} end]}} end)}
        }' "$@"
}

run() {
    : > "$log"
    PATH="$fakebin:$PATH" LOG="$log" ANSWER="$body" QUEUE_DONE_NOW="$now" sh "$tool" "$@" > "$scratch/out" 2> "$scratch/err"
}

ok() { passed=$((passed + 1)); echo "  ok    $1"; }
bad() { failed=$((failed + 1)); echo "  FAIL  $1"; echo "        out: $(cat "$scratch/out")"; echo "        err: $(cat "$scratch/err")"; }

echo "a pull request in the queue is not finished"
answer OPEN - "$(entry AWAITING_CHECKS 2 PENDING check:- check:2026-10-01T11:00:00Z)" - added
run 42; status=$?
if [ "$status" -eq 1 ] && [ ! -s "$scratch/out" ] && grep -q 'in the queue, AWAITING_CHECKS' "$scratch/err"; then ok "queued: exit 1, nothing on stdout"; else bad "queued (exit $status)"; fi

echo "a pull request with auto-merge set is not finished"
answer OPEN - - auto
run 42; status=$?
if [ "$status" -eq 1 ] && grep -q 'waits on its own checks' "$scratch/err"; then ok "auto-merge: exit 1"; else bad "auto-merge (exit $status)"; fi

echo "a merged pull request is finished and names its commit"
answer MERGED abc123 - - added removed:merged
run 42; status=$?
if [ "$status" -eq 0 ] && grep -qx 'queue-done: #42 merged abc123' "$scratch/out"; then ok "merged: exit 0 with the sha"; else bad "merged (exit $status)"; fi

echo "an ejected pull request is finished and says why"
answer OPEN - - - added 'removed:The merge queue group failed: Engine tests'
run 42; status=$?
if [ "$status" -eq 0 ] && grep -qx 'queue-done: #42 ejected: The merge queue group failed: Engine tests' "$scratch/out"; then ok "ejected: exit 0 with the reason"; else bad "ejected (exit $status)"; fi

echo "a reason with a tab or a newline in it stays one field"
answer OPEN - - - added "removed:group failed:	Lint
and more"
run 42; status=$?
if [ "$status" -eq 0 ] && grep -qx 'queue-done: #42 ejected: group failed: Lint and more' "$scratch/out"; then ok "a tab and a newline in the reason become spaces"; else bad "tab in reason (exit $status)"; fi

echo "an open pull request never queued is finished and is not called ejected"
answer OPEN - - -
run 42; status=$?
if [ "$status" -eq 0 ] && grep -q 'not queued' "$scratch/out" && ! grep -q ejected "$scratch/out"; then ok "never queued: exit 0, not queued"; else bad "never queued (exit $status)"; fi

echo "a removal the queue made because it merged is not an ejection (#1353)"
answer OPEN - - - added removed:merged
run 42; status=$?
if [ "$status" -eq 1 ] && [ ! -s "$scratch/out" ] && grep -q 'merge is not recorded yet' "$scratch/err"; then ok "#1353: exit 1, nothing on stdout, never ejected"; else bad "#1353 removed as merged (exit $status)"; fi

echo "the merged reason is read in any case"
answer OPEN - - - removed:MERGED
run 42; status=$?
if [ "$status" -eq 1 ] && [ ! -s "$scratch/out" ]; then ok "MERGED reason: exit 1, never ejected"; else bad "MERGED reason (exit $status)"; fi

# Two nodes, oldest first: the filter must read the last one.
echo "an add with no removal visible yet is not finished, and never not queued (#1412)"
answer OPEN - - - removed:failed_checks added
run 42; status=$?
if [ "$status" -eq 1 ] && [ ! -s "$scratch/out" ] && grep -q 'removal is not visible yet' "$scratch/err"; then ok "#1412: exit 1, nothing on stdout"; else bad "#1412 added, no entry (exit $status)"; fi

echo "a failed_checks removal after an earlier add is an ejection"
answer OPEN - - - added removed:failed_checks
run 42; status=$?
if [ "$status" -eq 0 ] && grep -qx 'queue-done: #42 ejected: failed_checks' "$scratch/out"; then ok "ejected: failed_checks"; else bad "failed_checks (exit $status)"; fi

echo "a removal with no reason is an ejection with reason -"
jq -n '{data: {repository: {pullRequest: {state: "OPEN", mergeCommit: null, mergeQueueEntry: null, autoMergeRequest: null, timelineItems: {nodes: [{__typename: "RemovedFromMergeQueueEvent", reason: null}]}}}}}' > "$body"
run 42; status=$?
if [ "$status" -eq 0 ] && grep -qx 'queue-done: #42 ejected: -' "$scratch/out"; then ok "a null reason reads as -"; else bad "null reason (exit $status)"; fi

echo "a reason that holds a colon is kept whole"
answer OPEN - - - added 'removed:group failed: Lint'
run 42; status=$?
if [ "$status" -eq 0 ] && grep -qx 'queue-done: #42 ejected: group failed: Lint' "$scratch/out"; then ok "a colon in the reason survives"; else bad "colon reason (exit $status)"; fi

echo "an unmergeable entry is finished at once, before the queue ejects it (#1328)"
answer OPEN - "$(entry UNMERGEABLE 1)" - added
run 42; status=$?
if [ "$status" -eq 0 ] && grep -qx 'queue-done: #42 unmergeable: the queue will eject it with merge_conflict' "$scratch/out"; then ok "#1328: exit 0 with the unmergeable line"; else bad "#1328 unmergeable (exit $status)"; fi

echo "a queue entry in another state still holds the wait, whatever the timeline says"
answer OPEN - "$(entry MERGEABLE 1 SUCCESS check:2026-10-01T09:00:00Z)" - added removed:failed_checks
run 42; status=$?
if [ "$status" -eq 1 ] && [ ! -s "$scratch/out" ] && grep -q 'in the queue, MERGEABLE' "$scratch/err"; then ok "requeued after an ejection: exit 1"; else bad "requeued (exit $status)"; fi

# #1548: the group of the entry at the head of the queue was green, and the
# entry stayed in AWAITING_CHECKS for 84 minutes or more until the
# integrator's deadline. The last check completed at 10:36, and the clock
# reads 12:00.
echo "a green group at the head of the queue that has not merged for 84 minutes is stalled (#1548)"
answer OPEN - "$(entry AWAITING_CHECKS 1 SUCCESS check:2026-10-01T10:20:00Z check:2026-10-01T10:36:00Z status:2026-10-01T10:05:00Z)" - added
run 42; status=$?
if [ "$status" -eq 0 ] && grep -qx 'queue-done: #42 stalled: every check of its merge group is green and it has not merged in 84 minutes' "$scratch/out"; then ok "#1548: exit 0 with the stalled line"; else bad "#1548 stalled (exit $status)"; fi

echo "a group that went green a minute ago is not stalled"
answer OPEN - "$(entry AWAITING_CHECKS 1 SUCCESS check:2026-10-01T10:20:00Z check:2026-10-01T11:59:00Z)" - added
run 42; status=$?
if [ "$status" -eq 1 ] && [ ! -s "$scratch/out" ] && grep -q 'in the queue, AWAITING_CHECKS' "$scratch/err"; then ok "fresh green: exit 1, nothing on stdout"; else bad "fresh green (exit $status)"; fi

echo "a green group behind another entry is not stalled"
answer OPEN - "$(entry AWAITING_CHECKS 2 SUCCESS check:2026-10-01T10:36:00Z)" - added
run 42; status=$?
if [ "$status" -eq 1 ] && [ ! -s "$scratch/out" ]; then ok "position 2: exit 1"; else bad "position 2 (exit $status)"; fi

echo "a group with a check still pending is not stalled, however old its last completion"
answer OPEN - "$(entry AWAITING_CHECKS 1 PENDING check:2026-10-01T10:36:00Z check:-)" - added
run 42; status=$?
if [ "$status" -eq 1 ] && [ ! -s "$scratch/out" ]; then ok "pending rollup: exit 1"; else bad "pending rollup (exit $status)"; fi

echo "the stall interval is fifteen minutes, counted from the latest completion"
answer OPEN - "$(entry AWAITING_CHECKS 1 SUCCESS check:2026-10-01T11:00:00Z status:2026-10-01T11:45:01Z)" - added
run 42; s1=$?
answer OPEN - "$(entry AWAITING_CHECKS 1 SUCCESS check:2026-10-01T11:00:00Z status:2026-10-01T11:45:00Z)" - added
run 42; s2=$?
if [ "$s1" -eq 1 ] && [ "$s2" -eq 0 ] && grep -q 'in 15 minutes' "$scratch/out"; then ok "14:59 waits, 15:00 is stalled, and a commit status counts as a completion"; else bad "interval (exit $s1, $s2)"; fi

echo "a green group with no completion time is not stalled"
answer OPEN - "$(entry AWAITING_CHECKS 1 SUCCESS)" - added
run 42; status=$?
if [ "$status" -eq 1 ] && [ ! -s "$scratch/out" ]; then ok "no completion: exit 1"; else bad "no completion (exit $status)"; fi

echo "an event this script does not know is asked again"
jq -n '{data: {repository: {pullRequest: {state: "OPEN", mergeCommit: null, mergeQueueEntry: null, autoMergeRequest: null, timelineItems: {nodes: [{__typename: "ReopenedEvent"}]}}}}}' > "$body"
run 42; status=$?
if [ "$status" -eq 1 ] && [ ! -s "$scratch/out" ] && grep -q "read 'ReopenedEvent'" "$scratch/err"; then ok "unknown event: exit 1"; else bad "unknown event (exit $status)"; fi

echo "a closed pull request is finished"
answer CLOSED - - - added 'removed:The merge queue group failed: Lint'
run 42; status=$?
if [ "$status" -eq 0 ] && grep -qx 'queue-done: #42 closed' "$scratch/out"; then ok "closed: exit 0"; else bad "closed (exit $status)"; fi

echo "a failed call is asked again, and never read as an ejection"
rm -f "$body"
run 42; status=$?
if [ "$status" -eq 1 ] && [ ! -s "$scratch/out" ] && grep -q 'query failed' "$scratch/err"; then ok "gh error: exit 1, nothing on stdout"; else bad "gh error (exit $status)"; fi

echo "an answer that is not a pull request is asked again"
echo '{"data": {"repository": {"pullRequest": null}}}' > "$body"
run 42; status=$?
if [ "$status" -eq 1 ] && [ ! -s "$scratch/out" ]; then ok "null pull request: exit 1"; else bad "null pull request (exit $status)"; fi

echo "the number is the pull request's, passed as an integer"
answer MERGED abc123 - -
run 42
if grep -q 'graphql' "$log" && grep -q 'n=42' "$log"; then ok "gh api graphql with n=42"; else bad "arguments: $(cat "$log")"; fi

# The body of a case is the answer to any query, so no case can see what
# the query asks for. The last event is the latest one only when the query
# asks for the last of both event types; `first:1` or removals alone would
# read a stale event. The stall reads the entry's position and the rollup
# and completion times of its group's head commit.
echo "the query asks for the last added or removed queue event, and for the group's checks"
if grep -qF 'timelineItems(itemTypes:[ADDED_TO_MERGE_QUEUE_EVENT,REMOVED_FROM_MERGE_QUEUE_EVENT],last:1)' "$log" && grep -qF '__typename' "$log" \
    && grep -qF 'mergeQueueEntry{state position headCommit{statusCheckRollup{state contexts(last:100){nodes{... on CheckRun{completedAt} ... on StatusContext{createdAt}}}}}}' "$log"; then
    ok "both event types, last:1, with __typename, and the entry's position and group checks"
else
    bad "query: $(cat "$log")"
fi

echo "a number that is not a number is refused before gh runs"
run '42; rm -rf /'; status=$?
if [ "$status" -eq 64 ] && [ ! -s "$log" ]; then ok "refused: exit 64, gh never ran"; else bad "non-number (exit $status)"; fi

echo
echo "queue-done: $passed passed, $failed failed"
[ "$failed" -eq 0 ]
