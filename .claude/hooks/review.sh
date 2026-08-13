#!/bin/sh
# The review-time hook: the commit gate, run at the end of a turn instead of at
# the end of a change. Registered on `Stop`, which takes no matcher.
#
# It runs `.githooks/pre-commit` and nothing else. That is deliberate and it is
# the whole design. A second script that ran `headwater check --strict` with a
# second opinion about which findings block would be two live paths over one
# question, and every disagreement between them would be a defect that only a
# reader could find. So this position invokes the other one. What blocks a
# commit is what blocks a turn, by construction rather than by agreement.
#
# What it passes to the engine: nothing. The commit hook reads the working tree.
# What it gets back: the exit status, and the error findings on standard output.
# What a refusal means: the harness does not let the turn end, and the agent
# reads the same report a commit would have printed.
#
# What happens when the harness ignores it: the turn ends. `disableAllHooks`
# turns this off with no record anywhere, and a harness that is not this one
# never had it. The commit hook still holds the commit, and the CI job still
# holds the pull request, and neither of those is skippable by the agent that
# wrote the defect. This position only makes the finding arrive sooner.

. "$(dirname "$0")/lib.sh"

input=$(cat)

# The harness sets this once it has already blocked a stop, and blocking again
# on the same turn is a loop rather than a gate.
active=$(hw_field "$input" stop_hook_active)
[ "$active" = "true" ] && exit 0

gate="$hw_root/.githooks/pre-commit"
[ -x "$gate" ] || exit 0

report=$(cd "$hw_root" && "$gate" 2>&1)
status=$?
[ "$status" -eq 0 ] && exit 0

printf 'The commit gate, run at the end of this turn rather than at the next commit.\n' >&2
printf '%s\n' "$report" >&2
exit 2
