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

# The main checkout the stale check judges (#1811). The tool judges its own
# repository by default, and the host's checkout can be any number of commits
# behind, so every case points it at a scratch clone. A bare origin, a clone
# that stays level with it, and three more: one behind by one commit, one
# ahead by a local-only commit, and one with no `origin/main` at all.
gitc() { git -c user.name=t -c user.email=t@t -c init.defaultBranch=main "$@"; }
repos="$scratch/git"
mkdir -p "$repos"
gitc init -q --bare "$repos/origin.git"
gitc clone -q "$repos/origin.git" "$repos/level" 2>/dev/null
gitc -C "$repos/level" commit -q --allow-empty -m first
gitc -C "$repos/level" branch -q -M main
gitc -C "$repos/level" push -q -u origin main 2>/dev/null
gitc clone -q -b main "$repos/origin.git" "$repos/behind" 2>/dev/null
gitc clone -q -b main "$repos/origin.git" "$repos/ahead" 2>/dev/null
gitc -C "$repos/ahead" commit -q --allow-empty -m local-only
gitc clone -q -b main "$repos/origin.git" "$repos/other" 2>/dev/null
gitc -C "$repos/other" commit -q --allow-empty -m second
gitc -C "$repos/other" push -q origin HEAD:main 2>/dev/null
gitc -C "$repos/behind" fetch -q origin
gitc -C "$repos/behind" worktree add -q "$repos/behind-wt" -b wt 2>/dev/null
gitc init -q "$repos/no-origin"
gitc -C "$repos/no-origin" commit -q --allow-empty -m first
stale_line="STALE: main checkout at $(git -C "$repos/behind" rev-parse --short HEAD), origin/main at $(git -C "$repos/behind" rev-parse --short origin/main), 1 commits behind. Run: git -C $repos/behind merge --ff-only origin/main"
export HW_RUN_CHECKOUT="$repos/level"
unset HW_RUN_ALLOW_STALE

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

