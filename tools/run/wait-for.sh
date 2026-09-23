#!/bin/sh
# One bounded attempt at a wait that might outlive the five-minute
# prompt-cache lifetime, the shape [HW-PD-0007] rules for exactly this case.
#
# A wait started in the background and left running to completion, however
# long that takes, wakes a turn whose prompt cache the harness has by then
# discarded — past about five minutes of silence, that turn pays to rewrite
# its whole context rather than read it, at 1.25x the input rate against
# 0.1x. Session `9ab3be93` paid $14.85 doing this thirteen times in one run.
# The remedy is a wait capped under that lifetime that reports whether to
# stop or to run again, not a wait that loops past its own cap: a loop still
# running when the cap is reached is the harness's own foreground-to-
# background move, a different thing this script never does.
#
#     sh tools/run/wait-for.sh '<condition-command>'
#     sh tools/run/wait-for.sh --cap 240 --poll 30 '<condition-command>'
#
# Start it with run_in_background: true. It runs the condition, as a shell
# command string, every <poll> seconds (default 30, never go under ten: one
# agent checking a status file every few seconds burned 29% of a whole run)
# and exits 0 the moment the condition succeeds. If <cap> seconds (default
# 240, under the five-minute lifetime) pass first, it exits 2 and prints
# RE-ISSUE: this attempt has genuinely ended, so starting the identical call
# again is a new wait and not the re-ask that cost run `cc7cc6c6` 7.8 hours
# waiting on a call already finished. The caller decides whether to re-issue;
# this script never loops itself past its own cap.

set -u

cap=240
poll=30

while :; do
    case "${1:-}" in
        --cap) cap=$2; shift 2 ;;
        --poll) poll=$2; shift 2 ;;
        --) shift; break ;;
        *) break ;;
    esac
done

cond=${1:-}
if [ -z "$cond" ]; then
    echo "wait-for: needs a condition command." >&2
    echo "  sh tools/run/wait-for.sh [--cap N] [--poll N] '<condition-command>'" >&2
    exit 2
fi

case $cap in '' | *[!0-9]*) echo "wait-for: --cap wants a number of seconds, got \`$cap\`." >&2; exit 2 ;; esac
case $poll in '' | *[!0-9]*) echo "wait-for: --poll wants a number of seconds, got \`$poll\`." >&2; exit 2 ;; esac

if [ "$cap" -gt 280 ]; then
    echo "wait-for: --cap $cap is at or past the five-minute prompt-cache lifetime; 240 is the default for a reason." >&2
fi
if [ "$poll" -lt 10 ]; then
    echo "wait-for: --poll $poll is under the floor a run measured burning 29% of itself on; use 30 or more." >&2
fi

elapsed=0
while [ "$elapsed" -lt "$cap" ]; do
    if sh -c "$cond"; then
        echo "wait-for: condition met after ${elapsed}s."
        exit 0
    fi
    sleep "$poll"
    elapsed=$((elapsed + poll))
done

echo "wait-for: RE-ISSUE - condition not met after ${cap}s. This attempt has ended; start a fresh one the same way." >&2
exit 2
