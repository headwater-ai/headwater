#!/bin/sh
# What holds `tools/run/gh-issue.sh`: the endpoint it builds for each verb,
# and that `close` refuses to report success when the state did not move.
#
# Run it from anywhere:
#     sh tools/run/gh-issue-fixtures.sh
#
# It needs no network and no token: `gh` on `PATH` is a fake for the length
# of this suite, a shell script that logs its own arguments and prints a
# canned answer. The cases below hold the endpoint and the argument shape,
# not what GitHub returns for one.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
tool="$root/tools/run/gh-issue.sh"

if [ ! -f "$tool" ]; then
    echo "no script at \`tools/run/gh-issue.sh\`." >&2
    exit 1
fi

scratch=$(mktemp -d) || exit 1
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

fakebin="$scratch/fakebin"
mkdir -p "$fakebin"
log="$scratch/gh.log"
: > "$log"

# The fake logs every call it sees and answers the shapes this tool makes: a
# body read, a comments read, the state re-read `close` makes after asking
# for the close, a pull request's body read and the closing-references
# query. CLOSE_STATE picks which state the second read answers, so both the
# honored and the refused close are held. ISSUE_BODY and PR_BODY name canned
# files for the clause cases, and CLOSES_JSON is what the query answers.
cat > "$fakebin/gh" <<'EOF'
#!/bin/sh
printf '%s\n' "$*" >> "$LOG"
case "$*" in
    *"/comments --jq .[].body"*) printf 'first comment\nsecond comment\n' ;;
    *"-X PATCH"*"-f state=closed"*) : ;;
    *"--jq .state"*) printf '%s\n' "${CLOSE_STATE:-closed}" ;;
    *"/pulls/"*"--jq .body"*) cat "$PR_BODY" ;;
    *"--jq .body"*)
        if [ "${ISSUE_FAIL:-}" = 1 ]; then echo "HTTP 502" >&2; exit 1; fi
        if [ -n "${ISSUE_BODY:-}" ]; then cat "$ISSUE_BODY"; else printf 'the issue body\n'; fi ;;
    *"api graphql"*) printf '%s\n' "${CLOSES_JSON:-[]}" ;;
    *) : ;;
esac
EOF
chmod +x "$fakebin/gh"

passed=0
failed=0

run() {
    : > "$log"
    PATH="$fakebin:$PATH" LOG="$log" CLOSE_STATE="${CLOSE_STATE:-closed}" \
        ISSUE_BODY="${ISSUE_BODY:-}" PR_BODY="${PR_BODY:-/dev/null}" CLOSES_JSON="${CLOSES_JSON:-[]}" \
        ISSUE_FAIL="${ISSUE_FAIL:-}" \
        sh "$tool" "$@"
}

# check_status NAME WANT(zero|nonzero) STATUS
check_status() {
    name=$1 want=$2 got=$3
    if { [ "$want" = zero ] && [ "$got" -eq 0 ]; } || { [ "$want" = nonzero ] && [ "$got" -ne 0 ]; }; then
        passed=$((passed + 1))
        echo "  ok    $name"
    else
        failed=$((failed + 1))
        echo "  FAIL  $name (wanted $want exit, got $got)"
        echo "        out: $(cat "$scratch/out")"
        echo "        err: $(cat "$scratch/err")"
    fi
}

# check_out NAME PATTERN: a fixed string the verb's stdout must hold.
check_out() {
    name=$1 pattern=$2
    if grep -qF -- "$pattern" "$scratch/out"; then
        passed=$((passed + 1))
        echo "  ok    $name"
    else
        failed=$((failed + 1))
        echo "  FAIL  $name"
        echo "        wanted in stdout: $pattern"
        echo "        stdout holds: $(cat "$scratch/out")"
    fi
}

# check_not_out NAME PATTERN: a fixed string the verb's stdout must not hold.
check_not_out() {
    name=$1 pattern=$2
    if grep -qF -- "$pattern" "$scratch/out"; then
        failed=$((failed + 1))
        echo "  FAIL  $name"
        echo "        stdout holds, and must not: $pattern"
    else
        passed=$((passed + 1))
        echo "  ok    $name"
    fi
}

