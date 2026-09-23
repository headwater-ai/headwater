#!/bin/sh
# What holds `tools/run/ci-done.sh`: a commit with no workflow run, a running
# workflow, or a check run still going is not finished; a finished commit is
# reported green or red by name and red still exits 0; a short sha is resolved
# before the runs endpoint is asked; and the legacy combined-status endpoint,
# which answers `pending` forever on this repository, is never read.
#
# Run it from anywhere:
#     sh tools/run/ci-done-fixtures.sh
#
# It needs no network and no token: `gh` is a fake on `PATH` for the length of
# this suite, which logs its arguments and prints the TSV that the real `gh`
# would print through the script's own `--jq` filter.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
tool="$root/tools/run/ci-done.sh"

if [ ! -f "$tool" ]; then
    echo "no script at \`tools/run/ci-done.sh\`." >&2
    exit 1
fi

scratch=$(mktemp -d) || exit 1
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

fakebin="$scratch/fakebin"
mkdir -p "$fakebin"
log="$scratch/gh.log"

full=0123456789abcdef0123456789abcdef01234567

cat > "$fakebin/gh" <<'EOF'
#!/bin/sh
printf '%s\n' "$*" >> "$LOG"
case "$*" in
    *"/commits/0123456"*"/check-runs"*) cat "$CHECKS" ;;
    *"actions/runs?head_sha=0123456789abcdef0123456789abcdef01234567"*) cat "$RUNS" ;;
    *"actions/runs"*) : ;;
    *"/commits/0123456 "* | *"/commits/0123456789abcdef0123456789abcdef01234567 "*) echo 0123456789abcdef0123456789abcdef01234567 ;;
    *) exit 1 ;;
esac
EOF
chmod +x "$fakebin/gh"

runs="$scratch/runs.tsv"
checks="$scratch/checks.tsv"
t=$(printf '\t')

passed=0
failed=0

run() {
    : > "$log"
    PATH="$fakebin:$PATH" LOG="$log" RUNS="$runs" CHECKS="$checks" sh "$tool" "$@" > "$scratch/out" 2> "$scratch/err"
}

ok() { passed=$((passed + 1)); echo "  ok    $1"; }
bad() { failed=$((failed + 1)); echo "  FAIL  $1"; echo "        out: $(cat "$scratch/out")"; echo "        err: $(cat "$scratch/err")"; }

echo "a commit with no workflow run is not finished"
: > "$runs"; : > "$checks"
run "$full"; status=$?
if [ "$status" -eq 1 ] && grep -q 'no workflow run yet' "$scratch/err"; then ok "no run: exit 1, and the line says a conflicting pull request starts none"; else bad "no run (exit $status)"; fi

echo "a workflow still running is not finished, whatever its check runs say"
printf '11%sin_progress%s-%sCI\n' "$t" "$t" "$t" > "$runs"
printf 'completed%ssuccess%sLint\n' "$t" "$t" > "$checks"
run "$full"; status=$?
if [ "$status" -eq 1 ] && grep -q '0 of 1 workflow runs and 1 of 1 check runs' "$scratch/err"; then ok "a running workflow behind finished check runs holds the wait"; else bad "running workflow (exit $status)"; fi

echo "a check run from outside Actions still going is not finished"
printf '11%scompleted%ssuccess%sCI\n' "$t" "$t" "$t" > "$runs"
printf 'completed%ssuccess%sLint\nin_progress%s-%sWorkers Builds\n' "$t" "$t" "$t" "$t" > "$checks"
run "$full"; status=$?
if [ "$status" -eq 1 ]; then ok "a pending external check holds the wait"; else bad "external check (exit $status)"; fi

echo "a finished green commit exits 0 and names its run"
printf '11%scompleted%ssuccess%sCI\n12%scompleted%sskipped%sCI\n' "$t" "$t" "$t" "$t" "$t" "$t" > "$runs"
printf 'completed%ssuccess%sLint\ncompleted%sskipped%sEngine tests\n' "$t" "$t" "$t" "$t" > "$checks"
run "$full"; status=$?
if [ "$status" -eq 0 ] && grep -q '^run 11 success CI$' "$scratch/out" && grep -q 'ci-done: 01234567 green' "$scratch/out"; then ok "green, with a skipped duplicate run not counted red"; else bad "green (exit $status)"; fi

echo "a finished red commit exits 0 and names what failed"
printf '11%scompleted%sfailure%sCI\n' "$t" "$t" "$t" > "$runs"
printf 'completed%ssuccess%sLint\ncompleted%sfailure%sEngine tests\n' "$t" "$t" "$t" "$t" > "$checks"
run "$full"; status=$?
if [ "$status" -eq 0 ] && grep -q 'ci-done: 01234567 red: Engine tests, workflow CI' "$scratch/out"; then ok "red ends the wait and names the check and the workflow"; else bad "red (exit $status)"; fi

echo "a workflow that failed before any job is red"
printf '11%scompleted%sstartup_failure%sCI\n' "$t" "$t" "$t" > "$runs"
: > "$checks"
run "$full"; status=$?
if [ "$status" -eq 0 ] && grep -q 'red: workflow CI' "$scratch/out"; then ok "a startup failure with no check run is red"; else bad "startup failure (exit $status)"; fi

echo "a short sha is resolved before the runs endpoint is asked"
printf '11%scompleted%ssuccess%sCI\n' "$t" "$t" "$t" > "$runs"
printf 'completed%ssuccess%sLint\n' "$t" "$t" > "$checks"
run 0123456; status=$?
if [ "$status" -eq 0 ] && grep -q "head_sha=$full" "$log"; then ok "the runs endpoint is asked with the full sha"; else bad "short sha (exit $status)"; fi

echo "the legacy combined-status endpoint is never read"
if ! grep -q '/status' "$log"; then ok "no call reaches commits/<sha>/status"; else bad "a call read the status endpoint: $(grep /status "$log")"; fi

echo "no commit, or one gh cannot resolve, is refused with exit 2"
run; s1=$?
run nonsense; s2=$?
if [ "$s1" -eq 2 ] && [ "$s2" -eq 2 ] && grep -q 'cannot resolve' "$scratch/err"; then ok "a missing and an unknown commit both exit 2"; else bad "refusal (exit $s1, $s2)"; fi

echo "$passed passed; $failed failed"
[ "$failed" -eq 0 ]
