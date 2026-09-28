#!/bin/sh
# Restart the build-order parent every few merges, so its context stays small.
#
# A parent that runs a whole build order in one session pays for its own
# length on every call. Run `20260927-0443` (session `b5554ef1`, #1275) spent
# 245M context tokens over 1,368 parent calls, a mean of 179k per call: the
# first call was 52k, and the context grew about 1.2k per call between seven
# compactions made by hand. The bar #1275 sets is a mean under 120k per call,
# counted over every parent session of a run.
#
#     sh tools/run/supervise.sh <run-id> [N]
#
# This is the loop that holds a run to that bar. It runs one parent session at
# a time as `claude -p "/next-run --resume <run-id>"` with stream-json output,
# reads the context of each call from the stream at no cost to the parent, and
# creates `<run>/drain` when a call passes the threshold or the session has
# made K merges. `tools/run/run-dir.sh` prints `DRAIN` on the `claim`, `stage`
# and `next` calls the parent already makes, the parent starts no new stage,
# and it exits once nothing is in flight. This script then removes `drain` and
# starts the next session, which resumes from `handover/`. A subagent runs
# inside its parent's process and ends with it, so a drain to zero before the
# exit is the only way a restart loses nothing (the owner's ruling 1 on #1275).
#
# The first session of a run stays interactive, because it puts the product
# owner's rulings to the owner. This script takes over from that run id.
#
# The defaults, and the measurement behind each (run 20260927-0443):
#
#     threshold 130000  HW_SUPERVISE_THRESHOLD, context of one call that starts
#                       the drain. A session starts near 60k and grows about
#                       1.2k per call; drained from 130k it peaks near 142k,
#                       with a mean near 100k, which is under the 120k bar.
#     K 4               HW_SUPERVISE_K, merges per session, a ceiling behind
#                       the threshold. About 30 parent calls per closed issue
#                       today; about 6 is right after #1274 and #1276.
#     five-hour 90      HW_SUPERVISE_FIVE_HOUR, the used percentage of the
#                       plan's 5-hour window in the last `usage.jsonl` sample
#                       at which no new session starts.
#     N                 the second argument, total merges for this loop; no
#                       cap when it is absent.
#
# The trade-off, measured: a drain to zero takes 25 min at the median and 35 at
# p75 (adjudicate 2.8/3.4 min, first build 27.8/37.9, rework build 15.6/24.1,
# verify 6.0/10.0, integrate 29.2/35.2, median/p75). A restart costs about 50k
# of one-hour cache write, paid back in about 20 calls at the smaller context.
# At 2.5 merges per hour and a restart every four merges, stop-and-drain loses
# an estimated 13-15% of throughput, which the owner accepted for the simpler
# design.
#
# It stops, and prints `STOPPED: <reason>`, on the first of these:
#
#     empty       `run-dir.sh next` prints EMPTY and every handover is logged
#     merges      N merges are reached
#     session     a session exited without a drain: it stopped on the stop
#                 condition of `.claude/commands/next-run.md`, or a human did
#     idle        two sessions in a row merged nothing
#     five-hour   the last usage sample is at or past HW_SUPERVISE_FIVE_HOUR
#     claude      the session exited with a status other than 0
#
# A merge is a line of `log.jsonl` whose `merge` is a commit sha, since the
# `verdict` field has held thirty spellings over the runs so far.
#
# Each session's stream is kept at `<run>/supervise/session-<n>.jsonl`, in the
# transcript's shape, so `sh tools/run/run-census.sh --context` reads the bar
# over all of them; the loop prints that line when it stops.
#
# Run it from its own worktree (`sh tools/repo/new-worktree.sh`), never the
# main checkout, which the integrator owns; HW_SUPERVISE_TREE names another
# directory for the sessions to run in, and a main checkout is refused.
# HW_SUPERVISE_CLAUDE names the binary (default `claude`), and
# HW_SUPERVISE_ARGS adds arguments after the fixed ones, such as a permission
# mode. `sh tools/run/supervise-fixtures.sh` holds every stop with a stub
# `claude`, so no fixture calls a model. The loop leaves no process behind: a
# signal to it ends the session it started.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
rundir_tool="$root/tools/run/run-dir.sh"

id=${1:-}
cap=${2:-}
[ -n "$id" ] || { echo "usage: sh tools/run/supervise.sh <run-id> [N]" >&2; exit 2; }
case $cap in ''|*[!0-9]*) [ -z "$cap" ] || { echo "supervise: N is a number of merges, got \`$cap\`." >&2; exit 2; } ;; esac
command -v jq >/dev/null 2>&1 || { echo "supervise: \`jq\` is not on the path." >&2; exit 3; }

threshold=${HW_SUPERVISE_THRESHOLD:-130000}
k=${HW_SUPERVISE_K:-4}
five_hour=${HW_SUPERVISE_FIVE_HOUR:-90}
claude=${HW_SUPERVISE_CLAUDE:-claude}
tree=${HW_SUPERVISE_TREE:-$root}

if [ -n "${HEADWATER_RUN_ROOT:-}" ]; then
    runs=$HEADWATER_RUN_ROOT
