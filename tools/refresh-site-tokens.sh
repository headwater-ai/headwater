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

# THE SECOND ARM: THE GENERATED HALF LINKS THE REGISTER AND NEVER RESTATES IT.
#
# HW-DR-0050 clause 4. The generated half is served from `/*`, whose policy
# carries `style-src 'self'`, so `mkdocs-hooks/site_tokens.py` serves this same
# file at `css/site-tokens.css` and `mkdocs-overrides/css/headwater.css` reads
# it through `var(--…)`. A theme stylesheet that pasted `#1d5c54` instead would
# be the exact second copy this record exists to prevent, and the arm above
# would never see it: it walks `site/` alone. That was the gap on the day this
# arm was written.
#
# It reads the values out of the block rather than listing them, so a token
# added to `tools/site-tokens.css` is covered on the commit that adds it.
declared = []
for line in body.splitlines():
    for part in line.split(";"):
        if "--" not in part or ":" not in part:
            continue
        name, _, value = part.partition(":")
        if name.strip().startswith("--"):
            declared.append((name.strip(), value.strip()))

theme = root / "mkdocs-overrides"
copied = []
unlinked = []
if theme.is_dir():
    entry = theme / "main.html"
    if not entry.is_file() or "css/site-tokens.css" not in entry.read_text():
        unlinked.append("mkdocs-overrides/main.html")
    for f in sorted(theme.rglob("*")):
        if not f.is_file() or f.suffix not in (".css", ".html", ".js"):
            continue
        text_of = f.read_text()
        for name, value in declared:
            if value and value in text_of:
                copied.append((f.relative_to(root).as_posix(), name, value))

for name, prop, value in copied:
    print("%s restates `%s: %s`, which tools/site-tokens.css declares"
          % (name, prop, value), file=sys.stderr)
    print("  the generated half links the register. Read it as `var(%s)`."
          % prop, file=sys.stderr)
for name in unlinked:
    print("%s does not link `css/site-tokens.css`, so the generated half is"
          % name, file=sys.stderr)
    print("  served with no visual register at all.", file=sys.stderr)

if MODE == "check":
    for name in stale:
        print("stale %s (its block disagrees with tools/site-tokens.css)" % name,
              file=sys.stderr)
    print("%d page(s) read, %d unmarked, %d stale"
          % (len(pages), len(unmarked), len(stale)))
    print("%d token(s) declared, %d restated under `mkdocs-overrides/`, "
          "%d file(s) that must link the register and do not"
          % (len(declared), len(copied), len(unlinked)))
    raise SystemExit(1 if (unmarked or stale or copied or unlinked) else 0)

for name in stale:
    print("wrote %s" % name, file=sys.stderr)
print("%d page(s) read, %d unmarked, %d written"
      % (len(pages), len(unmarked), len(stale)))
raise SystemExit(1 if unmarked else 0)
PY
