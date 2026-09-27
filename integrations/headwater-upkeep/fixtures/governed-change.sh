#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
#
# The decisive fixture of `integrations/headwater-upkeep` (#502): a change
# edits a code file that a document governs, and the report must name that
# document. That is the whole failure the action exists to prevent, a change
# made on any harness that stales a document and meets no signal.
#
# It builds the corpus of `headwater-check/fixtures/build-decisive-fixture.sh`
# with the engine it is given. The decision ACME-DR-0001 governs `src/a.rs`,
# and a second decision ACME-DR-0002 `traces_to` it. The corpus also holds
# `src/b.rs`, which nothing governs. That is committed as the base, and three
# changes are run over it, each from a clean base:
#
#   edit    edit `src/a.rs` and `src/b.rs`. Touched names ACME-DR-0001
#           against `src/a.rs`, and Unmeasured names `src/b.rs`.
#   rename  rename `src/a.rs` to `src/c.rs`. Touched names the old path and
#           the document that governed it, and Owed lists the finding the
#           broken edge raises on that document, marked new.
#   hop     delete ACME-DR-0001. Owed lists the new finding on ACME-DR-0002,
#           one relation away, which no path of the change names.
#
# For every case the run must write nothing inside the checkout: HEAD and
# `git status --porcelain` do not move, and no file under the corpus is newer
# than a marker set before the run. The last check is the one that sees a
# write to `.headwater/cache/`, which the corpus gitignores, so `git status`
# cannot.
#
# Usage: FIXTURE_TAXONOMY_URL=... FIXTURE_TAXONOMY_DIGEST=... \
#        sh governed-change.sh <engine> <dir>
# The engine is exported as HEADWATER_BIN for the corpus builder. <dir> is
# removed and rebuilt; the corpus is <dir>/corpus and each case writes
# <dir>/<case>.md.
set -eu

engine=${1:?usage: governed-change.sh <engine> <dir>}
dir=${2:?usage: governed-change.sh <engine> <dir>}
here=$(cd "$(dirname "$0")" && pwd)
HEADWATER_BIN=$engine
export HEADWATER_BIN

rm -rf "$dir"
mkdir -p "$dir"
corpus="$dir/corpus"
sh "$here/../../headwater-check/fixtures/build-decisive-fixture.sh" "$corpus" >"$dir/build.log" 2>&1 || {
    cat "$dir/build.log" >&2
    exit 1
}

"$engine" new decision --root "$corpus" --title "Retry a delivery three times" >>"$dir/build.log" 2>&1
first=$(find "$corpus/docs/decisions" -name '0001-*.md')
second=$(find "$corpus/docs/decisions" -name '0002-*.md')
python3 - "$first" "$second" <<'PY'
import sys
first, second = sys.argv[1], sys.argv[2]
text = open(first, encoding="utf-8").read()
old = "last_verified: 2026-09-22\n"
if old not in text:
    sys.exit(f"{first}: no `{old.strip()}` line to add the governs edge after")
text = text.replace(old, old + "relations:\n  governs:\n    - src/a.rs\n", 1)
open(first, "w", encoding="utf-8").write(text)
open(second, "w", encoding="utf-8").write("""---
id: ACME-DR-0002
status: current
status_since: 2026-09-22
summary: "A failed delivery is tried three times, and each attempt is a row in the same store as the rest of the service state."
last_verified: 2026-09-22
relations:
  traces_to:
    - ACME-DR-0001
---

# Retry a delivery three times

## Context

A delivery can fail for a reason that passes.

## Decision

A failed delivery is tried three times.

## Consequences

Three attempts are three rows.
""")
PY
mkdir -p "$corpus/src"
printf 'pub fn a() {}\n' >"$corpus/src/a.rs"
printf 'pub fn b() {}\n' >"$corpus/src/b.rs"
"$engine" generate --root "$corpus" >/dev/null 2>&1
"$engine" check --root "$corpus" --strict >"$dir/base-check.log" 2>&1 || {
    echo "$0: the base corpus does not pass check --strict; see $dir/base-check.log" >&2
    exit 1
}

git -C "$corpus" init -q
git -C "$corpus" add -A
git -C "$corpus" -c user.name=fixture -c user.email=fixture@example.invalid commit -q -m base
base=$(git -C "$corpus" rev-parse HEAD)

failed=0

