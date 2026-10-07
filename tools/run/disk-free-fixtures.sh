#!/bin/sh
# What holds `tools/run/disk-free.sh`: it measures the volume the cargo pool
# resolves to, through a symlink, and not the volume the configured path
# names; it takes the pool path from the same expression as `tools/hw-cargo`;
# it measures a path that does not exist yet at its deepest existing ancestor
# and creates nothing; it measures the worktrees directory on its own volume;
# and `--floor` fails on the pool's volume alone.
#
# Run it from anywhere:
#     sh tools/run/disk-free-fixtures.sh
#
# A fake `df` on PATH answers per path, so no case depends on the host's
# mounts: any path under `vol-b` is on `/mnt/fake-b`, and any other path is on
# `/mnt/fake-a`. The free figure of each comes from FAKE_A_K and FAKE_B_K, in
# KiB, and every path the fake was asked about is logged.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
tool="$root/tools/run/disk-free.sh"

if [ ! -f "$tool" ]; then
    echo "no script at \`tools/run/disk-free.sh\`." >&2
    exit 1
fi

scratch=$(mktemp -d) || exit 1
trap 'rm -rf "$scratch"' EXIT HUP INT TERM
scratch=$(cd "$scratch" && pwd -P)

passed=0
failed=0
pass() { passed=$((passed + 1)); echo "  ok    $1"; }
fail() { failed=$((failed + 1)); echo "  FAIL  $1"; [ -z "${2:-}" ] || echo "        $2"; }

mkdir -p "$scratch/bin" "$scratch/vol-a" "$scratch/vol-b/real-pool" "$scratch/vol-b/wt" "$scratch/home" "$scratch/xdg"
ln -s "$scratch/vol-b/real-pool" "$scratch/vol-a/cargo-pool"

