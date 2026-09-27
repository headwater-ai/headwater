#!/bin/sh
# check-site-figures.sh — refuse an assembled site that would serve a blank figure.
#
#   sh tools/site/check-site-figures.sh <assembled-dir>
#
# The pages under `site/` carry every `data-figure` element empty, and
# `tools/site/refresh-figures.sh --into <dir>` fills the copies when the site is
# published (#1273). A directory that reaches a deploy with an empty element
# still in it would serve a page with a hole where a number belongs, so this
# refuses it. Two callers run it:
#
#   `tools/site/deploy-site.sh`, after the fill and before `wrangler`, as a last
#   look at what is about to be served.
#
#   `tools/site/cloudflare-build.sh`, the Workers Builds command. Nothing there
#   fills the figures, so while that integration stays connected every build
#   it runs is refused here and publishes nothing, rather than publishing
#   blanks over the CI deploy.
#
# Exit 0 when every element holds a value, 1 when one is empty or the
# directory holds no page, 2 on a wrong argument.
set -eu

dir=${1:-}
if [ -z "$dir" ] || [ ! -d "$dir" ]; then
  echo "check-site-figures.sh: name the assembled directory to check" >&2
  exit 2
fi

python3 - "$dir" <<'PY'
import pathlib, re, sys

base = pathlib.Path(sys.argv[1])
pages = sorted(base.glob("**/*.html"))
if not pages:
    sys.exit("check-site-figures.sh: no page under %s" % base)
empty = re.compile(r'<(\w+)\b[^>]*\bdata-figure="([^"]*)"[^>]*>\s*</\1>')
blank, markers = [], 0
for page in pages:
    text = page.read_text(errors="replace")
    markers += text.count("data-figure=")
    for m in empty.finditer(text):
        blank.append((page.relative_to(base), m.group(2)))
for page, key in blank:
    print("blank figure %s in %s" % (key, page), file=sys.stderr)
if blank:
    print("check-site-figures.sh: %d of %d figures under %s are blank, so this "
          "directory is not deployed. A figure is filled by "
          "`sh tools/site/refresh-figures.sh --into %s`, which "
          "`tools/site/deploy-site.sh` runs." % (len(blank), markers, base, base),
          file=sys.stderr)
    raise SystemExit(1)
print("%d figures under %s, none blank" % (markers, base))
PY
