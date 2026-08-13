#!/bin/sh
# The intent-time hook: `headwater route`, on the task before the agent reads a
# file. Registered on `UserPromptSubmit`, which takes no matcher, so it sees
# every prompt.
#
# What it passes to the engine: the prompt, as text. Nothing else. The engine
# reads the checkout itself.
#
# What it hands back: what `route` wrote, verbatim, on standard output, which
# this harness adds to the agent's context. It hands back nothing at all when
# the route offered no pointer, because spec 5 makes silence a result and a
# wrong pointer costs more than a missing one.
#
# What a refusal means here: there is none. `route` exits 0 whether or not it
# offers a pointer, and this hook never blocks a prompt. An intent-time hook
# that refused would refuse the work rather than the mistake.
#
# It fails open and silently. Every other position in this repository prints one
# line when it cannot run, and this one does not, because it runs on every
# prompt and a line per prompt is a cost the reader pays forever. The commit
# hook says out loud when the engine is missing, once, at the moment it matters.

. "$(dirname "$0")/lib.sh"

input=$(cat)
engine=$(hw_engine) || exit 0

task=$(hw_field "$input" user_input) || task=$(hw_field "$input" prompt) || exit 0
[ -n "$task" ] || exit 0

route=$("$engine" route --root "$hw_root" "$task" 2>/dev/null) || exit 0

# A pointer line is `  <path> — <summary>`. Every silence line is prose with no
# em dash in it, so the presence of one separates "the corpus has something for
# this" from all four ways `route` reports that it has nothing.
printf '%s\n' "$route" | grep -q ' — ' || exit 0

printf 'Headwater routed this task to the documents that govern it, before you open a file.\n'
printf 'These are pointers, not content. Open the ones that bear on the task.\n\n'
printf '%s\n' "$route"
exit 0