cat > "$scratch/bin/df" <<'EOF'
#!/bin/sh
# The fake: df -P -k <path>. The last argument is the path.
for p; do path=$p; done
printf '%s\n' "$path" >> "$FAKE_DF_LOG"
case "$path" in
    */vol-b|*/vol-b/*) mount=/mnt/fake-b; avail=${FAKE_B_K:-5242880} ;;
    *) mount=/mnt/fake-a; avail=${FAKE_A_K:-209715200} ;;
esac
echo "Filesystem     1024-blocks      Used Available Capacity Mounted on"
echo "/dev/fake      300000000  1000 $avail 50% $mount"
EOF
chmod +x "$scratch/bin/df"

FAKE_DF_LOG="$scratch/df.log"
export FAKE_DF_LOG

# run <env assignments...> -- <args...>: runs the tool with the fake df,
# HW_WORKTREES on vol-a unless overridden, from $scratch.
run() {
    : > "$FAKE_DF_LOG"
    (
        cd "$scratch" || exit 2
        PATH="$scratch/bin:$PATH"
        HW_WORKTREES="$scratch/vol-a/wt"
        export PATH HW_WORKTREES
        while [ $# -gt 0 ] && [ "$1" != "--" ]; do
            export "$1"
            shift
        done
        [ $# -gt 0 ] && shift
        sh "$tool" "$@"
    ) > "$scratch/out" 2> "$scratch/err"
    status=$?
}

pool_line() { grep '^pool ' "$scratch/out"; }
wt_line() { grep '^worktrees ' "$scratch/out"; }

echo "a pool that is a symlink to another volume is measured on the volume it resolves to"
run HW_CARGO_POOL="$scratch/vol-a/cargo-pool" --
line=$(pool_line)
case "$line" in
    *"mount=/mnt/fake-b"*"free=5G"*)
        case "$line" in
            *fake-a*) fail "the pool line names /mnt/fake-b and not /mnt/fake-a" "pool: \`$line\`" ;;
            *) pass "the pool line names /mnt/fake-b and its 5G, and not /mnt/fake-a" ;;
        esac
        ;;
    *) fail "the pool line names /mnt/fake-b and its 5G, and not /mnt/fake-a" "status $status, pool: \`$line\`, err: $(cat "$scratch/err")" ;;
esac
if grep -qx "$scratch/vol-b/real-pool" "$FAKE_DF_LOG"; then
    pass "and df was asked about the resolved path"
else
    fail "df was asked about the resolved path" "asked: $(tr '\n' ' ' < "$FAKE_DF_LOG")"
fi

echo "with no HW_CARGO_POOL and no XDG_CACHE_HOME the pool is \$HOME/.cache/headwater/cargo-pool"
mkdir -p "$scratch/home/.cache/headwater/cargo-pool"
run HOME="$scratch/home" HW_CARGO_POOL= XDG_CACHE_HOME= --
case "$(pool_line)" in
    "pool       $scratch/home/.cache/headwater/cargo-pool  mount=/mnt/fake-a  free=200G") pass "the HOME default is measured" ;;
    *) fail "the HOME default is measured" "status $status, out: $(cat "$scratch/out"), err: $(cat "$scratch/err")" ;;
esac

echo "with XDG_CACHE_HOME set the pool is under it"
mkdir -p "$scratch/vol-b/xdg/headwater/cargo-pool"
run HOME="$scratch/home" HW_CARGO_POOL= XDG_CACHE_HOME="$scratch/vol-b/xdg" --
case "$(pool_line)" in
    "pool       $scratch/vol-b/xdg/headwater/cargo-pool  mount=/mnt/fake-b  free=5G") pass "the XDG_CACHE_HOME default is measured" ;;
    *) fail "the XDG_CACHE_HOME default is measured" "status $status, out: $(cat "$scratch/out")" ;;
esac

echo "a pool that does not exist yet is measured at its deepest existing ancestor and not created"
run HW_CARGO_POOL="$scratch/vol-b/not/yet/cargo-pool" --
case "$(pool_line)" in
    "pool       $scratch/vol-b  mount=/mnt/fake-b  free=5G") pass "measured at vol-b" ;;
    *) fail "measured at vol-b" "status $status, out: $(cat "$scratch/out"), err: $(cat "$scratch/err")" ;;
esac
if [ -e "$scratch/vol-b/not" ]; then
    fail "nothing is created" "\`$scratch/vol-b/not\` exists"
else
    pass "nothing is created"
fi

echo "the worktrees line is present and names its own mount"
run HW_CARGO_POOL="$scratch/vol-a" HW_WORKTREES="$scratch/vol-b/wt" --
case "$(wt_line)" in
    "worktrees  $scratch/vol-b/wt  mount=/mnt/fake-b  free=5G") pass "the worktrees line is on /mnt/fake-b" ;;
    *) fail "the worktrees line is on /mnt/fake-b" "status $status, out: $(cat "$scratch/out")" ;;
esac
case "$(pool_line)" in
    *"mount=/mnt/fake-a"*) pass "and the pool line is on /mnt/fake-a" ;;
    *) fail "and the pool line is on /mnt/fake-a" "out: $(cat "$scratch/out")" ;;
esac

echo "with no HW_WORKTREES the worktrees directory is <main>/.claude/worktrees"
: > "$FAKE_DF_LOG"
(cd "$root" && PATH="$scratch/bin:$PATH" HW_CARGO_POOL="$scratch/vol-a" HW_WORKTREES= sh "$tool") > "$scratch/out" 2> "$scratch/err"
status=$?
main=$(cd "$root" && cd "$(dirname "$(git rev-parse --path-format=absolute --git-common-dir)")" && pwd -P)
case "$(wt_line)" in
    "worktrees  $main/.claude/worktrees  "*|"worktrees  $main  "*) pass "the worktrees line is under the main checkout" ;;
    *) fail "the worktrees line is under the main checkout" "status $status, out: $(cat "$scratch/out"), err: $(cat "$scratch/err")" ;;
esac

echo "outside a git repository with no HW_WORKTREES it refuses, exit 2"
run HW_CARGO_POOL="$scratch/vol-a" HW_WORKTREES= GIT_CEILING_DIRECTORIES="$scratch" --
if [ "$status" -eq 2 ] && grep -q 'HW_WORKTREES' "$scratch/err"; then
    pass "exit 2, and the refusal names HW_WORKTREES"
else
    fail "exit 2, and the refusal names HW_WORKTREES" "status $status, err: $(cat "$scratch/err")"
fi

echo "--floor fails on the pool's volume, in three directions"
run HW_CARGO_POOL="$scratch/vol-a/cargo-pool" -- --floor 40
if [ "$status" -eq 1 ] && grep -q 'under the floor of 40G' "$scratch/err" && [ -n "$(pool_line)" ]; then
    pass "a pool's volume under the floor exits 1, and still prints its reading"
else
    fail "a pool's volume under the floor exits 1, and still prints its reading" "status $status, err: $(cat "$scratch/err")"
fi
run HW_CARGO_POOL="$scratch/vol-a" HW_WORKTREES="$scratch/vol-b/wt" -- --floor 40
if [ "$status" -eq 0 ]; then
    pass "a low worktrees volume beside a full pool's volume exits 0"
else
    fail "a low worktrees volume beside a full pool's volume exits 0" "status $status, err: $(cat "$scratch/err")"
fi
run HW_CARGO_POOL="$scratch/vol-b/real-pool" FAKE_B_K=41943040 -- --floor 40
if [ "$status" -eq 0 ] && pool_line | grep -q 'free=40G'; then
    pass "a pool's volume at exactly the floor exits 0"
else
    fail "a pool's volume at exactly the floor exits 0" "status $status, out: $(cat "$scratch/out")"
fi
run HW_CARGO_POOL="$scratch/vol-b/real-pool" FAKE_B_K=41943039 -- --floor 40
if [ "$status" -eq 1 ] && pool_line | grep -q 'free=39G'; then
    pass "one KiB under the floor rounds down to 39G and exits 1"
else
    fail "one KiB under the floor rounds down to 39G and exits 1" "status $status, out: $(cat "$scratch/out")"
fi

echo "a bad argument is refused, exit 2"
for args in "--floor" "--floor forty" "--bogus"; do
    # shellcheck disable=SC2086
    run HW_CARGO_POOL="$scratch/vol-a" -- $args
    if [ "$status" -eq 2 ]; then
        pass "\`$args\` exits 2"
    else
        fail "\`$args\` exits 2" "status $status"
    fi
done

echo "$passed passed; $failed failed"
[ "$failed" -eq 0 ]
