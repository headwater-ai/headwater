#!/bin/sh
# The intent-time hook: `headwater route`, on the task, before the agent reads
# a file. Registered on `UserPromptSubmit`, which takes no matcher, so it sees
# every prompt, and it answers only the ones a person wrote.
#
# It passes the prompt to the binary as text, and the binary reads the
# checkout itself. It hands back what `route` wrote, on standard output, which
# the harness adds to the context of the agent. It hands back nothing when the
# route offered no pointer, because a wrong pointer costs more than a missing
# one.
#
# It never blocks a prompt, and it fails open and silently: it runs on every
# prompt, and a line on each one is a cost the reader pays forever.
#
# It decides on the `pointers` member of `route --json`, which is written on
# every run and written empty where the route had nothing to offer. What
# reaches the agent is the `text` member, the rendering of the binary byte for
# byte, so the hook composes no line of its own.
#
# A repository that defines `hw_local_intent` in `lib.local.sh` hears of each
# route after it is decided. The function reads `input`, `task`, `route`,
# `skip` and `injected`, and it must write nothing to either stream.

. "${0%/*}/lib.sh"

# The extension of the repository, where it has one. Its output and its status
# reach nobody.
hw_intent_done() {
    if command -v hw_local_intent >/dev/null 2>&1; then
        { hw_local_intent; } >/dev/null 2>&1
    fi
    return 0
}

input=$(cat)
engine=$(hw_engine) || exit 0
# Every read below uses this binary, after `hw_root` moves to the worktree.
hw_engine_pin=$engine

task=$(hw_field "$input" user_input) || task=$(hw_field "$input" prompt) || exit 0
[ -n "$task" ] || exit 0

# Not every prompt was written by a person. A harness that runs agents in
# parallel submits a prompt each time one of them finishes, and its text is
# the closing report of that agent inside an envelope. That report is not a
# task, and it is already in the context of the reader. The test is on the
# first characters, so a person who writes the word in a question still gets
# pointers.
case $task in
    '<task-notification>'* | '<cross-session-message'* ) exit 0 ;;
esac

hw_root=$(hw_resolve_root "$input")

skip=
injected=false
route=$("$engine" route --root "$hw_root" --json "$task" 2>/dev/null) || {
    skip=route_failed
    hw_intent_done
    exit 0
}

# The count is the decision, and an empty `pointers` is a result rather than a
# failure. A document that does not parse ends the hook with the prompt
# untouched.
pointers=$(hw_count "$route" pointers) || {
    skip=no_pointers_count
    hw_intent_done
    exit 0
}

if [ "$pointers" -gt 0 ] 2>/dev/null; then
    report=$(hw_field "$route" text) || report=
else
    report=
fi

injected=false
if [ "$pointers" -gt 0 ] 2>/dev/null && [ -n "$report" ]; then
    injected=true
fi

hw_intent_done

if [ "$injected" = true ]; then
    printf 'Headwater routed this task to the documents that govern it, before you open a file.\n'
    printf 'These are pointers, not content. Open the ones that bear on the task.\n\n'
    printf '%s\n' "$report"
fi
exit 0
