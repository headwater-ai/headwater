#!/bin/sh
# assemble-site.sh — compose the one directory that https://headwater.tools/
# serves, from the two halves this repository keeps apart.
#
# THE TWO HALVES, AND WHY THEY ARE APART
#
#   `site/` is hand-built. Every byte of it is committed, HW-DR-0037 governs
#   it by name, and no build step writes into it: the #430 CI step and the
#   `site/` clause of `.githooks/pre-commit` both exist to keep that true.
#
#   `.headwater/site-build/` is what `mkdocs build --strict` writes. It is the
#   rendered projection of the corpus — the specification, the decisions, the
#   obligations, the tutorial — and it is git-ignored, because every byte of
#   it is recomputable from the tree beside it.
#
#   Until this script existed the second half was built on every push and then
#   thrown away, and `https://headwater.tools/spec/` answered 404.
#
# WHAT THIS WRITES, AND IN WHICH ORDER
#
#   One directory, `.headwater/site-deploy/`, holding the generated half first
#   and the hand-built half second. The order is the rule: `site/` is copied
#   last, so a path carried by both halves is served with the hand-built
#   bytes. Nothing on this tree collides today, and the summary below names
#   every collision it overwrites so that the first one to appear is read by a
#   person rather than absorbed in silence.
#
#   `.headwater/corpus.json` is copied to the root of the output as
#   `corpus.json`, which is the target every generated page names in its
#   `rel="describedby"` link. CI copies the same file into the build output
#   for its own assertion (#428, #432); this script repeats it so that an
#   assembly is complete on its own, without a CI step having run first.
#
# THE ONE FILE THIS SCRIPT COMPOSES RATHER THAN COPIES
#
#   `sitemap.xml`, written last, over whichever copy the two halves left
#   there. It is the one file whose correct contents neither half knows: the
#   assembled directory is the only place both halves exist together, so it
#   is the only place a list of every served page can be derived.
#   `tools/site/sitemap.py` walks the output and writes one `<loc>` per
#   `index.html`, and `tools/site/refresh-crawler-files.sh` calls the same module
#   with `site/` for the copy that is committed and served today.
#
#   Both halves carried a `sitemap.xml` and each was wrong on its own. The
#   hand-built one was a list of seven URLs a person typed, and it already
#   omitted `site/changelog/` on the day it was committed. The MkDocs one
#   knows only the generated half, which is the durable half of the argument
#   and the one that survives a change to `mkdocs.yml`. It carried zero URLs
#   when #554 reported this, because `mkdocs.yml` set no `site_url` and the
#   MkDocs template emits a `<loc>` only from a canonical URL; #556 set that
#   value, so it now carries one `<loc>` per generated page and still none
#   for the eight hand-built pages. So the rule at the top of this file — the
#   hand-built half wins a collision — would have served the stale typed
#   list, and reversing it would serve a list missing every hand-built page.
#   #554 is the report, and this is the disposition it took.
#
#   `sitemap.xml.gz`, which `mkdocs build` writes beside its own sitemap, is
#   removed rather than recomputed. Nothing links it, and a compressed second
#   copy of a list is one more thing that can disagree with the list.
#
# WHAT THIS NEVER WRITES
#
#   Anything under `site/`. This script only reads that half. `--check` runs
#   the whole assembly and then asserts `git status --porcelain -- site/` is
#   empty, which is the clause #527 asks a reviewer to verify.
#
# WHERE THIS RUNS
#
#   On every CI push, so that the assembly is exercised by the same runner
#   that already builds the site, and a break is found before a deploy runs
#   it. And on the deploy path, which serves what it writes.
#
# USAGE
#
#   sh tools/site/assemble-site.sh           assemble, and print what it composed
#   sh tools/site/assemble-site.sh --check   assemble, then assert `site/` is
#                                        untouched, and exit 1 if it is not
#
set -eu

MODE=assemble
case "${1:-}" in
  --check) MODE=check ;;
  "") ;;
  *)
    echo "assemble-site.sh: unknown argument: $1" >&2
    echo "  usage: sh tools/site/assemble-site.sh [--check]" >&2
    exit 2
    ;;
esac

root=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
cd "$root"

BUILD=.headwater/site-build
HAND=site
OUT=.headwater/site-deploy
DESCRIPTOR=.headwater/corpus.json

