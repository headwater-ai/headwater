#!/bin/sh
# The tutorial says what a reader should now see, forty-odd times. This runs it.
#
# `docs/tutorials/your-first-governed-corpus.md` is the on-ramp, and its bar is
# that an agent can drive it end to end with no context beyond the page. Every
# output block in it is a claim about this engine, and a claim in prose goes
# stale the day the engine changes under it. `.claude/tutorial/drive.py` reads the
# commands out of the document, runs them against a scratch repository under the
# temporary directory, and diffs each result against the block the document
# prints.
#
# It blocks in CI. A tutorial that a newcomer cannot follow is worse than no
# tutorial, because the newcomer concludes the tool is broken rather than the
# page, and nothing else here reports the drift.
#
# It writes nothing inside this checkout: the scratch repository is a temporary
# directory, `HOME` is redirected into it, and the directory is removed at the
# end.
set -eu

root=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
bin=${HEADWATER_BIN:-$root/engine/target/release/headwater}

if [ ! -x "$bin" ]; then
  echo "tutorial fixtures: no engine at $bin" >&2
  echo "  build one with: cargo build --release -p headwater-cli --manifest-path engine/Cargo.toml" >&2
  exit 1
fi

HEADWATER_BIN=$bin exec python3 "$root/.claude/tutorial/drive.py" "$root"
