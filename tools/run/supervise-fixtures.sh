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
# The loop calls `run-dir.sh next`, which judges the main checkout of its own
# repository by default, and the host's can be any number of commits behind
# (#1811). A scratch repository with no origin/main gives it nothing to judge.
git init -q "$scratch/checkout"
export HW_RUN_CHECKOUT="$scratch/checkout"
unset HW_RUN_ALLOW_STALE

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

# fresh <run-id> ['']: a run directory and a stub directory for it, with a
# queue of #10, #11 and #12, or an empty queue when a second argument is given.
fresh() {
    RUN="$HEADWATER_RUN_ROOT/$1" STUB="$scratch/stub-$1"
    mkdir -p "$RUN" "$STUB"
    : > "$RUN/log.jsonl"
    # The queue is written through run-dir.sh, as hw-queue writes it, because
    # `next` reads queue.jsonl and refuses a run that holds only a typed queue.md.
    : > "$RUN/queue.jsonl"
    if [ $# -le 1 ]; then
        for n in 10 11 12; do
            sh "$root/tools/run/run-dir.sh" queue "$RUN" "$n" "title=Issue $n" milestone=none reader=x artifacts=x scope=narrow >/dev/null
        done
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

printf '\n# the main checkout is fast-forwarded before each session, and only by a fast-forward\n'
# A bare origin, a main checkout cloned from it, and a worktree of that
# checkout as the tree the sessions run in. A second clone pushes a commit;
# the loop brings the main checkout to it before the first session starts.
ffdir="$scratch/ff"
mkdir -p "$ffdir"
git init -q --bare "$ffdir/origin.git"
git -C "$ffdir" clone -q "$ffdir/origin.git" main-checkout 2>/dev/null
git -C "$ffdir/main-checkout" -c user.name=t -c user.email=t@t commit -q --allow-empty -m first
git -C "$ffdir/main-checkout" branch -q -M main
git -C "$ffdir/main-checkout" push -q -u origin main 2>/dev/null
git -C "$ffdir/main-checkout" worktree add -q "$ffdir/wt" -b wt 2>/dev/null
git -C "$ffdir" clone -q -b main "$ffdir/origin.git" other 2>/dev/null
git -C "$ffdir/other" -c user.name=t -c user.email=t@t commit -q --allow-empty -m second
git -C "$ffdir/other" push -q origin HEAD:main 2>/dev/null
want=$(git -C "$ffdir/other" rev-parse --short HEAD)
fresh ff
printf 'call 1 140000\n' > "$STUB/1.sh"
printf 'call 2 140000\n' > "$STUB/2.sh"
out=$(HW_SUPERVISE_TREE="$ffdir/wt" sh "$tool" ff 2>&1)
same 'the loop prints the fast-forward before the first session' "FAST-FORWARD: main at $want" "$(printf '%s\n' "$out" | grep '^FAST-FORWARD' | head -n 1)"
same '  and the main checkout is at origin/main' "$want" "$(git -C "$ffdir/main-checkout" rev-parse --short HEAD)"
same '  and still on main' main "$(git -C "$ffdir/main-checkout" symbolic-ref --short HEAD)"
same '  and the worktree the sessions ran in did not move' wt "$(git -C "$ffdir/wt" symbolic-ref --short HEAD)"
same '  and the loop does not stop stale (#1811)' 0 "$(printf '%s\n' "$out" | grep -c '^STOPPED: stale')"
git -C "$ffdir/main-checkout" -c user.name=t -c user.email=t@t commit -q --allow-empty -m local-only
git -C "$ffdir/other" -c user.name=t -c user.email=t@t commit -q --allow-empty -m third
git -C "$ffdir/other" push -q origin HEAD:main 2>/dev/null
local_sha=$(git -C "$ffdir/main-checkout" rev-parse --short HEAD)
fresh ff2
printf 'call 1 140000\n' > "$STUB/1.sh"
printf 'call 2 140000\n' > "$STUB/2.sh"
out=$(HW_SUPERVISE_TREE="$ffdir/wt" sh "$tool" ff2 2>&1)
same 'a main checkout that has diverged is left where it is, and the loop says so' \
    "FAST-FORWARD: skipped, $ffdir/main-checkout does not fast-forward to origin/main" "$(printf '%s\n' "$out" | grep '^FAST-FORWARD' | head -n 1)"
same '  and its HEAD is untouched' "$local_sha" "$(git -C "$ffdir/main-checkout" rev-parse --short HEAD)"
same '  and no rebase or merge is in progress' 0 "$(ls "$ffdir/main-checkout/.git" | grep -cE '^(MERGE_HEAD|rebase-merge|rebase-apply)$')"
ff2_stale="STALE: main checkout at $local_sha, origin/main at $(git -C "$ffdir/other" rev-parse --short HEAD), 1 commits behind. Run: git -C $ffdir/main-checkout merge --ff-only origin/main"
same '  and the loop stops stale, with the STALE line after its prefix (#1811)' "STOPPED: stale, ${ff2_stale#STALE: }" "$(printf '%s\n' "$out" | grep '^STOPPED' | head -n 1)"
same '  and runs no session on it' 0 "$(cat "$STUB/count" 2>/dev/null || echo 0)"
git -C "$ffdir/main-checkout" reset -q --hard origin/main
git -C "$ffdir/other" -c user.name=t -c user.email=t@t commit -q --allow-empty -m fourth
git -C "$ffdir/other" push -q origin HEAD:main 2>/dev/null
git -C "$ffdir/main-checkout" fetch -q origin
: > "$ffdir/main-checkout/dirty"
fresh ffdirty
printf 'call 1 140000\n' > "$STUB/1.sh"
out=$(HW_SUPERVISE_TREE="$ffdir/wt" sh "$tool" ffdirty 2>&1)
same 'a main checkout that is dirty and behind is skipped by the fast-forward' "FAST-FORWARD: skipped, $ffdir/main-checkout is not clean" "$(printf '%s\n' "$out" | grep '^FAST-FORWARD' | head -n 1)"
same '  and the loop stops stale' 1 "$(printf '%s\n' "$out" | grep -c '^STOPPED: stale, main checkout at .*, 1 commits behind\. Run: git -C .* merge --ff-only origin/main$')"
same '  and calls claude 0 times' 0 "$(cat "$STUB/count" 2>/dev/null || echo 0)"
out=$(HW_SUPERVISE_FF=0 HW_SUPERVISE_TREE="$ffdir/wt" sh "$tool" ffdirty 2>&1)
same '  and with HW_SUPERVISE_FF=0 the check still runs and stops stale' 1 "$(printf '%s\n' "$out" | grep -c '^STOPPED: stale, ')"
# `next` prints a STALE line before EMPTY when the checkout it judges is
# behind, and the loop reads its last line, so an empty run still stops empty.
fresh ffempty ''
out=$(HW_RUN_CHECKOUT="$ffdir/main-checkout" sh "$tool" ffempty 2>&1)
same 'an empty queue stops empty when next prints a STALE line before EMPTY' 1 "$(printf '%s\n' "$out" | grep -c '^STOPPED: empty')"
rm -f "$ffdir/main-checkout/dirty"
git -C "$ffdir/main-checkout" merge -q --ff-only origin/main
: > "$ffdir/main-checkout/dirty"
fresh ff3
printf 'call 1 140000\n' > "$STUB/1.sh"
printf 'call 2 140000\n' > "$STUB/2.sh"
out=$(HW_SUPERVISE_TREE="$ffdir/wt" sh "$tool" ff3 2>&1)
same 'a main checkout that is not clean is skipped' "FAST-FORWARD: skipped, $ffdir/main-checkout is not clean" "$(printf '%s\n' "$out" | grep '^FAST-FORWARD' | head -n 1)"
rm -f "$ffdir/main-checkout/dirty"
fresh ff4
printf 'call 1 140000\n' > "$STUB/1.sh"
printf 'call 2 140000\n' > "$STUB/2.sh"
out=$(HW_SUPERVISE_FF=0 HW_SUPERVISE_TREE="$ffdir/wt" sh "$tool" ff4 2>&1)
same 'HW_SUPERVISE_FF=0 turns it off' 'FAST-FORWARD: skipped, HW_SUPERVISE_FF=0' "$(printf '%s\n' "$out" | grep '^FAST-FORWARD' | head -n 1)"
same 'a tree that is no worktree is skipped, and the loop runs' 1 "$(printf '%s\n' "$(sh "$tool" idle 2>&1)" | grep -c 'FAST-FORWARD: skipped, .* is not a worktree of a main checkout' | sed 's/^[1-9][0-9]*$/1/')"
same '  and it does not stop stale (#1811)' 0 "$(sh "$tool" idle 2>&1 | grep -c '^STOPPED: stale')"

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
