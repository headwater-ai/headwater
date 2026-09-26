#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
#
# Leave the fixture corpus at $1 with a lock that its sources no longer
# resolve to, and with nothing else wrong. This is the tree a merge leaves
# when somebody resolves a conflict on `.headwater/taxonomy.lock` by taking
# one side: the overlay carries the other side's edit, and the lock does not.
# #1118 is the case. `headwater check` and `headwater generate --check` read
# the lock and never the overlay, so both pass this tree, and only
# `headwater taxonomy resolve --check` sees the defect.
#
# The overlay edit is neutral on purpose. It adds one phrase to a purpose's
# `answers`, which routes a task and changes nothing a document is checked
# against, so a later `taxonomy resolve` gives a tree that passes again.
# Changing `identifier_schemes.decision_id.namespace` would not be neutral,
# because ACME-DR-0001 would then carry the wrong prefix.
#
# The script refuses to finish unless the tree is what it says: the two verbs
# the action ran before #1118 both exit 0, and `resolve --check` exits
# non-zero. A fixture that stales something else as well proves nothing
# about the lock.
#
# Usage: HEADWATER_BIN=<path> sh stale-lock-one-sided.sh <fixture-dir>
set -eu

dest=${1:?usage: stale-lock-one-sided.sh <fixture-dir>}
bin=${HEADWATER_BIN:?set HEADWATER_BIN to the pinned engine binary}
lock="$dest/.headwater/taxonomy.lock"
overlay="$dest/.headwater/overlay.yml"

cp "$lock" "$lock.ours"

python3 - "$overlay" <<'PY'
import sys
path = sys.argv[1]
text = open(path, encoding="utf-8").read()
marker = "\nadd:\n"
if marker not in text:
    sys.exit(f"{path}: no top-level `add:` block to extend")
text = text.replace(
    marker,
    '\noverride:\n  purposes.rationale.answers: ["why is it this way", "what was rejected", "what does this constrain", "why not a queue"]\nadd:\n',
    1,
)
open(path, "w", encoding="utf-8").write(text)
PY

# The other side's lock: what the edited overlay resolves to.
"$bin" taxonomy resolve --root "$dest"
if cmp -s "$lock" "$lock.ours"; then
    echo "$0: the overlay edit did not move the lock, so this fixture stales nothing" >&2
    exit 1
fi

# Take one side of the conflict: the saved lock goes back over the new one.
mv "$lock.ours" "$lock"

# The lock is the only defect.
"$bin" check --root "$dest" --strict
"$bin" generate --root "$dest" --check
if "$bin" taxonomy resolve --root "$dest" --check; then
    echo "$0: taxonomy resolve --check passed a lock resolved from a different overlay" >&2
    exit 1
fi

echo "$0: $dest passes check --strict and generate --check, and its lock is stale" >&2