# q <run> <issue> [field=value]...: one queue entry through the verb, with
# the fields a case names and a default for every other field `queue` needs.
# `queue` refuses a field named twice, so a default is added only where the
# case names none. No case types a queue.
q() {
    qrun=$1 n=$2
    shift 2
    for default in "title=Issue $n" milestone=none reader=x artifacts=x scope=narrow; do
        named=0
        for pair in "$@"; do [ "${pair%%=*}" = "${default%%=*}" ] && named=1; done
        [ "$named" = 1 ] || set -- "$@" "$default"
    done
    sh "$tool" queue "$qrun" "$n" "$@" >/dev/null 2>"$scratch/qerr" ||
        fail "the queue verb took #$n" "$(cat "$scratch/qerr")"
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

printf '\n# start: a main checkout behind origin/main is refused (#1811)\n'
# Under a root of their own, because `start` seeds a run from the newest run
# beside it, and a later case reads what it seeded.
out=$(HW_RUN_CHECKOUT="$repos/behind" HEADWATER_RUN_ROOT="$scratch/stale-runs" sh "$tool" start stale1 2>"$scratch/err"); status=$?
same 'start on a checkout one commit behind origin/main exits 1' 1 "$status"
same '  and prints the one STALE line' "$stale_line" "$out"
same '  and writes no run directory' absent "$([ -e "$scratch/stale-runs/stale1" ] && echo present || echo absent)"
out=$(HW_RUN_CHECKOUT="$repos/behind-wt" HEADWATER_RUN_ROOT="$scratch/stale-runs" sh "$tool" start stale2 2>/dev/null); status=$?
same 'a worktree of that checkout judges the main checkout, not itself' "1 $stale_line" "$status $out"
out=$(HW_RUN_CHECKOUT="$repos/level" HEADWATER_RUN_ROOT="$scratch/stale-runs" sh "$tool" start level1 2>/dev/null); status=$?
same 'start on a checkout level with origin/main exits 0, with no STALE line' "0 $scratch/stale-runs/level1" "$status $out"
out=$(HW_RUN_CHECKOUT="$repos/ahead" HEADWATER_RUN_ROOT="$scratch/stale-runs" sh "$tool" start ahead1 2>/dev/null); status=$?
same 'start on a checkout ahead of origin/main and behind it by none exits 0, with no STALE line' "0 $scratch/stale-runs/ahead1" "$status $out"
out=$(HW_RUN_CHECKOUT="$repos/no-origin" HEADWATER_RUN_ROOT="$scratch/stale-runs" sh "$tool" start noorigin1 2>/dev/null); status=$?
same 'start on a checkout with no origin/main exits 0, silent' "0 $scratch/stale-runs/noorigin1" "$status $out"
out=$(HW_RUN_ALLOW_STALE=1 HW_RUN_CHECKOUT="$repos/behind" HEADWATER_RUN_ROOT="$scratch/stale-runs" sh "$tool" start stale3 2>/dev/null); status=$?
same 'HW_RUN_ALLOW_STALE=1 turns the refusal into a warning with exit 0' 0 "$status"
same '  and the STALE line comes first' "$stale_line" "$(printf '%s\n' "$out" | head -n 1)"
same '  and the run directory is the last line' "$scratch/stale-runs/stale3" "$(printf '%s\n' "$out" | tail -n 1)"
same '  and the run directory is made' present "$([ -f "$scratch/stale-runs/stale3/log.jsonl" ] && echo present || echo absent)"

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
q "$resume" 10; q "$resume" 11; q "$resume" 12
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

printf '\n# mid-run, claim, stage and next print STALE first and do what they would have done (#1811)\n'
# A parent that never handed over to supervise.sh meets the line on a call it
# already makes. The run was made level, and the checkout fell behind later.
mid=$(sh "$tool" start mid 2>/dev/null)
q "$mid" 30; q "$mid" 31
out=$(HW_RUN_CHECKOUT="$repos/behind" sh "$tool" next "$mid" 2>&1); status=$?
same 'next prints the STALE line first and exits 0' "0 $stale_line" "$status $(printf '%s\n' "$out" | head -n 1)"
same '  and hands out the same issue on the line after it' '30|2' "$(printf '%s\n' "$out" | tail -n 1)|$(printf '%s\n' "$out" | wc -l | tr -d ' ')"
out=$(HW_RUN_CHECKOUT="$repos/behind" sh "$tool" claim "$mid" 30 issue-30 mid-file 2>&1); status=$?
same 'claim prints the STALE line first and exits 0' "0 $stale_line" "$status $(printf '%s\n' "$out" | head -n 1)"
same '  and then its own output' 'CLAIMED: mid-file' "$(printf '%s\n' "$out" | sed -n 2p)"
same '  and the claim is written' present "$([ -e "$mid/claims/artifacts/mid-file" ] && echo present || echo absent)"
out=$(HW_RUN_CHECKOUT="$repos/behind" sh "$tool" stage "$mid" 31 built branch=issue-31 pr=31 2>&1); status=$?
same 'stage prints the STALE line, and nothing else outside drain, and exits 0' "0 $stale_line" "$status $out"
same '  and the handover holds the stage' 'stage built' "$(head -n 1 "$mid/handover/31" 2>&1)"
touch "$mid/drain"
same 'in drain, next prints STALE, then DRAIN' "$stale_line|DRAIN" "$(HW_RUN_CHECKOUT="$repos/behind" sh "$tool" next "$mid" 2>&1 | paste -sd'|')"
same '  and stage prints STALE, then DRAIN' "$stale_line|DRAIN" "$(HW_RUN_CHECKOUT="$repos/behind" sh "$tool" stage "$mid" 31 verified-fail 2>&1 | paste -sd'|')"
same '  and claim prints STALE, then DRAIN, and claims nothing' "$stale_line|DRAIN|absent" \
    "$(HW_RUN_CHECKOUT="$repos/behind" sh "$tool" claim "$mid" 32 issue-32 mid-other 2>&1 | paste -sd'|')|$([ -e "$mid/claims/artifacts/mid-other" ] && echo present || echo absent)"
out=$(HW_RUN_CHECKOUT="$repos/behind" sh "$tool" accept "$mid" 33 branch=issue-33 note=/s/adj33.md footprint=mid-33 2>&1); status=$?
same '  and accept in drain prints STALE once, then DRAIN' "0|$stale_line|DRAIN" "$status|$(printf '%s\n' "$out" | paste -sd'|')"
rm -f "$mid/drain"
q "$mid" 34
out=$(HW_RUN_CHECKOUT="$repos/behind" sh "$tool" accept "$mid" 34 branch=issue-34 note=/s/adj34.md footprint=mid-34 2>&1); status=$?
same 'accept prints the STALE line once, first, and exits 0' "0|$stale_line|1" "$status|$(printf '%s\n' "$out" | head -n 1)|$(printf '%s\n' "$out" | grep -c '^STALE: ')"
same '  and still claims and renders the dispatch' 'CLAIMED: mid-34|1' "$(printf '%s\n' "$out" | grep '^CLAIMED')|$(printf '%s\n' "$out" | grep -c '^DISPATCH: ')"
q "$mid" 35
out=$(HW_RUN_CHECKOUT="$repos/behind" sh "$tool" accept "$mid" 35 branch=issue-35 note=/s/adj35.md footprint=none 2>&1)
same 'accept with footprint none still prints the STALE line, from its stage' "$stale_line|1" "$(printf '%s\n' "$out" | head -n 1)|$(printf '%s\n' "$out" | grep -c '^DISPATCH: ')"
same 'on a level checkout, next, stage and claim print no STALE line' 0 \
    "$( { sh "$tool" next "$mid"; sh "$tool" stage "$mid" 31 built; sh "$tool" claim "$mid" 36 issue-36 lvl-file; } 2>&1 | grep -c '^STALE')"

printf '\n# a bare run id, and the prose a parent writes through it\n'
prose=$(sh "$tool" start prose 2>/dev/null)
: > "$prose/decisions.md"
sh "$tool" decide prose 'ruled #1 MERGE' 2>"$scratch/err"; status=$?
same 'decide takes a bare run id and appends one dated line' "0 - $(date -u +%Y-%m-%d) — ruled #1 MERGE" \
    "$status $(cat "$prose/decisions.md")"

printf -- '- #1 | build | a finding\n- #2 | verify | another\n' > "$scratch/lines"
same 'intake appends a file of lines and says how many' 'INTAKE: 2 lines appended' \
    "$(sh "$tool" intake prose "$scratch/lines")"
same '  and both lines are in intake.md' 2 "$(wc -l < "$prose/intake.md" | tr -d ' ')"

sh "$tool" withdraw prose '#2 |' '- #2 | verify | WITHDRAWN' 2>"$scratch/err"; status=$?
same 'withdraw replaces the one line that holds the match' "0 - #2 | verify | WITHDRAWN" \
    "$status $(sed -n 2p "$prose/intake.md")"
same '  and leaves the other line as it was' '- #1 | build | a finding' "$(sed -n 1p "$prose/intake.md")"

sh "$tool" withdraw prose '| ' 'x' 2>"$scratch/err"; status=$?
same 'withdraw refuses a match that two lines hold, and writes nothing' "1 - #1 | build | a finding" \
    "$status $(sed -n 1p "$prose/intake.md")"

# The product owner rules a one-line remedy by ending its line with ` | APPLY`,
# and `applies` prints those lines and a count. A line that names APPLY in its
# text, or rules it in a field before the last, is not ruled.
same 'applies prints no line and a zero count before anything is ruled' 'APPLY: 0' \
    "$(sh "$tool" applies prose 2>&1)"
sh "$tool" withdraw prose '#1 |' '- #1 | build | a finding | none | APPLY' 2>"$scratch/err"
printf -- '- #3 | build | the APPLY ruling is named here | none\n- #4 | build | ruled early | APPLY | none\n- #5 | verify | a second one | none | APPLY\n' > "$scratch/lines"
sh "$tool" intake prose "$scratch/lines" >/dev/null
same 'applies prints each line ruled APPLY in its last field, then the count' \
    "- #1 | build | a finding | none | APPLY
- #5 | verify | a second one | none | APPLY
APPLY: 2" "$(sh "$tool" applies prose 2>&1)"
none_run=$(sh "$tool" start applies-none 2>/dev/null)
rm -f "$none_run/intake.md"
same 'applies reads a run with no intake.md as a zero count' 'APPLY: 0' \
    "$(sh "$tool" applies applies-none 2>&1)"

printf '## Parent\n\n- a lesson on two lines\n' > "$scratch/lesson"
before=$(wc -l < "$prose/lessons.md" | tr -d ' ')
same 'lesson takes a bare run id, appends a file of prose to lessons.md, and says how many lines' \
    "LESSON: 3 lines appended $((before + 3))" \
    "$(sh "$tool" lesson prose "$scratch/lesson") $(wc -l < "$prose/lessons.md" | tr -d ' ')"
: > "$scratch/empty"
sh "$tool" lesson prose "$scratch/empty" 2>"$scratch/err"; status=$?
same 'lesson refuses an empty file' 1 "$status"

sh "$tool" decide no-such-run 'x' 2>"$scratch/err"; status=$?
same 'a bare id that names no run is refused' 1 "$status"

printf '\n# every subcommand that names a run takes a bare run id\n'
# The dispatch of run-dir.sh resolves one <dir> for every subcommand. This
# case holds that for the handover verbs #1281 added, with no path in any call,
# as a worktree-isolated parent types them.
bare=$(sh "$tool" start bare 2>/dev/null)
q bare 70; q bare 71
same 'next takes a bare run id' 70 "$(sh "$tool" next bare 2>&1)"
sh "$tool" stage bare 70 adjudicated note=/s/a.md branch=issue-70 footprint=f70 >/dev/null 2>"$scratch/err"; status=$?
same 'stage takes a bare run id and writes the handover' '0 stage adjudicated' "$status $(sed -n 1p "$bare/handover/70" 2>/dev/null)"
same 'resume takes a bare run id' '70 adjudicated claim f70' "$(sh "$tool" resume bare 2>&1)"
sh "$tool" rule bare 71 deferred 'not this run' >/dev/null 2>"$scratch/err"; status=$?
same 'rule takes a bare run id and writes its RULED line' '0 1' "$status $(grep -c 'RULED #71 deferred' "$bare/decisions.md")"
same 'session takes a bare run id' 'SESSION: cccc3333' "$(CLAUDE_JOB_DIR=/jobs/cccc3333 sh "$tool" session bare 2>&1)"
same '  and appends to that run'"'"'s parent.session' cccc3333 "$(tail -n 1 "$bare/parent.session")"
sh "$tool" resume no-such-run >/dev/null 2>&1; status=$?
same 'a bare id that names no run is refused by resume' 1 "$status"
printf '\n# next skips what the run ruled out, and a ruling line the owner has not answered\n'
# Verify-1 of PR #1279 ran `next` on a copy of run 20260927-0443 and it
# printed #927, which decisions.md:2055 says stays gated. Nothing in claims/,
# handover/ or the log records a gate, a deferral or a refusal, and the queue
# line was marked `ruling` with no OWNER line for it.
ruled=$(sh "$tool" start ruled 2>/dev/null)
q "$ruled" 20 scope=wide mark=ruling 'note=embedding path'; q "$ruled" 21 scope=wide mark=ruling
q "$ruled" 22; q "$ruled" 23; q "$ruled" 24
printf -- '- 2026-09-27 — OWNER (2026-09-27) #21: build it.\n' >> "$ruled/decisions.md"
same 'a ruling line with no OWNER line is skipped, and one the owner answered is not' 21 "$(sh "$tool" next "$ruled" 2>&1)"
sh "$tool" claim "$ruled" 21 issue-21 f21 >/dev/null 2>&1
sh "$tool" rule "$ruled" 22 deferred 'owner deferred it for this run' >/dev/null 2>"$scratch/err"; status=$?
same 'rule exits 0' 0 "$status"
same '  and appends one RULED line to decisions.md, naming its run' 1 "$(grep -c '^- [0-9-]* — RULED #22 deferred (run ruled): owner deferred it for this run$' "$ruled/decisions.md")"
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

printf '\n# next holds a line marked blocked until the parent rules it open\n'
# The owner ruled on 2026-10-02 that hw-queue checks each issue's GitHub
# `blocked by` links and raises an open one to the parent, rather than
# dropping the issue or building it unremarked.
blocked=$(sh "$tool" start blocked 2>/dev/null)
q "$blocked" 80 milestone=0.12 blocked=79,0.11; q "$blocked" 81 milestone=0.12
printf -- '- 2026-10-02 — OWNER (2026-10-02) #80: build it.\n' >> "$blocked/decisions.md"
same 'a blocked line is skipped, and an OWNER line does not release it' 81 "$(sh "$tool" next "$blocked" 2>&1)"
sh "$tool" rule "$blocked" 80 open 'the blocker merged' >/dev/null 2>&1
same '  and the parent ruling it open releases it' 80 "$(sh "$tool" next "$blocked" 2>&1)"

printf '\n# a line marked both ruling and blocked is held by both, in either order\n'
# Run 20261012-1113 queued #1541 as `ruling; blocked (#1721, none)`. The sed
# that read the marks took the first one only, so the owner's answer to the
# ruling would have released an issue whose blocker was still open.
both=$(sh "$tool" start both 2>/dev/null)
q "$both" 90 scope=wide mark=ruling blocked=89,none; q "$both" 91 scope=wide blocked=89,none mark=ruling; q "$both" 92
printf -- '- 2026-10-12 — OWNER (2026-10-12) #90: stop warning.\n- 2026-10-12 — OWNER (2026-10-12) #91: stop warning.\n' >> "$both/decisions.md"
same 'an OWNER answer does not release a line that is also blocked, whichever mark comes first' 92 "$(sh "$tool" next "$both" 2>&1)"
sh "$tool" rule "$both" 91 open 'the blocker merged' >/dev/null 2>&1
same '  and ruling it open releases it' 91 "$(sh "$tool" next "$both" 2>&1)"

printf '\n# a run whose queue.md was typed by hand is refused, and converted once\n'
# Before #1724, hw-queue typed queue.md and next parsed it, so a malformed line
# was refused only when next read it, after the queue agent had exited. Run
# 20261012-1113's queue agent wrote `1 | #502 | ...`, and next printed EMPTY
# over sixteen eligible issues. Now next reads queue.jsonl alone, and a run
# that holds only a hand-typed queue.md is an error that names the way out.
shape=$(sh "$tool" start shape 2>/dev/null)
printf '# Queue\n\n1. #95 Typed | none | x | x | narrow | ruling\n2. #96 Typed too | 0.10 | a reader | docs | wide | blocked (#94, 0.10, not in queue: waits on Q18); owner added it\n' > "$shape/queue.md"
sh "$tool" next "$shape" >/dev/null 2>"$scratch/err"; status=$?
same 'next refuses a run that holds queue.md and no queue.jsonl with exit 1' 1 "$status"
case $(cat "$scratch/err") in
    *'run-dir.sh convert shape'*) pass '  and the refusal names the convert verb for that run' ;;
    *) fail '  and the refusal names the convert verb for that run' "$(cat "$scratch/err")" ;;
