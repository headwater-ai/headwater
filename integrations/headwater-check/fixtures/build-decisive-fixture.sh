#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
#
# Build a tiny, throwaway corpus at $1, using the SAME pinned engine release
# (`FIXTURE_TAG`, resolved by the caller and passed as `HEADWATER_BIN`) that
# `.github/workflows/integrations-headwater-check.yml` runs the composite
# action against. This corpus is NOT committed to this repository: the corpus
# this repository governs itself moves faster than any one release does, so a
# lock this repository's own current engine writes is routinely rule-set-newer
# than the newest tagged release — confirmed while building #833, where
# `v0.1.2` refused this repository's own committed lock outright ("the lock
# declares rule set 2 and this engine validates against 1"). A fixture corpus
# built with today's dev engine and checked against yesterday's release would
# carry exactly that defect, for a reason that has nothing to do with what
# this test exists to prove. So this script builds the corpus with the same
# binary that checks it, every run, and neither one is ever stale relative to
# the other.
#
# Usage: HEADWATER_BIN=<path> sh build-decisive-fixture.sh <dest-dir>
set -eu

dest=${1:?usage: build-decisive-fixture.sh <dest-dir>}
bin=${HEADWATER_BIN:?set HEADWATER_BIN to the pinned engine binary}
# The whole location of the published zip, and never a tag that this script
# turns into one: an asset name built by concatenation is what
# `release-taxonomy.yml` says not to do, because a release names its asset.
url=${FIXTURE_TAXONOMY_URL:?set FIXTURE_TAXONOMY_URL to the https:// location of a published taxonomy zip}
digest=${FIXTURE_TAXONOMY_DIGEST:?set FIXTURE_TAXONOMY_DIGEST, e.g. sha256:...}

rm -rf "$dest"
mkdir -p "$dest/docs/decisions"
printf '# Store attempts in Postgres\n\nThe queue keeps every delivery attempt in Postgres.\n' \
    >"$dest/docs/decisions/postgres-note.md"

"$bin" init --root "$dest"

# The engine fetches the zip itself: `vendor` takes a location from v0.2.1.
"$bin" taxonomy vendor "$url" --expect "$digest" --root "$dest"

# `init` proposes `taxonomy.version: 0.0.0`, because no package sat under
# `.headwater/packages/` yet when it ran. Pin it at the version the vendored
# package itself declares, read back rather than assumed, so a change to
# FIXTURE_TAXONOMY_URL never needs a second edit here.
version=$(sed -n 's/^version: *//p' "$dest/.headwater/packages/headwater-standard/package.yml" | head -1)
if [ -z "$version" ]; then
    echo "$0: .headwater/packages/headwater-standard/package.yml states no version" >&2
    exit 1
fi
sed -i "s/version: 0.0.0/version: $version/" "$dest/.headwater/taxonomy.yml"

# `init` writes a trailing `add: {}`; replace it rather than appending a
# second `add:` key, which the YAML loader refuses as a duplicate.
python3 - "$dest/.headwater/overlay.yml" <<'PY'
import sys
path = sys.argv[1]
text = open(path, encoding="utf-8").read()
text = text.replace(
    "add: {}",
    "add:\n  identifier_schemes.decision_id.namespace: ACME\n",
)
open(path, "w", encoding="utf-8").write(text)
PY

"$bin" taxonomy resolve --root "$dest"

"$bin" new decision --root "$dest" --title "Store attempts in Postgres, not a queue"

decision_file=$(find "$dest/docs/decisions" -name '0001-*.md')
cat >"$decision_file" <<'DOC'
---
id: ACME-DR-0001
status: current
status_since: 2026-09-22
summary: "This corpus keeps every delivery attempt in the same Postgres database as the rest of the service state, and never in a separate queue."
last_verified: 2026-09-22
---

# Store attempts in Postgres, not a queue

## Context

The queue keeps every delivery attempt in Postgres.

## Decision

A delivery attempt is a row in the same database as the rest of the service.

## Consequences

One store to back up, and one store to query.
DOC

"$bin" generate --root "$dest"

"$bin" check --root "$dest" --strict
"$bin" generate --root "$dest" --check
# The action runs this third verb since #1118. `new` above runs after the
# resolve and does not move the lock (measured with v0.2.1), so no second
# resolve is needed before this line.
"$bin" taxonomy resolve --root "$dest" --check

echo "$0: built a clean, passing fixture corpus at $dest" >&2
