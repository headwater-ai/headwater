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
line='{"iter":1,"issue":101,"pr":9,"merge":"abc123","verdict":"PASS","proved":"the new rule fired on an injected defect","opened":[201,202],"closed":[101]}'
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

with_total='{"iter":2,"issue":102,"pr":10,"merge":"def456","verdict":"PASS","proved":"x","opened":[],"closed":[102,104,105],"net_delta":-1}'
sh "$tool" log "$dir" "$with_total" 2>"$scratch/err"; status=$?
same 'a line that stores a total is refused' 1 "$status"
case $(cat "$scratch/err") in
    *'HW-DR-0049'*) pass '  and the refusal cites the ruling' ;;
    *) fail '  and the refusal cites the ruling' "$(cat "$scratch/err")" ;;
esac

sh "$tool" log "$dir" 'not json' 2>"$scratch/err"; status=$?
same 'a line that is not JSON is refused' 1 "$status"

as_count='{"iter":2,"issue":102,"pr":10,"merge":"def456","verdict":"PASS","proved":"x","opened":0,"closed":1}'
sh "$tool" log "$dir" "$as_count" 2>"$scratch/err"; status=$?
same 'a line that stores opened or closed as a count is refused' 1 "$status"
case $(cat "$scratch/err") in
    *'array of issue numbers'*) pass '  and the refusal names the shape' ;;
    *) fail '  and the refusal names the shape' "$(cat "$scratch/err")" ;;
esac
same '  and nothing was appended' 1 "$(wc -l < "$dir/log.jsonl" | tr -d ' ')"

printf '\n# tail and net\n'
sh "$tool" log "$dir" '{"iter":2,"issue":102,"pr":10,"merge":"def456","verdict":"PASS","proved":"x","opened":[],"closed":[102,104,105]}' >/dev/null 2>&1
sh "$tool" log "$dir" '{"iter":3,"issue":103,"pr":11,"merge":"","verdict":"FAIL","proved":"a regression stayed green","opened":[203],"closed":[]}' >/dev/null 2>&1
same 'tail returns the last n lines' 2 "$(sh "$tool" tail "$dir" 2 | wc -l | tr -d ' ')"
same '  and the last of them is the last written' 103 "$(sh "$tool" tail "$dir" 1 | jq -r .issue)"
same 'net is derived from the lines, never read from one' 'iterations 3  opened 3  closed 4  net -1' "$(sh "$tool" net "$dir")"
printf '%s\n' '{"iter":4,"issue":106,"pr":12,"merge":"x","verdict":"PASS","proved":"x","opened":0,"closed":1}' >> "$dir/log.jsonl"
same '  and an older line that stored counts is still counted' 'iterations 4  opened 3  closed 5  net -2' "$(sh "$tool" net "$dir")"

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

printf '\n# claims: a footprint quoted as one argument\n'
joined=$(sh "$tool" start joined 2>/dev/null)
sh "$tool" claim "$joined" 40 issue-40 '.headwater/export.json docs/decisions/README.md' >/dev/null 2>"$scratch/err"; status=$?
same 'an artifact holding whitespace is refused' 2 "$status"
case $(cat "$scratch/err") in
    *'its own argument'*) pass '  and the refusal says to pass each artifact separately' ;;
    *) fail '  and the refusal says to pass each artifact separately' "$(cat "$scratch/err")" ;;
esac
same '  and nothing was claimed' 0 "$(ls "$joined/claims/artifacts" 2>/dev/null | wc -l | tr -d ' ')"

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
merged='{"iter":1,"issue":1,"pr":2,"merge":"abc","verdict":"merged","proved":"p","opened":[],"closed":[1]}'

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
rm -f "$HEADWATER_USAGE_DIR/cccc3333-broken.json"

