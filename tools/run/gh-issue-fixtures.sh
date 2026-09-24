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

# The fake logs every call it sees and answers the three shapes this tool
# makes: a body read, a comments read, and the state re-read `close` makes
# after asking for the close. CLOSE_STATE picks which state the second read
# answers, so both the honored and the refused close are held.
cat > "$fakebin/gh" <<'EOF'
#!/bin/sh
printf '%s\n' "$*" >> "$LOG"
case "$*" in
    *"/comments --jq .[].body"*) printf 'first comment\nsecond comment\n' ;;
    *"-X PATCH"*"-f state=closed"*) : ;;
    *"--jq .state"*) printf '%s\n' "${CLOSE_STATE:-closed}" ;;
    *"--jq .body"*) printf 'the issue body\n' ;;
    *) : ;;
esac
EOF
chmod +x "$fakebin/gh"

passed=0
failed=0

run() {
    : > "$log"
    PATH="$fakebin:$PATH" LOG="$log" CLOSE_STATE="${CLOSE_STATE:-closed}" sh "$tool" "$@"
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

echo "$passed passed; $failed failed"
[ "$failed" -eq 0 ]