# check NAME EXIT-STATUS PATTERN
check_logged() {
    name=$1 pattern=$2
    if grep -qF -- "$pattern" "$log"; then
        passed=$((passed + 1))
        echo "  ok    $name"
    else
        failed=$((failed + 1))
        echo "  FAIL  $name"
        echo "        wanted in the log: $pattern"
        echo "        log holds: $(cat "$log")"
    fi
}

echo "body builds the single-field endpoint"
run body 42 > "$scratch/out"
check_logged "repo, issue number and --jq .body" "repos/headwater-ai/headwater/issues/42 --jq .body"

echo "comments builds the comments endpoint"
run comments 42 > "$scratch/out"
check_logged "the comments endpoint" "repos/headwater-ai/headwater/issues/42/comments --jq .[].body"

echo "patch-body reads the file rather than taking prose as an argument"
printf 'a new body\n' > "$scratch/body.md"
run patch-body 42 "$scratch/body.md" > "$scratch/out"
check_logged "PATCH with the file as -F body=@path" "-X PATCH repos/headwater-ai/headwater/issues/42 -F body=@$scratch/body.md"

echo "comment posts to the comments endpoint from a file"
printf 'a note\n' > "$scratch/note.md"
run comment 42 "$scratch/note.md" > "$scratch/out"
check_logged "POST to the comments endpoint" "repos/headwater-ai/headwater/issues/42/comments -F body=@$scratch/note.md"

echo "close asks for the close and then re-reads state"
CLOSE_STATE=closed run close 42 > "$scratch/out" 2>"$scratch/err"
status=$?
if [ "$status" -eq 0 ] && grep -q 'is closed' "$scratch/out"; then
    passed=$((passed + 1))
    echo "  ok    a close that took reports success"
else
    failed=$((failed + 1))
    echo "  FAIL  a close that took reports success (exit $status)"
fi

echo "close refuses to report success when the state did not move"
CLOSE_STATE=open run close 42 > "$scratch/out" 2>"$scratch/err"
status=$?
if [ "$status" -ne 0 ] && grep -q 'did not take' "$scratch/err"; then
    passed=$((passed + 1))
    echo "  ok    a close that did not take is reported, not swallowed"
else
    failed=$((failed + 1))
    echo "  FAIL  a close that did not take is reported (exit $status)"
fi

echo "closes asks GraphQL for the closing references of the pull request"
run closes 1050 > "$scratch/out"
check_logged "a graphql call with the number as a typed variable" "api graphql -F n=1050"
check_logged "  and the closingIssuesReferences field" "closingIssuesReferences"

echo "a non-numeric issue is refused before any gh call"
: > "$log"
PATH="$fakebin:$PATH" LOG="$log" sh "$tool" body abc > "$scratch/out" 2>"$scratch/err"
status=$?
if [ "$status" -ne 0 ] && [ ! -s "$log" ]; then
    passed=$((passed + 1))
    echo "  ok    a non-numeric issue is refused, and gh is never called"
else
    failed=$((failed + 1))
    echo "  FAIL  a non-numeric issue is refused before any gh call"
fi

# The #1460 shape (#1485). #1315 had two clauses under `## Done when`, three
# `### Folded` sections and a last fold at h2, eleven clauses in all, and
# PR #1460 said `Closes #1315` while accounting for about five. The canned
# body below keeps that shape, and adds the three things a clause reader
# must not count: a box above the Done-when heading, a box in an HTML
# comment, and a box in a fenced block.
cat > "$scratch/issue-1315.md" <<'EOF'
## ELI5

Plain words for a reader who never opened the repository.

## What happens today

- [ ] a box above the Done-when heading is not a clause

## Done when

<!--
- [ ] a template placeholder in a comment is not a clause
-->

- [ ] c1 the workspace pin test fails on a drifted version
- [ ] c2 the release guide names generate beside the bless step <!-- carried from the #1348 build -->

### Folded 2026-09-29, first pass

- [ ] c3 the README tag test
- [ ] c4 the step-9 search
- [ ] c5 the FIXTURE_TAG statement
- [x] c6 a box already ticked in the issue is still a clause

### Folded 2026-09-29, final pass

- [ ] c7 the rule case 8k enforces is recorded

### Folded 2026-09-30, fifth-merge pass

- [ ] c8 the workflow comment no longer names v0.4.0
- [ ] c9 the tutorial install block is checked