printf '\n# session: one line per parent session, and the sample takes the live one\n'
# A parent restarted by `tools/run/supervise.sh` is a new session under the
# same run. `start` wrote the first line, and each resumed session appends its
# own, so the review hook exempts every one of them and `usage` samples the one
# that is live. Before this, `sample` matched the whole file as one prefix and
# fell back to the freshest session, which is a different session's figures.
multi=$(CLAUDE_JOB_DIR=/jobs/aaaa1111 sh "$tool" start usage-resumed 2>/dev/null)
CLAUDE_JOB_DIR=/jobs/bbbb2222 sh "$tool" session "$multi" >/dev/null 2>&1
CLAUDE_JOB_DIR=/jobs/bbbb2222 sh "$tool" session "$multi" >/dev/null 2>&1
same 'session appends the resumed session once, after the first' 'aaaa1111 bbbb2222' "$(paste -sd' ' "$multi/parent.session")"
env -u CLAUDE_JOB_DIR sh "$tool" session "$multi" >/dev/null 2>&1; status=$?
same '  and outside the harness it is refused with exit 2' 2 "$status"
plant aaaa1111-parent 600 8 R2 52
plant bbbb2222-other 500 20 R2 60
: > "$multi/usage.jsonl"
sh "$tool" log "$multi" "$merged" >/dev/null
same '  and a sample under two sessions takes the last line, the live session, over a fresher first' 'parent bbbb2222' \
    "$(jq -r '"\(.from) \(.session)"' "$multi/usage.jsonl")"

printf '\n# resume: next skips what is held, and drain reaches every verb a parent already calls\n'
# The decisive case for #1275. A resumed parent learns what to dispatch from
# `next`. Before `next` existed it read queue.md by hand, where nothing said
# that #10 was claimed or that #11 already had a built branch, and `claim` on
# a held issue prints `(already held)` and exits 0, so nothing refused the
# re-dispatch either.
resume=$(sh "$tool" start resume 2>/dev/null)
printf '# Queue, run resume\n\nOrder: a fixture.\n\n1. #10 First | none | x\n2. #11 Second | none | x\n3. #12 Third | none | x\n' > "$resume/queue.md"
sh "$tool" claim "$resume" 10 issue-10 a-file >/dev/null 2>&1
sh "$tool" stage "$resume" 11 built branch=issue-11 pr=911 rounds=0 build=/s/build.md >/dev/null 2>"$scratch/err"; status=$?
same 'stage writes a checkpoint and exits 0' 0 "$status"
same 'next skips a claimed issue and an issue with a handover' 12 "$(sh "$tool" next "$resume" 2>&1)"
same '  and the handover holds the stage and each field, one per line' \
    'stage built|branch issue-11|pr 911|rounds 0|build /s/build.md' \
    "$(paste -sd'|' "$resume/handover/11" 2>&1)"
sh "$tool" stage "$resume" 11 verified-fail branch=issue-11 pr=911 rounds=1 verify=/s/verify.md attacks=a1,a2 >/dev/null 2>&1
same '  and the last write wins, since one actor owns the stage of an issue' 'stage verified-fail' "$(head -n 1 "$resume/handover/11" 2>&1)"
same '  and no temp file is left beside it' 1 "$(ls -A "$resume/handover" 2>/dev/null | wc -l | tr -d ' ')"
sh "$tool" stage "$resume" 11 merged >/dev/null 2>&1; status=$?
same 'a stage outside the five is refused with exit 2' 2 "$status"
same '  and the checkpoint is unchanged' 'stage verified-fail' "$(head -n 1 "$resume/handover/11" 2>&1)"
sh "$tool" stage "$resume" 11 built colour=red >/dev/null 2>&1; status=$?
same 'a field outside the eight is refused with exit 2' 2 "$status"
sh "$tool" stage "$resume" 11 built branch >/dev/null 2>&1; status=$?
same 'a field with no value is refused with exit 2' 2 "$status"
sh "$tool" stage "$resume" 11 verified-fail >"$scratch/out" 2>&1
sh "$tool" claim "$resume" 13 issue-13 b-file >>"$scratch/out" 2>&1
same 'without a drain file, stage and claim print no DRAIN' 0 "$(grep -c '^DRAIN$' "$scratch/out")"
sh "$tool" release "$resume" 13 >/dev/null 2>&1
printf '{"iter":1,"issue":12,"pr":3,"merge":"m","verdict":"refused","proved":"p","opened":[],"closed":[]}\n' >> "$resume/log.jsonl"
same 'next skips an issue the log records, and prints EMPTY when nothing is left' EMPTY "$(sh "$tool" next "$resume" 2>&1)"
: > "$resume/log.jsonl"