esac
legacy=$(cat "$shape/queue.md")
sh "$tool" queue shape 99 title=x milestone=none reader=x artifacts=x scope=narrow >/dev/null 2>"$scratch/err"; status=$?
same 'a queue call on a run whose queue.md was typed by hand is refused with exit 1, and overwrites nothing' \
    "1 1 $legacy" "$status $([ -e "$shape/queue.jsonl" ]; echo $?) $(cat "$shape/queue.md")"
sh "$tool" convert shape >"$scratch/out" 2>"$scratch/err"; status=$?
same 'convert exits 0 and converts every issue line' '0 CONVERTED: 2 entries into queue.jsonl; the old file is queue-legacy.md' "$status $(cat "$scratch/out")"
same '  and keeps the typed file whole as queue-legacy.md' "$legacy" "$(cat "$shape/queue-legacy.md")"
same '  and the marks become fields, with the words past the milestone kept as a note' \
    '95 true [] []|96 false [{"issue":94,"milestone":"0.10"}] ["not in queue: waits on Q18","owner added it"]' \
    "$(jq -rc '"\(.issue) \(.ruling) \(.blocked) \(.notes)"' "$shape/queue.jsonl" | paste -sd'|')"
same '  and next reads the converted queue' 96 "$(sh "$tool" rule shape 96 open 'the blocker merged' >/dev/null; sh "$tool" next shape 2>&1)"
sh "$tool" convert shape >/dev/null 2>&1; status=$?
same '  and a second convert is refused, because a run converts once' 1 "$status"
bad=$(sh "$tool" start bad-shape 2>/dev/null)
printf '# Queue\n\n1. #97 Fine | none | x | x | narrow |\n2. #98 No scope | none | x | x\n' > "$bad/queue.md"
before=$(cat "$bad/queue.md")
sh "$tool" convert bad-shape >/dev/null 2>"$scratch/err"; status=$?
same 'convert refuses a line the queue verb would refuse, with exit 2' 2 "$status"
case $(cat "$scratch/err") in
    *'#98'*'scope'*) pass '  and the refusal names the line and the field' ;;
    *) fail '  and the refusal names the line and the field' "$(cat "$scratch/err")" ;;
esac
same '  and writes nothing: no queue.jsonl, and queue.md as it was' "1 $before" "$([ -e "$bad/queue.jsonl" ]; echo $?) $(cat "$bad/queue.md")"

