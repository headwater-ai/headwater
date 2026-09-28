#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
#
# The decisive fixture of `docs/how-to/publish-your-corpus-as-a-site.md`
# (#977). It follows that guide against the corpus the tutorial finishes with,
# and it proves two things:
#
# 1. The recipe builds. The overlay block is read out of the guide itself and
#    not copied here, and the configuration is `integrations/site-generator/
#    mkdocs.yml` as shipped, so a guide or a configuration that goes stale
#    fails this script rather than an adopter's first build.
# 2. The build refuses a navigation that names a missing page. A navigation
#    entry for `decisions/0999-missing.md` is appended to the generated
#    `.headwater/nav.yml`, and `mkdocs build --strict` must exit non-zero and
#    name that path. This is the one case that holds `--strict` in the guide:
#    without it MkDocs prints a warning and exits 0.
#
# No adopter runs this. It is a fixture of this repository, as
# `integrations/headwater-check/fixtures/` is.
#
# Usage: sh build-site.sh [<finished-tutorial-corpus>]
#
# With no argument it runs `.claude/tutorial/fixtures.sh` itself, with
# `HEADWATER_TUTORIAL_KEEP` set, which fetches the package once. CI passes the
# corpus the tutorial step already kept. The script edits the corpus it is
# given.
#
# Needs: `headwater` (`HEADWATER_BIN`, else the engine this checkout built,
# else `PATH`), MkDocs 1.6.1 (`MKDOCS`, default `mkdocs`), `awk`, `grep`,
# `mktemp`. The tutorial it may run needs `python3` and `git` as well.
set -eu

here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../../.." && pwd)
guide=$root/docs/how-to/publish-your-corpus-as-a-site.md
config=$root/integrations/site-generator/mkdocs.yml
mkdocs=${MKDOCS:-mkdocs}
work=$(mktemp -d)

fail() {
  echo "site fixture: $*" >&2
  exit 1
}

[ -f "$guide" ] || fail "no guide at $guide"
[ -f "$config" ] || fail "no configuration at $config"

. "$root/tools/repo/resolve-engine.sh"
bin=${HEADWATER_BIN:-}
if [ -z "$bin" ]; then
  bin=$(hw_resolve_engine_bin "$root") || bin=$(command -v headwater) || bin=
fi
[ -n "$bin" ] && [ -x "$bin" ] || fail "no headwater binary; set HEADWATER_BIN"
PATH=$(dirname "$bin"):$PATH
export PATH

if [ $# -ge 1 ]; then
  corpus=$1
else
  corpus=$work/corpus
  HEADWATER_TUTORIAL_KEEP=$corpus HEADWATER_BIN=$bin sh "$root/.claude/tutorial/fixtures.sh" >"$work/tutorial.out" 2>&1 \
    || fail "the tutorial failed; see $work/tutorial.out"
fi
[ -f "$corpus/.headwater/overlay.yml" ] || fail "$corpus holds no .headwater/overlay.yml"
cd "$corpus"

# Step 1 of the guide: the first fenced block that opens with `add_to:`.
awk '
  /^```/ { if (inside) { if (take) exit; inside = 0; next } inside = 1; first = 1; next }
  inside && first { first = 0; if ($0 ~ /^add_to:/) take = 1 }
  inside && take { print }
' "$guide" >"$work/overlay-block.yml"
grep -q 'kind: site_nav' "$work/overlay-block.yml" || fail "the guide holds no add_to block that declares site_nav"
printf '\n' >>.headwater/overlay.yml
cat "$work/overlay-block.yml" >>.headwater/overlay.yml

headwater taxonomy resolve >"$work/resolve.out" 2>&1 || fail "resolve refused the guide's overlay block; see $work/resolve.out"
headwater generate >"$work/generate.out" 2>&1 || fail "generate failed; see $work/generate.out"
[ -f .headwater/nav.yml ] || fail "generate wrote no .headwater/nav.yml"
headwater check --strict >"$work/check.out" 2>&1 || fail "check --strict failed after the recipe; see $work/check.out"

# Step 2 and 3 of the guide: the shipped configuration, built strictly.
cp "$config" mkdocs.yml
$mkdocs build --strict --site-dir "$work/site" >"$work/build.out" 2>"$work/build.err" \
  || fail "mkdocs build --strict failed on the recipe; see $work/build.err"
[ -f "$work/site/decisions/0002-deliver-at-least-once/index.html" ] \
  || fail "the site holds no page for decisions/0002-deliver-at-least-once.md"
echo "site fixture: the recipe builds, strict"

# The negative case: a navigation entry that names a page that does not exist.
printf '  - "Missing": "decisions/0999-missing.md"\n' >>.headwater/nav.yml
if $mkdocs build --strict --site-dir "$work/site-bad" >"$work/bad.out" 2>"$work/bad.err"; then
  fail "mkdocs build --strict exited 0 on a navigation entry for a missing page"
fi
grep -q 'decisions/0999-missing.md' "$work/bad.err" \
  || fail "mkdocs build --strict failed, but its stderr does not name decisions/0999-missing.md; see $work/bad.err"
echo "site fixture: a navigation entry for a missing page fails the strict build and is named"

# The generated file is held: `headwater generate --check` reports the edit.
if headwater generate --check >"$work/gencheck.out" 2>&1; then
  fail "headwater generate --check passed a hand-edited .headwater/nav.yml"
fi
headwater generate >/dev/null 2>&1
rm -rf "$work"
echo "site fixture: passed"