else
    common=$(git -C "$root" rev-parse --git-common-dir) || exit 2
    case $common in /*) ;; *) common="$root/$common" ;; esac
    runs="$common/headwater-run"
fi
run="$runs/$id"
[ -d "$run" ] || { echo "supervise: $run is not a run directory. The first session makes it with \`run-dir.sh start\`." >&2; exit 2; }

gitdir=$(git -C "$tree" rev-parse --absolute-git-dir 2>/dev/null)
gitcommon=$(cd "$tree" 2>/dev/null && cd "$(git rev-parse --git-common-dir 2>/dev/null)" 2>/dev/null && pwd)
if [ -n "$gitdir" ] && [ "$gitdir" = "$gitcommon" ]; then
    echo "supervise: $tree is a main checkout, which the integrator owns. Run from a worktree made by tools/repo/new-worktree.sh." >&2
    exit 2
fi

mkdir -p "$run/supervise"
fifo="$run/supervise/.stream.$$"
cpid=''

cleanup() {
    [ -n "$cpid" ] && kill "$cpid" 2>/dev/null
    rm -f "$fifo"
}
trap 'cleanup; exit 130' INT TERM HUP
trap cleanup EXIT

merges() {
    [ -s "$run/log.jsonl" ] || { echo 0; return; }
    jq -s '[.[] | select((.merge // "") | type == "string" and test("^[0-9a-f]{7,40}$"))] | length' "$run/log.jsonl" 2>/dev/null || echo 0
}

stop() {
    echo "STOPPED: $1"
    set -- "$run"/supervise/session-*.jsonl
    [ -f "$1" ] && sh "$root/tools/run/run-census.sh" --context "$@"
    exit 0
}

# Every handover issue is in the log, so nothing is in flight.
handovers_open() {
    [ -d "$run/handover" ] || return 1
    logged=' '
    [ -s "$run/log.jsonl" ] && logged=" $(jq -r '.issue // empty' "$run/log.jsonl" 2>/dev/null | tr '\n' ' ')"
    for h in "$run"/handover/*; do
        [ -f "$h" ] || continue
        case "$logged" in *" ${h##*/} "*) ;; *) return 0 ;; esac
    done
    return 1
}

start_merges=$(merges)
idle=0
n=0
while :; do
    total=$(( $(merges) - start_merges ))
    [ -n "$cap" ] && [ "$total" -ge "$cap" ] && stop "merges, $total of $cap reached"
    if [ -s "$run/usage.jsonl" ]; then
        used=$(tail -n 1 "$run/usage.jsonl" | jq -r '.five_hour.used_percentage // 0 | floor' 2>/dev/null)
        [ "${used:-0}" -ge "$five_hour" ] && stop "five-hour, the window is at ${used}% and the limit is ${five_hour}%"
    fi
    rm -f "$run/drain"
    if [ "$(sh "$rundir_tool" next "$run" 2>/dev/null)" = EMPTY ] && ! handovers_open; then
        stop "empty, the queue and handover/ hold nothing to dispatch"
    fi

    n=$((n + 1))
    log="$run/supervise/session-$n.jsonl"
    before=$(merges)
    echo "SESSION: $n"
    rm -f "$fifo"
    mkfifo "$fifo" || exit 1
    # shellcheck disable=SC2086
    (cd "$tree" && exec "$claude" -p "/next-run --resume $id" --output-format stream-json --verbose ${HW_SUPERVISE_ARGS:-}) > "$fifo" &
    cpid=$!
    # One reader for the whole stream: keep it, and turn each call into its
    # context. The loop sees the merge count after each call too. The reader
    # runs in the background and the loop blocks in `wait`, because a shell
    # runs a trap only once its foreground command returns, and a foreground
    # pipeline would hold a signal until the session ended by itself.
    tee -a "$log" < "$fifo" \
        | jq --unbuffered -R -r 'try fromjson catch empty | select(type == "object" and .type == "assistant")
            | (.message.usage // {}) | (.input_tokens // 0) + (.cache_read_input_tokens // 0) + (.cache_creation_input_tokens // 0)' \
        | while read -r ctx; do
            [ -e "$run/drain" ] && continue
            if [ "$ctx" -gt "$threshold" ]; then
                printf 'context %s past %s\n' "$ctx" "$threshold" > "$run/drain"
                echo "DRAIN: session $n, a call at context $ctx"
            elif [ $(( $(merges) - before )) -ge "$k" ]; then
                printf 'merges %s\n' "$k" > "$run/drain"
                echo "DRAIN: session $n, $k merges"
            fi
        done &
    rpid=$!
    wait "$cpid"
    status=$?
    cpid=''
    wait "$rpid"
    rm -f "$fifo"
    made=$(( $(merges) - before ))
    echo "SESSION-END: $n, $made merges, exit $status"
    [ "$status" -eq 0 ] || stop "claude, session $n exited $status"
    if [ ! -e "$run/drain" ] && [ "$made" -ge "$k" ]; then
        printf 'merges %s\n' "$made" > "$run/drain"
    fi
    [ -e "$run/drain" ] || stop "session, session $n exited without a drain"
    if [ "$made" -eq 0 ]; then
        idle=$((idle + 1))
        [ "$idle" -ge 2 ] && stop "idle, two sessions in a row merged nothing"
    else
        idle=0
    fi
done
