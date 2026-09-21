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
# The engine this suite drives, resolved the way `tools/repo/resolve-engine.sh`
# resolves it: either profile counts and the newer answers, `HEADWATER_BIN`
# still overrides both. This suite read the `release` path alone until #647,
# so a worktree built the way the build order tells a session to build it —
# `--profile dev-release` and not `--release` — had no `release` binary and
# this suite reported "no engine" against a real one sitting beside it.
set -eu

root=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
. "$root/tools/repo/resolve-engine.sh"
bin=${HEADWATER_BIN:-}
if [ -z "$bin" ]; then
  bin=$(hw_resolve_engine_bin "$root") || bin=
fi

if [ -z "$bin" ] || [ ! -x "$bin" ]; then
  echo "n8n fixtures:" >&2
  hw_resolve_engine_missing_message "$root" >&2
  exit 1
fi

HEADWATER_BIN=$bin exec python3 "$root/tools/taxonomy/drive_n8n.py" "$root"
