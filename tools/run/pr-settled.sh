#!/bin/sh
# The condition a verifier waits on after its attacks: GitHub has computed
# `mergeable` for the pull request, and CI on the commit it detached at has
# finished.
#
#     sh tools/run/pr-settled.sh <PR> <sha>
#     sh tools/run/wait-for.sh 'sh tools/run/pr-settled.sh <PR> <sha>'
#
# `.claude/agents/hw-verify.md` gave the verifier two waits, one on
# `mergeable` by branch through `gh pr list` and one on `ci-done.sh`, each
# capped at four minutes and each re-issued on its own. Two waits are two
# Bash timeouts and two `RE-ISSUE` loops in a stage whose whole job after the
# attacks is to wait, and the first condition was retyped by every verifier
# with its own quoting. One condition, one wait, in the shape `ci-done.sh`
# and `queue-done.sh` already have.
#
# Exit 1 while either half is not ready, with one line on stderr saying
# which. Exit 0 once both are, printing:
#
#     mergeable: <true|false> (<mergeable_state>)
#     <what ci-done.sh printed, its last line green or red>
#     pr-settled: #<PR> at <sha> settled
#
# and `head: <api sha>, not <sha>` before the last line when the pull
# request's head has moved past the commit the verifier detached at. CI was
# read on the commit the attacks ran on, and the newer push is unverified:
# the verifier reports that line under `RAN`.
#
# `gh pr view` fails on this repository (projectCards), so the read is
# `gh api` on `pulls/<PR>`, where `mergeable` is null until GitHub has
# computed it and `mergeable_state` reads `unknown` for the same window. An
# empty answer is an error and not a verdict (#1357), so the condition fails
# on it. `HW_CI_DONE` names the CI reader, for a fixture.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
repo=headwater-ai/headwater
ci_done=${HW_CI_DONE:-$root/tools/run/ci-done.sh}

usage() {
    sed -n '/^#     sh tools\/run\/pr-settled.sh/p' "$0" | sed 's/^# *//' >&2
    exit 2
}

[ $# -eq 2 ] || usage
pr=$1 sha=$2
case $pr in '' | *[!0-9]*) echo "pr-settled: \`$pr\` is not a pull request number." >&2; exit 2 ;; esac
case $sha in '' | *[!0-9a-f]*) echo "pr-settled: \`$sha\` is not a commit: seven to forty lower-case hex digits." >&2; exit 2 ;; esac
[ ${#sha} -ge 7 ] && [ ${#sha} -le 40 ] || { echo "pr-settled: \`$sha\` is not a commit: seven to forty lower-case hex digits." >&2; exit 2; }

fields=$(gh api "repos/$repo/pulls/$pr" --jq '"\(.mergeable) \(.mergeable_state) \(.head.sha)"' 2>/dev/null)
# shellcheck disable=SC2086
set -- $fields
if [ $# -ne 3 ]; then
    echo "pr-settled: #$pr: GitHub answered nothing for the pull request, which is an error and not a verdict." >&2
    exit 1
fi
mergeable=$1 state=$2 head=$3
case $mergeable in
    true | false) ;;
    *) echo "pr-settled: #$pr: mergeable is still being computed (mergeable=$mergeable, state=$state)." >&2; exit 1 ;;
esac
if [ "$state" = unknown ]; then
    echo "pr-settled: #$pr: mergeable_state is still unknown." >&2
    exit 1
fi

ci=$(sh "$ci_done" "$sha" 2>&1)
status=$?
if [ "$status" -ne 0 ]; then
    printf '%s\n' "$ci" | tail -n 1 >&2
    exit 1
fi

printf 'mergeable: %s (%s)\n' "$mergeable" "$state"
printf '%s\n' "$ci"
case $head in
    "$sha"*) ;;
    *) printf 'head: %s, not %s\n' "$head" "$sha" ;;
esac
printf 'pr-settled: #%s at %s settled\n' "$pr" "$sha"
