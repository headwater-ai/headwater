#!/bin/sh
# What holds `tools/run/pr-settled.sh`: the condition is not met while
# GitHub has not computed `mergeable`, not met while CI on the commit is
# unfinished, met once both are, and a head that moved is said.
#
# Run it from anywhere:
#     sh tools/run/pr-settled-fixtures.sh
#
# It needs no network and no token: `gh` on `PATH` is a fake that answers
# the one endpoint the tool reads, and `HW_CI_DONE` names a stub CI reader.
# The stub's answer is `HW_CI`, never `CI`: GitHub Actions exports `CI=true`.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
tool="$root/tools/run/pr-settled.sh"

scratch=$(mktemp -d) || exit 1
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

passed=0
failed=0
pass() { printf 'ok   %s\n' "$1"; passed=$((passed + 1)); }
fail() { printf 'FAIL %s\n  %s\n' "$1" "$2"; failed=$((failed + 1)); }
same() {
    if [ "$2" = "$3" ]; then pass "$1"; else fail "$1" "expected \`$2\`, got \`$3\`"; fi
}

mkdir -p "$scratch/bin"
cat > "$scratch/bin/gh" <<'EOF'
#!/bin/sh
printf '%s\n' "$*" >> "$LOG"
case "$*" in
    *"repos/headwater-ai/headwater/pulls/77 --jq"*)
        [ "${GH_EMPTY:-}" = 1 ] && exit 0
        printf '%s %s %s\n' "$MERGEABLE" "$STATE" "$HEAD" ;;
    *) exit 1 ;;
esac
EOF
chmod +x "$scratch/bin/gh"
cat > "$scratch/ci-done.sh" <<'EOF'
#!/bin/sh
printf 'ci-done %s\n' "$1" >> "$LOG"
case ${HW_CI:-green} in
    green) echo "run 1 completed success"; echo "ci-done: $(printf '%.8s' "$1") green" ;;
    red) echo "ci-done: $(printf '%.8s' "$1") red: fixtures" ;;
    *) echo "ci-done: $(printf '%.8s' "$1") not finished: 1 of 2 workflow runs and 3 of 5 check runs complete." >&2; exit 1 ;;
esac
EOF
export HW_CI_DONE="$scratch/ci-done.sh"
export LOG="$scratch/log"
sha=0123456789abcdef0123456789abcdef01234567

run() {
    : > "$LOG"
    PATH="$scratch/bin:$PATH" MERGEABLE="${MERGEABLE:-true}" STATE="${STATE:-clean}" HEAD="${HEAD:-$sha}" \
        HW_CI="${HW_CI:-green}" GH_EMPTY="${GH_EMPTY:-}" sh "$tool" "$@"
}

printf '# not settled while mergeable is null or the state is unknown, and CI is not asked\n'
out=$(MERGEABLE=null STATE=unknown run 77 "$sha" 2>&1); status=$?
same 'mergeable null: exit 1' 1 "$status"
same '  and the line says it is being computed' 1 "$(printf '%s\n' "$out" | grep -c 'mergeable is still being computed')"
same '  and ci-done was not run' 0 "$(grep -c '^ci-done' "$LOG")"
out=$(MERGEABLE=true STATE=unknown run 77 "$sha" 2>&1); status=$?
same 'mergeable true with state unknown: exit 1' 1 "$status"
out=$(GH_EMPTY=1 run 77 "$sha" 2>&1); status=$?
same 'an empty answer from GitHub is exit 1, an error and not a verdict' 1 "$status"
same '  and the line says so' 1 "$(printf '%s\n' "$out" | grep -c 'answered nothing')"

printf '\n# not settled while CI is unfinished\n'
out=$(HW_CI=pending run 77 "$sha" 2>&1); status=$?
same 'CI unfinished: exit 1' 1 "$status"
same '  and the ci-done line is the one on stderr' 1 "$(printf '%s\n' "$out" | grep -c 'not finished: 1 of 2')"
same '  and ci-done was asked about the sha the verifier detached at' "ci-done $sha" "$(grep '^ci-done' "$LOG")"

printf '\n# settled: mergeable computed and CI finished, green or red alike\n'
out=$(run 77 "$sha" 2>&1); status=$?
same 'exit 0' 0 "$status"
same '  and the block' "mergeable: true (clean)|run 1 completed success|ci-done: 01234567 green|pr-settled: #77 at $sha settled" "$(printf '%s\n' "$out" | paste -sd'|')"
out=$(MERGEABLE=false STATE=dirty HW_CI=red run 77 "$sha" 2>&1); status=$?
same 'red CI and an unmergeable state still settle, with exit 0: the verdict is the verifier'"'"'s' 0 "$status"
same '  and the lines carry both' 'mergeable: false (dirty)|ci-done: 01234567 red: fixtures' "$(printf '%s\n' "$out" | grep -E '^(mergeable|ci-done)' | paste -sd'|')"

printf '\n# a head that moved past the commit is said\n'
moved=fedcba9876543210fedcba9876543210fedcba98
out=$(HEAD=$moved run 77 "$sha" 2>&1); status=$?
same 'exit 0' 0 "$status"
same '  and the head line names both' "head: $moved, not $sha" "$(printf '%s\n' "$out" | grep '^head:')"
out=$(run 77 0123456 2>&1); status=$?
same 'a short sha that is a prefix of the head is the same commit' 0 "$(printf '%s\n' "$out" | grep -c '^head:')"

printf '\n# refusals, each with exit 2 and no call made\n'
: > "$LOG"
PATH="$scratch/bin:$PATH" sh "$tool" 77 >/dev/null 2>&1; status=$?
same 'one argument is a usage error' 2 "$status"
PATH="$scratch/bin:$PATH" sh "$tool" seventy "$sha" >/dev/null 2>&1; status=$?
same 'a pull request that is not a number is refused' 2 "$status"
PATH="$scratch/bin:$PATH" sh "$tool" 77 HEAD >/dev/null 2>&1; status=$?
same 'a sha that is not hex is refused' 2 "$status"
PATH="$scratch/bin:$PATH" sh "$tool" 77 abc >/dev/null 2>&1; status=$?
same 'a sha under seven digits is refused' 2 "$status"
same '  and gh was never called' 0 "$(grep -c . "$LOG")"

printf '\n%s passed, %s failed\n' "$passed" "$failed"
[ "$failed" -eq 0 ]
