#!/bin/sh
# What holds `tools/run/run-dir.sh`, the run directory of a build-order run.
#
# The tool makes one directory and refuses three things: a second start under
# one id, a log line missing a key, and a log line that stores a total. Each
# refusal is provoked below, and so is each pass, because a ledger tool that
# refused nothing would be `cat >>` with a longer name. The derived total is
# held against the arithmetic of the lines that were written, which is the
# whole of what [HW-PD-0005] and [HW-DR-0049] ask of a ledger.
#
# Run it from anywhere:
#     sh tools/run/run-dir-fixtures.sh
#
# It needs `jq` and nothing else, it points the tool at a scratch root through
# `HEADWATER_RUN_ROOT`, and it writes nothing under the git common dir or the
# checkout.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
tool="$root/tools/run/run-dir.sh"

if ! command -v jq >/dev/null 2>&1; then
    echo "no \`jq\` on the path, and the tool under test reads JSON lines with it." >&2
    exit 3
fi

scratch=$(mktemp -d) || exit 1
trap 'rm -rf "$scratch"' EXIT HUP INT TERM
export HEADWATER_RUN_ROOT="$scratch/runs"
# No host usage snapshot reaches a fixture unless a case plants one.
export HEADWATER_USAGE_DIR="$scratch/no-usage"

passed=0
failed=0

pass() {
    printf 'ok   %s\n' "$1"
    passed=$((passed + 1))
}

fail() {
    printf 'FAIL %s\n  %s\n' "$1" "$2"
    failed=$((failed + 1))
}

same() {
    name=$1 want=$2 got=$3
    if [ "$want" = "$got" ]; then
        pass "$name"
    else
        fail "$name" "expected \`$want\`, got \`$got\`"
    fi
}

printf '# start\n'
dir=$(sh "$tool" start first 2>"$scratch/err"); status=$?
same 'start exits 0 and prints the directory' "0 $scratch/runs/first" "$status $dir"
for file in doctrine.md log.jsonl findings.jsonl lessons.md decisions.md; do
    if [ -f "$dir/$file" ]; then
        pass "  and $file is there"
    else
        fail "  and $file is there" 'it is not'
    fi
done
if cmp -s "$dir/doctrine.md" "$root/.claude/run/doctrine.md"; then
    pass '  and the doctrine is the checked-in doctrine, byte for byte'
else
    fail '  and the doctrine is the checked-in doctrine, byte for byte' 'it differs'
fi
same '  and the log starts empty' 0 "$(wc -c < "$dir/log.jsonl" | tr -d ' ')"

sh "$tool" start first >/dev/null 2>"$scratch/err"; status=$?
same 'a second start under the same id is refused' 1 "$status"
case $(cat "$scratch/err") in
    *'never reused'*) pass '  and the refusal says a run directory is never reused' ;;
    *) fail '  and the refusal says a run directory is never reused' "$(cat "$scratch/err")" ;;
esac

printf '\n# log\n'
line='{"iter":1,"issue":101,"pr":9,"merge":"abc123","verdict":"PASS","proved":"the new rule fired on an injected defect","opened":2,"closed":1}'
sh "$tool" log "$dir" "$line" 2>"$scratch/err"; status=$?
same 'a line with every key is appended' 0 "$status"
same '  and the log holds one line' 1 "$(wc -l < "$dir/log.jsonl" | tr -d ' ')"
same '  and it round-trips through jq' 101 "$(jq -r .issue "$dir/log.jsonl")"

sh "$tool" log "$dir" '{"iter":2,"issue":102}' 2>"$scratch/err"; status=$?
same 'a line missing a key is refused' 1 "$status"
case $(cat "$scratch/err") in
    *'lacks `pr`'*) pass '  and the refusal names the first missing key' ;;
    *) fail '  and the refusal names the first missing key' "$(cat "$scratch/err")" ;;
esac
same '  and nothing was appended' 1 "$(wc -l < "$dir/log.jsonl" | tr -d ' ')"

