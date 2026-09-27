#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
#
# Reproduce every number of docs/evaluations/harper-spike-results.md.
#
#   sh tools/engine/harper-spike/build.sh <headwater-binary>
#
# Run it from the repository root. It needs docker and the rust:latest image
# (rustc 1.98.1 on 2026-09-27), because harper-core 2.11.0 does not compile on
# rustc 1.93.1; see in-container.sh. It writes the tables under results/ and
# prints the timings on standard error.
set -eu
hw=$1
spike=tools/engine/harper-spike
bin="$HOME/.cache/headwater/harper-spike/rust_latest/release/harper-spike"
out=${TMPDIR:-/tmp}/harper-spike
mkdir -p "$out"

# The decisive fixture and the seeded defects.
sh "$spike/in-container.sh" rust:latest test --release --locked -- --nocapture

# The sample, from the corpus as it stands.
"$hw" check --root . --format json > "$out/check.json"
python3 "$spike/sample.py" "$out/check.json" . > "$spike/sample.txt"

# Both modes. The authored mode runs twice and the two outputs must match.
"$bin" . "$spike/sample.txt" > "$spike/results/authored.tsv"
"$bin" . "$spike/sample.txt" > "$out/authored-2.tsv"
cmp "$spike/results/authored.tsv" "$out/authored-2.tsv"
"$bin" --raw . "$spike/sample.txt" > "$spike/results/raw.tsv"

# The adjudication draw. The verdict columns of results/verdicts.tsv are a
# person's reading of results/drawn.tsv and are not regenerated.
python3 "$spike/draw.py" . "$spike/results/authored.tsv" > "$spike/results/drawn.tsv"

# What harper-core adds.
sh "$spike/in-container.sh" rust:latest metadata --format-version 1 --locked > "$out/metadata.json"
python3 "$spike/footprint.py" "$out/metadata.json"
ls -l "$bin"
