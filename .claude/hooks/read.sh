#!/bin/sh
# The read-time hook. A document can declare that it governs a code path, and
# a session that opens that path hears of the document before it reads a
# line of it.
#
#   PreToolUse  Read   route the one path the read names, and add the pointer
#                      lines as context. It writes no permission decision, so
#                      it never blocks a read.
#
# It passes one path to `headwater route` and gets back the pointers. It is
# silent when nothing governs the path, when the path is outside the
# repository, when the payload names no path, and when there is no binary.
#
# A read through a shell command (`cat`, `sed -n`) reaches no hook. Copilot
# names the path `tool_input.path`, so this reads that field too.

. "${0%/*}/lib.sh"

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
# installed by headwater init --harness, digest sha256:fa4ac1596337207024790bfa5fa9efda39cf34392f9220062a70e19e65170f0f
