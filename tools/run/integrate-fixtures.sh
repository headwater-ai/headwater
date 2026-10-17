#!/bin/sh
# What holds `tools/run/integrate.sh`: `main` reads CI once and never waits,
# `check` reads the head, the closes and the clause check and refuses a head
# that moved, and `settle` waits once on the queue, then closes and releases
# on a merge, names an ejection, and passes a `RE-ISSUE` through as exit 2.
#
# Run it from anywhere:
#     sh tools/run/integrate-fixtures.sh
#
# It needs `jq` and `git` and no network: `gh` on `PATH` is a fake, and
# `HW_QUEUE_DONE` and `HW_CI_DONE` name stub readers. The run directory is
# real, under `HEADWATER_RUN_ROOT`, so the release is held against the
# claims `run-dir.sh` wrote.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
tool="$root/tools/run/integrate.sh"
rundir="$root/tools/run/run-dir.sh"

if ! command -v jq >/dev/null 2>&1; then
    echo "no \`jq\` on the path, and run-dir.sh reads JSON lines with it." >&2
    exit 3
fi

scratch=$(mktemp -d) || exit 1
trap 'rm -rf "$scratch"' EXIT HUP INT TERM
export HEADWATER_RUN_ROOT="$scratch/runs"
export HEADWATER_USAGE_DIR="$scratch/no-usage"
export LOG="$scratch/log"

passed=0
failed=0
pass() { printf 'ok   %s\n' "$1"; passed=$((passed + 1)); }
fail() { printf 'FAIL %s\n  %s\n' "$1" "$2"; failed=$((failed + 1)); }
same() {
    if [ "$2" = "$3" ]; then pass "$1"; else fail "$1" "expected \`$2\`, got \`$3\`"; fi
}

mkdir -p "$scratch/bin"
# The fake answers the pulls head read, the closing-references query, the
# PATCH that closes an issue and the state re-read after it, and the two
# body reads clause-check makes. A pull request body that accounts for every
# clause of a one-clause issue is canned, so clause-check passes unless a
# case asks for a body that accounts for none.
cat > "$scratch/bin/gh" <<'EOF'
#!/bin/sh
printf '%s\n' "$*" >> "$LOG"
case "$*" in
    *"/pulls/"*"--jq .head.sha") printf '%s\n' "${HEAD:-0123456789abcdef0123456789abcdef01234567}" ;;
    *"api graphql"*) printf '%s\n' "${CLOSES:-[]}" ;;
    *"-X PATCH"*"-f state=closed"*) : ;;
    *"--jq .state") printf '%s\n' "${CLOSE_STATE:-closed}" ;;
    *"/pulls/"*"--jq .body") if [ "${ACCOUNTED:-1}" = 1 ]; then printf 'Closes #9\n\n- [x] #9.1 held\n'; else printf 'Closes #9\n'; fi ;;
    *"/issues/"*"--jq .body") printf '## Done when\n\n- [ ] the one clause\n' ;;
    *) exit 1 ;;
esac
EOF
chmod +x "$scratch/bin/gh"
cat > "$scratch/queue-done.sh" <<'EOF'
#!/bin/sh
printf 'queue-done %s\n' "$1" >> "$LOG"
case ${QUEUE:-merged} in
    merged) echo "queue-done: #$1 merged 1234567890abcdef1234567890abcdef12345678" ;;
    ejected) echo "queue-done: #$1 ejected: CI failed" ;;
    unmergeable) echo "queue-done: #$1 unmergeable: the queue will eject it with merge_conflict" ;;
    stalled) echo "queue-done: #$1 stalled: every check of its merge group is green and it has not merged in 12 minutes" ;;
    notqueued) echo "queue-done: #$1 not queued: open, in no queue, and never removed from one" ;;
    waiting) echo "queue-done: #$1 is in the queue, position 2" >&2; exit 1 ;;
esac
EOF
cat > "$scratch/ci-done.sh" <<'EOF'
#!/bin/sh
printf 'ci-done %s\n' "$1" >> "$LOG"
case ${CI:-green} in
    green) echo "ci-done: $(printf '%.8s' "$1") green" ;;
    red) echo "ci-done: $(printf '%.8s' "$1") red: fixtures clippy" ;;
    *) echo "ci-done: $(printf '%.8s' "$1") not finished: 1 of 2 workflow runs and 3 of 5 check runs complete." >&2; exit 1 ;;