with_total='{"iter":2,"issue":102,"pr":10,"merge":"def456","verdict":"PASS","proved":"x","opened":0,"closed":3,"net_delta":-1}'
sh "$tool" log "$dir" "$with_total" 2>"$scratch/err"; status=$?
same 'a line that stores a total is refused' 1 "$status"
case $(cat "$scratch/err") in
    *'HW-DR-0049'*) pass '  and the refusal cites the ruling' ;;
    *) fail '  and the refusal cites the ruling' "$(cat "$scratch/err")" ;;
esac

sh "$tool" log "$dir" 'not json' 2>"$scratch/err"; status=$?
same 'a line that is not JSON is refused' 1 "$status"

printf '\n# tail and net\n'
sh "$tool" log "$dir" '{"iter":2,"issue":102,"pr":10,"merge":"def456","verdict":"PASS","proved":"x","opened":0,"closed":3}' >/dev/null 2>&1
sh "$tool" log "$dir" '{"iter":3,"issue":103,"pr":11,"merge":"","verdict":"FAIL","proved":"a regression stayed green","opened":1,"closed":0}' >/dev/null 2>&1
same 'tail returns the last n lines' 2 "$(sh "$tool" tail "$dir" 2 | wc -l | tr -d ' ')"
same '  and the last of them is the last written' 103 "$(sh "$tool" tail "$dir" 1 | jq -r .issue)"
same 'net is derived from the lines, never read from one' 'iterations 3  opened 3  closed 4  net -1' "$(sh "$tool" net "$dir")"

printf '\n# a second run seeds its prose from the first\n'
printf 'A lesson the first run learned.\n' > "$dir/lessons.md"
second=$(sh "$tool" start second 2>/dev/null)
same 'the lesson is carried into the next run' 'A lesson the first run learned.' "$(cat "$second/lessons.md")"
same '  and its log starts empty' 0 "$(wc -c < "$second/log.jsonl" | tr -d ' ')"

printf '\n# import\n'
cat > "$scratch/ledger.md" <<'EOF'
# Headwater build order — ledger

## Lessons

- One lesson.
- Another.

## Log

| 1 | #1 | merged |

## Open findings

- A finding.

## Decisions needing an owner

- A decision.

# Run 23 — notes

## Lessons

Not the ledger's Lessons section.
EOF
third=$(sh "$tool" start third 2>/dev/null)
sh "$tool" import "$scratch/ledger.md" "$third" >"$scratch/out" 2>&1; status=$?
same 'import exits 0' 0 "$status"
same '  and the Lessons section reaches lessons.md' 2 "$(grep -c '^- ' "$third/lessons.md")"
same '  and a later heading of the same name is not taken' 0 "$(grep -c 'Not the ledger' "$third/lessons.md")"
same '  and the decisions reach decisions.md' '- A decision.' "$(grep '^- ' "$third/decisions.md")"
same '  and the old log is kept whole rather than parsed' 1 "$(grep -c '^|' "$third/log-imported.md")"

