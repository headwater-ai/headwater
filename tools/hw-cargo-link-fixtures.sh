#!/bin/sh
# What holds `tools/hw-cargo` to building each worktree's own code (#849).
#
# A slot of `tools/hw-cargo` is one target directory that every worktree on
# the host builds into. Cargo gives a workspace crate one metadata hash in
# every checkout and records its sources in dep-info relative to the crate,
# so a unit another worktree compiled more recently reads as fresh, and its
# code is linked into this tree's binary. The build exits 0. The script's
# answer is an owner file per slot: when a different worktree builds, every
# file under its `engine/crates` is touched, so each workspace crate rebuilds.
#
# `tools/hw-cargo-fixtures.sh` holds that the touch happens, with a fake
# cargo. It cannot hold that the touch is enough, because nothing there
# compiles. This suite compiles: it needs a real `cargo` on `PATH`, and it
# fails with one line when there is none rather than passing over nothing.
#
# Run it from anywhere:
#     sh tools/hw-cargo-link-fixtures.sh
#
# The shape, per case: a toy workspace with one library crate and a binary
# named `headwater`, committed under `mktemp -d`, and two `git worktree add`
# trees. Tree B changes the library's signature and its string literal, and
# its sources carry a 2020 modification time, as a file edited before tree A
# built would. A builds, then B builds into the same target directory, and
# the case runs B's binary. Exit 0 is not evidence; the printed literal is.
#
# The control case runs plain cargo with one shared `CARGO_TARGET_DIR` and
# asserts that B's binary prints A's literal. That is the defect. If a future
# cargo stops reproducing it, the control fails and says so, rather than the
# real case passing because nothing could go wrong.
#
# A rustc wrapper is turned off for every build here. `sccache` hashes file
# contents rather than reading cargo's freshness, so it neither causes nor
# hides this defect, and a server it started would outlive the suite.
#
# Nothing inside this checkout is written.

set -u

root=$(cd "$(dirname "$0")/.." && pwd)
tool="$root/tools/hw-cargo"

if [ ! -f "$tool" ]; then
    echo "no script at \`tools/hw-cargo\`." >&2
    exit 1
fi
if ! command -v cargo >/dev/null 2>&1; then
    echo "no \`cargo\` on PATH: this suite compiles a toy workspace and cannot run without a toolchain." >&2
    exit 1
fi
if ! command -v git >/dev/null 2>&1; then
    echo "no \`git\` on PATH: this suite builds two worktrees and cannot run without git." >&2
    exit 1
fi

scratch=$(mktemp -d) || exit 1
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

RUSTC_WRAPPER=
CARGO_BUILD_RUSTC_WRAPPER=
export RUSTC_WRAPPER CARGO_BUILD_RUSTC_WRAPPER
unset CARGO_TARGET_DIR HW_CARGO_SLOT

passed=0
failed=0

# setup DIR: a repository at DIR/repo with worktrees DIR/a and DIR/b, where
# b carries the changed signature and literal with 2020 mtimes.
setup() {
    base=$1
    mkdir -p "$base/repo/engine/crates/lib/src" "$base/repo/engine/crates/cli/src"
    cat >"$base/repo/engine/Cargo.toml" <<'EOF'
[workspace]
members = ["crates/lib", "crates/cli"]
resolver = "2"

[profile.dev-release]
inherits = "release"
EOF
    printf '[package]\nname = "plib"\nversion = "0.1.0"\nedition = "2021"\n' \
        >"$base/repo/engine/crates/lib/Cargo.toml"
    printf '[package]\nname = "headwater"\nversion = "0.1.0"\nedition = "2021"\n\n[dependencies]\nplib = { path = "../lib" }\n' \
        >"$base/repo/engine/crates/cli/Cargo.toml"
    printf 'pub fn greet() -> &%sstatic str { "OLD_LITERAL" }\n' "'" \
        >"$base/repo/engine/crates/lib/src/lib.rs"
    printf 'fn main() { println!("{}", plib::greet()); }\n' \
        >"$base/repo/engine/crates/cli/src/main.rs"
    git -C "$base/repo" init -q
    git -C "$base/repo" add -A
    git -C "$base/repo" -c user.email=fixture@example.invalid -c user.name=fixture commit -qm init
    git -C "$base/repo" worktree add -q "$base/a"
    git -C "$base/repo" worktree add -q "$base/b"
    printf 'pub fn greet(x: u32) -> String { format!("NEW_LITERAL {x}") }\n' \
        >"$base/b/engine/crates/lib/src/lib.rs"
    printf 'fn main() { println!("{}", plib::greet(7)); }\n' \
        >"$base/b/engine/crates/cli/src/main.rs"
    age_b "$base"
}

age_b() {
    find "$1/b/engine/crates" -type f -exec touch -t 202001010000 {} +
}

echo "control: plain cargo, one shared target directory, links a stale crate"
base="$scratch/control"
setup "$base"
target="$base/shared"
a_status=0
b_status=0
CARGO_TARGET_DIR="$target" cargo build -q --profile dev-release \
    --manifest-path "$base/a/engine/Cargo.toml" >"$base/a.out" 2>"$base/a.err" || a_status=$?
age_b "$base"
CARGO_TARGET_DIR="$target" cargo build -q --profile dev-release \
    --manifest-path "$base/b/engine/Cargo.toml" >"$base/b.out" 2>"$base/b.err" || b_status=$?
printed=$("$target/dev-release/headwater" 2>/dev/null || true)
if [ "$a_status" -eq 0 ] && [ "$b_status" -eq 0 ] && [ "$printed" = "OLD_LITERAL" ]; then
    passed=$((passed + 1))
    echo "  ok    b exits 0 and its binary prints a's literal"
else
    failed=$((failed + 1))
    echo "  FAIL  control did not reproduce the stale link (a exit $a_status, b exit $b_status, b printed '$printed')"
    echo "        this cargo may no longer share a workspace unit across checkouts; the case below proves nothing until this is read"
    sed 's/^/        /' "$base/b.err"
fi

echo "hw-cargo: a slot another worktree built in last rebuilds this tree's crates"
base="$scratch/hwcargo"
setup "$base"
pool="$base/pool"
a_status=0
b_status=0
HW_CARGO_POOL="$pool" HW_CARGO_SLOT=shared sh "$tool" build -q --profile dev-release \
    --manifest-path "$base/a/engine/Cargo.toml" >"$base/a.out" 2>"$base/a.err" || a_status=$?
age_b "$base"
HW_CARGO_POOL="$pool" HW_CARGO_SLOT=shared sh "$tool" build -q --profile dev-release \
    --manifest-path "$base/b/engine/Cargo.toml" >"$base/b.out" 2>"$base/b.err" || b_status=$?
printed=$("$base/b/engine/target/dev-release/headwater" 2>/dev/null || true)
if [ "$a_status" -eq 0 ] && [ "$b_status" -eq 0 ] && [ "$printed" = "NEW_LITERAL 7" ]; then
    passed=$((passed + 1))
    echo "  ok    b exits 0 and its binary prints b's literal"
else
    failed=$((failed + 1))
    echo "  FAIL  b linked code that is not its own (a exit $a_status, b exit $b_status, b printed '$printed')"
    sed 's/^/        /' "$base/b.err"
fi

echo "$passed passed; $failed failed"
[ "$failed" -eq 0 ]
