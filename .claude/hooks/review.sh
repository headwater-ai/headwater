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
#
# Two harnesses, one blocking mechanism apiece, confirmed live against each.
# Claude Code and Codex both fail this position closed on exit 2 with the
# report on standard error. GitHub Copilot's CLI does not: an exit-2 `Stop`
# hook there is logged and the turn ends anyway, and the block instead reads a
# `{"decision":"block","reason":...}` object on standard output at exit 0.
# `COPILOT_CLI` is the one environment variable that told the two mechanisms
# apart in that test, so it decides which shape this hook writes. Getting the
# detection wrong fails the same way either mechanism already fails open: the
# turn ends and the commit gate holds the result regardless.

. "$(dirname "$0")/lib.sh"

input=$(cat)

# The harness sets this once it has already blocked a stop, and blocking again
# on the same turn is a loop rather than a gate.
#
# The engine is what reads it, so a host with no engine cannot decide, and this
# position ends the turn rather than running a gate it could not stop twice.
# That is the hook contract's term at the one branch that used to break it:
# HW-OBL-0146 measured a machine with no `python3` re-blocking the same turn
# forever, because the read resolved to an empty string and the guard was gone
# rather than unreadable. A missing engine costs nothing here. The commit gate
# below reads the same tree at the next commit, and it prints its own line when
# there is no engine to run.
hw_engine >/dev/null || exit 0
active=$(hw_field "$input" stop_hook_active)
[ "$active" = "true" ] && exit 0

gate="$hw_root/.githooks/pre-commit"
[ -x "$gate" ] || exit 0

report=$(cd "$hw_root" && "$gate" 2>&1)
status=$?
[ "$status" -eq 0 ] && exit 0

message="The commit gate, run at the end of this turn rather than at the next commit.
$report"

if [ -n "${COPILOT_CLI:-}" ]; then
    quoted=$(hw_quote "$message") || exit 0
    printf '{"decision":"block","reason":%s}\n' "$quoted"
    exit 0
fi

printf '%s\n' "$message" >&2
exit 2