## Folded by the product owner, 2026-09-30

- [ ] c10 the release-guide fixture runs the step-9 search
  - [ ] c11 a relation reaches publish_order.rs

```text
- [ ] a box in a fenced block is not a clause
```
EOF

# account FILE FIRST LAST: a pull request body that marks clauses FIRST..LAST
# of #1315 met, one line each, in the grammar clause-check reads.
account() {
    file=$1 first=$2 last=$3
    printf 'Closes #1315\n\n## Done-when accounting\n\n' > "$file"
    k=$first
    while [ "$k" -le "$last" ]; do
        printf -- '- [x] #1315.%s held by a case in the fixture\n' "$k" >> "$file"
        k=$((k + 1))
    done
}

echo "clauses reads every box from Done when to the end, folds at any level included"
ISSUE_BODY="$scratch/issue-1315.md" run clauses 1315 > "$scratch/out" 2>"$scratch/err"
check_status "clauses exits 0 on a body it can read" zero $?
if [ "$(grep -c . "$scratch/out")" -eq 11 ]; then
    passed=$((passed + 1)); echo "  ok    eleven clauses, no more and no fewer"
else
    failed=$((failed + 1)); echo "  FAIL  eleven clauses, no more and no fewer (got $(grep -c . "$scratch/out"))"
    echo "        stdout holds: $(cat "$scratch/out")"
fi
check_out "  clause 1 is the first box under Done when" "1 [ ] c1 the workspace pin test"
check_out "  clause 10 is under the h2 fold" "10 [ ] c10 the release-guide fixture"
check_out "  clause 11 is the indented box under the h2 fold" "11 [ ] c11 a relation"
check_out "  a ticked box in the issue keeps its mark" "6 [x] c6"
check_not_out "  no box above Done when" "above the Done-when heading"
check_not_out "  no box in an HTML comment" "template placeholder"
check_not_out "  no box in a fenced block" "fenced block"
check_out "  a comment closed on its own line does not hide its clause" "2 [ ] c2 the release guide"

echo "clauses drops the carriage returns of a body edited in the web form"
sed 's/$/\r/' "$scratch/issue-1315.md" > "$scratch/issue-1315-crlf.md"
ISSUE_BODY="$scratch/issue-1315-crlf.md" run clauses 1315 > "$scratch/out" 2>"$scratch/err"
if [ "$(grep -c . "$scratch/out")" -eq 11 ] && ! grep -q "$(printf '\r')" "$scratch/out"; then
    passed=$((passed + 1)); echo "  ok    eleven clauses and no carriage return"
else
    failed=$((failed + 1)); echo "  FAIL  eleven clauses and no carriage return"
fi

echo "clause-check refuses the #1460 shape: five of eleven accounted"
account "$scratch/pr-five.md" 1 5
printf -- '- [ ] #1315.6 not done yet\n' >> "$scratch/pr-five.md"
ISSUE_BODY="$scratch/issue-1315.md" PR_BODY="$scratch/pr-five.md" CLOSES_JSON='[1315]' \
    run clause-check 1460 > "$scratch/out" 2>"$scratch/err"
check_status "a Closes that accounts for five of eleven refuses" nonzero $?
for k in 6 7 8 9 10 11; do
    check_out "  it names #1315 clause $k" "#1315 clause $k:"
done
check_not_out "  it does not name clause 5, which the body marks met" "#1315 clause 5:"
if grep -qF 'clause-check: #1460 refused' "$scratch/err"; then
    passed=$((passed + 1)); echo "  ok    the refusal names the pull request, not the issue it read last"
else
    failed=$((failed + 1)); echo "  FAIL  the refusal names the pull request, not the issue it read last"
    echo "        stderr holds: $(cat "$scratch/err")"
fi
check_logged "  it reads the pull request body from the pulls endpoint" "repos/headwater-ai/headwater/pulls/1460 --jq .body"

echo "clause-check (a): every clause marked met passes, CRLF line ends included"
account "$scratch/pr-all.lf" 1 11
sed 's/$/\r/' "$scratch/pr-all.lf" > "$scratch/pr-all.md"
ISSUE_BODY="$scratch/issue-1315.md" PR_BODY="$scratch/pr-all.md" CLOSES_JSON='[1315]' \
    run clause-check 1460 > "$scratch/out" 2>"$scratch/err"
