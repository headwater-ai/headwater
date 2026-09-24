#!/bin/sh
# The read-time position. A document may declare that it governs a code path
# (HW-DR-0074), and a session that opens that path hears the documents before
# it reads a line of it, not only after it has changed the file (#953).
#
#   PreToolUse  Read   route the one path the read names, and add the pointer
#                      lines as context. It emits no permission decision, so it
#                      never blocks a read and never asks for one.
#
# This is not spec 5's read-time rule loading, which is a generated rule file a
# harness attaches on a glob (capability C2). This is injected context, a hook
# position, and spec 16 records it as C11.
#
# A script of its own rather than a branch of `write.sh`: the two positions
# share one answer, `hw_governing_pointers` in `lib.sh`, and nothing else. The
# write position refuses and this one never does, so a branch in one file would
# carry a refusal path that a read must never reach.
#
# What it passes to the engine: one path. What it gets back: the pointers
# `headwater route` renders. No verb is new, per spec 5's hook contract.
#
# It is silent when nothing governs the path, when the path is outside the
# root, when the payload names no path, and when there is no engine. It fails
# open on every path it cannot decide, like every other position.
#
# A read through `Bash` (`cat`, `sed -n`) matches no matcher and reaches no
# hook. The same bypass holds for a write, and spec 5 records it.
#
# Registered on Claude Code alone. Codex has no read tool that this repository
# has seen: it reads through its shell, so a matcher here would match nothing.
# Copilot's read tool name is not yet measured, and `fixtures-live.sh` records
# it. Spec 16's C11 row carries both cells. The payload shapes still differ as
# they do for `write.sh`, so this reads `tool_input.path` too.

. "$(dirname "$0")/lib.sh"

input=$(cat)
event=$(hw_field "$input" hook_event_name) || exit 0
[ "$event" = PreToolUse ] || exit 0
path=$(hw_field "$input" tool_input file_path) \
    || path=$(hw_field "$input" tool_input path) \
    || exit 0
[ -n "$path" ] || exit 0

hw_root=$(hw_resolve_root "$input")

case $path in
    "$hw_root"/*) rel=${path#"$hw_root"/} ;;
    /*) exit 0 ;;
    *) rel=$path ;;
esac

pointers=$(hw_governing_pointers "$rel") || exit 0

advisory="Headwater: a document in this corpus declares that it governs \`$rel\`, which you are about to read.

$pointers

Read the ones that bear on your task before you change this file. This is context, and it blocks nothing."
quoted=$(hw_quote "$advisory") || exit 0
printf '{"hookSpecificOutput":{"hookEventName":"PreToolUse","additionalContext":%s}}\n' "$quoted"
exit 0
