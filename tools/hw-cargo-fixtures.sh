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
# below hold where the tool copies that file, not what building one produces.
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

echo "$passed passed; $failed failed"
[ "$failed" -eq 0 ]