printf '\n# an OWNER line naming two issues answers both, and a malformed call is refused when it is written\n'
# The decisive case for #1724. Run 20261012-1113 recorded the owner's answer
# to #990 and #1275 as one line, `OWNER (2026-10-12) #990 #1275: "Build both"`
# (decisions.md:1215), and answered_issues took the first #N alone, so next
# held #1275 for a ruling the owner had already given and handed out #7.
two=$(sh "$tool" start two-answers 2>/dev/null)
q two-answers 990 mark=ruling
q two-answers 1275 mark=ruling
q two-answers 7
sh "$tool" decide two-answers --owner 990 1275 -- '"Build both"' >"$scratch/out" 2>"$scratch/err"; status=$?
same 'decide --owner exits 0 and names each issue it answers' '0 OWNER: #990 #1275' "$status $(cat "$scratch/out")"
same '  and writes the one line every reader parses' 1 "$(grep -c '^- [0-9-]* — OWNER ([0-9-]*) #990 #1275: "Build both"$' "$two/decisions.md")"
same 'next hands out the first issue the line answers' 990 "$(sh "$tool" next two-answers 2>&1)"
sh "$tool" claim two-answers 990 issue-990 f990 >/dev/null 2>&1
same '  and the second, which the first-#N reader held' 1275 "$(sh "$tool" next two-answers 2>&1)"
# The line already in run 20261012-1113, typed before the verb existed, is read the same way.
hand=$(sh "$tool" start hand-typed 2>/dev/null)
q hand-typed 990 mark=ruling; q hand-typed 1275 mark=ruling; q hand-typed 7
printf -- '- 2026-10-12 — OWNER (2026-10-12) #990 #1275: "Build both (Recommended)" — build in this run despite the self-audit label.\n' >> "$hand/decisions.md"
sh "$tool" claim hand-typed 990 issue-990 f990 >/dev/null 2>&1
same 'a hand-typed OWNER line in the same form answers both of its issues too' 1275 "$(sh "$tool" next hand-typed 2>&1)"
# A #N after the colon is the owner's prose. Read as an answer, this one
# line of the same run would have answered twelve issues.
prose=$(sh "$tool" start prose-owner 2>/dev/null)
q prose-owner 502 mark=ruling; q prose-owner 504 mark=ruling; q prose-owner 8
printf -- '- 2026-10-12 — OWNER (2026-10-12): this run rules out probe work; Target set: #502 #504.\n' >> "$prose/decisions.md"
same 'an OWNER line whose issues follow the colon answers none of them' 8 "$(sh "$tool" next prose-owner 2>&1)"
sh "$tool" queue two-answers 9 title=x milestone=none reader=x artifacts=x >/dev/null 2>"$scratch/err"; status=$?
same 'a queue call missing a field exits 2' 2 "$status"
case $(cat "$scratch/err") in *'scope='*) pass '  and names the field' ;; *) fail '  and names the field' "$(cat "$scratch/err")" ;; esac
sh "$tool" queue two-answers 9 title=x milestone=none reader=x artifacts=x scope=narrow mark=rulng >/dev/null 2>"$scratch/err"; status=$?
same 'a queue call carrying an unknown mark exits 2' 2 "$status"
case $(cat "$scratch/err") in *'mark=rulng'*) pass '  and names the mark' ;; *) fail '  and names the mark' "$(cat "$scratch/err")" ;; esac
# Each bad value replaces the good one, so the refusal is the value's own
# and not the one for a field named twice.
for call in 'scope=both' 'blocked=x' 'colour=red' 'title='; do
    set --
    for pair in title=x milestone=none reader=x artifacts=x scope=narrow; do
        [ "${pair%%=*}" = "${call%%=*}" ] || set -- "$@" "$pair"
    done
    sh "$tool" queue two-answers 9 "$@" "$call" >/dev/null 2>"$scratch/err"; status=$?
    case $(cat "$scratch/err") in *'given twice'*) twice=1 ;; *) twice=0 ;; esac
    same "a queue call carrying \`$call\` exits 2, for its own reason" '2 0' "$status $twice"
done
sh "$tool" queue two-answers x9 title=x milestone=none reader=x artifacts=x scope=narrow >/dev/null 2>&1; status=$?
same 'a queue call whose issue is not a number exits 2' 2 "$status"
sh "$tool" queue two-answers 7 title=x milestone=none reader=x artifacts=x scope=narrow >/dev/null 2>&1; status=$?
same 'a queue call for an issue already queued exits 2' 2 "$status"
same '  and no refused call wrote an entry' 3 "$(wc -l < "$two/queue.jsonl" | tr -d ' ')"
same 'queue.md is rendered from queue.jsonl, one line per entry in the shape hw-queue states' \
    '1. #990 Issue 990 | none | x | x | narrow | ruling|2. #1275 Issue 1275 | none | x | x | narrow | ruling|3. #7 Issue 7 | none | x | x | narrow | ' \
    "$(grep '^[0-9]*\. #' "$two/queue.md" | paste -sd'|')"
printf '# Queue\n\n1. #7 Edited by hand | none | x | x | narrow | ruling\n' > "$two/queue.md"
sh "$tool" release two-answers 990 >/dev/null 2>&1
same '  and next never reads it: an edit there changes nothing' 990 "$(sh "$tool" next two-answers 2>&1)"
sh "$tool" decide two-answers 'OWNER (2026-10-12) #7: typed by hand' >/dev/null 2>"$scratch/err"; status=$?
same 'a plain decide whose text opens OWNER exits 2' 2 "$status"
case $(cat "$scratch/err") in *'--owner'*) pass '  and names --owner' ;; *) fail '  and names --owner' "$(cat "$scratch/err")" ;; esac
same '  and writes nothing' 0 "$(grep -c 'typed by hand' "$two/decisions.md")"
sh "$tool" decide two-answers 'the parent filled slot 2' >/dev/null 2>&1; status=$?
same 'a plain decide of prose is unchanged' '0 1' "$status $(grep -c ' — the parent filled slot 2$' "$two/decisions.md")"
for call in '--owner -- "yes"' '--owner 12x -- "yes"' '--owner 12 -- ' '--owner 12 "yes"'; do
    # shellcheck disable=SC2086
    eval "sh \"\$tool\" decide two-answers $call" >/dev/null 2>&1; status=$?
    same "decide $call exits 2" 2 "$status"
done

printf '\n# the rulings the owner owes are written through owe, and rendered\n'
owes=$(sh "$tool" start owes 2>/dev/null)
sh "$tool" owe owes 1541 'title=Declared conflict' 'question=Stop warning?' 'options=Stop warning, because the edge says it | Keep warning | defer' 'unblocks=#1541 in 0.10' >"$scratch/out" 2>&1; status=$?
same 'owe exits 0' '0 OWED: #1541, ruling 1' "$status $(cat "$scratch/out")"
same '  and rulings.md holds the block in the product owner'"'"'s shape, not yet answered' \
    'RULING #1541 Declared conflict|  question: Stop warning?|  options:  Stop warning, because the edge says it | Keep warning | defer|  unblocks: #1541 in 0.10|  answered: no' \
    "$(grep . "$owes/rulings.md" | paste -sd'|')"
sh "$tool" decide owes --owner 1541 -- '"Stop warning (Recommended)"' >/dev/null 2>&1
same '  and an OWNER line marks it answered, with no block edited' '  answered: yes, an OWNER line in decisions.md' "$(sh "$tool" rulings owes | grep answered)"
for call in 'options=a | b' 'options=defer' 'options=a |  | defer' 'colour=red'; do
    sh "$tool" owe owes 1542 title=t question=q unblocks=u "$call" >/dev/null 2>"$scratch/err"; status=$?
    same "owe carrying \`$call\` exits 2" 2 "$status"
done
sh "$tool" owe owes 1542 title=t 'options=a | defer' unblocks=u >/dev/null 2>"$scratch/err"; status=$?
same 'owe missing the question exits 2' 2 "$status"
case $(cat "$scratch/err") in *'question='*) pass '  and names the field' ;; *) fail '  and names the field' "$(cat "$scratch/err")" ;; esac
same '  and no refused call wrote a ruling' 1 "$(wc -l < "$owes/rulings.jsonl" | tr -d ' ')"
handrul=$(sh "$tool" start hand-rulings 2>/dev/null)
printf 'RULING #1471 Typed by hand\n  question: q\n' > "$handrul/rulings.md"
sh "$tool" owe hand-rulings 1472 title=t question=q 'options=a | defer' unblocks=u >/dev/null 2>&1; status=$?
same 'the first owe in a run whose rulings.md was written by hand keeps that file whole as rulings-legacy.md' \
    "0 RULING #1471 Typed by hand|  question: q RULING #1472 t" \
    "$status $(paste -sd'|' "$handrul/rulings-legacy.md") $(head -n 1 "$handrul/rulings.md")"

