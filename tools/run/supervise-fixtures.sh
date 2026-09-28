#!/bin/sh
# What holds `tools/run/supervise.sh`, the loop that restarts a build-order
# parent every few merges (#1275).
#
# A stub `claude` stands in for the model. Each session it runs the script
# `<stub>/<n>.sh` if there is one, which prints canned stream-json calls and
# appends merges to the run's log, and it records its arguments, its pid and
# whether `drain` existed when it started. Every stop condition is provoked
# once, the drain is provoked on each of its two triggers, and a signal to the
# loop is held to leave no session behind.
#
# Run it from anywhere:
#     sh tools/run/supervise-fixtures.sh
#
# It needs `jq` and `mkfifo`, and it writes only under a temporary directory.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
tool="$root/tools/run/supervise.sh"

if ! command -v jq >/dev/null 2>&1; then
    echo "no \`jq\` on the path, and the tool under test reads JSON with it." >&2
    exit 3
fi

scratch=$(mktemp -d) || exit 1
trap 'rm -rf "$scratch"' EXIT HUP INT TERM
export HEADWATER_RUN_ROOT="$scratch/runs"
export HW_SUPERVISE_TREE="$scratch/tree"
export HW_SUPERVISE_CLAUDE="$scratch/bin/claude"
mkdir -p "$HEADWATER_RUN_ROOT" "$HW_SUPERVISE_TREE" "$scratch/bin"

passed=0
failed=0
pass() { printf 'ok   %s\n' "$1"; passed=$((passed + 1)); }
fail() { printf 'FAIL %s\n  %s\n' "$1" "$2"; failed=$((failed + 1)); }
same() {
    if [ "$2" = "$3" ]; then pass "$1"; else fail "$1" "expected \`$2\`, got \`$3\`"; fi
}

cat > "$HW_SUPERVISE_CLAUDE" <<'EOF'
#!/bin/sh
n=$(( $(cat "$STUB/count" 2>/dev/null || echo 0) + 1 ))
echo "$n" > "$STUB/count"
printf '%s\n' "$*" > "$STUB/args-$n"
if [ -e "$RUN/drain" ]; then echo yes; else echo no; fi > "$STUB/drain-at-start-$n"
echo "$$" > "$STUB/pid-$n"
call() { printf '{"type":"assistant","message":{"id":"m%s","usage":{"input_tokens":0,"cache_read_input_tokens":%s}}}\n' "$1" "$2"; }
merge() { printf '{"iter":1,"issue":%s,"pr":1,"merge":"abc1234","verdict":"MERGE","proved":"p","opened":[],"closed":[]}\n' "$1" >> "$RUN/log.jsonl"; }
[ -f "$STUB/$n.sh" ] && . "$STUB/$n.sh"
printf '{"type":"result","subtype":"success"}\n'
exit 0
EOF
chmod u+x "$HW_SUPERVISE_CLAUDE"

