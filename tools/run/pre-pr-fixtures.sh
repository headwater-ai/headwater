#!/bin/sh
# What holds `tools/run/pre-pr.sh`: the steps run in the order the build
# definition states, the first failure stops the rest and is named, a
# conflict with `origin/main` is aborted and named, what the regenerate
# moved is listed, and the main checkout and `main` are refused.
#
# Run it from anywhere:
#     sh tools/run/pre-pr-fixtures.sh
#
# It needs `git` and nothing else. A bare origin, a main checkout and a
# builder's worktree live under a temporary directory. `HW_CARGO` names a
# stub wrapper that logs what it was asked and fails on request,
# `HEADWATER_ENGINE` a stub engine that logs, moves a file on `generate`
# and fails on request, and `HW_PRE_PR_FIXTURES` two fixture scripts in the
# tree, one of which fails on request.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
tool="$root/tools/run/pre-pr.sh"

scratch=$(mktemp -d) || exit 1
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

passed=0
failed=0
pass() { printf 'ok   %s\n' "$1"; passed=$((passed + 1)); }
fail() { printf 'FAIL %s\n  %s\n' "$1" "$2"; failed=$((failed + 1)); }
same() {
    if [ "$2" = "$3" ]; then pass "$1"; else fail "$1" "expected \`$2\`, got \`$3\`"; fi
}

cat > "$scratch/cargo" <<'EOF'
#!/bin/sh
printf 'cargo %s\n' "$1" >> "$LOG"
[ "${FAIL_AT:-}" = "cargo-$1" ] && { echo "$1 failed"; exit 1; }
exit 0
EOF
cat > "$scratch/engine" <<'EOF'
#!/bin/sh
printf 'engine %s\n' "$*" >> "$LOG"
case "$1 ${2:-}" in
    'generate --root') printf 'regenerated\n' > "$3/docs/derived.md" ;;
esac
[ "${FAIL_AT:-}" = "engine-$1${2:+ $2}" ] && { echo "$1 failed"; exit 1; }
exit 0
EOF
chmod +x "$scratch/cargo" "$scratch/engine"
export HW_CARGO="$scratch/cargo" HEADWATER_ENGINE="$scratch/engine" LOG="$scratch/log"
export HEADWATER_SCRATCH="$scratch/jobtmp"
export HW_PRE_PR_FIXTURES='t/ok.sh t/bad.sh'

git init -q --bare "$scratch/origin.git"
git -C "$scratch" clone -q "$scratch/origin.git" main 2>/dev/null
main="$scratch/main"
git -C "$main" config user.name t
git -C "$main" config user.email t@t
mkdir -p "$main/docs" "$main/t"
printf 'stale\n' > "$main/docs/derived.md"
printf 'exit 0\n' > "$main/t/ok.sh"
printf '[ "${FAIL_AT:-}" = fixture ] && exit 1; exit 0\n' > "$main/t/bad.sh"
printf 'one\n' > "$main/shared.txt"
git -C "$main" add -A
git -C "$main" -c user.name=t -c user.email=t@t commit -q -m first
git -C "$main" branch -q -M main
git -C "$main" push -q -u origin main 2>/dev/null
git -C "$main" worktree add -q "$scratch/wt" -b issue-7 2>/dev/null
wt="$scratch/wt"
printf 'the build\n' > "$wt/build.txt"
git -C "$wt" add -A
git -C "$wt" -c user.name=t -c user.email=t@t commit -q -m 'the build'
# origin/main moves on, by one commit the branch does not have.
git -C "$scratch" clone -q -b main "$scratch/origin.git" other 2>/dev/null
printf 'later\n' > "$scratch/other/later.txt"
git -C "$scratch/other" add -A
git -C "$scratch/other" -c user.name=t -c user.email=t@t commit -q -m later
git -C "$scratch/other" push -q origin HEAD:main 2>/dev/null
later=$(git -C "$scratch/other" rev-parse --short HEAD)

run() {
    : > "$LOG"
    (cd "$wt" && FAIL_AT="${FAIL_AT:-}" sh "$tool" "$@")
}

printf '# every step passes, in the stated order, and the block says what moved\n'
out=$(run 7 2>&1); status=$?
same 'exit 0' 0 "$status"
same '  and the first line is PRE-PR: ok' 'PRE-PR: ok' "$(printf '%s\n' "$out" | head -n 1)"
same '  and origin/main was merged, one commit brought in' "MERGED: origin/main at $later, 1 commits brought in" "$(printf '%s\n' "$out" | grep '^MERGED:')"
same '  and the branch now has that commit' 1 "$(ls "$wt/later.txt" 2>/dev/null | grep -c .)"
same '  and the steps ran in order: build, generate, test, fmt, generate --check, check --strict' \
    "cargo build|engine generate --root $wt|cargo test|cargo fmt|engine generate --check --root $wt|engine check --strict --root $wt" \
    "$(paste -sd'|' "$LOG")"
same '  and the block names each' 'BUILT: '"$scratch/engine"'|GENERATED: ok|SUITE: pass|FMT: ok|CHECK: ok|FIXTURES: ok' \
    "$(printf '%s\n' "$out" | grep -E '^(BUILT|GENERATED|SUITE|FMT|CHECK|FIXTURES):' | paste -sd'|')"
same '  and what the regenerate moved is listed' 'MOVED: docs/derived.md' "$(printf '%s\n' "$out" | grep '^MOVED:')"
same '  and the logs are under the issue'"'"'s scratch' "LOGS: $scratch/jobtmp/issue-7/pre-pr" "$(printf '%s\n' "$out" | grep '^LOGS:')"
same '  and one log per step is there' 1 "$(ls "$scratch/jobtmp/issue-7/pre-pr/suite.log" 2>/dev/null | grep -c .)"
same '  and the fixture logs are named by their path' 2 "$(ls "$scratch/jobtmp/issue-7/pre-pr/" | grep -c '^t-')"
git -C "$wt" checkout -q -- docs/derived.md

