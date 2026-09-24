#!/bin/sh
# What holds `tools/hw-cargo`'s copy-back root.
#
# The script picks one worktree to copy the built binary into. #856 is what
# happens when it picks by asking `git rev-parse --show-toplevel` about the
# caller's ambient cwd: a session pinned to, or drifted into, a worktree other
# than the one its own `--manifest-path` names gets the binary copied into the
# wrong one, silently overwriting whatever sat there. The fix reads the root
# from `--manifest-path` itself, the same project cargo already builds
# regardless of cwd, and falls back to cwd only when no manifest path is
# given.
#
# Run it from anywhere:
#     sh tools/hw-cargo-fixtures.sh
#
# It needs no toolchain: `cargo` on `PATH` is a fake for the length of this
# suite, a shell script that plants an empty executable at
# `$CARGO_TARGET_DIR/<profile>/headwater` and does nothing else. The cases
# below hold where the tool copies that file, which slot and lock it takes,
# and that a process the build leaves behind does not keep the lock. What
# building produces is `tools/hw-cargo-link-fixtures.sh`'s, which compiles.
# Every worktree is a real `git init` under `mktemp -d`, and nothing inside
# this checkout is written.

set -u

root=$(cd "$(dirname "$0")/.." && pwd)
tool="$root/tools/hw-cargo"

if [ ! -f "$tool" ]; then
    echo "no script at \`tools/hw-cargo\`." >&2
    exit 1
fi

scratch=$(mktemp -d) || exit 1
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

fakebin="$scratch/fakebin"
mkdir -p "$fakebin"
cat >"$fakebin/cargo" <<'EOF'
#!/bin/sh
profile=debug
prev=
for arg in "$@"; do
    case "$prev" in --profile) profile=$arg ;; esac
    case "$arg" in --release) profile=release ;; esac
    prev=$arg
done
case "$profile" in dev | test) profile=debug ;; esac
mkdir -p "$CARGO_TARGET_DIR/$profile"
: >"$CARGO_TARGET_DIR/$profile/headwater"
chmod +x "$CARGO_TARGET_DIR/$profile/headwater"
EOF
chmod +x "$fakebin/cargo"

pool="$scratch/pool"

worktree_a="$scratch/a"
worktree_b="$scratch/b"
mkdir -p "$worktree_a/engine" "$worktree_b/engine"
git init -q "$worktree_a"
git init -q "$worktree_b"
: >"$worktree_a/engine/Cargo.toml"
: >"$worktree_b/engine/Cargo.toml"

passed=0
failed=0

# report NAME PATH-THAT-SHOULD-EXIST PATH-THAT-SHOULD-NOT
report() {
    name=$1
    want=$2
    unwant=$3
    if [ -x "$want" ] && [ ! -e "$unwant" ]; then
        passed=$((passed + 1))
        echo "  ok    $name"
    else
        failed=$((failed + 1))
        echo "  FAIL  $name"
    fi
}

echo "no --manifest-path: falls back to the ambient cwd"
rm -rf "$worktree_a/engine/target" "$worktree_b/engine/target"
(
    cd "$worktree_b" || exit 1
    PATH="$fakebin:$PATH" HW_CARGO_POOL="$pool" sh "$tool" build --profile dev-release -p headwater-cli --locked
) >"$scratch/out" 2>"$scratch/err"
report "cwd b, no manifest path, lands in b" \
    "$worktree_b/engine/target/dev-release/headwater" \
    "$worktree_a/engine/target/dev-release/headwater"

echo "--manifest-path names worktree a while cwd is worktree b"
rm -rf "$worktree_a/engine/target" "$worktree_b/engine/target"
(
    cd "$worktree_b" || exit 1
    PATH="$fakebin:$PATH" HW_CARGO_POOL="$pool" sh "$tool" build --profile dev-release -p headwater-cli \
        --manifest-path "$worktree_a/engine/Cargo.toml" --locked
) >"$scratch/out" 2>"$scratch/err"
report "cwd b, manifest path a, lands only in a" \
    "$worktree_a/engine/target/dev-release/headwater" \
    "$worktree_b/engine/target/dev-release/headwater"

echo "HW_CARGO_SLOT reserves a target dir outside the numbered pool"
rm -rf "$worktree_b/engine/target" "$pool"
(
    cd "$worktree_b" || exit 1
    PATH="$fakebin:$PATH" HW_CARGO_POOL="$pool" HW_CARGO_SLOT=integrate sh "$tool" build --profile dev-release -p headwater-cli --locked
) >"$scratch/out" 2>"$scratch/err"
report "a reserved slot writes its own named target dir" \
    "$pool/target-integrate/dev-release/headwater" \
    "$pool/target-1/dev-release/headwater"

