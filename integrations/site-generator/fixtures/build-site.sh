#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
#
# The decisive fixture of `docs/how-to/publish-your-corpus-as-a-site.md`
# (#977). It follows that guide against the corpus the tutorial finishes with,
# and it runs what the guide says rather than a copy of it:
#
# - The overlay block is the guide's first fenced block that opens with
#   `add_to:`, appended to the corpus overlay.
# - The configuration is `integrations/site-generator/mkdocs.yml` as shipped,
#   which is what the guide says to copy.
# - Every command is a line of one of the guide's `sh` blocks, run in the
#   guide's order. A guide that loses `headwater generate` leaves no
#   `.headwater/nav.yml`, and the build that `INHERIT`s it fails here.
#
# - The guide's last command must be `headwater site site`, after its last
#   `mkdocs build` (#978). It runs with the other commands, so it must exit 0
#   over the site the guide built. Then a page is removed from a copy of that
#   site, and the guide's own `headwater site` line runs again over the copy.
#   It must exit non-zero and report `site.page.missing` for that document. A
#   guide that drops the step, or moves it before the build, fails here.
#
# Then the negative case: a navigation entry for `decisions/0999-missing.md`
# is appended to the generated `.headwater/nav.yml`, and the guide's own
# `mkdocs build` line runs again. It must exit non-zero and name that path.
# This is what holds the guide's `--strict`: without it MkDocs prints a
# warning and exits 0, and this script fails.
#
# Two things the script does not take from the guide. The `pip install` line
# is skipped, because the caller supplies MkDocs 1.6.1 (`MKDOCS`). A
# `mkdocs` line gets `--site-dir <temporary directory>` appended, so that
# nothing is written into the corpus; the guide's own flags are kept. The
# guide's `headwater site site` reads that same temporary directory in place of
# `site`, the MkDocs default output directory that the guide names.
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
# else `PATH`), MkDocs 1.6.1 (`MKDOCS`, default `mkdocs`), `awk`, `cp`,
# `cut`, `grep`, `head`, `mktemp`, `tail`. The tutorial it may run needs `python3` and `git` as well.
set -eu

here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../../.." && pwd)
guide=$root/docs/how-to/publish-your-corpus-as-a-site.md
config=$root/integrations/site-generator/mkdocs.yml
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