esac
EOF
export HW_QUEUE_DONE="$scratch/queue-done.sh" HW_CI_DONE="$scratch/ci-done.sh"
export HW_SETTLE_CAP=1 HW_SETTLE_POLL=1
sha=0123456789abcdef0123456789abcdef01234567

run() {
    : > "$LOG"
    PATH="$scratch/bin:$PATH" HEAD="${HEAD:-$sha}" CLOSES="${CLOSES:-[]}" CLOSE_STATE="${CLOSE_STATE:-closed}" \
        ACCOUNTED="${ACCOUNTED:-1}" QUEUE="${QUEUE:-merged}" CI="${CI:-green}" sh "$tool" "$@"
}

printf '# main: one read of CI on the tip, never a wait, exit 0 whatever it says\n'
out=$(run main "$sha" 2>&1); status=$?
same 'green: exit 0 and the MAIN line' "0 MAIN: $sha green" "$status $out"
out=$(CI=red run main "$sha" 2>&1); status=$?
same 'red: exit 0 and the checks named' "0 MAIN: $sha red: fixtures clippy" "$status $out"
out=$(CI=pending run main "$sha" 2>&1); status=$?
same 'unfinished: exit 0, said, and not waited on' "0 1 1" "$status $(printf '%s' "$out" | grep -c "^MAIN: $sha unfinished") $(grep -c '^ci-done' "$LOG")"
git init -q "$scratch/repo"
git -C "$scratch/repo" -c user.name=t -c user.email=t@t commit -q --allow-empty -m first
git -C "$scratch/repo" branch -q -M main
git init -q --bare "$scratch/origin.git"
git -C "$scratch/repo" remote add origin "$scratch/origin.git"
git -C "$scratch/repo" push -q -u origin main 2>/dev/null
tip=$(git -C "$scratch/repo" rev-parse HEAD)
out=$(cd "$scratch/repo" && run main 2>&1); status=$?
same 'with no sha, the tip of origin/main is read' "0 MAIN: $tip green" "$status $out"
run main xyz >/dev/null 2>&1; status=$?
same 'a sha that is not hex is refused with exit 2' 2 "$status"

printf '\n# check: the head, the closes and the clause check, before the enqueue\n'
out=$(CLOSES='[9]' run check 77 2>&1); status=$?
same 'exit 0 when the clause check passes' 0 "$status"
same '  and the block' "HEAD: $sha|CLOSES: [9]|clause-check: #9, 1 of 1 clauses marked met." "$(printf '%s\n' "$out" | paste -sd'|')"
out=$(CLOSES='[9]' ACCOUNTED=0 run check 77 2>&1); status=$?
same 'exit 1 when the clause check refuses' 1 "$status"
same '  and the refusal reaches the integrator' 1 "$(printf '%s\n' "$out" | grep -c 'clause-check: #77 refused')"
out=$(CLOSES='[9]' run check 77 "$sha" 2>&1); status=$?
same 'with the ruled sha, and the head at it: exit 0' 0 "$status"
out=$(CLOSES='[9]' run check 77 0123456 2>&1); status=$?
same 'a seven-digit ruled sha that prefixes the head is the same commit' 0 "$status"
out=$(CLOSES='[9]' run check 77 fedcba9876543210fedcba9876543210fedcba98 2>&1); status=$?
same 'a head that moved past the ruled sha: exit 1' 1 "$status"
same '  and the HEAD MOVED line names both' "HEAD MOVED: #77 is at $sha, and the ruling was at fedcba9876543210fedcba9876543210fedcba98. The parent rules it again." "$(printf '%s\n' "$out" | grep '^HEAD MOVED')"
same '  and nothing further was read' 0 "$(grep -c graphql "$LOG")"
out=$(CLOSES='[]' run check 77 2>&1); status=$?
same 'a Refs pull request closes nothing and owes no clause' "0 clause-check: #77 closes no issue, so it owes no clause." "$status $(printf '%s\n' "$out" | tail -n 1)"

