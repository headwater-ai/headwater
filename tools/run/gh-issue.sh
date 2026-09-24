#!/bin/sh
# One issue of headwater-ai/headwater, read and written the way that works.
#
# `gh issue view` and `gh pr edit` both fail on this repository with a
# `projectCards` GraphQL deprecation, a fact three separate agent definitions
# and the `hw-run-policy` skill each stated on their own before this script
# existed. `gh api` on a single field is the workaround, and it was retyped by
# hand at every call site: the repository slug, the endpoint shape and the
# `--jq` filter were free to drift apart one call at a time, and did.
#
#     sh tools/run/gh-issue.sh body <N>                  the issue body, plain text
#     sh tools/run/gh-issue.sh comments <N>               every comment body, one per line
#     sh tools/run/gh-issue.sh patch-body <N> <file>      replace the body from a file
#     sh tools/run/gh-issue.sh comment <N> <file>         post a new comment from a file
#     sh tools/run/gh-issue.sh close <N>                  close the issue, and confirm the close took
#     sh tools/run/gh-issue.sh closes <PR>                the issues a pull request's merge closes, as a JSON array
#
# `patch-body` and `comment` read the file rather than a shell argument,
# because a long note as an argument is context as permanent as any result,
# the same reason `run-dir.sh log` takes JSON and not a string of flags.
#
# `close` re-reads the issue's state after asking `gh` to change it. An agent
# can state a write-back and not land it — hw-run-policy names this as a
# measured failure mode of its own, and a close that did not take is silent
# without the re-read.
#
# `closes` asks GraphQL for `closingIssuesReferences`, the one field that says
# which issues a merge closes. `gh pr view --json` on this host's gh has no
# such field, and a title's `(#N)` or a body's `Refs #N` closes nothing. The
# query lives here because its braces, typed inline, are refused by worktree
# isolation as a construct too complex to verify.
#
# The repository is fixed at headwater-ai/headwater, the one this tree's
# tools already hardcode; this is not a general gh wrapper.

set -u

repo=headwater-ai/headwater

usage() {
    sed -n '/^#     sh tools\/run\/gh-issue.sh/p' "$0" | sed 's/^# *//' >&2
    exit 2
}

need_number() {
    case $1 in
        '' | *[!0-9]*)
            echo "gh-issue: \`$2\` wants an issue number, got \`$1\`." >&2
            exit 2
            ;;
    esac
}

body() {
    n=$1
    need_number "$n" body
    gh api "repos/$repo/issues/$n" --jq .body
}

comments() {
    n=$1
    need_number "$n" comments
    gh api "repos/$repo/issues/$n/comments" --jq '.[].body'
}

patch_body() {
    n=$1 file=$2
    need_number "$n" patch-body
    [ -f "$file" ] || { echo "gh-issue: no file at $file." >&2; exit 1; }
    gh api -X PATCH "repos/$repo/issues/$n" -F "body=@$file"
}

comment() {
    n=$1 file=$2
    need_number "$n" comment
    [ -f "$file" ] || { echo "gh-issue: no file at $file." >&2; exit 1; }
    gh api "repos/$repo/issues/$n/comments" -F "body=@$file"
}

close() {
    n=$1
    need_number "$n" close
    gh api -X PATCH "repos/$repo/issues/$n" -f state=closed >/dev/null
    state=$(gh api "repos/$repo/issues/$n" --jq .state)
    if [ "$state" = closed ]; then
        echo "gh-issue: #$n is closed."
    else
        echo "gh-issue: asked to close #$n, but it reads \`$state\`. The close did not take." >&2
        exit 1
    fi
}

closes() {
    n=$1
    need_number "$n" closes
    q='query($n:Int!){repository(owner:"headwater-ai",name:"headwater"){pullRequest(number:$n){closingIssuesReferences(first:50){nodes{number}}}}}'
    gh api graphql -F "n=$n" -f "query=$q" --jq '[.data.repository.pullRequest.closingIssuesReferences.nodes[].number]'
}

case ${1:-} in
    body) [ $# -eq 2 ] || usage; body "$2" ;;
    comments) [ $# -eq 2 ] || usage; comments "$2" ;;
    patch-body) [ $# -eq 3 ] || usage; patch_body "$2" "$3" ;;
    comment) [ $# -eq 3 ] || usage; comment "$2" "$3" ;;
    close) [ $# -eq 2 ] || usage; close "$2" ;;
    closes) [ $# -eq 2 ] || usage; closes "$2" ;;
    *) usage ;;
esac
