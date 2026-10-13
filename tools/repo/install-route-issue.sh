#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
#
# install-route-issue.sh RUN_URL JOB=RESULT... — open or update the one issue
# that says an install route of the README failed on its scheduled run (#1490).
#
# The last job of `.github/workflows/install-routes.yml` runs this when a route
# job did not succeed, with each route job's id and its result. It names every
# job whose result is not `success`. Then it comments on the open issue labeled
# `bug` whose title is the one below, or creates that issue with the label when
# none is open, so that a route that stays broken for a week is one issue with
# seven comments and not seven issues.
#
# It needs `gh` with a token that may write issues, and `GITHUB_REPOSITORY`.
# Group 14 of `tools/repo/readme-fixtures.sh` runs it against a stub `gh` with
# no open issue, with one, and with a `gh` that fails.

set -u

title="An install route the README documents failed on its scheduled run"

if [ $# -lt 2 ]; then
    echo "usage: install-route-issue.sh RUN_URL JOB=RESULT..." >&2
    exit 2
fi
run_url=$1
shift

repo=${GITHUB_REPOSITORY:-}
if [ -z "$repo" ]; then
    echo "install-route-issue: GITHUB_REPOSITORY is not set, so no repository is named" >&2
    exit 2
fi
# In a workflow `gh` reads its token from `GH_TOKEN` and nothing else, so a
# report job that hands it none files nothing. Say so before calling it.
if [ -z "${GH_TOKEN:-}${GITHUB_TOKEN:-}" ]; then
    echo "install-route-issue: neither GH_TOKEN nor GITHUB_TOKEN is set, so gh cannot write the issue" >&2
    exit 2
fi

failed=""
for pair in "$@"; do
    case $pair in
        *=success) ;;
        *=*) failed="$failed- \`${pair%%=*}\`: ${pair#*=}
" ;;
        *)
            echo "install-route-issue: \`$pair\` is not JOB=RESULT" >&2
            exit 2
            ;;
    esac
done

# The workflow runs this only when a route job did not succeed. A list with no
# failure in it means the report job dropped the job that failed, from its
# `needs` or from its arguments, so it still files, and says that.
if [ -z "$failed" ]; then
    failed="- no route job handed to this script failed, so the report job dropped the one that did from its \`needs\` or its arguments
"
fi

work=$(mktemp -d) || exit 1
body="$work/body.md"

number=$(gh issue list --repo "$repo" --state open --label bug --limit 200 \
    --json number,title --jq "map(select(.title == \"$title\")) | .[0].number // empty") || {
    echo "install-route-issue: \`gh issue list\` failed, so the open issue cannot be found" >&2
    rm -rf "$work"
    exit 1
}

if [ -n "$number" ]; then
    {
        printf 'The scheduled run %s failed again.\n\n' "$run_url"
        printf 'The route jobs that did not succeed:\n\n%s' "$failed"
    } >"$body"
    gh issue comment "$number" --repo "$repo" --body-file "$body"
    status=$?
else
    {
        printf '## ELI5\n\n'
        printf 'The README tells a reader several ways to install Headwater: download a file, use apt, or build it with Rust. '
        printf 'Every day a robot follows each of those instructions on a clean computer, the way a new reader would. '
        printf 'On its last run at least one way did not work, so a reader who follows the README today may not get a working install. '
        printf 'Fixed means the robot finishes every way again, or the README stops offering the way that broke.\n\n'
        printf '## What happens today\n\n'
        printf 'The scheduled run %s of `.github/workflows/install-routes.yml` failed. ' "$run_url"
        printf 'Each job runs the commands `tools/repo/readme-install-block.sh` prints from `README.md` and checks what `headwater --version` prints.\n\n'
        printf 'The route jobs that did not succeed:\n\n%s\n' "$failed"
        printf 'Each later failed run adds a comment here while this issue is open.\n\n'
        printf '## Done when\n\n'
        printf -- '- [ ] A dispatched run of `install-routes.yml` passes every route job, or the README no longer documents the route that failed.\n'
    } >"$body"
    gh issue create --repo "$repo" --title "$title" --label bug --body-file "$body"
    status=$?
fi

rm -rf "$work"
exit "$status"
