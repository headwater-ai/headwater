#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
#
# Run one cargo command for the spike inside a pinned Rust image.
#
# harper-core 2.11.0 declares no rust-version and does not compile on rustc
# 1.93.1 (four E0308 errors in its own linting/ modules, measured 2026-09-27).
# The engine's floor is 1.91. So the spike builds in a container whose rustc
# is newer, and the evaluation reports the lowest image that built it.
#
#   sh tools/engine/harper-spike/in-container.sh <image> <cargo args...>
#
# The repository root is mounted read-write at /src, because the spike takes
# headwater-doc by a path dependency. The host's cargo registry is reused, and
# the target directory is per image under ~/.cache/headwater/harper-spike/.
set -eu
image=$1
shift
here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../../.." && pwd)
tag=$(printf '%s' "$image" | tr ':/' '__')
target="$HOME/.cache/headwater/harper-spike/$tag"
mkdir -p "$target" "$HOME/.cargo/registry"
exec docker run --rm --user "$(id -u):$(id -g)" \
  -e CARGO_HOME=/cargo -e CARGO_TARGET_DIR=/target -e HOME=/tmp \
  -v "$HOME/.cargo/registry:/cargo/registry" \
  -v "$target:/target" \
  -v "$root:/src" -w /src/tools/engine/harper-spike \
  "$image" cargo "$@"