printf '\n# the integrator queue only appends, and no verb rewrites an earlier entry\n'
ints=$(sh "$tool" start ints 2>/dev/null)
sh "$tool" integrate ints 1723 issue=none branch=1723-queue sha=5cd20cbb ruling=MERGE footprint=tools/run/run-dir.sh >"$scratch/out" 2>&1; status=$?
same 'integrate exits 0 for a pull request with no issue' '0 INTEGRATE: PR #1723 queued, entry 1' "$status $(cat "$scratch/out")"
first=$(sed -n 1p "$ints/integrate.jsonl")
sh "$tool" integrate ints 1730 issue=1724 branch=1724-run-files sha=abc1234 'ruling=MERGE, held by its fixture' footprint=a,b >/dev/null 2>&1
sh "$tool" integrate ints 1723 issue=none branch=1723-queue sha=5cd20cbb ruling=MERGE footprint=tools/run/run-dir.sh >/dev/null 2>"$scratch/err"; status=$?
same 'the same pull request at the same commit is refused with exit 2' 2 "$status"
sh "$tool" integrate ints 1723 issue=none branch=1723-queue sha=def5678 ruling='MERGE again, after an ejection' footprint=tools/run/run-dir.sh >/dev/null 2>&1; status=$?
same '  and a new ruling of it at a new head is appended' '0 3' "$status $(wc -l < "$ints/integrate.jsonl" | tr -d ' ')"
for call in 'issue=x' 'sha=XYZ1234' 'sha=abc' 'branch=a b' 'colour=red'; do
    set --
    for pair in issue=1 branch=b sha=abc1234 ruling=r footprint=f; do
        [ "${pair%%=*}" = "${call%%=*}" ] || set -- "$@" "$pair"
    done
    sh "$tool" integrate ints 1731 "$@" "$call" >/dev/null 2>"$scratch/err"; status=$?
    case $(cat "$scratch/err") in *'given twice'*) twice=1 ;; *) twice=0 ;; esac
    same "integrate carrying \`$call\` exits 2, for its own reason" '2 0' "$status $twice"
done
sh "$tool" integrate ints 1731 issue=1 branch=b sha=abc1234 ruling=r >/dev/null 2>"$scratch/err"; status=$?
same 'integrate missing the footprint exits 2' 2 "$status"
case $(cat "$scratch/err") in *'footprint='*) pass '  and names the field' ;; *) fail '  and names the field' "$(cat "$scratch/err")" ;; esac
same 'integrations lists each pull request once, at its last ruling, in the order first ruled' \
    'PR #1723 issue=none branch=1723-queue sha=def5678 footprint=tools/run/run-dir.sh ruling=MERGE again, after an ejection|PR #1730 issue=1724 branch=1724-run-files sha=abc1234 footprint=a,b ruling=MERGE, held by its fixture' \
    "$(sh "$tool" integrations ints | paste -sd'|')"
printf '{"iter":1,"issue":0,"pr":1723,"merge":"abc1234","verdict":"MERGE","proved":"p","opened":[],"closed":[]}\n' >> "$ints/log.jsonl"
same '  and drops one the log records merged' 'PR #1730' "$(sh "$tool" integrations ints | cut -d' ' -f1-2)"
sh "$tool" integrate ints 1730 issue=1724 branch=1724-run-files sha=abc1234 ruling=ejected footprint=a,b >/dev/null 2>&1; status=$?
same '  and an ejection is one more entry, which takes the pull request off the list' '0 4 0' \
    "$status $(wc -l < "$ints/integrate.jsonl" | tr -d ' ') $(sh "$tool" integrations ints | wc -l | tr -d ' ')"
sh "$tool" integrate ints 1730 issue=1724 branch=1724-run-files sha=abc5678 ruling=MERGE footprint=a,b >/dev/null 2>&1
same '  until the parent rules it again at its new head' 'PR #1730 sha=abc5678' "$(sh "$tool" integrations ints | cut -d' ' -f1,2,5)"
# Every verb that writes the run, once each. None may touch the first entry.
q ints 5
sh "$tool" owe ints 5 title=t question=q 'options=a | defer' unblocks=u >/dev/null 2>&1
sh "$tool" decide ints --owner 5 -- 'yes' >/dev/null 2>&1
sh "$tool" decide ints 'prose' >/dev/null 2>&1
sh "$tool" rule ints 5 open 'r' >/dev/null 2>&1
printf -- '- #5 | build | a finding | none\n' > "$scratch/intake"
sh "$tool" intake ints "$scratch/intake" >/dev/null 2>&1
sh "$tool" withdraw ints 'a finding' '- #5 | build | withdrawn | none' >/dev/null 2>&1
sh "$tool" stage ints 5 ruled-merge pr=1730 >/dev/null 2>&1
sh "$tool" rulings ints >/dev/null 2>&1
sh "$tool" integrations ints >/dev/null 2>&1
sh "$tool" resume ints >/dev/null 2>&1
same 'after every verb that writes the run, the first entry of integrate.jsonl is byte for byte as written' "$first" "$(sed -n 1p "$ints/integrate.jsonl")"
same '  and the file only grew' 5 "$(wc -l < "$ints/integrate.jsonl" | tr -d ' ')"
same 'resume does not send a ruled-merge whose pull request is already queued to integrate again' '5 ruled-merge queued' "$(sh "$tool" resume ints 2>&1)"
sh "$tool" stage ints 6 ruled-merge pr=1740 >/dev/null 2>&1
same '  and one that is not queued still goes to integrate' '6 ruled-merge integrate' "$(sh "$tool" resume ints 2>&1 | grep '^6 ')"
sh "$tool" stage ints 1724 ruled-merge >/dev/null 2>&1
same '  and an issue queued under its own number is queued, with no pr on its handover' '1724 ruled-merge queued' "$(sh "$tool" resume ints 2>&1 | grep '^1724 ')"
# Verify-1 of PR #1726: an ejected pull request's re-ruling read as queued,
# so a parent that restarted before appending it was told to do nothing, and
# the pull request never merged.
sh "$tool" integrate ints 1750 issue=17 branch=b17 sha=aaa1750 ruling=MERGE footprint=f >/dev/null 2>&1
sh "$tool" integrate ints 1750 issue=17 branch=b17 sha=aaa1750 ruling=ejected footprint=f >/dev/null 2>&1
sh "$tool" stage ints 17 ruled-merge pr=1750 >/dev/null 2>&1
same 'resume sends a ruled-merge whose pull request was last ejected to integrate, not queued' '17 ruled-merge integrate' "$(sh "$tool" resume ints 2>&1 | grep '^17 ')"
sh "$tool" integrate ints 1750 issue=17 branch=b17 sha=bbb1750 ruling=MERGE footprint=f >/dev/null 2>&1
same '  and once it is ruled again at its new head, it is queued' '17 ruled-merge queued' "$(sh "$tool" resume ints 2>&1 | grep '^17 ')"
sh "$tool" stage ints 18 ruled-merge >/dev/null 2>&1
sh "$tool" integrate ints 1760 issue=18 branch=b18 sha=aaa1760 ruling=ejected footprint=f >/dev/null 2>&1
same '  and an ejected issue with no pr on its handover goes to integrate too' '18 ruled-merge integrate' "$(sh "$tool" resume ints 2>&1 | grep '^18 ')"
for verb in "queue ints 19 title=a milestone=none reader=x artifacts=x scope=narrow title=b" \
    "owe ints 19 title=a question=q 'options=a | defer' unblocks=u title=b" \
    "integrate ints 1770 issue=19 branch=b sha=abc1234 ruling=r footprint=f sha=abc5678"; do
    eval "sh \"\$tool\" $verb" >/dev/null 2>"$scratch/err"; status=$?
    case $(cat "$scratch/err") in *'given twice'*) said=1 ;; *) said=0 ;; esac
    same "${verb%% *} refuses a field named twice with exit 2, and says so" '2 1' "$status $said"
done
sh "$tool" queue ints 29 'title=a | b' milestone=none reader=x artifacts=x scope=narrow >/dev/null 2>"$scratch/err"; status=$?
case $(cat "$scratch/err") in *'separates the columns'*) said=1 ;; *) said=0 ;; esac
same 'queue refuses a title that holds `|`, which separates the columns of queue.md' '2 1' "$status $said"

