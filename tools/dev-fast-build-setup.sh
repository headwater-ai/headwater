#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
#
# Optional, machine-local speedup for engine builds: wires mold (linker) and
# sccache (compile cache) into ~/.cargo/config.toml. Never required by this
# repo or CI, and this script never touches engine/.cargo/config.toml (the
# repo's own, tracked, alias-only config) — see
# .claude/skills/headwater-engine/SKILL.md, "Optional: a faster linker and
# compile cache", for what each tool actually buys you and the one gotcha
# (sccache does not cache an incremental build, so it does ~nothing for
# `cargo check`/`cargo test`; mold helps every link regardless).
#
# Idempotent: safe to re-run. Only appends a marked block to
# ~/.cargo/config.toml and never edits anything outside it.
#
#   tools/dev-fast-build-setup.sh          # install the block
#   tools/dev-fast-build-setup.sh --remove # remove exactly that block

set -euo pipefail

CARGO_CONFIG="${CARGO_HOME:-$HOME/.cargo}/config.toml"
MARKER_BEGIN="# BEGIN headwater dev-fast-build-setup"
MARKER_END="# END headwater dev-fast-build-setup"

if [ "${1:-}" = "--remove" ]; then
  if [ ! -f "$CARGO_CONFIG" ] || ! grep -qF "$MARKER_BEGIN" "$CARGO_CONFIG"; then
    echo "No headwater dev-fast-build-setup block found in $CARGO_CONFIG." >&2
    exit 0
  fi
  sed -i "/$MARKER_BEGIN/,/$MARKER_END/d" "$CARGO_CONFIG"
  echo "Removed the block from $CARGO_CONFIG." >&2
  exit 0
fi

missing=()
command -v mold >/dev/null 2>&1 || missing+=(mold)
command -v sccache >/dev/null 2>&1 || missing+=(sccache)

if [ "${#missing[@]}" -gt 0 ]; then
  echo "Missing: ${missing[*]}" >&2
  echo "Install first, e.g.: sudo apt install -y ${missing[*]}" >&2
  echo "(or your distro's equivalent), then re-run this script." >&2
  exit 1
fi

mkdir -p "$(dirname "$CARGO_CONFIG")"
touch "$CARGO_CONFIG"

if grep -qF "$MARKER_BEGIN" "$CARGO_CONFIG"; then
  echo "Already configured in $CARGO_CONFIG — nothing to do." >&2
  exit 0
fi

host_triple="$(rustc -vV | sed -n 's/^host: //p')"

cat >> "$CARGO_CONFIG" <<EOF

$MARKER_BEGIN
# mold: faster linker, used for every link.
# sccache: compile cache — helps non-incremental builds only (--release,
# --profile dev-release, a clean or cross-branch rebuild); cargo check/test
# use incremental compilation by default and see ~0 benefit from it.
# Written by tools/dev-fast-build-setup.sh — remove with --remove.
[target.$host_triple]
rustflags = ["-C", "link-arg=-fuse-ld=mold"]

[build]
rustc-wrapper = "sccache"
$MARKER_END
EOF

echo "Wired mold + sccache into $CARGO_CONFIG for target $host_triple." >&2
echo "Verify with: cargo test -p headwater-hash -v 2>&1 | grep fuse-ld" >&2
