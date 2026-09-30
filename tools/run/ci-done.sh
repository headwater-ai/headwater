#!/bin/sh
# Whether CI on one commit of headwater-ai/headwater has finished, and how.
#
# The condition every build-order wait on CI hands to `tools/run/wait-for.sh`.
# Before this script each stage wrote its own, and each one was wrong in its
# own way. `hw-integrate.md` read `commits/<sha>/status`, the legacy
# combined-status endpoint, which answers `pending` with an empty status list
# forever on a repository whose CI is GitHub Actions: every attempt ended in
# RE-ISSUE until the integrator of run 20260923-0733 noticed and changed the
# endpoint itself, and #1046 then wrote an inline check-runs condition into
# the definition with a count guard, so that a wait cannot clear before CI has
# registered a run. The guard lives on here as "no workflow run yet". Check
# runs alone still miss a job behind `needs:`, as the paragraph on both halves
# below says. A builder first slept and listed runs to learn a run id to
# wait on, and another waited on `gh pr checks | grep -qv pending`, which
# succeeds the moment any one check is no longer pending.
#
#     sh tools/run/ci-done.sh <sha>
#
# Exit 0: at least one workflow run exists for the commit, and every workflow
# run and every check run on it has completed. It prints one line per workflow
# run, `run <id> <conclusion> <workflow>`, then `ci-done: <sha> green`, or
# `ci-done: <sha> red: <check>, <check>` naming each check run whose
# conclusion is not success, skipped or neutral. Finished is not green, so a
# red run still ends the wait and is read from the last line.
#
# A cancelled run that another run of the same workflow on the same commit
# superseded is printed as `run <id> cancelled <workflow> (superseded,
# <branch>)` and counts for nothing, and neither do the check runs of its
# check suite. A commit is the head of more than one branch at once, and in
# run 20260924-0411 the merge commit d5aa24bd of main read red because the
# CI run of fix/809-derived-check-attr on the same sha was cancelled. The
# superseding run must have run: a skipped or neutral one does not count. A
# lone cancelled run is still red. The rule is where it is applied, below.
#
# Exit 1: not finished yet, with one line on standard error saying how far it
# has got. No workflow run at all is not finished either: a push takes a few
# seconds to start one, and a conflicting pull request starts none, which the
# line says so that a wait re-issued on it reads the cause.
#
# Exit 2: no commit named, or `gh` could not resolve it.
#
# Both halves are read because each misses something the other has. A job
# behind `needs:` has no check run until the job before it finishes, so check
# runs alone can all be complete while a workflow is still running, and a
# check posted from outside Actions (the Workers build) has no workflow run.
#
#     sh tools/run/wait-for.sh 'sh tools/run/ci-done.sh <sha>'

set -u

repo=headwater-ai/headwater

ref=${1:-}
if [ -z "$ref" ]; then
    echo "ci-done: needs a commit." >&2
    echo "  sh tools/run/ci-done.sh <sha>" >&2
    exit 2
fi

# `head_sha` on the runs endpoint matches only a full sha, so a short one
# would read as a commit with no CI at all.
sha=$(gh api "repos/$repo/commits/$ref" --jq .sha 2>/dev/null) || sha=
if [ -z "$sha" ]; then
    echo "ci-done: cannot resolve \`$ref\` to a commit of $repo." >&2
    exit 2
fi
short=$(printf '%.8s' "$sha")

runs=$(gh api "repos/$repo/actions/runs?head_sha=$sha&per_page=100" \
    --jq '.workflow_runs[] | [.id, .status, (.conclusion // "-"), .name, (.check_suite_id // "-"), (.head_branch // "-")] | @tsv') || exit 1
checks=$(gh api "repos/$repo/commits/$sha/check-runs?per_page=100" \
    --jq '.check_runs[] | [.status, (.conclusion // "-"), .name, (.check_suite.id // "-")] | @tsv') || exit 1

if [ -z "$runs" ]; then
    echo "ci-done: $short has no workflow run yet. A fresh push starts one within seconds; a conflicting pull request starts none." >&2
    exit 1
fi

tab=$(printf '\t')

# A cancelled run is superseded when another run of the same workflow on the
# same commit completed and ran: its conclusion is neither cancelled, skipped
# nor neutral. The superseded run and every check run of its check suite are
# left out of what follows, so they neither hold the wait nor turn it red.
# d5aa24bd, the merge commit of main in run 20260924-0411, was also the head
# of a branch whose CI run was cancelled, and the cancelled suite's two check
# runs read as red on a commit whose main run had passed. A lone cancelled
# run supersedes nothing and stays red.
# A run is keyed by its id, and its check runs by its check suite's id. A
# run whose suite id is missing (`-`) is still dropped, but `-` is never a
# key for check runs: an external check with no suite would match it.
superseded=$(printf '%s\n' "$runs" | awk -F"$tab" '
    NF { id[NR] = $1; c[NR] = $3; n[NR] = $4; s[NR] = $5
         if ($2 == "completed" && $3 != "cancelled" && $3 != "skipped" && $3 != "neutral" && $3 != "-") ran[$4] = 1 }
    END { for (i = 1; i <= NR; i++) if (c[i] == "cancelled" && (n[i] in ran)) print id[i] "\t" s[i] }')
dropped_runs=$(printf '%s\n' "$superseded" | awk -F"$tab" 'NF { print $1 }' | paste -sd' ' -)
dropped_suites=$(printf '%s\n' "$superseded" | awk -F"$tab" 'NF && $2 != "-" { print $2 }' | paste -sd' ' -)
all_runs=$runs
runs=$(printf '%s\n' "$all_runs" | awk -F"$tab" -v d=" $dropped_runs " 'NF && index(d, " " $1 " ") == 0')
checks=$(printf '%s\n' "$checks" | awk -F"$tab" -v d=" $dropped_suites " 'NF && index(d, " " $4 " ") == 0')

runs_total=$(printf '%s\n' "$runs" | grep -c .)
runs_done=$(printf '%s\n' "$runs" | awk -F"$tab" '$2 == "completed"' | grep -c .)
checks_total=$(printf '%s\n' "$checks" | grep -c .)
checks_done=$(printf '%s\n' "$checks" | awk -F"$tab" '$1 == "completed"' | grep -c .)

if [ "$runs_done" -lt "$runs_total" ] || [ "$checks_done" -lt "$checks_total" ]; then
    echo "ci-done: $short not finished: $runs_done of $runs_total workflow runs and $checks_done of $checks_total check runs complete." >&2
    exit 1
fi

printf '%s\n' "$all_runs" | awk -F"$tab" -v d=" $dropped_runs " 'NF {
    if (index(d, " " $1 " ") > 0) print "run " $1 " " $3 " " $4 " (superseded, " $6 ")"
    else print "run " $1 " " $3 " " $4 }'

# A workflow that fails before it starts a job, a startup failure, leaves no
# check run behind, so a failed workflow run is named as well as a failed check.
red=$( {
    printf '%s\n' "$checks" | awk -F"$tab" '$2 != "success" && $2 != "skipped" && $2 != "neutral" && NF { print $3 }'
    printf '%s\n' "$runs" | awk -F"$tab" '$3 != "success" && $3 != "skipped" && $3 != "neutral" && NF { print "workflow " $4 }'
} | paste -sd, - | sed 's/,/, /g')
if [ -n "$red" ]; then
    echo "ci-done: $short red: $red"
else
    echo "ci-done: $short green"
fi
exit 0
