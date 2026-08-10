#!/usr/bin/env bash
# Run the whole Q1 risk-retirement spike and print every number the results
# document quotes. Any item can reopen Q1, so this script fails on the first
# one that fails.
#
#   ./build.sh
#
# Requires a Rust toolchain with the wasm32-unknown-unknown target, and node.

set -euo pipefail
cd "$(dirname "$0")"

hr() { printf '\n\033[1m== %s\033[0m\n' "$1"; }

hr "toolchain"
rustc --version
cargo --version
node --version

hr "item 2 — scope enforcement (compile-fail, runs first: it can falsify Q1)"
cargo test -p headwater-core --test compile_fail

hr "item 1 — spans reach rendered findings"
cargo test -p headwater-core --lib

hr "item 4 — change-scoped budget"
cargo run --release -q -p headwater-core --example bench

hr "item 3a — native Node addon, loaded in-process"
cargo build --release -q -p headwater-node
cp target/release/libheadwater_node.so target/release/headwater.node
node crates/node/smoke.js

hr "item 3b — wasm32, instantiated and called"
cargo build -q -p headwater-wasm --target wasm32-unknown-unknown --release
cargo build -q -p headwater-wasm --target wasm32-unknown-unknown --profile release-small
node crates/wasm/smoke.mjs release-small

hr "artifact sizes"
printf '%-42s %10s\n' "artifact" "bytes"
for f in \
  target/release/headwater.node \
  target/wasm32-unknown-unknown/release/headwater_wasm.wasm \
  target/wasm32-unknown-unknown/release-small/headwater_wasm.wasm
do
  printf '%-42s %10s\n' "$f" "$(stat -c%s "$f")"
done

printf '\n\033[1mall four items passed\033[0m\n'
