#!/bin/sh
# refresh-crawler-files.sh — write `site/llms.txt` and `site/robots.txt` from
# the pages already committed under `site/`.
#
# WHAT THIS MEASURES, AND FROM WHERE
#
#   Every line either file carries comes from one source: the `<title>` and
#   `<meta name="description">` of a page already committed under `site/`,
#   read fresh on each run. Nothing here is typed by a person. A ninth page
#   appearing under `site/` and not yet in `llms.txt` is exactly the drift
#   `--check` exists to catch.
#
# WHY THIS IS A SEPARATE SCRIPT, AND NOT PART OF refresh-figures.sh
#
#   `refresh-figures.sh` patches numeric spans inside already-authored HTML
#   prose, and needs the full `check --json` / census / conformance pipeline
#   to do it. This script writes two whole new files from a directory
#   listing of `site/`, and needs none of that. Coupling the two would make
#   `llms.txt`/`robots.txt` regeneration depend on inputs it doesn't need.
#
# WHAT `llms.txt` AND `robots.txt` ARE FOR HERE
#
#   Q16 (docs/spec/09-decisions.md#q16) disposes of both files in one line:
#   ship them, and count them as nothing. Neither is referenced by, or
#   relied on by, any claim elsewhere on the site — this script only writes
#   the two files. `robots.txt` is a global allow-all: every crawler, AI
#   crawlers included, may read this corpus. `llms.txt` is the curated
#   index the `llms.txt` convention describes: an H1 title, a one-line
#   summary, and a list of every page with its own title and description.
#   No `Sitemap:` line in `robots.txt` — the only sitemap that exists
#   (`.headwater/site-build/sitemap.xml`) belongs to the MkDocs half of this
#   repository, which is not deployed, and a pointer into a 404 is exactly
#   the kind of hand-typed, unbacked claim HW-DR-0037 forbids on a
#   hand-built page.
#
# THE DISCIPLINE
#
#   Run this before any commit that adds, removes or retitles a page under
#   `site/`, and read what it prints. `--check` writes nothing and exits
#   non-zero when either committed file disagrees with a fresh run, which is
#   the form to put in front of a reviewer. `--check` runs in CI, in the
#   step named "The crawler files are what the committed pages say", so a
#   stale file is a red build rather than something a person had to
#   remember. It was left out of CI while CI was down org-wide and while
#   `wrangler.jsonc` ran no build step between the commit and the served
#   bytes; neither holds now. Run it by hand as well, before any commit
#   that adds, removes or retitles a page, so the diff you push is the one
#   you meant.
#
# USAGE
#
#   sh tools/refresh-crawler-files.sh            measure, and write the files
#   sh tools/refresh-crawler-files.sh --check     measure, write nothing, and
#                                                  exit 1 on any disagreement
#   sh tools/refresh-crawler-files.sh --print     measure, and print what
#                                                  would be written
#
set -eu

MODE=write
case "${1:-}" in
  --check) MODE=check ;;
  --print) MODE=print ;;
  "") ;;
  *) echo "refresh-crawler-files.sh: unknown argument '$1'" >&2; exit 2 ;;
esac

ROOT=$(cd "$(dirname "$0")/.." && pwd)
cd "$ROOT"

python3 - "$MODE" <<'PY'
import difflib
import html as _html
import pathlib
import re
import sys

mode = sys.argv[1]
root = pathlib.Path.cwd()
site = root / "site"
base = "https://headwater.tools"

title_re = re.compile(r"<title>([^<]*)</title>")
desc_re = re.compile(r'<meta name="description" content="([^"]*)"')


def read_page(path):
    text = path.read_text()
    tm = title_re.search(text)
    dm = desc_re.search(text)
    if not tm or not dm:
        sys.exit("refresh-crawler-files.sh: %s has no <title> or no "
                  "<meta name=\"description\">, and this script only reads "
                  "pages that carry both" % path.relative_to(root))
    title = _html.unescape(tm.group(1))
    desc = _html.unescape(dm.group(1))
    rel = path.relative_to(site)
    if rel.name != "index.html":
        sys.exit("refresh-crawler-files.sh: %s is not an index.html, and "
                  "this script only reads site/index.html and "
                  "site/*/index.html" % rel)
    url_path = "" if rel.parent == pathlib.Path(".") else str(rel.parent) + "/"
    return {"title": title, "desc": desc, "url": base + "/" + url_path}


pages = sorted(site.glob("**/index.html"))
if not pages:
    sys.exit("refresh-crawler-files.sh: no site/**/index.html pages found")

entries = [read_page(p) for p in pages]
home = next((e for e in entries if e["url"] == base + "/"), None)
if home is None:
    sys.exit("refresh-crawler-files.sh: no site/index.html among the pages "
              "found, and the home page supplies llms.txt's title and "
              "summary")

lines = [
    "# %s" % home["title"],
    "",
    "> %s" % home["desc"],
    "",
    "## Docs",
    "",
]
for e in entries:
    lines.append("- [%s](%s): %s" % (e["title"], e["url"], e["desc"]))
llms_txt = "\n".join(lines) + "\n"

robots_txt = (
    "# Every crawler is welcome to read this corpus, AI crawlers included.\n"
    "# llms.txt (https://headwater.tools/llms.txt) is the curated index.\n"
    "# This file only says nothing here is disallowed.\n"
    "User-agent: *\n"
    "Allow: /\n"
)

targets = {"site/llms.txt": llms_txt, "site/robots.txt": robots_txt}

if mode == "print":
    for name, content in targets.items():
        print("--- %s ---" % name)
        print(content, end="")
    raise SystemExit(0)

stale = []
missing = []
before = {}
for name, content in targets.items():
    path = root / name
    if not path.exists():
        missing.append(name)
    else:
        before[name] = path.read_text()
        if before[name] != content:
            stale.append(name)


def report_stale(name):
    # Name the actual lines that differ, not just the file, so a page that
    # is new, removed or retitled under `site/` is named in the report
    # rather than left as an unexplained "drift detected".
    diff = list(difflib.unified_diff(
        before[name].splitlines(), targets[name].splitlines(),
        fromfile="committed/%s" % name, tofile="measured/%s" % name,
        lineterm="", n=0))
    for line in diff:
        print("  %s" % line, file=sys.stderr)


if mode == "check":
    for name in missing:
        print("missing %s (never committed)" % name, file=sys.stderr)
    for name in stale:
        print("stale %s (committed copy disagrees with a fresh run):"
              % name, file=sys.stderr)
        report_stale(name)
    print("%d pages read, %d file(s) missing, %d file(s) stale"
          % (len(entries), len(missing), len(stale)))
    if missing or stale:
        raise SystemExit(1)
    raise SystemExit(0)

# mode == "write"
for name, content in targets.items():
    (root / name).write_text(content)
for name in missing:
    print("wrote %s (new)" % name, file=sys.stderr)
for name in stale:
    print("wrote %s (was stale)" % name, file=sys.stderr)
print("%d pages read, %d file(s) written" % (len(entries), len(targets)))
PY
