#!/bin/sh
# Run the clippy step of CI, on the toolchain CI uses, in a container.
#
#     sh tools/engine/clippy.sh
#
# The `Lint` step of `.github/workflows/ci.yml` runs
# `cargo clippy --all-targets --locked -- -D warnings` on the version that
# `tools/ci/rust-version` pins. A host clippy answers for the host's version, and
# one has reported zero warnings on a tree CI rejected with two errors
# (DEVELOPING.md). This script runs the same command on the same version, so
# its exit status is evidence about CI and a host `cargo clippy` is not. Seven of
# the 40 failed CI runs read on 2026-10-03 were this step; each cost a push and a
# wait on the pool this host also builds on.
#
# What it shares and what it costs. It compiles the whole workspace under the
# pinned toolchain, once; the target directory and the cargo home persist under
# `~/.cache/headwater/clippy/<version>/`, so a second run compiles only what the
# change touched. A lock allows one run at a time on the host, because three
# runner slots and the build pool already share the cores; a second caller waits
# for the first rather than adding to the load. It runs `nice`d. Run it once
# before the first push of a branch and again only after a change that could move
# a lint, never in a loop.
#
# The image is `rust:<version>-slim` plus `clippy` and `rustfmt`, built once and
# tagged `headwater-clippy:<version>`. The container runs as the caller, so the
# caches are owned by the caller, and the worktree is mounted read-only.
#
# This is not the floor recipe of `engine/README.md`. That one runs the lowest
# version the crates declare, so it carries no `-D warnings` and answers a
# different question. This one runs CI's version and carries it.
set -eu

here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../.." && pwd)
version=$(tr -d '[:space:]' < "$root/tools/ci/rust-version")
image="headwater-clippy:$version"
cache="${HEADWATER_CLIPPY_CACHE:-$HOME/.cache/headwater/clippy}/$version"

if ! command -v docker >/dev/null 2>&1; then
  echo "clippy.sh: no docker here, so clippy cannot run on CI's version. Push and read CI." >&2
  exit 2
fi

if ! docker image inspect "$image" >/dev/null 2>&1; then
  echo "clippy.sh: building $image once" >&2
  printf 'FROM rust:%s-slim\nRUN rustup component add clippy rustfmt\n' "$version" |
    docker build -q -t "$image" - >/dev/null
fi

mkdir -p "$cache/target" "$cache/cargo"

exec 9>"$cache/lock"
if ! flock -n 9; then
  echo "clippy.sh: another clippy run holds the lock, waiting" >&2
  flock 9
fi

nice -n 10 docker run --rm \
  --user "$(id -u):$(id -g)" \
  -v "$root":/w:ro -w /w/engine \
  -v "$cache/target":/target -v "$cache/cargo":/cargo \
  -e CARGO_HOME=/cargo -e CARGO_TARGET_DIR=/target -e HOME=/tmp \
  "$image" cargo clippy --all-targets --locked -- -D warnings