printf '\n# claims: two claimants over one artifact\n'
run=$(sh "$tool" start claims 2>/dev/null)
first=$(sh "$tool" claim "$run" 41 issue-41 .headwater/export.json docs/decisions/README.md 2>&1); status=$?
same 'the first claimant takes both artifacts' 0 "$status"
same '  and says so, one line each' 2 "$(printf '%s\n' "$first" | grep -c '^CLAIMED:')"
second=$(sh "$tool" claim "$run" 42 issue-42 .headwater/export.json engine/crates/census/fixtures/corpus.census 2>&1); status=$?
same 'the second claimant exits 0, because a claim orders and never refuses' 0 "$status"
same '  and takes the artifact nobody held' 1 "$(printf '%s\n' "$second" | grep -c '^CLAIMED: engine')"
same '  and reports who holds the other' 'HELD: .headwater/export.json by #41' "$(printf '%s\n' "$second" | grep '^HELD:')"
same '  and ends with WAITS-ON naming the holder' 'WAITS-ON: 41' "$(printf '%s\n' "$second" | grep '^WAITS-ON:')"
same '  and the wait is recorded on the issue claim' 1 "$(grep -c '^WAITS-ON: 41' "$run/claims/issues/42")"
empties=0
for owner in "$run"/claims/artifacts/*; do
    [ -s "$owner" ] || empties=$((empties + 1))
done
same 'no claim is empty' 0 "$empties"
same '  and the artifact list shows three claims' 3 "$(sh "$tool" claims "$run" | wc -l | tr -d ' ')"
same '  and a slash in an artifact name is folded, not nested' 1 "$(ls "$run"/claims/artifacts/docs-decisions-README.md 2>/dev/null | wc -l | tr -d ' ')"
same '  and a leading dot is folded, so the glob that frees it can see it' 1 "$(ls "$run"/claims/artifacts/headwater-export.json 2>/dev/null | wc -l | tr -d ' ')"

again=$(sh "$tool" claim "$run" 41 issue-41 .headwater/export.json 2>&1)
same 'a holder re-claiming its own artifact is not made to wait on itself' 0 "$(printf '%s\n' "$again" | grep -c '^WAITS-ON')"

printf '\n# release, and the second claimant goes through\n'
same 'release frees what the issue held and nothing else' 'RELEASED: 2 artifacts held by #41' "$(sh "$tool" release "$run" 41)"
same '  and the other claim stands' 'engine/crates/census/fixtures/corpus.census #42 issue-42' "$(sh "$tool" claims "$run")"
third=$(sh "$tool" claim "$run" 42 issue-42 .headwater/export.json 2>&1)
same '  and the released artifact is taken by the one that waited' 'CLAIMED: .headwater/export.json' "$(printf '%s\n' "$third" | grep '^CLAIMED')"

printf '\n# end: a stale claim from a run that will never finish does not block a fresh claimant\n'
# The decisive case. Issue 41 claims an artifact and the run that held it ends
# without ever releasing -- the crash this issue exists for. Before `end`
# exists to collect it, a second claimant is made to wait on a run that will
# never finish, which is the bug: assert that first, so this fails for the
# issue's own reason rather than for a typo.
ended=$(sh "$tool" start ended 2>/dev/null)
sh "$tool" claim "$ended" 41 issue-41 shared-artifact >/dev/null 2>&1
still_waits=$(sh "$tool" claim "$ended" 42 issue-42 shared-artifact 2>&1)
same 'before end, a fresh claimant on the abandoned artifact is made to wait' 'WAITS-ON: 41' "$(printf '%s\n' "$still_waits" | grep '^WAITS-ON:')"

same 'end exits 0 and reports how many claims it dropped' 'ENDED: '"$ended"', 1 claims dropped' "$(sh "$tool" end "$ended" 2>&1)"
same '  and the claim file is gone' 0 "$(find "$ended/claims" -type f 2>/dev/null | wc -l | tr -d ' ')"
after=$(sh "$tool" claim "$ended" 42 issue-42 shared-artifact 2>&1)
same '  and a fresh claimant on the same artifact is no longer made to wait' 0 "$(printf '%s\n' "$after" | grep -c '^WAITS-ON:')"
same '  and takes the artifact outright' 'CLAIMED: shared-artifact' "$(printf '%s\n' "$after" | grep '^CLAIMED')"
same '  and lessons.md and decisions.md survive end, unlike claims' 0 "$([ -e "$ended/lessons.md" ] && [ -e "$ended/decisions.md" ]; echo $?)"

bare=$(sh "$tool" start ended-bare 2>/dev/null)
same 'end on a directory with no claims subtree at all reports zero and does not fail' 'ENDED: '"$bare"', 0 claims dropped' "$(sh "$tool" end "$bare" 2>&1)"

# This case is the reason the claim is a file and not a directory. On a host
# whose `mkdir` is uutils coreutils, two racing `mkdir` calls on one path both
# succeeded in 17 of 20 races while every sequential case above passed. A
# primitive that stops being atomic is reported here rather than trusted.
printf '\n# concurrency: two claimants racing for one artifact, ten times\n'
lost=0
for round in 1 2 3 4 5 6 7 8 9 10; do
    race=$(sh "$tool" start "race-$round" 2>/dev/null)
    sh "$tool" claim "$race" 1 a x >"$race/a.out" 2>&1 &
    sh "$tool" claim "$race" 2 b x >"$race/b.out" 2>&1 &
    wait
    claimed=$(cat "$race/a.out" "$race/b.out" | grep -c '^CLAIMED: x$')
    waited=$(cat "$race/a.out" "$race/b.out" | grep -c '^WAITS-ON:')
    [ "$claimed" -eq 1 ] && [ "$waited" -eq 1 ] || lost=$((lost + 1))
done
same 'in every race exactly one claimant owns and exactly one waits' 0 "$lost"

printf '\n# usage: a sample per start, log and end, and the figures derived from them\n'
bare=$(sh "$tool" start usage-bare 2>/dev/null)
same 'a host with no usage snapshot records no sample' 0 "$( [ -s "$bare/usage.jsonl" ] && echo 1 || echo 0)"
same '  and usage says so rather than failing' "0 no usage samples" \
    "$(sh "$tool" usage "$bare" >"$scratch/out" 2>&1; echo $?) $(cut -c1-16 "$scratch/out")"

export HEADWATER_USAGE_DIR="$scratch/usage"
mkdir -p "$HEADWATER_USAGE_DIR"
# plant <session id> <last_activity> <5h %> <5h resets_at> <7d %>
plant() {
    printf '{"session_id":"%s","last_activity":%s,"five_hour":{"used_percentage":%s,"resets_at":"%s"},"seven_day":{"used_percentage":%s,"resets_at":"W"}}\n' \
        "$1" "$2" "$3" "$4" "$5" > "$HEADWATER_USAGE_DIR/$1.json"
}
merged='{"iter":1,"issue":1,"pr":2,"merge":"abc","verdict":"merged","proved":"p","opened":0,"closed":1}'

plant aaaa1111-parent 100 10 R1 40
plant bbbb2222-other 200 99 R1 99
run=$(CLAUDE_JOB_DIR=/jobs/aaaa1111 sh "$tool" start usage-parent 2>/dev/null)
same 'start takes the parent session over a fresher one' "parent aaaa1111 10" \
    "$(jq -r '"\(.from) \(.session) \(.five_hour.used_percentage)"' "$run/usage.jsonl")"
plant aaaa1111-parent 300 30 R1 46
sh "$tool" log "$run" "$merged" >/dev/null
plant aaaa1111-parent 400 8 R2 52
sh "$tool" log "$run" "$merged" >/dev/null
sh "$tool" end "$run" > "$scratch/ended"
same 'start, two logs and end leave four samples' 4 "$(wc -l < "$run/usage.jsonl" | tr -d ' ')"
same '  and end prints the usage after its own line' 4 "$(wc -l < "$scratch/ended" | tr -d ' ')"
sh "$tool" usage "$run" > "$scratch/out"
same '  the 5-hour rise counts across a reset as a floor' \
    '5-hour  spent 28+ (1 reset) points, now at 8%, 14 per closed issue, room for 6 more' \
    "$(sed -n 2p "$scratch/out")"
same '  the 7-day rise is divided by the issues closed' \
    '7-day   spent 12 points, now at 52%, 6 per closed issue, room for 8 more' \
    "$(sed -n 3p "$scratch/out")"

plant bbbb2222-other 500 99 R2 99
loose=$(sh "$tool" start usage-freshest 2>/dev/null)
same 'without parent.session a sample takes the freshest session' "freshest bbbb2222" \
    "$(jq -r '"\(.from) \(.session)"' "$loose/usage.jsonl")"

printf 'not json' > "$HEADWATER_USAGE_DIR/cccc3333-broken.json"
sh "$tool" log "$loose" "$merged" >/dev/null 2>"$scratch/err"; status=$?
same 'a broken snapshot never fails the ledger write, and prints nothing' "0 0 1" \
    "$status $(wc -c < "$scratch/err" | tr -d ' ') $(wc -l < "$loose/log.jsonl" | tr -d ' ')"

printf '\n%s passed, %s failed\n' "$passed" "$failed"
[ "$failed" -eq 0 ]
