#!/bin/sh
# What holds `tools/run/verify-setup.sh`: it detaches the worktree at the
# branch's remote tip, names the head and the slot, refuses to build under
# the disk floor, refuses the main checkout, and `--clean` drops the slot.
#
# Run it from anywhere:
#     sh tools/run/verify-setup-fixtures.sh
#
# It needs `git` and nothing else. A bare origin, a main checkout and a
# worktree of it live under a temporary directory, and a stub disk reader
# on `HW_DISK_FREE` answers whatever free space a case names.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
tool="$root/tools/run/verify-setup.sh"

scratch=$(mktemp -d) || exit 1
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

passed=0
failed=0
pass() { printf 'ok   %s\n' "$1"; passed=$((passed + 1)); }
fail() { printf 'FAIL %s\n  %s\n' "$1" "$2"; failed=$((failed + 1)); }
same() {
    if [ "$2" = "$3" ]; then pass "$1"; else fail "$1" "expected \`$2\`, got \`$3\`"; fi
}

# The stub disk reader prints the two lines the real one prints, with the
# free space FAKE_FREE names, and refuses under the floor as the real one does.
cat > "$scratch/disk-free.sh" <<'EOF'
#!/bin/sh
floor=''
[ "${1:-}" = --floor ] && floor=$2
echo "pool       /p  mount=/mnt/build  free=${FAKE_FREE}G"
echo "worktrees  /w  mount=/home  free=500G"
if [ -n "$floor" ] && [ "$FAKE_FREE" -lt "$floor" ]; then
    echo "disk-free: the pool's volume has ${FAKE_FREE}G free, under the floor of ${floor}G." >&2
    exit 1
fi
EOF
export HW_DISK_FREE="$scratch/disk-free.sh"
export HW_CARGO_POOL="$scratch/pool"

git init -q --bare "$scratch/origin.git"
git -C "$scratch" clone -q "$scratch/origin.git" main 2>/dev/null
main="$scratch/main"
git -C "$main" -c user.name=t -c user.email=t@t commit -q --allow-empty -m first
git -C "$main" branch -q -M main
git -C "$main" push -q -u origin main 2>/dev/null
git -C "$main" worktree add -q "$scratch/wt" -b wt 2>/dev/null
wt="$scratch/wt"
# The branch under verify, pushed from another clone, so the worktree only
# has it after the fetch the tool makes.
git -C "$scratch" clone -q -b main "$scratch/origin.git" builder 2>/dev/null
git -C "$scratch/builder" switch -q -c issue-5
git -C "$scratch/builder" -c user.name=t -c user.email=t@t commit -q --allow-empty -m 'the build'
git -C "$scratch/builder" push -q origin issue-5 2>/dev/null
tip=$(git -C "$scratch/builder" rev-parse HEAD)

printf '# the setup detaches at the branch, names the head and the slot, and reads the disk\n'
out=$(cd "$wt" && FAKE_FREE=120 sh "$tool" 5 issue-5 2>&1); status=$?
same 'exit 0 with the pool above the floor' 0 "$status"
same '  and HEAD is the remote tip of the branch' "HEAD: $tip" "$(printf '%s\n' "$out" | grep '^HEAD:')"
same '  and the worktree is detached there' "$tip" "$(git -C "$wt" rev-parse HEAD)"
same '  and detached, not on a branch' '' "$(git -C "$wt" symbolic-ref --short -q HEAD)"
same '  and the slot is the issue'"'"'s' 'SLOT: HW_CARGO_SLOT=verify-5' "$(printf '%s\n' "$out" | grep '^SLOT:')"
same '  and the disk lines follow' 'pool       /p  mount=/mnt/build  free=120G' "$(printf '%s\n' "$out" | grep '^pool')"
same '  and no disk: line' 0 "$(printf '%s\n' "$out" | grep -c '^disk:')"

printf '\n# under the floor: exit 1, and disk: is the first line\n'
out=$(cd "$wt" && FAKE_FREE=12 sh "$tool" 5 issue-5 2>&1); status=$?
same 'exit 1' 1 "$status"
same '  and the first line is the reading, for the FAIL block' "disk: the pool's volume has 12G free, under the floor of 40G." "$(printf '%s\n' "$out" | head -n 1)"
same '  and the head is still named, for the report' "HEAD: $tip" "$(printf '%s\n' "$out" | grep '^HEAD:')"
out=$(cd "$wt" && FAKE_FREE=12 HW_VERIFY_FLOOR=10 sh "$tool" 5 issue-5 2>&1); status=$?
same 'HW_VERIFY_FLOOR lowers the floor' 0 "$status"

printf '\n# the run-lessons verify has no issue, and its own slot\n'
out=$(cd "$wt" && FAKE_FREE=120 sh "$tool" none issue-5 2>&1); status=$?
same 'none is accepted' 0 "$status"
same '  and the slot is the lessons slot' 'SLOT: HW_CARGO_SLOT=verify-lessons' "$(printf '%s\n' "$out" | grep '^SLOT:')"

printf '\n# refusals, each with exit 2\n'
(cd "$main" && FAKE_FREE=120 sh "$tool" 5 issue-5) >/dev/null 2>"$scratch/err"; status=$?
same 'the main checkout is refused' 2 "$status"
same '  and says why' 1 "$(grep -c 'main checkout' "$scratch/err")"
same '  and the main checkout is still on main' main "$(git -C "$main" symbolic-ref --short -q HEAD)"
(cd "$wt" && FAKE_FREE=120 sh "$tool" 5 no-such-branch) >/dev/null 2>"$scratch/err"; status=$?
same 'a branch origin does not have is refused' 2 "$status"
same '  and says which' 1 "$(grep -c 'no `origin/no-such-branch`' "$scratch/err")"
(cd "$wt" && FAKE_FREE=120 sh "$tool" five issue-5) >/dev/null 2>&1; status=$?
same 'an issue that is not a number is refused' 2 "$status"
(cd "$wt" && FAKE_FREE=120 sh "$tool" 5 -- 2>/dev/null) >/dev/null 2>&1; status=$?
same 'a branch that opens with a dash is refused' 2 "$status"
(cd "$wt" && sh "$tool" 5) >/dev/null 2>&1; status=$?
same 'one argument is a usage error' 2 "$status"
(cd "$scratch" && FAKE_FREE=120 sh "$tool" 5 issue-5) >/dev/null 2>&1; status=$?
same 'outside a git tree is refused' 2 "$status"

printf '\n# --clean drops the slot from the pool\n'
mkdir -p "$HW_CARGO_POOL/target-verify-5/dev-release" "$HW_CARGO_POOL/target-verify-7"
: > "$HW_CARGO_POOL/target-verify-5.root"
out=$(sh "$tool" --clean 5 2>&1); status=$?
same 'exit 0 and the CLEANED line' "0 CLEANED: $HW_CARGO_POOL/target-verify-5 and its .root" "$status $out"
same '  and the slot and its .root are gone' '0' "$(ls -d "$HW_CARGO_POOL"/target-verify-5* 2>/dev/null | grep -c .)"
same '  and another slot is untouched' 1 "$(ls -d "$HW_CARGO_POOL/target-verify-7" 2>/dev/null | grep -c .)"
sh "$tool" --clean 5 >/dev/null 2>&1; status=$?
same 'a second clean of a slot that is gone is still exit 0' 0 "$status"
sh "$tool" --clean five >/dev/null 2>&1; status=$?
same 'a non-numeric issue is refused' 2 "$status"

printf '\n%s passed, %s failed\n' "$passed" "$failed"
[ "$failed" -eq 0 ]
