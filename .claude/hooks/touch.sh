#!/bin/sh
# The record of what a session wrote, which the review hook reads at the end
# of a turn.
#
# Registered on `PreToolUse` for the tools that write a file: `Write` and
# `Edit` on Claude Code and Copilot, and `apply_patch` on Codex. Not on a
# shell command: most shell commands read, and a session that only reads must
# not be held to the state of a tree it did not change. A shell command that
# does write a file leaves no record, and the commit holds that file.
#
# What it writes: the path the tool names, relative to the repository, one
# line per path and each path once, in
# `<git common dir>/headwater-session/<session id>/touched`. A path outside
# the repository creates the file and adds no line.
#
# It fails open, silently: with no session id, no git common dir, or a
# directory it cannot create, the tool call goes on and nothing is recorded.
#
#     sh .claude/hooks/touch.sh collect   remove the record of every session that has ended
#
# `collect` is not a hook position, and nothing runs it automatically. A
# session id is never used again, so the record of an ended session names
# nobody. It reads the `state.json` of each harness job under
# `HEADWATER_JOBS_ROOT`, or `~/.claude/jobs`, and removes a record only when
# every job of that session id is `done`, `failed` or `stopped`. A session it
# cannot find is kept, so it is wrong only in the direction of keeping a
# record too long.

. "${0%/*}/lib.sh"

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
        _session=${_dir%/}
        _session=${_session##*/}
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
path=$(hw_field "$input" tool_input file_path) \
    || path=$(hw_field "$input" tool_input path) \
    || path=$(hw_patch_path "$input") \
    || path=

hw_root=$(hw_resolve_root "$input")
file=$(hw_touched_file "$session") || exit 0
mkdir -p "${file%/*}" 2>/dev/null || exit 0
{ : >> "$file"; } 2>/dev/null || exit 0

case $path in
    "") exit 0 ;;
    "$hw_root"/*) rel=${path#"$hw_root"/} ;;
    /*) exit 0 ;;
    *) rel=$path ;;
esac
grep -Fqx -- "$rel" "$file" 2>/dev/null && exit 0
{ printf '%s\n' "$rel" >> "$file"; } 2>/dev/null
exit 0
# installed by headwater init --harness, digest sha256:2b6e4a5a8556f7e1b14dd98ed3d6033ea8e691dcbcab27b86fa555254eb276f7
