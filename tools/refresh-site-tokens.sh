#!/bin/sh
# refresh-site-tokens.sh — write `tools/site-tokens.css` into every hand-built
# page under `site/`, between the two `headwater:tokens` markers in its
# `<style>` element.
#
# WHAT THIS WRITES, AND FROM WHERE
#
#   One source, `tools/site-tokens.css`, and the region of it between its own
#   pair of markers. Nothing is measured and nothing is derived: this is a copy
#   with one origin, which is the whole of what it is for.
#
# WHY A COPY RATHER THAN A LINK
#
#   `site/_headers` gives every hand-built path `default-src 'none'` with
#   `style-src 'unsafe-inline'` and no `'self'`, so a hand-built page fetches
#   nothing at read time. HW-DR-0037 states that as a measurement and HW-DR-0039
#   relies on it. A linked stylesheet would need `'self'` added to `style-src`
#   on all eight paths. So the copy runs before the commit, and the committed
#   bytes are what Cloudflare serves — the same move HW-DR-0039 made for a
#   figure on the same pages, and HW-DR-0050 records this one.
#
# WHAT COUNTS AS A PAGE
#
#   Every `index.html` under `site/`, found by a directory walk rather than by
#   a list typed here. A ninth page added under `site/` is therefore covered by
#   the commit that adds it, and a page with no marker pair is an error rather
#   than a silent skip. That is the rule `--check` exists to hold: a page that
#   opted out of the shared register would do it by carrying no marker, which
#   is exactly the drift this refuses.
#
# THE DISCIPLINE
#
#   `--check` runs in `.githooks/pre-commit` and in CI, so a page edited by
#   hand is a refused commit and a red build rather than something a person had
#   to remember. Run the write form after editing `tools/site-tokens.css`, and
#   read the diff before committing it.
#
# USAGE
#
#   sh tools/refresh-site-tokens.sh           write every page
#   sh tools/refresh-site-tokens.sh --check   write nothing, and exit 1 on any
#                                             page that disagrees or carries no
#                                             marker pair
set -eu

MODE=write
case "${1:-}" in
  --check) MODE=check ;;
  "") ;;
  *) echo "refresh-site-tokens.sh: unknown argument '$1'" >&2; exit 2 ;;
esac

ROOT=$(cd "$(dirname "$0")/.." && pwd)
cd "$ROOT"

MODE="$MODE" python3 - <<'PY'
import os
import sys
from pathlib import Path

MODE = os.environ["MODE"]
OPEN = "  /* headwater:tokens */"
CLOSE = "  /* headwater:tokens end */"

root = Path.cwd()
source = root / "tools" / "site-tokens.css"
text = source.read_text()

# The source carries the same marker pair the pages do, so the header comment
# that explains the file is not copied into eight pages that cannot use it.
try:
    body = text.split(OPEN + "\n", 1)[1].split(CLOSE, 1)[0]
except IndexError:
    sys.exit("refresh-site-tokens.sh: no marker pair in tools/site-tokens.css")

block = OPEN + "\n" + body + CLOSE + "\n"

pages = sorted(p for p in (root / "site").rglob("index.html"))
if not pages:
    sys.exit("refresh-site-tokens.sh: no page found under `site/`")

unmarked, stale = [], []
for page in pages:
    name = page.relative_to(root).as_posix()
    current = page.read_text()
    start = current.find(OPEN + "\n")
    end = current.find(CLOSE + "\n")
    if start < 0 or end < 0 or end < start:
        unmarked.append(name)
        continue
    if current[start:end + len(CLOSE) + 1] == block:
        continue
    stale.append(name)
    if MODE == "write":
        page.write_text(current[:start] + block + current[end + len(CLOSE) + 1:])

for name in unmarked:
    print("no marker pair in %s" % name, file=sys.stderr)
    print("  a hand-built page carries the shared block. Paste these two lines",
          file=sys.stderr)
    print("  into its `<style>` element and run this script again:",
          file=sys.stderr)
    print("      %s\n      %s" % (OPEN.strip(), CLOSE.strip()), file=sys.stderr)

if MODE == "check":
    for name in stale:
        print("stale %s (its block disagrees with tools/site-tokens.css)" % name,
              file=sys.stderr)
    print("%d page(s) read, %d unmarked, %d stale"
          % (len(pages), len(unmarked), len(stale)))
    raise SystemExit(1 if (unmarked or stale) else 0)

for name in stale:
    print("wrote %s" % name, file=sys.stderr)
print("%d page(s) read, %d unmarked, %d written"
      % (len(pages), len(unmarked), len(stale)))
raise SystemExit(1 if unmarked else 0)
PY
