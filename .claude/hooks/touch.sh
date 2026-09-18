#!/bin/sh
# The mark a session leaves so the review position can tell whether it could
# have caused whatever the tree already fails.
#
# Registered on `PreToolUse` for the tools that write a file: `Write`, `Edit`
# on Claude Code and GitHub Copilot, `apply_patch` on Codex. Not `Bash`. A
# session's `Bash` calls are far more often a `git log` or a `cat` than a
# mutation, and marking every one of them touched would defeat the case this
# exists for: a session that reads and runs a few inspection commands and
# edits nothing. A `Bash` call that does write a file leaves this session
# unmarked, and that is an accepted gap rather than an oversight — see
# `review.sh` for what holds the tree regardless.
#
# What it writes: an empty file at
# `<git common dir>/headwater-session/<session id>/touched`. Never the
# session's content, never the path a tool named — a mark that this session
# crossed the line at all is the whole fact `review.sh` reads back.
#
# It fails open like the other three hooks, silently: no session id, no
# common git dir, or a directory it cannot create, and the tool call proceeds
# having stamped nothing. `review.sh` then keeps gating that session on every
# Stop, which is the same posture as before this hook existed.
#
#     sh .claude/hooks/touch.sh collect   remove the marker of every session that has ended
#
# `collect` is not a hook position; the harness never calls it, and nothing
# else does automatically either — it is a tool a maintainer or a sweep runs
# by hand, the same posture `tools/run/run-dir.sh end` takes for a run's
# claims. A session id is never reused, so a marker a live session left
# behind names nobody once that session is gone, and #926 is the finding that
# nothing collected it.
#
# The fixture the harness does not supply is a clock: nothing here can read
# wall-clock time and call a marker old enough. What the harness's own
# bookkeeping supplies instead is `state.json` under a job directory
# (`$CLAUDE_JOB_DIR`'s parent, `HEADWATER_JOBS_ROOT` for a fixture): a job
# still running has no `state.json` at all, and one the harness has finished
# carries a `state` field — `done`, `failed` or `stopped` here read as ended,
# `blocked` does not, because a blocked job is paused rather than gone and may
# still resume without a new session. A session id `collect` cannot find in
# any job's `sessionId` is read the same as a live one: this only ever
# collects a marker it has positive evidence for, and the direction it is
# allowed to be wrong in is keeping a marker too long, never dropping one
# still in use.
#
# A session id is not proven to name one job directory alone, and this reads
# every `state.json` that names it rather than the first: an early return on
# the first match let one ended job's `done` answer for a second, later
# sorting job that had not finished, which is exactly the direction this is
# not allowed to be wrong in. `session_ended` below is ended only when every
# job it finds for that session id agrees; one non-terminal or unreadable
# match anywhere in the scan keeps the marker.
#
# What this still cannot see: a job that has never written `state.json` at
# all carries no on-disk record of its session id — read directly off this
# host, a running job's directory holds nothing else. A live job sharing a
# session id with an already-`done` job is therefore indistinguishable from
# "no other job for this session exists", because both produce zero matches
# beyond the `done` one. Nothing in this repository can close that gap from
# the outside; it would take the harness recording a live job's session id
# somewhere this can read. Recorded rather than assumed away.
. "$(dirname "$0")/lib.sh"

session_ended() {
    _session=$1 _jobs=$2
    _found=0
    for _state in "$_jobs"/*/state.json; do
        [ -f "$_state" ] || continue
        _json=$(cat "$_state") || continue
        _sid=$(hw_field "$_json" sessionId) || continue
        [ "$_sid" = "$_session" ] || continue
        _st=$(hw_field "$_json" state) || return 1
        case $_st in
            done | failed | stopped) _found=1 ;;
            *) return 1 ;;
        esac
    done
    [ "$_found" -eq 1 ]
}

collect() {
    _common=$(hw_common_dir) || return 0
    _sessions="$_common/headwater-session"
    [ -d "$_sessions" ] || return 0
    _jobs=${HEADWATER_JOBS_ROOT:-"$HOME/.claude/jobs"}
    if [ ! -d "$_jobs" ]; then
        echo "touch.sh collect: no jobs root at $_jobs; nothing to compare a marker against, so nothing is collected." >&2
        return 0
    fi
    _dropped=0 _kept=0
    for _dir in "$_sessions"/*/; do
        [ -d "$_dir" ] || continue
        _session=$(basename "$_dir")
        if session_ended "$_session" "$_jobs"; then
            rm -rf "$_dir"
            printf 'COLLECTED: %s\n' "$_session"
            _dropped=$((_dropped + 1))
        else
            _kept=$((_kept + 1))
        fi
    done
    printf 'collected %s, kept %s\n' "$_dropped" "$_kept"
}

if [ -n "${1:-}" ]; then
    if [ "$1" = 'collect' ]; then
        collect
        exit 0
    fi
    echo "touch.sh: \`$1\` is not a verb. The one CLI verb is \`collect\`; with no argument this reads a hook payload from standard input." >&2
    exit 2
fi

input=$(cat)
session=$(hw_field "$input" session_id) || exit 0
[ -n "$session" ] || exit 0

common=$(hw_common_dir) || exit 0
dir="$common/headwater-session/$session"
[ -e "$dir/touched" ] && exit 0
mkdir -p "$dir" 2>/dev/null && : > "$dir/touched" 2>/dev/null
exit 0