printf '\n# settle: one wait on the queue, then the closes and the release on a merge\n'
r=$(sh "$rundir" start s1 2>/dev/null)
sh "$rundir" queue "$r" 9 'title=Nine' milestone=none reader=x artifacts=x scope=narrow >/dev/null
sh "$rundir" claim "$r" 9 issue-9 a9 b9 >/dev/null
out=$(CLOSES='[9]' run settle "$r" 77 2>&1); status=$?
same 'merged: exit 0' 0 "$status"
same '  and the block: MERGED, CLOSED, RELEASED' "MERGED: 1234567890abcdef1234567890abcdef12345678 for #77|CLOSED: #9|RELEASED: 2 artifacts held by #9" "$(printf '%s\n' "$out" | paste -sd'|')"
same '  and the close was asked and re-read' 1 "$(grep -c 'X PATCH repos/headwater-ai/headwater/issues/9 -f state=closed' "$LOG")"
same '  and the claims are gone' 0 "$(ls "$r/claims/artifacts" 2>/dev/null | grep -c .)"
same '  and the queue was asked once' 1 "$(grep -c '^queue-done' "$LOG")"
sh "$rundir" claim "$r" 9 issue-9 a9 >/dev/null
out=$(CLOSES='[9]' CLOSE_STATE=open run settle "$r" 77 2>&1); status=$?
same 'a close that did not take is a LEFT line, and the release still happens' "MERGED: 1234567890abcdef1234567890abcdef12345678 for #77|LEFT: #9 close did not take; run \`sh tools/run/gh-issue.sh close 9\`|RELEASED: 1 artifacts held by #9" "$(printf '%s\n' "$out" | paste -sd'|')"
sh "$rundir" queue "$r" 10 'title=Ten' milestone=none reader=x artifacts=x scope=narrow >/dev/null
sh "$rundir" claim "$r" 10 issue-10 a10 >/dev/null
out=$(CLOSES='[]' run settle "$r" 78 10 2>&1); status=$?
same 'a Refs merge closes nothing, and releases the ruled issue it was given' "0|MERGED: 1234567890abcdef1234567890abcdef12345678 for #78|RELEASED: 1 artifacts held by #10" "$status|$(printf '%s\n' "$out" | paste -sd'|')"

printf '\n# settle: ejected, unmergeable, stalled, not queued\n'
out=$(QUEUE=ejected run settle "$r" 77 2>&1); status=$?
same 'ejected: exit 0 and the reason' "0 EJECTED: #77: CI failed" "$status $out"
same '  and no close was asked' 0 "$(grep -c 'state=closed' "$LOG")"
out=$(QUEUE=unmergeable run settle "$r" 77 2>&1); status=$?
same 'unmergeable: the merge_conflict ejection' "0 EJECTED: #77: merge_conflict (unmergeable)" "$status $out"
out=$(QUEUE=stalled run settle "$r" 77 2>&1); status=$?
same 'stalled: a LEFT line with what queue-done said' "0 LEFT: #77 stalled: every check of its merge group is green and it has not merged in 12 minutes" "$status $out"
out=$(QUEUE=notqueued run settle "$r" 77 2>&1); status=$?
same 'not queued: a LEFT line' "0 LEFT: #77 not queued: open, in no queue, and never removed from one" "$status $out"

printf '\n# settle: the wait that runs out is a RE-ISSUE, passed through as exit 2\n'
out=$(QUEUE=waiting run settle "$r" 77 2>&1); status=$?
same 'exit 2' 2 "$status"
same '  and the RE-ISSUE line is the last' 1 "$(printf '%s\n' "$out" | tail -n 1 | grep -c 'RE-ISSUE')"

printf '\n# refusals\n'
run settle "$r" seventy >/dev/null 2>&1; status=$?
same 'a pull request that is not a number is refused with exit 2' 2 "$status"
run settle "$r" >/dev/null 2>&1; status=$?
same 'settle without a pull request is a usage error' 2 "$status"
run check >/dev/null 2>&1; status=$?
same 'check without a pull request is a usage error' 2 "$status"
run polish 1 >/dev/null 2>&1; status=$?
same 'a verb that does not exist is a usage error' 2 "$status"

printf '\n%s passed, %s failed\n' "$passed" "$failed"
[ "$failed" -eq 0 ]
