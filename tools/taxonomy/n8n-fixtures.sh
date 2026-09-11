#!/bin/sh
# The three n8n fixture READMEs print a recipe and then state what it reports. This runs it.
#
# `docs/taxonomies/design-spec/fixtures/n8n/`, `.../standards-spec/fixtures/n8n/`
# and `.../diataxis-site/fixtures/n8n/` each type a real corpus of the n8n
# monorepo at one pin. Each carries a *How to run this corpus* block and a
# *What a run reports* paragraph, and every number in the second is a claim
# about this engine copied into prose. Prose does not re-derive.
#
# It went stale unseen. All three recipes refused at `headwater taxonomy
# resolve` for four minor versions of the vendored package, because a fixture
# pinned `headwater/standard` at a version `packages/` no longer carried, and
# `docs/taxonomies/**` is outside this repository's own corpus so no check read
# either scalar. The published pages named commands that could not run.
#
# `tools/taxonomy/drive_n8n.py` reads the commands out of each README, runs them
# verbatim against a scratch root, and diffs each figure against the block the
# README prints. It carries no second copy of any number.
#
# It blocks in CI, and it holds three arms a green run cannot show you: the
# count of corpora it found, an edited figure that must fail it naming the file
# and both values, and a version pin the vendored package does not carry.
#
# It writes nothing inside this checkout: every scratch root is under a
# temporary directory, and the directory is removed at the end.
set -eu

root=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
bin=${HEADWATER_BIN:-$root/engine/target/release/headwater}

if [ ! -x "$bin" ]; then
  echo "n8n fixtures: no engine at $bin" >&2
  echo "  build one with: cargo build --release -p headwater-cli --manifest-path engine/Cargo.toml --locked" >&2
  exit 1
fi

HEADWATER_BIN=$bin exec python3 "$root/tools/taxonomy/drive_n8n.py" "$root"
