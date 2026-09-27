#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
#
# Download the release archive `$TAG` names, verify its checksum, unpack the
# `headwater` binary into `$STAGE`, and put `$STAGE` on `$GITHUB_PATH`.
#
# `integrations/headwater-check` and `integrations/headwater-upkeep` both run
# this one script, after `resolve-version.sh`, so the two actions install the
# engine the same way and a fix to one reaches both.
#
# Usage: TAG=<v-tag> STAGE=<dir> sh install.sh
set -eu

tag=${TAG:?set TAG to a release tag, as resolve-version.sh writes it}
stage=${STAGE:?set STAGE to the directory to unpack the engine into}

asset="headwater-${tag}-x86_64-unknown-linux-gnu.tar.gz"
base="https://github.com/headwater-ai/headwater/releases/download/${tag}"
mkdir -p "$stage"
curl -fsSL -o "$stage/$asset" "$base/$asset"
curl -fsSL -o "$stage/$asset.sha256" "$base/$asset.sha256"
( cd "$stage" && sha256sum -c "$asset.sha256" )
tar -xzf "$stage/$asset" -C "$stage" headwater
chmod +x "$stage/headwater"
echo "$stage" >>"$GITHUB_PATH"
"$stage/headwater" --version