# run_case <name>: run upkeep.sh over the tree as it stands, hold the
# no-write assertions, and leave the report at <dir>/<name>.md.
run_case() {
    status_before=$(git -C "$corpus" status --porcelain)
    head_before=$(git -C "$corpus" rev-parse HEAD)
    touch "$dir/marker"
    sh "$here/../upkeep.sh" "$corpus" "$base" "$dir/work-$1" "$dir/$1.md"
    echo "--- $1 ---"
    cat "$dir/$1.md"
    if [ "$status_before" != "$(git -C "$corpus" status --porcelain)" ]; then
        echo "FAIL $1: the run changed git status in the checkout" >&2
        failed=1
    fi
    if [ "$head_before" != "$(git -C "$corpus" rev-parse HEAD)" ]; then
        echo "FAIL $1: the run moved HEAD" >&2
        failed=1
    fi
    written=$(find "$corpus" -path "$corpus/.git" -prune -o -type f -newer "$dir/marker" -print)
    if [ -n "$written" ]; then
        echo "FAIL $1: the run wrote inside the checkout: $written" >&2
        failed=1
    fi
}

# reset_to_base: put the corpus back at the base commit, clean.
reset_to_base() {
    git -C "$corpus" reset -q --hard "$base"
    git -C "$corpus" clean -q -fd
}

# assert <case> <python condition over T, S, O, U: the four parts>
assert() {
    python3 - "$dir/$1.md" "$2" "$3" <<'PY' || failed=1
import sys
path, cond, why = sys.argv[1], sys.argv[2], sys.argv[3]
text = open(path, encoding="utf-8").read()
def part(name):
    head = f"## {name}\n"
    if head not in text:
        return None
    return text.split(head, 1)[1].split("\n## ", 1)[0]
def entries(block):
    out, cur = [], None
    for line in (block or "").splitlines():
        if line.startswith("- "):
            cur = [line]
            out.append(cur)
        elif line.startswith("  ") and cur is not None:
            cur.append(line)
    return ["\n".join(e) for e in out]
T, S, O, U = part("Touched"), part("Stale"), part("Owed"), part("Unmeasured")
if not eval(cond):
    print(f"FAIL {path}: {why}", file=sys.stderr)
    sys.exit(1)
PY
}

d1=$(basename "$first")
d2=$(basename "$second")

# The edit case, the one the issue exists for.
printf 'pub fn a() -> u8 { 1 }\n' >"$corpus/src/a.rs"
printf 'pub fn b() -> u8 { 2 }\n' >"$corpus/src/b.rs"
run_case edit
assert edit "any(e.startswith('- \`src/a.rs\`') and '$d1' in e for e in entries(T))" \
    "Touched does not name $d1 against src/a.rs"
assert edit "'src/b.rs' not in T" "Touched names src/b.rs, which no edge governs"
assert edit "U is not None and U.strip() != ''" "Unmeasured is missing or empty, and it must never be"
assert edit "'src/b.rs' in U" "Unmeasured does not name src/b.rs, which no edge governs"

# The rename case: the edge that named the old path now names nothing.
reset_to_base
git -C "$corpus" mv src/a.rs src/c.rs
git -C "$corpus" -c user.name=fixture -c user.email=fixture@example.invalid commit -q -m rename
run_case rename
assert rename "any(e.startswith('- \`src/a.rs\`: renamed to \`src/c.rs\`') and '$d1' in e for e in entries(T))" \
    "Touched does not name the old path src/a.rs, its new name and $d1, which governed it"
assert rename "any('$d1' in line and '(new with this change)' in line for line in (O or '').splitlines())" \
    "Owed does not list the new finding the broken edge raises on $d1"

# The hop case: a finding one relation away from any changed path.
reset_to_base
git -C "$corpus" rm -q "docs/decisions/$d1"
git -C "$corpus" -c user.name=fixture -c user.email=fixture@example.invalid commit -q -m delete
run_case hop
assert hop "any('$d2' in line and '(new with this change)' in line for line in (O or '').splitlines())" \
    "Owed does not list the new finding on $d2, which traces_to the deleted $d1"

# Leave the tree at the edit case, so a later step can run the packaged
# action over the base commit and this same change.
reset_to_base
printf 'pub fn a() -> u8 { 1 }\n' >"$corpus/src/a.rs"
printf 'pub fn b() -> u8 { 2 }\n' >"$corpus/src/b.rs"
if [ "$failed" -ne 0 ]; then
    exit 1
fi
echo "$0: edit, rename and hop each name what they must, and no run wrote in the checkout" >&2
