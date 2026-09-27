#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
#
# The decisive fixture of `integrations/headwater-upkeep` (#502): a change
# edits a code file that a document governs, and the report must name that
# document. That is the whole failure the action exists to prevent, a change
# made on any harness that stales a document and meets no signal.
#
# It builds the corpus of `headwater-check/fixtures/build-decisive-fixture.sh`
# with the engine it is given, adds `governs: src/a.rs` to the decision,
# creates `src/a.rs` and `src/b.rs`, commits that as the base, edits both
# files, and runs `upkeep.sh` over the change. Then it asserts:
#
#   1. Touched names the decision against `src/a.rs`.
#   2. Unmeasured is present, is not empty, and names `src/b.rs`, which no
#      edge governs.
#   3. The run wrote nothing inside the checkout: `git status --porcelain`
#      and `HEAD` are what they were before it.
#
# Usage: HEADWATER_BIN=<engine> FIXTURE_TAXONOMY_URL=... FIXTURE_TAXONOMY_DIGEST=... \
#        sh governed-change.sh <engine> <dir>
# The engine is the first argument and is exported as HEADWATER_BIN for the
# corpus builder. <dir> is removed and rebuilt; the corpus is <dir>/corpus and
# the report is <dir>/report.md.
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

decision=$(find "$corpus/docs/decisions" -name '0001-*.md')
python3 - "$decision" <<'PY'
import sys
path = sys.argv[1]
text = open(path, encoding="utf-8").read()
old = "last_verified: 2026-09-22\n"
if old not in text:
    sys.exit(f"{path}: no `{old.strip()}` line to add the governs edge after")
text = text.replace(old, old + "relations:\n  governs:\n    - src/a.rs\n", 1)
open(path, "w", encoding="utf-8").write(text)
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

printf 'pub fn a() -> u8 { 1 }\n' >"$corpus/src/a.rs"
printf 'pub fn b() -> u8 { 2 }\n' >"$corpus/src/b.rs"

status_before=$(git -C "$corpus" status --porcelain)
head_before=$(git -C "$corpus" rev-parse HEAD)

sh "$here/../upkeep.sh" "$corpus" "$base" "$dir/work" "$dir/report.md"

status_after=$(git -C "$corpus" status --porcelain)
head_after=$(git -C "$corpus" rev-parse HEAD)

echo "--- report ---"
cat "$dir/report.md"
echo "--------------"

python3 - "$dir/report.md" "$(basename "$decision")" <<'PY'
import sys
report_path, decision = sys.argv[1], sys.argv[2]
text = open(report_path, encoding="utf-8").read()

def section(name):
    head = f"## {name}\n"
    if head not in text:
        return None
    rest = text.split(head, 1)[1]
    return rest.split("\n## ", 1)[0]

failed = []
touched = section("Touched")
if touched is None:
    failed.append("the report has no Touched section")
else:
    lines = [l for l in touched.splitlines() if "src/a.rs" in l and decision in l]
    if not lines:
        failed.append(f"Touched does not name {decision} against src/a.rs")
unmeasured = section("Unmeasured")
if unmeasured is None:
    failed.append("the report has no Unmeasured section")
elif not unmeasured.strip():
    failed.append("Unmeasured is empty, and it must never be")
elif "src/b.rs" not in unmeasured:
    failed.append("Unmeasured does not name src/b.rs, which no edge governs")
if touched is not None and "src/b.rs" in touched:
    failed.append("Touched names src/b.rs, which no edge governs")
for f in failed:
    print(f"FAIL {f}", file=sys.stderr)
sys.exit(1 if failed else 0)
PY

if [ "$status_before" != "$status_after" ]; then
    echo "FAIL the run changed the checkout: before <$status_before>, after <$status_after>" >&2
    exit 1
fi
if [ "$head_before" != "$head_after" ]; then
    echo "FAIL the run moved HEAD from $head_before to $head_after" >&2
    exit 1
fi
echo "$0: the report names the governing document, names the ungoverned path, and wrote nothing in the checkout" >&2