# The overlay block: the first fenced block that opens with `add_to:`.
awk '
  /^```/ { if (inside) { if (take) exit; inside = 0; next } inside = 1; first = 1; next }
  inside && first { first = 0; if ($0 ~ /^add_to:/) take = 1 }
  inside && take { print }
' "$guide" >"$work/overlay-block.yml"
grep -q 'kind: site_nav' "$work/overlay-block.yml" || fail "the guide holds no add_to block that declares site_nav"
printf '\n' >>.headwater/overlay.yml
cat "$work/overlay-block.yml" >>.headwater/overlay.yml

# The configuration the guide says to copy. The guide's `docs_dir` is the
# tutorial's `corpus.root`, `docs`, so the file is copied unedited.
cp "$config" mkdocs.yml

# The commands: every line of every `sh` block, in order, less `pip install`.
awk '
  /^```/ { if (inside) { inside = 0; next } if ($0 ~ /^```sh[ \t]*$/) inside = 1; else inside = 2; next }
  inside == 1 && NF && $0 !~ /pip install/ { print }
' "$guide" >"$work/commands.sh"
grep -q '^mkdocs build' "$work/commands.sh" || fail "the guide runs no mkdocs build"
grep '^mkdocs build' "$work/commands.sh" >"$work/build-commands.sh"

# The last step of a publish is `headwater site` over the directory MkDocs
# wrote, and nothing the guide runs comes after it. The line is exactly
# `headwater site site`, because step 8 tells a reader with another `site_dir`
# to put it as the last word of the command.
last=$(awk 'NF { line = $0 } END { print line }' "$work/commands.sh")
case $last in
  'headwater site site') ;;
  *) fail "the guide's last command is not \`headwater site site\`: $last" ;;
esac
last_build=$(grep -n '^mkdocs build' "$work/commands.sh" | tail -n 1 | cut -d: -f1)
first_site=$(grep -n '^headwater site' "$work/commands.sh" | head -n 1 | cut -d: -f1)
[ "$first_site" -gt "$last_build" ] || fail "the guide runs \`headwater site\` before its last mkdocs build"
printf '%s\n' "$last" >"$work/site-commands.sh"

# `mkdocs` runs the caller's MkDocs and writes the site outside the corpus, and
# `headwater site site` reads that directory.
{
  printf 'set -e\n'
  printf 'mkdocs() { command %s "$@" --site-dir "$SITE_DIR"; }\n' "${MKDOCS:-mkdocs}"
  printf 'headwater() { if [ "${1:-}" = site ] && [ "${2:-}" = site ]; then shift 2; command headwater site "$SITE_DIR" "$@"; else command headwater "$@"; fi; }\n'
} >"$work/prelude.sh"
cat "$work/prelude.sh" "$work/commands.sh" >"$work/run-guide.sh"
cat "$work/prelude.sh" "$work/build-commands.sh" >"$work/run-build.sh"
cat "$work/prelude.sh" "$work/site-commands.sh" >"$work/run-site.sh"

SITE_DIR=$work/site sh "$work/run-guide.sh" >"$work/guide.out" 2>"$work/guide.err" \
  || fail "the guide's commands failed; see $work/guide.err"
[ -f .headwater/nav.yml ] || fail "the guide's commands wrote no .headwater/nav.yml"
[ -f "$work/site/decisions/0002-deliver-at-least-once/index.html" ] \
  || fail "the site holds no page for decisions/0002-deliver-at-least-once.md"
headwater check --strict >"$work/check.out" 2>&1 || fail "check --strict failed after the guide; see $work/check.out"
headwater generate --check >"$work/gencheck.out" 2>&1 || fail "generate --check failed after the guide; see $work/gencheck.out"
echo "site fixture: the guide's commands build the site, and its last step holds it"

# The last step catches a page the build lost: the guide's own `headwater site`
# line over a copy of the site with one page removed.
cp -R "$work/site" "$work/site-gap"
rm -f "$work/site-gap/decisions/0002-deliver-at-least-once/index.html"
if SITE_DIR=$work/site-gap sh "$work/run-site.sh" >"$work/gap.out" 2>"$work/gap.err"; then
  fail "the guide's headwater site exited 0 on a site that lacks a page; see $work/gap.out"
fi
grep 'site\.page\.missing' "$work/gap.out" | grep -q 'decisions/0002-deliver-at-least-once' \
  || fail "the guide's headwater site failed, but reported no site.page.missing for decisions/0002-deliver-at-least-once; see $work/gap.out and $work/gap.err"
echo "site fixture: a page the build lost fails the guide's last step and is named"

# The negative case: a navigation entry that names a page that does not exist,
# built with the guide's own `mkdocs build` line.
printf '  - "Missing": "decisions/0999-missing.md"\n' >>.headwater/nav.yml
if SITE_DIR=$work/site-bad sh "$work/run-build.sh" >"$work/bad.out" 2>"$work/bad.err"; then
  fail "the guide's mkdocs build exited 0 on a navigation entry for a missing page"
fi
grep -q 'decisions/0999-missing.md' "$work/bad.err" \
  || fail "the guide's mkdocs build failed, but its stderr does not name decisions/0999-missing.md; see $work/bad.err"
echo "site fixture: a navigation entry for a missing page fails the guide's build and is named"

# The generated file is held: `headwater generate --check` reports the edit.
if headwater generate --check >"$work/gencheck-bad.out" 2>&1; then
  fail "headwater generate --check passed a hand-edited .headwater/nav.yml"
fi
headwater generate >/dev/null 2>&1
rm -rf "$work"
echo "site fixture: passed"