printf '\n# an OWNER line an old ledger carries in is read like one decide --owner writes\n'
imp=$(sh "$tool" start imported 2>/dev/null)
printf '# Ledger\n\n## Decisions needing an owner\n\n- 2026-09-01 — OWNER (2026-09-01) #310 #311: build both.\n\n## Log\n\nrow\n' > "$scratch/ledger.md"
sh "$tool" import "$scratch/ledger.md" imported >/dev/null 2>&1
q imported 310 mark=ruling; q imported 311 mark=ruling; q imported 312
sh "$tool" claim imported 310 issue-310 f310 >/dev/null 2>&1
same 'import carries an OWNER line that answers both of its issues' 311 "$(sh "$tool" next imported 2>&1)"

printf '\n# drain while an adjudicate is in flight: the claim waits for the next session\n'
# Verify-1 of PR #1279: the parent claims on an adjudicate report, which can
# arrive after drain has started. `claim` then writes nothing. The parent
# writes `stage adjudicated` with the note and the footprint, so the next
# session claims it and dispatches hw-iterate, and the issue is not
# adjudicated twice. `resume` says which, for every open handover.
inflight=$(sh "$tool" start inflight 2>/dev/null)
for n in 30 31 32 33 34; do q "$inflight" "$n"; done
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

printf '\n# a ruling binds the run that wrote it, and an OWNER line binds every run\n'
# Verify-1 of PR #1281: `start` seeds decisions.md from the previous run, so
# a deferral written in run A skipped the issue again in run B, although
# hw-run-policy defers "for this run alone". An OWNER line is different: the
# owner's answer is posted on the issue and holds until the owner changes it.
rm -rf "$HEADWATER_RUN_ROOT"
mkdir -p "$HEADWATER_RUN_ROOT"
runa=$(sh "$tool" start 2026-a 2>/dev/null)
q "$runa" 40; q "$runa" 41 mark=ruling; q "$runa" 42
sh "$tool" rule "$runa" 40 deferred 'not this run' >/dev/null 2>&1
printf -- '- 2026-09-27 — OWNER (2026-09-27) #41: build it.\n' >> "$runa/decisions.md"
same 'in the run that deferred it, next skips the issue' 41 "$(sh "$tool" next "$runa" 2>&1)"
runb=$(sh "$tool" start 2026-b 2>/dev/null)
same '  and the next run is seeded with no queue, no rulings and no integrator queue' '1 1 1' \
    "$([ -e "$runb/queue.jsonl" ]; echo $?) $([ -e "$runb/rulings.jsonl" ]; echo $?) $([ -e "$runb/integrate.jsonl" ]; echo $?)"
q "$runb" 40; q "$runb" 41 mark=ruling; q "$runb" 42
same '  and the next run is seeded with the RULED line' 1 "$(grep -c 'RULED #40 deferred (run 2026-a)' "$runb/decisions.md")"
same '  but the next run hands the issue out again' 40 "$(sh "$tool" next "$runb" 2>&1)"
sh "$tool" claim "$runb" 40 issue-40 f40 >/dev/null 2>&1
same '  and the OWNER line of the earlier run still answers the ruling line' 41 "$(sh "$tool" next "$runb" 2>&1)"

printf '\n# the accept form next-run.md gives the parent leaves a handover resume can act on\n'
# Verify-2 of PR #1281: next-run.md told the parent to run a bare
# `stage <run> <issue> adjudicated`. Followed into a drain, that left a
# handover `resume` read as `claim ` with no artifacts and no note. The form
# is now `accept`, which writes the checkpoint, claims and dispatches in one
# call. This case takes the form from next-run.md itself, fills its
# placeholders, and follows it into a drain and a resume, so a command file
# that drops a field goes red.
cmd="$root/.claude/commands/next-run.md"
form=$(grep -o 'run-dir\.sh accept <run> <issue> [^`]*' "$cmd" | head -n 1)
same 'next-run.md gives the parent an accept form for a BUILD report' 1 "$([ -n "$form" ] && echo 1 || echo 0)"
own=$(sh "$tool" start own-form 2>/dev/null)
q "$own" 60
touch "$own/drain"
filled=$(printf '%s' "$form" | sed -e "s|^run-dir\.sh accept <run> <issue>|accept $own 60|" \
    -e 's|<path>|/s/adjudication.md|g' -e 's|<name>|issue-60|g' -e 's|<a,b>|f60,g60|g')
# shellcheck disable=SC2086
out=$(HEADWATER_SCRATCH="$scratch/accept-form" sh "$tool" $filled 2>&1); status=$?
same '  and in drain that form writes the checkpoint and stops' '0 DRAIN stage adjudicated' "$status $out $(head -n 1 "$own/handover/60" 2>/dev/null)"
same '  and claims nothing in drain' 0 "$(ls "$own/claims/artifacts" 2>/dev/null | grep -c .)"
rm -f "$own/drain"
same '  and the next session claims the footprint it names' '60 adjudicated claim f60 g60' "$(sh "$tool" resume "$own" 2>&1)"
same '  and the note it names reaches hw-iterate' 'note /s/adjudication.md' "$(grep '^note ' "$own/handover/60" 2>/dev/null)"
sh "$tool" stage "$own" 61 adjudicated >/dev/null 2>"$scratch/err"; status=$?
same 'a bare stage adjudicated, with no note and no footprint, is refused with exit 2' '2 0' \
    "$status $(ls "$own/handover" 2>/dev/null | grep -c '^61$')"

printf '\n# a veto after a restart: verified-pass goes back to verified-fail, and the fields carry\n'
# Verify-1 of PR #1281: after a restart, the parent rules a verified-pass
# handover and may veto it, but no path took the veto back to a builder. The
# veto writes the checkpoint back to verified-fail with the veto file as the
# finding, and every field it does not name carries from the checkpoint before.
veto=$(sh "$tool" start veto 2>/dev/null)
sh "$tool" stage "$veto" 50 adjudicated note=/s/adj.md branch=issue-50 footprint=f50 >/dev/null 2>&1
same 'resume, adjudicated and unclaimed: claim the footprint' '50 adjudicated claim f50' "$(sh "$tool" resume "$veto" 2>&1)"
sh "$tool" stage "$veto" 50 built pr=500 build=/s/build.md >/dev/null 2>&1
same 'a stage carries every field it does not name from the checkpoint before' \
    'stage built|branch issue-50|pr 500|note /s/adj.md|build /s/build.md|footprint f50' \
    "$(paste -sd'|' "$veto/handover/50")"
same 'resume, built: iterate' '50 built iterate' "$(sh "$tool" resume "$veto" 2>&1)"
sh "$tool" stage "$veto" 50 verified-pass verify=/s/verify.md rounds=1 >/dev/null 2>&1
same 'resume, verified-pass: the parent rules' '50 verified-pass rule' "$(sh "$tool" resume "$veto" 2>&1)"
sh "$tool" stage "$veto" 50 verified-fail verify=/s/veto-1.md >/dev/null 2>&1
same 'a veto writes verified-fail with the veto as the finding, and resume sends it to hw-iterate' \
    '50 verified-fail iterate|verify /s/veto-1.md|pr 500|note /s/adj.md' \
    "$(sh "$tool" resume "$veto" 2>&1)|$(grep -E '^(verify|note|pr) ' "$veto/handover/50" | sort -r | paste -sd'|')"
sh "$tool" stage "$veto" 50 ruled-merge >/dev/null 2>&1
same 'resume, ruled-merge: integrate' '50 ruled-merge integrate' "$(sh "$tool" resume "$veto" 2>&1)"
sh "$tool" stage "$veto" 51 adjudicated note=/s/adj51.md branch=issue-51 footprint=none >/dev/null 2>&1
same 'resume, adjudicated with footprint none: iterate, with nothing to claim' '51 adjudicated iterate' "$(sh "$tool" resume "$veto" 2>&1 | grep '^51 ')"

