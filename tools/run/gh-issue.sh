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

# visible FILE: the lines of a Markdown body that GitHub renders as text,
# with carriage returns dropped (a body edited in the web form comes back
# with CRLF line ends). A fenced block is dropped whole: it closes only on a
# fence of its own character at least as long as the one that opened it,
# with nothing after it, so a ``` line inside a ~~~ fence stays inside. An
# HTML comment is cut out of the line it sits on, and the text before `<!--`
# and after `-->` stays. No fence opens inside a comment, and a `<!--` in a
# code span opens none. A `<!--` after other text on its line is a comment
# only where a `-->` closes it before the paragraph ends, as GitHub reads
# it; otherwise it is text. Where a fence or a comment is still open at the
# end of the body, the reader cannot tell what GitHub shows inside a list or
# a quote, so it names the line on stderr and exits 3 rather than guess.
visible() {
    tr -d '\r' < "$1" | awk '
        function fence_of(s,   t) {
            t = s
            sub(/^[ \t]*/, "", t)
            if (match(t, /^```+/) || match(t, /^~~~+/)) return substr(t, 1, RLENGTH)
            return ""
        }
        # the line with every code span blanked to spaces, so an index into
        # it finds a `<!--` only where GitHub would read one
        function mask(s,   out, i, run, j, r2, found) {
            out = ""
            i = 1
            while (i <= length(s)) {
                if (substr(s, i, 1) != "`") { out = out substr(s, i, 1); i++; continue }
                run = 0
                while (substr(s, i + run, 1) == "`") run++
                j = i + run
                found = 0
                while (j <= length(s)) {
                    if (substr(s, j, 1) != "`") { j++; continue }
                    r2 = 0
                    while (substr(s, j + r2, 1) == "`") r2++
                    if (r2 == run) { found = 1; break }
                    j += r2
                }
                if (!found) { out = out substr(s, i, run); i += run; continue }
                out = out sprintf("%" (j + run - i) "s", "")
                i = j + run
            }
            return out
        }
        function closes_in_paragraph(ln, rest,   k) {
            if (index(rest, "-->")) return 1
            for (k = ln + 1; k <= n; k++) {
                if (L[k] ~ /^[ \t]*$/) return 0
                if (index(L[k], "-->")) return 1
            }
            return 0
        }
        { L[++n] = $0 }
        END {
            for (ln = 1; ln <= n; ln++) {
                line = L[ln]
                if (fence != "") {
                    f = fence_of(line)
                    rest = line
                    sub(/^[ \t]*[`~]+/, "", rest)
                    if (f != "" && substr(f, 1, 1) == substr(fence, 1, 1) && length(f) >= length(fence) && rest ~ /^[ \t]*$/) fence = ""
                    continue
                }
                if (!incomment) {
                    f = fence_of(line)
                    if (f != "") { fence = f; opened = ln; continue }
                }
                out = ""
                while (1) {
                    if (incomment) {
                        i = index(line, "-->")
                        if (!i) break
                        line = substr(line, i + 3)
                        incomment = 0
                        continue
                    }
                    i = index(mask(line), "<!--")
                    if (!i) { out = out line; break }
                    pre = substr(line, 1, i - 1)
                    if (!(out == "" && pre ~ /^[ \t]*$/) && !closes_in_paragraph(ln, substr(line, i + 4))) {
                        out = out substr(line, 1, i + 3)
                        line = substr(line, i + 4)
                        continue
                    }
                    out = out pre
                    line = substr(line, i + 4)
                    incomment = 1
                    opened = ln
                }
                print out
            }
            if (fence != "" || incomment) {
                print "a " (fence != "" ? "fence" : "comment") " opened on line " opened " never closes" > "/dev/stderr"
                exit 3
            }
        }
    '
}

# clauses_of FILE: the checkbox clauses of an issue body held in FILE, one
# per line as `<k> [<mark>] <text>`, numbered from 1 in body order. Reading
# starts at the first heading of any level whose text opens `Done when`, in
# any case, and runs to the end of the body; no later heading of any level
# ends it. A clause is a task item GitHub renders: a `-`, `*` or `+` bullet,
# or an ordered `1.` or `1)` item, in a quote or not, then `[ ]`, `[x]` or
# `[X]`. Only what `visible` keeps is read, so a box in a fence or a comment
# is no clause, and where `visible` refuses, this returns 3 and reads none.
clauses_of() {
    v=$(mktemp) || exit 1
    if ! visible "$1" > "$v"; then
        rm -f "$v"
        return 3
    fi
    awk '
        !started && tolower($0) ~ /^#+[ \t]+done when/ { started = 1; next }
        !started { next }
        /^[ \t]*(>[ \t]*)*([-*+]|[0-9]+[.)])[ \t]+\[[ xX]\]/ {
            i = index($0, "[")
            mark = substr($0, i + 1, 1)
            text = substr($0, i + 3)
            sub(/^[ \t]+/, "", text)
            k++
            printf "%d [%s] %s\n", k, mark, text
        }
    ' "$v"
    rm -f "$v"
}

# account ISSUE CLAUSES-FILE PR-BODY-FILE: print one line for each clause of
# ISSUE that the pull request body does not mark met, and exit 1 if there is
# any. A clause is met only when a line of the body reads
# `- [x] #<issue>.<k> <evidence>` and no line reads `- [ ] #<issue>.<k>`.
# PR-BODY-FILE is what `visible` kept, so a line in a fence is no accounting.
account() {
    awk -v n="$1" -v prfile="$3" '
        BEGIN {
            while ((getline line < prfile) > 0) {
                if (!match(line, /^[ \t]*(>[ \t]*)*([-*+]|[0-9]+[.)])[ \t]+\[[ xX]\][ \t]+#[0-9]+\.[0-9]+/)) continue
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
    st=$?
    rm -f "$tmp"
    [ "$st" -eq 0 ] || { echo "gh-issue: #$n cannot be read for clauses." >&2; exit 1; }
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
    if ! visible "$tmp/pr" > "$tmp/pr.visible" 2> "$tmp/why"; then
        echo "#$pr body: $(cat "$tmp/why"), so no line of it is read as accounting."
        rm -rf "$tmp"
        echo "clause-check: #$pr refused. Close it in the pull request body." >&2
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
        if ! clauses_of "$tmp/issue" > "$tmp/clauses" 2> "$tmp/why"; then
            echo "#$i body: $(cat "$tmp/why"), so its clauses cannot be counted."
            refused=1
            continue
        fi
        if [ ! -s "$tmp/clauses" ]; then
            echo "#$i has no clause: no checkbox under a Done-when heading, so nothing can account for it."
            refused=1
            continue
        fi
        account "$i" "$tmp/clauses" "$tmp/pr.visible" || refused=1
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