check_status "a Closes that accounts for all eleven passes" zero $?

echo "clause-check (b): nine accounted, the h2 fold left out, refuses"
account "$scratch/pr-nine.md" 1 9
ISSUE_BODY="$scratch/issue-1315.md" PR_BODY="$scratch/pr-nine.md" CLOSES_JSON='[1315]' \
    run clause-check 1460 > "$scratch/out" 2>"$scratch/err"
check_status "a reader that stops at the next h2 would pass this; it must refuse" nonzero $?
check_out "  it names clause 10" "#1315 clause 10:"
check_out "  it names clause 11" "#1315 clause 11:"

echo "clause-check (c): a Refs-only pull request closes nothing and reads no clause"
PR_BODY="$scratch/pr-five.md" CLOSES_JSON='[]' run clause-check 1460 > "$scratch/out" 2>"$scratch/err"
check_status "an empty closes list passes" zero $?
if grep -q 'issues/' "$log"; then
    failed=$((failed + 1)); echo "  FAIL  no issue body is read when nothing closes"
else
    passed=$((passed + 1)); echo "  ok    no issue body is read when nothing closes"
fi

echo "clause-check (d): clause 1 missing is not met by the line for clause 10"
account "$scratch/pr-no1.md" 2 11
ISSUE_BODY="$scratch/issue-1315.md" PR_BODY="$scratch/pr-no1.md" CLOSES_JSON='[1315]' \
    run clause-check 1460 > "$scratch/out" 2>"$scratch/err"
check_status "#1315.10 does not account for #1315.1" nonzero $?
check_out "  it names clause 1" "#1315 clause 1:"

echo "clause-check (e): a clause marked both met and unmet refuses"
account "$scratch/pr-both.md" 1 11
printf -- '- [ ] #1315.4 the search regressed after review\n' >> "$scratch/pr-both.md"
ISSUE_BODY="$scratch/issue-1315.md" PR_BODY="$scratch/pr-both.md" CLOSES_JSON='[1315]' \
    run clause-check 1460 > "$scratch/out" 2>"$scratch/err"
check_status "an unmet mark wins over a met one" nonzero $?
check_out "  it names clause 4" "#1315 clause 4:"

echo "clause-check (f): every closed issue is accounted, not only the first"
ISSUE_BODY="$scratch/issue-1315.md" PR_BODY="$scratch/pr-all.md" CLOSES_JSON='[1315,1316]' \
    run clause-check 1460 > "$scratch/out" 2>"$scratch/err"
check_status "a second closed issue with nothing accounted refuses" nonzero $?
check_out "  it names #1316 clause 1" "#1316 clause 1:"

echo "clause-check (g): an issue with no box under Done when refuses"
printf '## ELI5\n\nWords.\n\n## Done when\n\nIt works.\n' > "$scratch/issue-bare.md"
ISSUE_BODY="$scratch/issue-bare.md" PR_BODY="$scratch/pr-all.md" CLOSES_JSON='[1315]' \
    run clause-check 1460 > "$scratch/out" 2>"$scratch/err"
check_status "no clause to account for is not a pass" nonzero $?
check_out "  it says the issue has no clause" "#1315 has no clause"

# The shapes the first verify of #1485 found. The reader stopped at no h2,
# and a fixture whose later headings were all h2 or h3 and all said
# "Folded" held nothing more. So this body has an h1 fold and a later heading
# that is no fold, a heading in capitals, every task-item marker GitHub
# renders, a ``` line inside a ~~~ fence, a lone ``` inside a comment, and
# a clause whose line opens a comment. It has nine clauses, s1 to s9.
cat > "$scratch/issue-104.md" <<'EOF'
## DONE WHEN

* [ ] s1 a star bullet
+ [ ] s2 a plus bullet
1. [ ] s3 an ordered item
2) [ ] s4 an ordered item with a paren

~~~markdown
```
- [ ] a box inside a tilde fence is not a clause
~~~

- [ ] s5 after the tilde fence <!-- a closed comment --> keeps its tail
- [ ] s6 opens a comment <!-- a note
that runs on
```
-->
- [ ] s7 after a comment that held a lone backtick fence

# Folded at h1

- [ ] s8 under an h1 fold

## Review notes