printf '\n# claim refuses a comma-joined footprint, and takes over an empty claim file\n'
# Run 20261012-1113: `claim <run> 41 b a,b` stored one slug `a,b`, collided
# with nothing, and a later claim printed `HELD: ... by #unknown` against an
# empty file. Neither shape stores anything now.
cm=$(sh "$tool" start comma 2>/dev/null)
sh "$tool" claim "$cm" 41 issue-41 'f41,g41' >/dev/null 2>"$scratch/err"; status=$?
same 'a comma in an artifact is refused with exit 2' 2 "$status"
same '  and the refusal says what to pass' 1 "$(grep -c 'each artifact as its own argument' "$scratch/err")"
same '  and nothing was claimed' 0 "$(ls "$cm/claims/artifacts" 2>/dev/null | grep -c .)"
# An empty claim file is a writer that died between open and write. The slug
# of `f41` is whatever the tool makes of it: claim `f41` once under #1, copy
# its slug name, empty it, and claim it again under #41.
sh "$tool" claim "$cm" 1 issue-1 f41 >/dev/null 2>&1
slug41=$(grep -l '^artifact f41$' "$cm"/claims/artifacts/* | head -n 1)
: > "$slug41"
out=$(sh "$tool" claim "$cm" 41 issue-41 f41 2>&1); status=$?
same 'an empty claim file is taken over after the retries, and says so' '0 CLAIMED: f41 (replaced an empty claim file)' "$status $out"
same '  and the file now names the claimant' 'issue 41' "$(sed -n 's/^\(issue .*\)$/\1/p' "$slug41" | head -n 1)"
same '  and no WAITS-ON names #unknown' 0 "$(printf '%s' "$out" | grep -c unknown)"

printf '\n# integrate reads the head of the pull request when sha= is absent\n'
# The parent types the number it ruled on, and the sha comes from the one
# field on GitHub, never copied from a report. A fake `gh` on PATH answers
# the pulls endpoint; an answer that is not a sha refuses.
mkdir -p "$scratch/ghbin"
cat > "$scratch/ghbin/gh" <<'EOF'
#!/bin/sh
printf '%s\n' "$*" >> "$GH_LOG"
case "$*" in
    *"/pulls/700 --jq .head.sha") printf '%s\n' "${GH_HEAD:-abcdef1234567}" ;;
    *) exit 1 ;;
esac
EOF
chmod +x "$scratch/ghbin/gh"
ig=$(sh "$tool" start inthead 2>/dev/null)
: > "$scratch/gh.log"
out=$(PATH="$scratch/ghbin:$PATH" GH_LOG="$scratch/gh.log" sh "$tool" integrate "$ig" 700 issue=70 branch=issue-70 ruling=MERGE footprint=f70 2>&1); status=$?
same 'integrate without sha= exits 0 and reads the head from the pulls endpoint' '0 1' "$status $(grep -c 'repos/headwater-ai/headwater/pulls/700 --jq .head.sha' "$scratch/gh.log")"
same '  and the entry carries that sha' abcdef1234567 "$(jq -r 'select(.pr == 700) | .sha' "$ig/integrate.jsonl")"
PATH="$scratch/ghbin:$PATH" GH_LOG="$scratch/gh.log" GH_HEAD='not a sha' sh "$tool" integrate "$ig" 700 issue=71 branch=issue-71 ruling=MERGE footprint=f71 >/dev/null 2>"$scratch/err"; status=$?
same 'an answer that is not a sha refuses with exit 2' 2 "$status"
same '  and says to pass sha= or retry' 1 "$(grep -c 'pass `sha=` or retry' "$scratch/err")"
same '  and wrote no second entry' 1 "$(grep -c . "$ig/integrate.jsonl")"
PATH="$scratch/ghbin:$PATH" GH_LOG="$scratch/gh.log" sh "$tool" integrate "$ig" 701 issue=72 branch=issue-72 ruling=MERGE footprint=f72 sha=1234567abc >/dev/null 2>&1; status=$?
same 'with sha= given, gh is not asked' '0 0' "$status $(grep -c 'pulls/701' "$scratch/gh.log")"

printf '\n# dispatch renders the fields of each stage, and refuses a stage that lacks one\n'
# The dispatch template lived in next-run.md and the parent typed it into a
# Write per stage. The verb is the one renderer. Each line below is a field
# the command names, so a renderer that drops one goes red.
export HEADWATER_SCRATCH="$scratch/dispatch"
dp=$(sh "$tool" start dp 2>/dev/null)
q "$dp" 80 'title=A titled issue'
out=$(sh "$tool" dispatch "$dp" 80 adjudicate deadline=25 2>&1); status=$?
same 'adjudicate: exit 0 and the DISPATCH line names the file' "0 DISPATCH: $scratch/dispatch/issue-80/dispatch-adjudicate.md" "$status $out"
f="$scratch/dispatch/issue-80/dispatch-adjudicate.md"
same '  issue line carries the number and the queue title' 'issue:        #80 A titled issue' "$(grep '^issue:' "$f")"
same '  run line' "run:          $dp" "$(grep '^run:' "$f")"
same '  scratch line' "scratch:      $scratch/dispatch/issue-80/" "$(grep '^scratch:' "$f")"
same '  deadline line in minutes' 'deadline:     25 minutes' "$(grep '^deadline:' "$f")"
same '  report line' 'report:       the fixed block your definition names and nothing before it' "$(grep '^report:' "$f")"
same '  and no branch or note line for an adjudicate' 0 "$(grep -cE '^(branch|note|waits-on|slot):' "$f")"
sh "$tool" dispatch "$dp" 80 iterate note=/s/adj.md >/dev/null 2>"$scratch/err"; status=$?
same 'iterate without branch= is refused with exit 2' 2 "$status"
same '  and names the field' 1 "$(grep -c 'needs `branch=`' "$scratch/err")"
sh "$tool" dispatch "$dp" 80 iterate branch=issue-80 >/dev/null 2>"$scratch/err"; status=$?
same 'iterate without note= or handover= is refused with exit 2' 2 "$status"
sh "$tool" dispatch "$dp" 80 iterate branch=issue-80 note=/s/adj.md waits-on='#79' attacks='a, b' deadline=90 >/dev/null 2>&1
f="$scratch/dispatch/issue-80/dispatch-iterate.md"
same 'iterate: branch, waits-on, note, attacks and deadline lines' \
    'branch:       issue-80|waits-on:     #79|note:         /s/adj.md|attacks:      a, b|deadline:     90 minutes' \
    "$(grep -E '^(branch|waits-on|note|attacks|deadline):' "$f" | paste -sd'|')"
sh "$tool" dispatch "$dp" 80 iterate branch=issue-80 handover="$dp/handover/80" >/dev/null 2>&1
same 'iterate from a checkpoint: a handover line, waits-on none, and no note line' \
    "handover:     $dp/handover/80|waits-on:     none|0" \
    "$(grep -E '^(handover|waits-on):' "$f" | sort | paste -sd'|')|$(grep -c '^note:' "$f")"
sh "$tool" dispatch "$dp" 80 build branch=issue-80 note=/s/adj.md attacks=x >/dev/null 2>&1
same 'build: the same fields as an iterate' 'branch:       issue-80|note:         /s/adj.md|attacks:      x' \
    "$(grep -E '^(branch|note|attacks):' "$scratch/dispatch/issue-80/dispatch-build.md" | paste -sd'|')"
sh "$tool" dispatch "$dp" 80 verify branch=issue-80 >/dev/null 2>"$scratch/err"; status=$?
same 'verify without pr= is refused with exit 2' 2 "$status"
sh "$tool" dispatch "$dp" 80 verify branch=issue-80 pr=800 build=/s/build.md note=/s/adj.md attacks='mutant 3' >/dev/null 2>&1
f="$scratch/dispatch/issue-80/dispatch-verify.md"
same 'verify: pull request, note, build note, slot and attacks lines' \
    'pull request: #800|note:         /s/adj.md|build note:   /s/build.md|slot:         HW_CARGO_SLOT=verify-80|attacks:      mutant 3' \
    "$(grep -E '^(pull request|note|build note|slot|attacks):' "$f" | paste -sd'|')"
sh "$tool" dispatch "$dp" none build >/dev/null 2>&1; status=$?
f="$scratch/dispatch/run-lessons-dp/dispatch-build.md"
same 'issue none with build: the run-lessons branch, note and scratch' \
    "0|issue:        none (run lessons dp)|scratch:      $scratch/dispatch/run-lessons-dp/|branch:       run-lessons-dp" \
    "$status|$(grep -E '^(issue|branch|scratch):' "$f" | paste -sd'|')"
same '  and the note names the applies verb' 1 "$(grep -c '^note:.*run-dir.sh applies dp' "$f")"
sh "$tool" dispatch "$dp" none verify branch=run-lessons-dp pr=801 >/dev/null 2>&1
same 'issue none with verify: the lessons slot' 'slot:         HW_CARGO_SLOT=verify-lessons' "$(grep '^slot:' "$scratch/dispatch/run-lessons-dp/dispatch-verify.md")"
sh "$tool" dispatch "$dp" none integrate >/dev/null 2>"$scratch/err"; status=$?
same 'integrate with an empty integrator queue is refused with exit 2' 2 "$status"
sh "$tool" integrate "$dp" 800 issue=80 branch=issue-80 ruling=MERGE footprint=f80 sha=abc1234 >/dev/null 2>&1
sh "$tool" integrate "$dp" 801 issue=none branch=run-lessons-dp ruling=MERGE footprint=none sha=def5678 >/dev/null 2>&1
out=$(sh "$tool" dispatch "$dp" none integrate 2>&1); status=$?
f=${out#DISPATCH: }
same 'integrate: exit 0, the issue line names the queue, and one pull request line per ruling' \
    '0|issue:        none (the integrator queue)|2' \
    "$status|$(grep '^issue:' "$f")|$(grep -c '^pull request:' "$f")"
same '  and each pull request line is what integrations prints' "$(sh "$tool" integrations "$dp" | paste -sd'|')" "$(sed -n 's/^pull request: *//p' "$f" | paste -sd'|')"
sh "$tool" dispatch "$dp" 80 integrate >/dev/null 2>"$scratch/err"; status=$?
same 'integrate with an issue is refused with exit 2' 2 "$status"
sh "$tool" dispatch "$dp" 80 adjudicate colour=red >/dev/null 2>"$scratch/err"; status=$?
same 'a field no stage reads is refused with exit 2' 2 "$status"
sh "$tool" dispatch "$dp" 80 polish >/dev/null 2>"$scratch/err"; status=$?
same 'a stage that does not exist is refused with exit 2' 2 "$status"
out=$(HEADWATER_SCRATCH= CLAUDE_JOB_DIR= sh "$tool" dispatch "$dp" 80 adjudicate 2>&1); status=$?
same 'no scratch directory at all is refused with exit 2' 2 "$status"

printf '\n# accept: checkpoint, claim and dispatch in one call\n'
ac=$(sh "$tool" start ac 2>/dev/null)
q "$ac" 90 'title=Ninety'; q "$ac" 91
sh "$tool" claim "$ac" 91 issue-91 shared >/dev/null 2>&1
out=$(sh "$tool" accept "$ac" 90 branch=issue-90 note=/s/adj90.md footprint=f90,shared attacks='attack one' 2>&1); status=$?
same 'accept exits 0' 0 "$status"
same '  and the checkpoint is adjudicated with every field' \
    'stage adjudicated|branch issue-90|note /s/adj90.md|footprint f90,shared' \
    "$(grep -E '^(stage|branch|note|footprint) ' "$ac/handover/90" | paste -sd'|')"
same '  and the claim lines show what was claimed and what is held' 'CLAIMED: f90|HELD: shared by #91|WAITS-ON: 91' "$(printf '%s\n' "$out" | grep -E '^(CLAIMED|HELD|WAITS-ON)' | paste -sd'|')"
same '  and the DISPATCH line names the iterate file' "DISPATCH: $scratch/dispatch/issue-90/dispatch-iterate.md" "$(printf '%s\n' "$out" | grep '^DISPATCH')"
f="$scratch/dispatch/issue-90/dispatch-iterate.md"
same '  and the dispatch carries the waits-on the claim printed, as issue references' 'waits-on:     #91' "$(grep '^waits-on:' "$f")"
same '  and the branch, note, title and attacks' 'issue:        #90 Ninety|branch:       issue-90|note:         /s/adj90.md|attacks:      attack one' \
    "$(grep -E '^(issue|branch|note|attacks):' "$f" | paste -sd'|')"
q "$ac" 92
out=$(sh "$tool" accept "$ac" 92 branch=issue-92 note=/s/adj92.md footprint=none 2>&1); status=$?
same 'footprint none: no claim, waits-on none, and a dispatch' "0|waits-on:     none|0" \
    "$status|$(grep '^waits-on:' "$scratch/dispatch/issue-92/dispatch-iterate.md")|$(printf '%s\n' "$out" | grep -c '^CLAIMED')"
sh "$tool" accept "$ac" 92 branch=issue-92 note=/s/adj92.md >/dev/null 2>"$scratch/err"; status=$?
same 'accept without footprint= is refused with exit 2' 2 "$status"

printf '\n# po-rule records every block of a rulings file, or none\n'
pr=$(sh "$tool" start porule 2>/dev/null)
q "$pr" 95; q "$pr" 96
cat > "$scratch/rulings.md" <<'EOF'
RULING #95 Which install route
  question: Keep the APT fence or drop it?
  options: keep | drop | defer
  unblocks: #95 and the install page

RULING #96 Milestone of the LSP work
  question: Is #96 in 0.7 or 0.8?
  options: 0.7 | 0.8 | defer
  unblocks: the 0.7 close
  answered: not yet
EOF
out=$(sh "$tool" po-rule "$pr" "$scratch/rulings.md" 2>&1); status=$?
same 'a good file records every block, through owe, and says how many' \
    "0|OWED: #95, ruling 1|OWED: #96, ruling 2|PO-RULE: 2 rulings recorded from $scratch/rulings.md" "$status|$(printf '%s\n' "$out" | paste -sd'|')"
same '  and the ledger holds both' '95 96' "$(jq -r 'select(.issue == 95 or .issue == 96) | .issue' "$pr/rulings.jsonl" | paste -sd' ')"
same '  and the options reached the ledger whole, the last one defer' 'keep|drop|defer' "$(jq -r 'select(.issue == 95) | .options | join("|")' "$pr/rulings.jsonl" | head -n 1)"
cat > "$scratch/rulings-bad.md" <<'EOF'
RULING #97 A block with a bad option list
  question: Is this recorded?
  options: yes
  unblocks: nothing

RULING #98 A good block after a bad one
  question: Is this recorded?
  options: yes | no | defer
  unblocks: nothing
EOF
sh "$tool" po-rule "$pr" "$scratch/rulings-bad.md" >/dev/null 2>"$scratch/err"; status=$?
same 'a file with one bad block is refused with exit 2' 2 "$status"
same '  and names the block' 1 "$(grep -c 'RULING #97, block 1' "$scratch/err")"
same '  and records none of it, not even the good block' 0 "$(grep -cE '"issue":9[78]' "$pr/rulings.jsonl")"
printf 'RULING #99 Stray\n  question: q\n  options: a | defer\n  unblocks: u\nthis line is prose\n' > "$scratch/rulings-prose.md"
sh "$tool" po-rule "$pr" "$scratch/rulings-prose.md" >/dev/null 2>"$scratch/err"; status=$?
same 'a line that is neither a RULING line nor a field is refused, with its number' '2 1' "$status $(grep -c 'line 5 of the rulings file' "$scratch/err")"
unset HEADWATER_SCRATCH

printf '\n%s passed, %s failed\n' "$passed" "$failed"
[ "$failed" -eq 0 ]