touch "$resume/drain"
same 'in drain, next prints DRAIN instead of an issue' DRAIN "$(sh "$tool" next "$resume" 2>&1)"
same '  and claim prints DRAIN and nothing else' DRAIN "$(sh "$tool" claim "$resume" 12 issue-12 c-file 2>&1)"
same '  and claims nothing, so a later session can still dispatch the issue' 1 \
    "$([ -e "$resume/claims/issues/12" ] || [ -e "$resume/claims/artifacts/c-file" ]; echo $?)"
same '  and stage still writes its checkpoint, then prints DRAIN' 'DRAIN stage verified-pass' \
    "$(sh "$tool" stage "$resume" 11 verified-pass 2>&1) $(head -n 1 "$resume/handover/11" 2>&1)"
rm -f "$resume/drain"
same 'once drain is gone, next hands out the issue again' 12 "$(sh "$tool" next "$resume" 2>&1)"

printf '\n# next skips what the run ruled out, and a ruling line the owner has not answered\n'
# Verify-1 of PR #1279 ran `next` on a copy of run 20260927-0443 and it
# printed #927, which decisions.md:2055 says stays gated. Nothing in claims/,
# handover/ or the log records a gate, a deferral or a refusal, and the queue
# line was marked `ruling` with no OWNER line for it.
ruled=$(sh "$tool" start ruled 2>/dev/null)
printf '# Queue\n\n1. #20 Gated | none | x | wide | ruling (embedding path)\n2. #21 Answered | none | x | wide | ruling\n3. #22 Deferred | none | x | narrow\n4. #23 Refused | none | x | narrow\n5. #24 Open | none | x | narrow\n' > "$ruled/queue.md"
printf -- '- 2026-09-27 — OWNER (2026-09-27) #21: build it.\n' >> "$ruled/decisions.md"
same 'a ruling line with no OWNER line is skipped, and one the owner answered is not' 21 "$(sh "$tool" next "$ruled" 2>&1)"
sh "$tool" claim "$ruled" 21 issue-21 f21 >/dev/null 2>&1
sh "$tool" rule "$ruled" 22 deferred 'owner deferred it for this run' >/dev/null 2>"$scratch/err"; status=$?
same 'rule exits 0' 0 "$status"
same '  and appends one RULED line to decisions.md' 1 "$(grep -c '^- [0-9-]* — RULED #22 deferred: owner deferred it for this run$' "$ruled/decisions.md")"
sh "$tool" rule "$ruled" 23 refused 'kind 2, better done another way' >/dev/null 2>&1
same 'next skips a deferred and a refused issue' 24 "$(sh "$tool" next "$ruled" 2>&1)"
sh "$tool" rule "$ruled" 22 open 'the owner lifted the deferral' >/dev/null 2>&1
same '  and the last ruling of an issue wins, so open lifts a deferral' 22 "$(sh "$tool" next "$ruled" 2>&1)"
sh "$tool" rule "$ruled" 20 open 'answered in the session' >/dev/null 2>&1
same '  and open also answers a ruling line' 20 "$(sh "$tool" next "$ruled" 2>&1)"
sh "$tool" rule "$ruled" 20 gated 'waits on HW-DR-0064' >/dev/null 2>&1
same '  and gated takes it out again' 22 "$(sh "$tool" next "$ruled" 2>&1)"
sh "$tool" rule "$ruled" 20 maybe 'x' >/dev/null 2>&1; status=$?
same 'a ruling outside gated, deferred, refused and open is refused with exit 2' 2 "$status"
sh "$tool" rule "$ruled" 20 gated >/dev/null 2>&1; status=$?
same 'a ruling with no reason is refused with exit 2' 2 "$status"

