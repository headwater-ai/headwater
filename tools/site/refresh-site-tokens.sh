#!/bin/sh
# refresh-site-tokens.sh — write `tools/site/site-tokens.css` into every hand-built
# page under `site/`, between the two `headwater:tokens` markers in its
# `<style>` element.
#
# WHAT THIS WRITES, AND FROM WHERE
#
#   One source, `tools/site/site-tokens.css`, and the region of it between its own
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
# THE FOOTER, WHICH IS THE SAME COPY MADE TWICE MORE
#
#   `tools/site/site-footer.html` is the one copy of the footer every page of
#   the site carries, and `tools/site/site-footer.css` is the one copy of its
#   styles. Each has a `headwater:footer` marker pair, as an HTML comment in
#   the first and a CSS comment in the second. This script writes the markup
#   between the same pair in every page under `site/` and in
#   `mkdocs/overrides/main.html`, and the styles between the same pair in
#   every page's `<style>` element and in `mkdocs/overrides/css/headwater.css`.
#
#   It is here rather than in a script of its own because it is the same move
#   on the same pages, and the gates that hold the register already run this
#   file. Before it, the landing page split its footer in two groups, seven
#   pages carried a flat list, and the generated half carried a third.
#
#   The generated half takes a copy too, where it links the register. A
#   template cannot link markup, and one check that reads both halves the same
#   way is worth more than an include only one half could use.
#
# THE DISCIPLINE
#
#   `--check` runs in `.githooks/pre-commit` and in CI, so a page edited by
#   hand is a refused commit and a red build rather than something a person had
#   to remember. Run the write form after editing `tools/site/site-tokens.css`,
#   `tools/site/site-footer.html` or `tools/site/site-footer.css`, and read the
#   diff before committing it.
#
# USAGE
#
#   sh tools/site/refresh-site-tokens.sh           write every page
#   sh tools/site/refresh-site-tokens.sh --check   write nothing, and exit 1 on any
#                                             page that disagrees or carries no
#                                             marker pair
set -eu

MODE=write
case "${1:-}" in
  --check) MODE=check ;;
  "") ;;
  *) echo "refresh-site-tokens.sh: unknown argument '$1'" >&2; exit 2 ;;
esac

ROOT=$(cd "$(dirname "$0")/../.." && pwd)
cd "$ROOT"

MODE="$MODE" python3 - <<'PY'
import os
import sys
from pathlib import Path

MODE = os.environ["MODE"]
OPEN = "  /* headwater:tokens */"
CLOSE = "  /* headwater:tokens end */"

root = Path.cwd()
source = root / "tools" / "site" / "site-tokens.css"
text = source.read_text()

# The source carries the same marker pair the pages do, so the header comment
# that explains the file is not copied into eight pages that cannot use it.
try:
    body = text.split(OPEN + "\n", 1)[1].split(CLOSE, 1)[0]
except IndexError:
    sys.exit("refresh-site-tokens.sh: no marker pair in tools/site/site-tokens.css")

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
# carries `style-src 'self'`, so `mkdocs/hooks/site_tokens.py` serves this same
# file at `css/site-tokens.css` and `mkdocs/overrides/css/headwater.css` reads
# it through `var(--…)`. A theme stylesheet that pasted `#1d5c54` instead would
# be the exact second copy this record exists to prevent, and the arm above
# would never see it: it walks `site/` alone. That was the gap on the day this
# arm was written.
#
# It reads the values out of the block rather than listing them, so a token
# added to `tools/site/site-tokens.css` is covered on the commit that adds it.
declared = []
for line in body.splitlines():
    for part in line.split(";"):
        if "--" not in part or ":" not in part:
            continue
        name, _, value = part.partition(":")
        if name.strip().startswith("--"):
            declared.append((name.strip(), value.strip()))

theme = root / "mkdocs/overrides"
copied = []
unlinked = []
if theme.is_dir():
    entry = theme / "main.html"
    if not entry.is_file() or "css/site-tokens.css" not in entry.read_text():
        unlinked.append("mkdocs/overrides/main.html")
    for f in sorted(theme.rglob("*")):
        if not f.is_file() or f.suffix not in (".css", ".html", ".js"):
            continue
        text_of = f.read_text()
        for name, value in declared:
            if value and value in text_of:
                copied.append((f.relative_to(root).as_posix(), name, value))

