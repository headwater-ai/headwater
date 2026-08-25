#!/bin/sh
# What the three harness hooks share, and nothing else.
#
# A hook is a place where a harness hands control to this engine and takes it
# back. The contract is in spec 5. This file holds the two things all three
# hooks need and no rule of any kind: where the engine is, and how to read one
# field off the JSON object the harness puts on standard input.
#
# Every function here fails open. A hook that cannot find the engine, or cannot
# read its input, returns nothing and lets the harness proceed. The commit hook
# and the CI job are downstream of all three, so a hook that failed closed would
# stop work that the position below it already holds.

# The repository these hooks answer for. The harness sets `CLAUDE_PROJECT_DIR`,
# and a fixture run sets `HEADWATER_HOOK_ROOT` instead so that a test can point
# a hook at a corpus that is not this one.
hw_root=${HEADWATER_HOOK_ROOT:-${CLAUDE_PROJECT_DIR:-$PWD}}

# The built engine, or empty when there is none.
hw_engine() {
    engine="$hw_root/engine/target/release/headwater"
    [ -x "$engine" ] || return 1
    printf '%s' "$engine"
}

# One top-level or one nested field of the JSON object on standard input, as a
# bare string. `hw_field "$input" tool_input file_path` reads `.tool_input
# .file_path`. It prints nothing when the field is absent or the input will not
# parse, and the caller treats that as "the harness said nothing".
#
# This is the one place a hook needs an interpreter. The engine parses YAML and
# Markdown and never the harness's own wire format, so the hook does it here.
hw_field() {
    _json=$1
    shift
    command -v python3 >/dev/null 2>&1 || return 1
    printf '%s' "$_json" | python3 -c '
import json, sys
try:
    value = json.load(sys.stdin)
except Exception:
    sys.exit(1)
for key in sys.argv[1:]:
    if not isinstance(value, dict):
        sys.exit(1)
    value = value.get(key)
    if value is None:
        sys.exit(1)
if isinstance(value, bool):
    sys.stdout.write("true" if value else "false")
elif isinstance(value, (str, int, float)):
    sys.stdout.write(str(value))
else:
    sys.exit(1)
' "$@" 2>/dev/null
}

# A JSON string literal, so a hook can put a path or a report into the object it
# writes back without breaking it.
hw_quote() {
    command -v python3 >/dev/null 2>&1 || return 1
    printf '%s' "$1" | python3 -c 'import json, sys; sys.stdout.write(json.dumps(sys.stdin.read()))' 2>/dev/null
}

# The one path an `apply_patch` tool call names, for a harness that hands this
# hook a unified-diff-style command instead of the `file_path` field the other
# two harnesses pass. Codex's own edit tool is `apply_patch`, and its
# `tool_input` carries the patch text under `command` rather than a path
# (spec 16's C4 row: "exit 2 with the reason on standard error", the same
# document, names no `file_path`). A patch's own header lines say what it
# touches: `*** Add File: <path>`, `*** Update File: <path>`, `*** Delete
# File: <path>`. This reads the first one and stops.
#
# Only the first path a patch names reaches the check below. A patch that
# creates several files in one call holds this hook to the first of them, and
# the commit gate holds the rest, the same as it holds every write a `Bash`
# call makes that no matcher here ever sees.
hw_patch_path() {
    _json=$1
    command -v python3 >/dev/null 2>&1 || return 1
    printf '%s' "$_json" | python3 -c '
import json, sys
try:
    value = json.load(sys.stdin)
except Exception:
    sys.exit(1)
if not isinstance(value, dict):
    sys.exit(1)
tool_input = value.get("tool_input")
if not isinstance(tool_input, dict):
    sys.exit(1)
command = tool_input.get("command")
if not isinstance(command, str):
    sys.exit(1)
for line in command.splitlines():
    for label in ("Add File", "Update File", "Delete File"):
        prefix = "*** " + label + ": "
        if line.startswith(prefix):
            sys.stdout.write(line[len(prefix):].strip())
            sys.exit(0)
sys.exit(1)
' 2>/dev/null
}