printf '\n# drain while an adjudicate is in flight: the claim waits for the next session\n'
# Verify-1 of PR #1279: the parent claims on an adjudicate report, which can
# arrive after drain has started. `claim` then writes nothing. The parent
# writes `stage adjudicated` with the note and the footprint, so the next
# session claims it and dispatches hw-iterate, and the issue is not
# adjudicated twice. `resume` says which, for every open handover.
inflight=$(sh "$tool" start inflight 2>/dev/null)
printf '# Queue\n\n1. #30 A | none | x\n2. #31 B | none | x\n3. #32 C | none | x\n4. #33 D | none | x\n5. #34 E | none | x\n' > "$inflight/queue.md"
touch "$inflight/drain"
same 'in drain the claim on an adjudicate report writes nothing' 'DRAIN 1' \
    "$(sh "$tool" claim "$inflight" 30 issue-30 f30 g30 2>&1) $([ -e "$inflight/claims/issues/30" ]; echo $?)"
same '  and stage adjudicated keeps the note and the footprint' 'DRAIN' \
    "$(sh "$tool" stage "$inflight" 30 adjudicated note=/s/adjudication.md branch=issue-30 footprint=f30,g30 2>&1)"
sh "$tool" stage "$inflight" 31 built branch=issue-31 pr=31 >/dev/null 2>&1
sh "$tool" stage "$inflight" 32 verified-pass branch=issue-32 pr=32 >/dev/null 2>&1
sh "$tool" stage "$inflight" 33 ruled-merge branch=issue-33 pr=33 >/dev/null 2>&1
sh "$tool" stage "$inflight" 34 verified-fail branch=issue-34 pr=34 verify=/s/v.md >/dev/null 2>&1
rm -f "$inflight/drain"
same 'the next session is not handed the adjudicated issue by next' EMPTY "$(sh "$tool" next "$inflight" 2>&1)"
sh "$tool" resume "$inflight" > "$scratch/resume" 2>&1; status=$?
same 'resume exits 0 and prints one line per open handover' '0 5' "$status $(wc -l < "$scratch/resume" | tr -d ' ')"
same '  an unclaimed adjudicated issue is claimed with its footprint, then iterated' \
    '30 adjudicated claim f30 g30' "$(grep '^30 ' "$scratch/resume")"
same '  a built or failed issue goes to a fresh hw-iterate' '31 built iterate|34 verified-fail iterate' \
    "$(grep -E '^3[14] ' "$scratch/resume" | paste -sd'|')"
same '  a pass is the parent'"'"'s to rule, and a merge ruling goes to the integrator' '32 verified-pass rule|33 ruled-merge integrate' \
    "$(grep -E '^3[23] ' "$scratch/resume" | paste -sd'|')"
sh "$tool" claim "$inflight" 30 issue-30 f30 g30 >/dev/null 2>&1
same '  and once claimed, the adjudicated issue goes to hw-iterate' '30 adjudicated iterate' \
    "$(sh "$tool" resume "$inflight" 2>&1 | grep '^30 ')"
printf '{"iter":1,"issue":33,"pr":33,"merge":"abc1234","verdict":"MERGE","proved":"p","opened":[],"closed":[33]}\n' >> "$inflight/log.jsonl"
same '  and a handover the log already records is not resumed' 0 "$(sh "$tool" resume "$inflight" 2>&1 | grep -c '^33 ')"
touch "$inflight/drain"
same '  and in drain resume prints DRAIN and nothing else' DRAIN "$(sh "$tool" resume "$inflight" 2>&1)"
rm -f "$inflight/drain"

printf '\n%s passed, %s failed\n' "$passed" "$failed"
[ "$failed" -eq 0 ]
