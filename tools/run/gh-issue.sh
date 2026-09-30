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
#     sh tools/run/gh-issue.sh clauses <N>                the Done-when clauses of an issue, numbered, folds included
#     sh tools/run/gh-issue.sh clause-check <PR>          refuse a pull request that closes an issue with a clause unaccounted
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
# `clauses` reads every checkbox from the first `Done when` heading to the
# end of the body, and no later heading stops it. The product owner folds
# findings into an issue under headings of its own, at h3 and at h2 alike,
# and a reader that stopped at the next `## ` heading missed the last fold of
# #1315: PR #1460 said `Closes #1315` and accounted for about five of its
# eleven clauses, and nothing that merged it read a clause (#1485).
#
# `clause-check` is the integrator's gate before it enqueues. For each issue
# in `closes <PR>`, every clause must appear in the pull request body as one
# line, `- [x] #<issue>.<k> <evidence>`, so the squash message on `main`
# carries the accounting. A clause with no such line, or with a `- [ ]` line
# for it anywhere in the body, refuses, and so does a closed issue with no
# clause at all. The check reads the accounting and never judges the
# evidence; whether the evidence is true is the verifier's attack.
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

# clauses_of FILE: the checkbox clauses of an issue body held in FILE, one
# per line as `<k> [<mark>] <text>`, numbered from 1 in body order. Reading
# starts at the first heading of any level whose text opens `Done when` and
# runs to the end of the body; no later heading ends it. A box in a fenced
# block or an HTML comment is not a clause. Carriage returns are dropped,
# because a body edited in the web form comes back with CRLF line ends.
clauses_of() {
    tr -d '\r' < "$1" | awk '
        /^[ \t]*(```|~~~)/ { fence = !fence; next }
        fence { next }
        incomment { if (index($0, "-->")) incomment = 0; next }
        /<!--/ {
            rest = substr($0, index($0, "<!--") + 4)
            if (!index(rest, "-->")) { incomment = 1; next }
        }
        !started && tolower($0) ~ /^#+[ \t]+done when/ { started = 1; next }
        !started { next }
        /^[ \t]*[-*+][ \t]+\[[ xX]\]/ {
            i = index($0, "[")
            mark = substr($0, i + 1, 1)
            if (mark == "X") mark = "x"
            text = substr($0, i + 3)
            sub(/^[ \t]+/, "", text)
            k++
            printf "%d [%s] %s\n", k, mark, text
        }
    '
}

# account ISSUE CLAUSES-FILE PR-BODY-FILE: print one line for each clause of
# ISSUE that the pull request body does not mark met, and exit 1 if there is
# any. A clause is met only when a line of the body reads
# `- [x] #<issue>.<k> <evidence>` and no line reads `- [ ] #<issue>.<k>`.
account() {
    awk -v n="$1" -v prfile="$3" '
        BEGIN {
            while ((getline line < prfile) > 0) {
                sub(/\r$/, "", line)
                if (!match(line, /^[ \t]*[-*+][ \t]+\[[ xX]\][ \t]+#[0-9]+\.[0-9]+/)) continue
                head = substr(line, RSTART, RLENGTH)
                key = substr(head, index(head, "#") + 1)
                mark = substr(head, index(head, "[") + 1, 1)
                if (mark == " ") unmet[key] = 1
                else met[key] = 1
            }
        }
        {
            key = n "." $1
            if (!(key in met) || (key in unmet)) {
                print "#" n " clause " $1 ": not marked met in the pull request body. " substr($0, length($1) + 2)
                bad++
            }
            total++
        }
        END {
            if (bad) exit 1
            print "clause-check: #" n ", " total " of " total " clauses marked met."
        }
    ' "$2"
}

clauses() {
    n=$1
    need_number "$n" clauses
    tmp=$(mktemp) || exit 1
    if ! body "$n" > "$tmp"; then
        rm -f "$tmp"
        echo "gh-issue: could not read the body of #$n." >&2
        exit 1
    fi
    clauses_of "$tmp"
    rm -f "$tmp"
}

clause_check() {
    pr=$1  # not `n`: body and closes assign the global `n`, and sh has no local
    need_number "$pr" clause-check
    tmp=$(mktemp -d) || exit 1
    if ! closes "$pr" > "$tmp/closes"; then
        rm -rf "$tmp"
        echo "gh-issue: could not read what #$pr closes." >&2
        exit 1
    fi
    issues=$(tr -d '[] \n' < "$tmp/closes" | tr ',' ' ')
    if [ -z "$issues" ]; then
        rm -rf "$tmp"
        echo "clause-check: #$pr closes no issue, so it owes no clause."
        exit 0
    fi
    if ! gh api "repos/$repo/pulls/$pr" --jq .body > "$tmp/pr"; then
        rm -rf "$tmp"
        echo "gh-issue: could not read the body of pull request #$pr." >&2
        exit 1
    fi
    refused=0
    for i in $issues; do
        need_number "$i" clause-check
        if ! body "$i" > "$tmp/issue"; then
            rm -rf "$tmp"
            echo "gh-issue: could not read the body of #$i." >&2
            exit 1
        fi
        clauses_of "$tmp/issue" > "$tmp/clauses"
        if [ ! -s "$tmp/clauses" ]; then
            echo "#$i has no clause: no checkbox under a Done-when heading, so nothing can account for it."
            refused=1
            continue
        fi
        account "$i" "$tmp/clauses" "$tmp/pr" || refused=1
    done
    rm -rf "$tmp"
    if [ "$refused" -ne 0 ]; then
        echo "clause-check: #$pr refused. Mark each clause above \`- [x] #<issue>.<k> <evidence>\` in its body, or make it \`Refs\`." >&2
        exit 1
    fi
}

case ${1:-} in
    body) [ $# -eq 2 ] || usage; body "$2" ;;
    comments) [ $# -eq 2 ] || usage; comments "$2" ;;
    patch-body) [ $# -eq 3 ] || usage; patch_body "$2" "$3" ;;
    comment) [ $# -eq 3 ] || usage; comment "$2" "$3" ;;
    close) [ $# -eq 2 ] || usage; close "$2" ;;
    closes) [ $# -eq 2 ] || usage; closes "$2" ;;
    clauses) [ $# -eq 2 ] || usage; clauses "$2" ;;
    clause-check) [ $# -eq 2 ] || usage; clause_check "$2" ;;
    *) usage ;;
esac
