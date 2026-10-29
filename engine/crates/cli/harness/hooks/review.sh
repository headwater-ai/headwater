#!/bin/sh
# The review-time hook: `headwater check --strict`, run at the end of a turn
# and held to the files the session wrote. Registered on `Stop`, which takes
# no matcher.
#
# It reads the paths the touch hook recorded for this session. A session that
# wrote nothing ends its turn with no check, because it could not have caused
# what the tree already fails. Otherwise it runs `headwater check --strict`,
# keeps the error findings on the paths the session wrote, and prints them.
# An error on a file that the session did not write is not this session's to
# fix, and the hook leaves it to the commit and the pull request.
#
# What a refusal means: the harness does not let the turn end, and the agent
# reads the findings. Claude Code and Codex block on exit 2 with the report on
# standard error. The Copilot CLI ends the turn on exit 2, and blocks on a
# JSON object with a block and a reason, on standard output at exit 0.
# `COPILOT_CLI` tells the two apart. When the harness ignores the hook, the
# turn ends, and the commit and the pull request still hold the tree.
#
# A repository that defines `hw_local_review` in `lib.local.sh` replaces the
# check below with its own.

. "${0%/*}/lib.sh"

input=$(cat)

# The harness sets this once it has blocked a stop, and a second block of the
# same turn is a loop rather than a gate. The binary reads it, so with no
# binary the hook cannot decide and ends the turn.
hw_engine >/dev/null || exit 0
active=$(hw_field "$input" stop_hook_active)
[ "$active" = "true" ] && exit 0

if command -v hw_local_review >/dev/null 2>&1; then
    hw_local_review
    exit $?
fi

session=$(hw_field "$input" session_id) || exit 0
[ -n "$session" ] || exit 0
hw_root=$(hw_resolve_root "$input")
touched=$(hw_touched_file "$session") || exit 0
[ -s "$touched" ] || exit 0

engine=$(hw_engine) || exit 0
report=$("$engine" check --strict --no-color --root "$hw_root" 2>/dev/null) && exit 0

# The blocks of the report under a head line `  <path> ✗ error` or
# `  <path>:<line>:<column> ✗ error`, for each path the session wrote. A block
# that the report marks `not reported` does not fail the check, so it is not
# printed.
findings=$(printf '%s\n' "$report" | awk -v list="$touched" '
    BEGIN { while ((getline line < list) > 0) wrote[line] = 1 }
    function flush() { if (block != "" && !held) out = out block; block = ""; held = 0; on = 0 }
    /^  [^ ]/ {
        flush()
        if ($0 ~ / ✗ error$/) {
            path = substr($0, 3)
            sub(/ ✗ error$/, "", path)
            sub(/:[0-9]+:[0-9]+$/, "", path)
            if (path in wrote) { on = 1; block = $0 "\n" }
        }
        next
    }
    on && /^    / {
        block = block $0 "\n"
        if ($0 ~ /^    not reported \(/) held = 1
        next
    }
    { flush() }
    END { flush(); printf "%s", out }')
[ -n "$findings" ] || exit 0

message="Headwater: \`headwater check --strict\` reports an error on a file this session wrote. Fix it before the turn ends.

$findings"

if [ -n "${COPILOT_CLI:-}" ]; then
    quoted=$(hw_quote "$message") || exit 0
    printf '{"decision":"block","reason":%s}\n' "$quoted"
    exit 0
fi

printf '%s\n' "$message" >&2
exit 2