echo "a reserved slot never takes a numbered slot's lock"
if [ -e "$pool/slot.1.lock" ]; then
    failed=$((failed + 1))
    echo "  FAIL  a reserved-slot build left slot.1.lock behind"
else
    passed=$((passed + 1))
    echo "  ok    no numbered slot lock was touched"
fi

echo "a slot another worktree built in last makes this tree's workspace crates rebuild"
# Cargo gives a workspace crate one metadata hash in every checkout, so a
# unit a second worktree compiled more recently reads as fresh in the first.
# The tool marks the building tree's crate sources newer than anything in a
# slot it did not build in last; here the fake cargo builds nothing, so the
# source's modification time is what is held.
rm -rf "$pool"
mkdir -p "$worktree_a/engine/crates/verbs/src" "$worktree_b/engine/crates/verbs/src"
: >"$worktree_a/engine/crates/verbs/src/lib.rs"
: >"$worktree_b/engine/crates/verbs/src/lib.rs"
build_in() {
    PATH="$fakebin:$PATH" HW_CARGO_POOL="$pool" HW_CARGO_SLOT=shared sh "$tool" build --profile dev-release -p headwater-cli \
        --manifest-path "$1/engine/Cargo.toml" --locked >"$scratch/out" 2>"$scratch/err"
}
is_old() { [ "$(find "$1" -newer "$scratch/stamp" | wc -l)" -eq 0 ]; }
touch -t 202001010000 "$scratch/stamp"
build_in "$worktree_a"
touch -t 202001010000 "$worktree_b/engine/crates/verbs/src/lib.rs"
build_in "$worktree_b"
if ! is_old "$worktree_b/engine/crates/verbs/src/lib.rs" && grep -q 'every workspace crate rebuilds from this tree' "$scratch/err"; then
    passed=$((passed + 1))
    echo "  ok    b, after a, has its crate sources marked newer, and says so"
else
    failed=$((failed + 1))
    echo "  FAIL  b, after a, was not made to rebuild its workspace crates"
fi
touch -t 202001010000 "$worktree_b/engine/crates/verbs/src/lib.rs"
build_in "$worktree_b"
if is_old "$worktree_b/engine/crates/verbs/src/lib.rs"; then
    passed=$((passed + 1))
    echo "  ok    b, building in the slot it built in last, is left incremental"
else
    failed=$((failed + 1))
    echo "  FAIL  b rebuilding in its own slot had its sources touched again"
fi

echo "a daemon the build starts does not keep the slot"
# A rustc wrapper such as sccache starts a server from inside the build, and
# the server inherits every open descriptor and outlives the build. The slot's
# lock is descriptor 9 of the tool's shell, so the tool runs cargo with that
# descriptor closed. The fake cargo here stands in for the wrapper: it starts
# a background `sleep` that inherits whatever cargo was handed, and exits.
if command -v flock >/dev/null 2>&1; then
    daemonbin="$scratch/daemonbin"
    mkdir -p "$daemonbin"
    cat >"$daemonbin/cargo" <<EOF
#!/bin/sh
sleep 30 &
echo \$! >"$scratch/daemon.pid"
EOF
    chmod +x "$daemonbin/cargo"
    rm -rf "$pool" "$scratch/daemon.pid"
    (
        cd "$worktree_b" || exit 1
        PATH="$daemonbin:$PATH" HW_CARGO_POOL="$pool" HW_CARGO_SLOTS=1 sh "$tool" build --profile dev-release -p headwater-cli --locked
    ) >"$scratch/out" 2>"$scratch/err"
    # A free lock proves nothing unless the tool took it and the daemon it
    # would have leaked to is still alive when the lock is probed.
    daemon=$(cat "$scratch/daemon.pid" 2>/dev/null || true)
    if [ ! -e "$pool/slot.1.lock" ]; then
        failed=$((failed + 1))
        echo "  FAIL  the tool never opened slot 1's lock, so the probe below would prove nothing"
    elif [ -z "$daemon" ] || ! kill -0 "$daemon" 2>/dev/null; then
        failed=$((failed + 1))
        echo "  FAIL  the fake cargo's daemon is not running, so the tool never ran cargo or the daemon died"
    elif (flock -n 9) 9>>"$pool/slot.1.lock"; then
        passed=$((passed + 1))
        echo "  ok    slot 1 is free once the tool returns, with its daemon still running"
    else
        failed=$((failed + 1))
        echo "  FAIL  a process the build started still holds slot 1's lock"
    fi
    [ -n "$daemon" ] && kill "$daemon" 2>/dev/null
else
    failed=$((failed + 1))
    echo "  FAIL  no flock on this host, so the lock this case holds is never taken; unrun"
fi

echo "$passed passed; $failed failed"
[ "$failed" -eq 0 ]