- [ ] s9 under a later heading that is no fold
EOF

# account104 FILE LAST: a body that marks clauses 1..LAST of #104 met.
account104() {
    printf 'Closes #104\n\n' > "$1"
    k=1
    while [ "$k" -le "$2" ]; do
        printf -- '- [x] #104.%s held\n' "$k" >> "$1"
        k=$((k + 1))
    done
}

echo "clauses reads past any later heading, at any level, and every task-item marker"
ISSUE_BODY="$scratch/issue-104.md" run clauses 104 > "$scratch/out" 2>"$scratch/err"
if [ "$(grep -c . "$scratch/out")" -eq 9 ]; then
    passed=$((passed + 1)); echo "  ok    nine clauses"
else
    failed=$((failed + 1)); echo "  FAIL  nine clauses (got $(grep -c . "$scratch/out"))"
    echo "        stdout holds: $(cat "$scratch/out")"
fi
check_out "  a heading in capitals opens the section, and a star bullet is clause 1" "1 [ ] s1 a star bullet"
check_out "  a plus bullet is clause 2" "2 [ ] s2 a plus bullet"
check_out "  an ordered item is clause 3" "3 [ ] s3 an ordered item"
check_out "  an ordered item with a paren is clause 4" "4 [ ] s4 an ordered"
check_out "  a backtick line does not close a tilde fence" "5 [ ] s5 after the tilde fence"
check_out "  the text after a closed comment stays, and the comment goes" "5 [ ] s5 after the tilde fence  keeps its tail"
check_out "  a clause whose line opens a comment is read" "6 [ ] s6 opens a comment"
check_out "  a backtick line in a comment opens no fence" "7 [ ] s7 after a comment"
check_out "  an h1 fold does not end the section" "8 [ ] s8 under an h1 fold"
check_out "  a later heading that is no fold does not end it" "9 [ ] s9 under a later heading"
check_not_out "  no box inside the tilde fence" "inside a tilde fence"

echo "clause-check refuses when only a clause past the h1 fold is left out"
account104 "$scratch/pr-104-eight.md" 8
ISSUE_BODY="$scratch/issue-104.md" PR_BODY="$scratch/pr-104-eight.md" CLOSES_JSON='[104]' \
    run clause-check 1600 > "$scratch/out" 2>"$scratch/err"
check_status "eight of nine refuses" nonzero $?
check_out "  it names #104 clause 9" "#104 clause 9:"

echo "clause-check reads an ordered accounting line, and all nine pass"
account104 "$scratch/pr-104-all.md" 8
printf -- '9. [x] #104.9 held\n' >> "$scratch/pr-104-all.md"
ISSUE_BODY="$scratch/issue-104.md" PR_BODY="$scratch/pr-104-all.md" CLOSES_JSON='[104]' \
    run clause-check 1600 > "$scratch/out" 2>"$scratch/err"
check_status "nine of nine passes" zero $?

echo "clause-check does not read a met line inside a fence of the pull request body"
account104 "$scratch/pr-104-fenced.md" 8
printf -- '```\n- [x] #104.9 quoted, not accounted\n```\n' >> "$scratch/pr-104-fenced.md"
ISSUE_BODY="$scratch/issue-104.md" PR_BODY="$scratch/pr-104-fenced.md" CLOSES_JSON='[104]' \
    run clause-check 1600 > "$scratch/out" 2>"$scratch/err"
check_status "a fenced met line accounts for nothing" nonzero $?
check_out "  it names #104 clause 9" "#104 clause 9:"

echo "clause-check exits 1 when an issue body cannot be read, and passes nothing"
ISSUE_FAIL=1 PR_BODY="$scratch/pr-104-all.md" CLOSES_JSON='[104]' \
    run clause-check 1600 > "$scratch/out" 2>"$scratch/err"
check_status "a failed read refuses" nonzero $?
check_not_out "  it prints no pass line" "marked met"
if grep -qF 'could not read the body of #104' "$scratch/err"; then
    passed=$((passed + 1)); echo "  ok    it names the issue it could not read"
else
    failed=$((failed + 1)); echo "  FAIL  it names the issue it could not read"
    echo "        stderr holds: $(cat "$scratch/err")"
fi

echo "$passed passed; $failed failed"
[ "$failed" -eq 0 ]