if [ ! -d "$BUILD" ]; then
  echo "assemble-site.sh: no generated half at \`$BUILD\`." >&2
  echo "  \`mkdocs build --strict\` writes it, and \`mkdocs.yml\` sets the path." >&2
  echo "  Run the build first:" >&2
  echo "        python3 -m mkdocs build --strict" >&2
  exit 1
fi

if [ ! -d "$HAND" ]; then
  echo "assemble-site.sh: no hand-built half at \`$HAND\`." >&2
  exit 1
fi

rm -rf "$OUT"
mkdir -p "$OUT"

cp -R "$BUILD"/. "$OUT"/

if [ -f "$DESCRIPTOR" ]; then
  cp "$DESCRIPTOR" "$OUT"/corpus.json
else
  echo "assemble-site.sh: no corpus descriptor at \`$DESCRIPTOR\`," >&2
  echo "  so every generated page's \`rel=\"describedby\"\` link will 404." >&2
  echo "  \`headwater generate\` writes it." >&2
  exit 1
fi

# Name every path the hand-built half is about to shadow, before it does.
# `$OUT` holds the generated half alone at this point, so the two listings
# below are exactly the two halves as they will be served.
scratch=$(mktemp -d)
trap 'rm -rf "$scratch"' EXIT INT TERM
# `LC_ALL=C` on both the sort and the comparison: `comm` collates bytewise and
# refuses input a locale-aware `sort` ordered differently, which is a failure
# that appears on one machine and not on the next.
( cd "$HAND" && find . -type f ) | sed 's|^\./||' | LC_ALL=C sort >"$scratch/hand"
( cd "$OUT" && find . -type f ) | sed 's|^\./||' | LC_ALL=C sort >"$scratch/generated"
collisions=$(LC_ALL=C comm -12 "$scratch/hand" "$scratch/generated")

cp -R "$HAND"/. "$OUT"/

# The sitemap of the whole site, derived from the whole site. This runs after
# both halves are in place, because the directory is the input. Written to a
# temporary file first, so a failing walk leaves the copied sitemap in place
# rather than truncating the served one.
if ! python3 "$root/tools/site/sitemap.py" "$OUT" >"$OUT/.sitemap.xml.new"; then
  rm -f "$OUT/.sitemap.xml.new"
  echo "assemble-site.sh: could not derive the sitemap from \`$OUT\`." >&2
  exit 1
fi
mv "$OUT/.sitemap.xml.new" "$OUT/sitemap.xml"
sitemap_urls=$(grep -c '<loc>' "$OUT/sitemap.xml")
rm -f "$OUT/sitemap.xml.gz"

generated=$(find "$BUILD" -type f | wc -l | tr -d ' ')
handbuilt=$(find "$HAND" -type f | wc -l | tr -d ' ')
served=$(find "$OUT" -type f | wc -l | tr -d ' ')

echo "assembled \`$OUT\`:"
echo "  $generated files from the generated half (\`$BUILD\`)"
echo "  $handbuilt files from the hand-built half (\`$HAND\`), copied last"
echo "  $served files served"

echo "  $sitemap_urls URLs in \`sitemap.xml\`, derived from the output above"

# `sitemap.xml` is carried by both halves and served with neither half's
# bytes, so it is reported on its own line rather than counted as a shadowing.
# Every other collision is the case the rule at the top of this file decides,
# and it is named here so that the first one to appear is read by a person.
shadowed=$(printf '%s\n' "$collisions" | grep -v '^$' | grep -vx 'sitemap.xml' || true)
if [ -n "$shadowed" ]; then
  echo "  paths carried by both halves, served with the hand-built bytes:"
  printf '%s\n' "$shadowed" | sed 's/^/    /'
else
  echo "  no path is carried by both halves and shadowed"
fi

if [ "$MODE" = check ]; then
  moved=$(git status --porcelain -- "$HAND"/)
  if [ -n "$moved" ]; then
    echo "assemble-site.sh: the assembly wrote into \`$HAND/\`, which is the" >&2
    echo "  hand-built half and never an output. What moved:" >&2
    printf '%s\n' "$moved" | sed 's/^/    /' >&2
    exit 1
  fi
  echo "  \`$HAND/\` is unchanged"
fi
