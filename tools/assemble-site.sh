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
#   sh tools/assemble-site.sh           assemble, and print what it composed
#   sh tools/assemble-site.sh --check   assemble, then assert `site/` is
#                                        untouched, and exit 1 if it is not
#
set -eu

MODE=assemble
case "${1:-}" in
  --check) MODE=check ;;
  "") ;;
  *)
    echo "assemble-site.sh: unknown argument: $1" >&2
    echo "  usage: sh tools/assemble-site.sh [--check]" >&2
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

generated=$(find "$BUILD" -type f | wc -l | tr -d ' ')
handbuilt=$(find "$HAND" -type f | wc -l | tr -d ' ')
served=$(find "$OUT" -type f | wc -l | tr -d ' ')

echo "assembled \`$OUT\`:"
echo "  $generated files from the generated half (\`$BUILD\`)"
echo "  $handbuilt files from the hand-built half (\`$HAND\`), copied last"
echo "  $served files served"

if [ -n "$collisions" ]; then
  echo "  paths carried by both halves, served with the hand-built bytes:"
  printf '%s\n' "$collisions" | sed 's/^/    /'
else
  echo "  no path is carried by both halves"
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