# fresh <run-id> [queue line]: a run directory and a stub directory for it.
fresh() {
    RUN="$HEADWATER_RUN_ROOT/$1" STUB="$scratch/stub-$1"
    mkdir -p "$RUN" "$STUB"
    : > "$RUN/log.jsonl"
    if [ $# -gt 1 ]; then
        printf '# Queue\n\n%s\n' "$2" > "$RUN/queue.md"
    else
        printf '# Queue\n\n1. #10 One | none | x\n2. #11 Two | none | x\n3. #12 Three | none | x\n' > "$RUN/queue.md"
    fi
    export RUN STUB
}

printf '# a call past the threshold starts the drain, and the next session starts clean\n'
fresh ctx
printf 'call 1 60000\ncall 2 140000\nmerge 10\n' > "$STUB/1.sh"
printf 'call 3 50000\n' > "$STUB/2.sh"
out=$(sh "$tool" ctx 2>&1); status=$?
same 'the loop exits 0 when it stops' 0 "$status"
same '  and the call past 130000 drains session 1' 1 "$(printf '%s\n' "$out" | grep -c '^DRAIN: session 1, a call at context 140000$')"
same '  and a second session starts once the first exits' 2 "$(cat "$STUB/count")"
same '  and drain is removed before it starts' no "$(cat "$STUB/drain-at-start-2")"
same '  and each session is the resume form of next-run' '-p /next-run --resume ctx --output-format stream-json --verbose' "$(cat "$STUB/args-2")"
same '  and a session that exits without a drain stops the loop, and says why' \
    'STOPPED: session, session 2 exited without a drain' "$(printf '%s\n' "$out" | grep '^STOPPED')"
same '  and the context line is read over both sessions' \
    'context  transcripts 2  calls 3  mean 83333  max 140000' "$(printf '%s\n' "$out" | grep '^context')"
same '  and no stream pipe is left in the run' 0 "$(ls -A "$RUN/supervise" | grep -c '^\.stream')"

printf '\n# K merges start the drain, and N merges stop the loop\n'
fresh kay
printf 'merge 10\nmerge 11\ncall 1 1000\n' > "$STUB/1.sh"
out=$(HW_SUPERVISE_K=2 sh "$tool" kay 2 2>&1)
same 'K merges drain the session' 1 "$(printf '%s\n' "$out" | grep -c '^DRAIN: session 1, 2 merges$')"
same '  and N merges stop the loop before another session' 'STOPPED: merges, 2 of 2 reached' "$(printf '%s\n' "$out" | grep '^STOPPED')"
same '  and one session ran' 1 "$(cat "$STUB/count")"

fresh late
printf 'call 1 1000\nmerge 10\nmerge 11\n' > "$STUB/1.sh"
printf 'call 2 1000\n' > "$STUB/2.sh"
out=$(HW_SUPERVISE_K=2 sh "$tool" late 2>&1)
same 'merges made after the last call still count as a drain at the end of the session' 2 "$(cat "$STUB/count")"

printf '\n# two sessions in a row that merge nothing\n'
fresh idle
printf 'call 1 140000\n' > "$STUB/1.sh"
printf 'call 2 140000\n' > "$STUB/2.sh"
printf 'call 3 140000\n' > "$STUB/3.sh"
out=$(sh "$tool" idle 2>&1)
same 'stop the loop' 'STOPPED: idle, two sessions in a row merged nothing' "$(printf '%s\n' "$out" | grep '^STOPPED')"
same '  after the second' 2 "$(cat "$STUB/count")"

printf '\n# the queue and handover/ hold nothing\n'
fresh empty ''
out=$(sh "$tool" empty 2>&1)
same 'the loop stops before any session' 'STOPPED: empty, the queue and handover/ hold nothing to dispatch' "$(printf '%s\n' "$out" | grep '^STOPPED')"
same '  and never calls claude' 0 "$(cat "$STUB/count" 2>/dev/null || echo 0)"
fresh inflight ''
mkdir -p "$RUN/handover"
printf 'stage built\n' > "$RUN/handover/10"
printf 'merge 10\n' > "$STUB/1.sh"
out=$(sh "$tool" inflight 2>&1)
same 'an unlogged handover is in flight, so a session runs' 1 "$(cat "$STUB/count" 2>/dev/null || echo 0)"

printf '\n# the 5-hour window\n'
fresh usage
printf '{"five_hour":{"used_percentage":95.2}}\n' > "$RUN/usage.jsonl"
out=$(sh "$tool" usage 2>&1)
same 'at 95% the loop stops before any session' 'STOPPED: five-hour, the window is at 95% and the limit is 90%' "$(printf '%s\n' "$out" | grep '^STOPPED')"
same '  and never calls claude' 0 "$(cat "$STUB/count" 2>/dev/null || echo 0)"

printf '\n# a session that fails\n'
fresh broken
printf 'exit 3\n' > "$STUB/1.sh"
out=$(sh "$tool" broken 2>&1)
same 'stops the loop and names the status' 'STOPPED: claude, session 1 exited 3' "$(printf '%s\n' "$out" | grep '^STOPPED')"

printf '\n# refusals\n'
sh "$tool" no-such-run >/dev/null 2>&1; status=$?
same 'a run directory that does not exist is refused with exit 2' 2 "$status"
fresh main
git init -q "$scratch/main-checkout" 2>/dev/null
HW_SUPERVISE_TREE="$scratch/main-checkout" sh "$tool" main >/dev/null 2>"$scratch/err"; status=$?
same 'a main checkout is refused with exit 2' 2 "$status"
same '  and never calls claude' 0 "$(cat "$STUB/count" 2>/dev/null || echo 0)"
sh "$tool" main x >/dev/null 2>&1; status=$?
same 'an N that is not a number is refused with exit 2' 2 "$status"

printf '\n# a signal to the loop leaves no session behind\n'
fresh signal
printf 'call 1 1000\nexec sleep 60\n' > "$STUB/1.sh"
sh "$tool" signal > "$scratch/signal.out" 2>&1 &
loop=$!
tries=0
while [ ! -s "$STUB/pid-1" ] && [ "$tries" -lt 50 ]; do
    sleep 0.1
    tries=$((tries + 1))
done
stub=$(cat "$STUB/pid-1" 2>/dev/null)
sleep 0.3
kill -TERM "$loop"
wait "$loop"; status=$?
same 'the loop exits on TERM' 130 "$status"
sleep 0.2
if [ -n "$stub" ] && kill -0 "$stub" 2>/dev/null; then
    fail '  and the session it started is gone' "pid $stub is alive"
    kill "$stub" 2>/dev/null
else
    pass '  and the session it started is gone'
fi
same '  and no stream pipe is left in the run' 0 "$(ls -A "$RUN/supervise" | grep -c '^\.stream')"

printf '\n%s passed, %s failed\n' "$passed" "$failed"
[ "$failed" -eq 0 ]