for name, prop, value in copied:
    print("%s restates `%s: %s`, which tools/site/site-tokens.css declares"
          % (name, prop, value), file=sys.stderr)
    print("  the generated half links the register. Read it as `var(%s)`."
          % prop, file=sys.stderr)
for name in unlinked:
    print("%s does not link `css/site-tokens.css`, so the generated half is"
          % name, file=sys.stderr)
    print("  served with no visual register at all.", file=sys.stderr)

# THE THIRD ARM: ONE FOOTER, ON BOTH HALVES.
#
# Two sources, each with its own marker pair, and each copied into every
# hand-built page and into the one file of the generated half that carries it.
# A file with no pair is an error and not a skip, for the reason the first arm
# gives: a page that opted out of the shared footer would do it by carrying no
# marker.
#
# A file under `mkdocs/overrides/` that is absent is reported as unmarked. The
# second arm skips a missing theme directory. This one does not, because a
# generated half with no footer copy is the drift this arm exists to refuse.
FOOTER = [
    ("tools/site/site-footer.html",
     "    <!-- headwater:footer -->", "    <!-- headwater:footer end -->",
     "mkdocs/overrides/main.html"),
    ("tools/site/site-footer.css",
     "  /* headwater:footer */", "  /* headwater:footer end */",
     "mkdocs/overrides/css/headwater.css"),
]
foot_read = 0
foot_unmarked, foot_stale = [], []
for src, f_open, f_close, generated in FOOTER:
    src_text = (root / src).read_text() if (root / src).is_file() else ""
    try:
        f_body = src_text.split(f_open + "\n", 1)[1].split(f_close + "\n", 1)[0]
    except IndexError:
        sys.exit("refresh-site-tokens.sh: no marker pair in %s" % src)
    f_block = f_open + "\n" + f_body + f_close + "\n"
    for target in pages + [root / generated]:
        name = target.relative_to(root).as_posix()
        foot_read += 1
        current = target.read_text() if target.is_file() else ""
        start = current.find(f_open + "\n")
        end = current.find(f_close + "\n")
        if start < 0 or end < 0 or end < start:
            foot_unmarked.append((name, src, f_open, f_close))
            continue
        if current[start:end + len(f_close) + 1] == f_block:
            continue
        foot_stale.append((name, src))
        if MODE == "write":
            target.write_text(current[:start] + f_block + current[end + len(f_close) + 1:])

for name, src, f_open, f_close in foot_unmarked:
    print("no footer marker pair in %s" % name, file=sys.stderr)
    print("  every page carries the block of %s. Paste these two lines" % src,
          file=sys.stderr)
    print("  where that block belongs and run this script again:",
          file=sys.stderr)
    print("      %s\n      %s" % (f_open.strip(), f_close.strip()), file=sys.stderr)

if MODE == "check":
    for name in stale:
        print("stale %s (its block disagrees with tools/site/site-tokens.css)" % name,
              file=sys.stderr)
    for name, src in foot_stale:
        print("stale %s (its footer block disagrees with %s)" % (name, src),
              file=sys.stderr)
    print("%d page(s) read, %d unmarked, %d stale"
          % (len(pages), len(unmarked), len(stale)))
    print("%d token(s) declared, %d restated under `mkdocs/overrides/`, "
          "%d file(s) that must link the register and do not"
          % (len(declared), len(copied), len(unlinked)))
    print("%d footer block(s) read, %d unmarked, %d stale"
          % (foot_read, len(foot_unmarked), len(foot_stale)))
    raise SystemExit(1 if (unmarked or stale or copied or unlinked
                           or foot_unmarked or foot_stale) else 0)

for name in stale:
    print("wrote %s" % name, file=sys.stderr)
for name, src in foot_stale:
    print("wrote %s (footer, from %s)" % (name, src), file=sys.stderr)
print("%d page(s) read, %d unmarked, %d written"
      % (len(pages), len(unmarked), len(stale)))
print("%d footer block(s) read, %d unmarked, %d written"
      % (foot_read, len(foot_unmarked), len(foot_stale)))
raise SystemExit(1 if (unmarked or foot_unmarked) else 0)
PY