printf '\n# the first failure stops the rest and is named\n'
out=$(FAIL_AT=cargo-test run 7 2>&1); status=$?
same 'a failed suite is exit 1' 1 "$status"
same '  and the first line names the step' 'PRE-PR: failed at suite' "$(printf '%s\n' "$out" | head -n 1)"
same '  and the SUITE line names its log' "SUITE: fail ($scratch/jobtmp/issue-7/pre-pr/suite.log)" "$(printf '%s\n' "$out" | grep '^SUITE:')"
same '  and the later steps were not reached' 'FMT: not reached|CHECK: not reached|FIXTURES: not reached' "$(printf '%s\n' "$out" | grep -E '^(FMT|CHECK|FIXTURES):' | paste -sd'|')"
same '  and fmt was not run' 0 "$(grep -c 'cargo fmt' "$LOG")"
git -C "$wt" checkout -q -- docs/derived.md
out=$(FAIL_AT='engine-check --strict' run 7 2>&1); status=$?
same 'a failed commit gate names which half' "CHECK: fail, check --strict ($scratch/jobtmp/issue-7/pre-pr/check.log)" "$(printf '%s\n' "$out" | grep '^CHECK:')"
git -C "$wt" checkout -q -- docs/derived.md
out=$(FAIL_AT=fixture run 7 2>&1); status=$?
same 'a failed fixture suite is exit 1, and every suite still ran' "1 2" "$status $(ls "$scratch/jobtmp/issue-7/pre-pr/" | grep -c '^t-')"
same '  and the FIXTURES line names the one that failed' 'FIXTURES: failed: t/bad.sh' "$(printf '%s\n' "$out" | grep '^FIXTURES:')"
git -C "$wt" checkout -q -- docs/derived.md

printf '\n# a conflict with origin/main is aborted and named\n'
printf 'two\n' > "$scratch/other/shared.txt"
git -C "$scratch/other" -c user.name=t -c user.email=t@t commit -q -am 'theirs'
git -C "$scratch/other" push -q origin HEAD:main 2>/dev/null
printf 'three\n' > "$wt/shared.txt"
git -C "$wt" -c user.name=t -c user.email=t@t commit -q -am 'ours'
before=$(git -C "$wt" rev-parse HEAD)
out=$(run 7 2>&1); status=$?
same 'exit 1' 1 "$status"
same '  and the first line names the merge' 'PRE-PR: failed at merge' "$(printf '%s\n' "$out" | head -n 1)"
same '  and the MERGED line names the file' 'MERGED: conflict on shared.txt' "$(printf '%s\n' "$out" | grep '^MERGED:' | sed 's/ *$//')"
same '  and the merge was aborted: HEAD is where it was' "$before" "$(git -C "$wt" rev-parse HEAD)"
same '  and no MERGE_HEAD is left' 0 "$(ls "$main/.git/worktrees/wt" | grep -c '^MERGE_HEAD$')"
same '  and nothing was built' 0 "$(grep -c . "$LOG")"
git -C "$wt" reset -q --hard origin/main

printf '\n# refusals, each with exit 2 and nothing run\n'
: > "$LOG"
(cd "$main" && sh "$tool" 7) >/dev/null 2>"$scratch/err"; status=$?
same 'the main checkout is refused' 2 "$status"
same '  and says so' 1 "$(grep -c 'main checkout' "$scratch/err")"
# `main` can be checked out in one worktree at a time, so the main checkout
# steps off it for the length of this case.
git -C "$main" switch -q --detach
git -C "$wt" switch -q main
(cd "$wt" && sh "$tool" 7) >/dev/null 2>"$scratch/err"; status=$?
same 'a worktree on main is refused' 2 "$status"
same '  and says so' 1 "$(grep -c 'runs on its own branch' "$scratch/err")"
git -C "$wt" switch -q --detach
(cd "$wt" && sh "$tool" 7) >/dev/null 2>"$scratch/err"; status=$?
same 'a detached worktree is refused' 2 "$status"
git -C "$wt" switch -q issue-7
git -C "$main" switch -q main
(cd "$wt" && sh "$tool" seven) >/dev/null 2>&1; status=$?
same 'an issue that is not a number is refused' 2 "$status"
(cd "$wt" && sh "$tool") >/dev/null 2>&1; status=$?
same 'no argument is a usage error' 2 "$status"
same '  and nothing ran' 0 "$(grep -c . "$LOG")"
out=$(cd "$wt" && sh "$tool" none 2>&1); status=$?
same 'none is the run-lessons build, with its own scratch' "0 LOGS: $scratch/jobtmp/run-lessons/pre-pr" "$status $(printf '%s\n' "$out" | grep '^LOGS:')"

printf '\n# the default list holds the probe suites CI runs\n'
# PR #1894 and #1902 each printed PRE-PR: ok and failed one of these in CI.
defaults=$(sed -n 's/^fixtures=\${HW_PRE_PR_FIXTURES:-"\(.*\)"}$/\1/p' "$tool")
for suite in tools/probe/probe-record-fixtures.sh tools/probe/oracle-fixtures.sh tools/probe/probe-oracle-fixtures.sh; do
    case " $defaults " in
        *" $suite "*) pass "the default fixture list runs $suite" ;;
        *) fail "the default fixture list runs $suite" "it is not in \`$defaults\`" ;;
    esac
done

printf '\n%s passed, %s failed\n' "$passed" "$failed"
[ "$failed" -eq 0 ]
