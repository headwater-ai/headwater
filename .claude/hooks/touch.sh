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
# crossed the line at all is the whole fact `review.sh` reads back. Nothing
# ever removes it; a session id is never reused, so a marker a finished
# session left behind names nobody and costs one empty file.
#
# It fails open like the other three hooks, silently: no session id, no
# common git dir, or a directory it cannot create, and the tool call proceeds
# having stamped nothing. `review.sh` then keeps gating that session on every
# Stop, which is the same posture as before this hook existed.

. "$(dirname "$0")/lib.sh"

input=$(cat)
session=$(hw_field "$input" session_id) || exit 0
[ -n "$session" ] || exit 0

common=$(hw_common_dir) || exit 0
dir="$common/headwater-session/$session"
[ -e "$dir/touched" ] && exit 0
mkdir -p "$dir" 2>/dev/null && : > "$dir/touched" 2>/dev/null
exit 0
