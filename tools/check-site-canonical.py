#!/usr/bin/env python3
"""Assert that every generated page carries its own canonical URL.

WHAT THIS READS

  The generated half of the site, as built bytes. `mkdocs build` writes one
  `<link rel="canonical">` per page from `site_url` in `mkdocs.yml`, and it
  writes none at all when that setting is absent. HW-DR-0048 recorded that
  absence and #556 closed it. Nothing in the build says so: with `site_url`
  deleted, `mkdocs build --strict` exits 0 with no warning while every page
  loses the element, which is the same failure mode the corpus-descriptor step
  in `.github/workflows/ci.yml` was added for.

WHAT IT ASSERTS, AND WHY PRESENCE ALONE IS NOT ENOUGH

  1. Every `*.html` page under the directory carries exactly one canonical URL.
  2. Each page's canonical URL is the URL that page is served at.

  A gate that greps for the string `rel="canonical"` passes a page whose
  canonical names some other page, and that is worse than no canonical at all:
  it hands a search engine a wrong answer confidently, and the wrong answer is
  the one a crawler believes. A wrong origin, a dropped trailing slash and a
  typo in `site_url` all pass assertion 1 and all fail assertion 2.

WHERE THE EXPECTED ORIGIN COMES FROM

  `--site-url`, which defaults to `https://headwater.tools/`. That default is a
  second statement of the origin, beside the one in `mkdocs.yml`, and it is
  deliberate rather than a duplicate to keep in step. A gate that read the
  origin out of the file it is checking would confirm that a typo agrees with
  itself. `tools/refresh-crawler-files.sh --check` takes the same posture,
  comparing what is committed against what is measured rather than deriving
  one from the other.

WHAT IS EXEMPT, AND WHY

  `404.html`, which is the one generated page that carries no canonical URL.
  MkDocs renders it from the theme's own `404.html`, which extends `base.html`;
  the canonical element comes from the same template chain as the rest, but a
  404 is served under whatever address the reader mistyped, so it has no URL of
  its own for a canonical to name. The corpus-descriptor step exempts the same
  file for the neighboring reason. The exemption is by exact path at the root of
  the built directory, not by suffix, so a page named `404.html` inside a shelf
  is checked like any other.

  The generated half has no root `index.html` at all: `docs/README.md` does not
  exist and `.headwater/nav.yml` names no root page, so `https://headwater.tools/`
  is served by `site/index.html` from the hand-built half. This tool still
  derives the right URL for a root `index.html` should one ever appear, and says
  nothing about the hand-built half.

WHY IT IS NOT POINTED AT THE ASSEMBLED SITE

  `tools/check-site-fragments.py` reads the assembled root, both halves
  composed, because a dead fragment is a defect wherever it is served. This one
  cannot: none of the eight hand-built pages carries a canonical URL, so
  assertion 1 over the assembled root would be red on correct input from its
  first run. Whether those eight should carry one is a separate question.

WHAT IT REFUSES TO DO

  Pass over an empty population. A directory with no page in it satisfies both
  assertions vacuously, and a check that reports success while looking at
  nothing is the failure this repository has met more than once. Zero pages is
  exit 2 and not exit 0.

USAGE

    python3 tools/check-site-canonical.py <directory> [--site-url URL]

  Exit 0 when every page passes, 1 on a finding, 2 when the run could not be
  made at all. `sh tools/site-canonical-fixtures.sh` provokes each of those.
"""

import argparse
import pathlib
import sys
from html.parser import HTMLParser

DEFAULT_SITE_URL = "https://headwater.tools/"

# Relative to the root of the built directory. See the header.
EXEMPT = frozenset({"404.html"})


class CanonicalFinder(HTMLParser):
    """Collect the `href` of every `<link rel="canonical">` on a page."""

    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.hrefs = []

    def handle_starttag(self, tag, attrs):
        if tag != "link":
            return
        attributes = dict(attrs)
        rel = (attributes.get("rel") or "").split()
        if "canonical" in [token.lower() for token in rel]:
            self.hrefs.append(attributes.get("href"))


def served_path(relative):
    """The path a page is served at, relative to the site root.

    A directory URL for an `index.html`, and the file path itself otherwise.
    `use_directory_urls` is at its MkDocs default, so every generated page is
    the first form and the trailing slash is part of the URL.
    """
    parts = relative.split("/")
    if parts[-1] == "index.html":
        parent = "/".join(parts[:-1])
        return f"{parent}/" if parent else ""
    return relative


def main(argv):
    parser = argparse.ArgumentParser(add_help=True, description=__doc__)
    parser.add_argument("directory")
    parser.add_argument("--site-url", default=DEFAULT_SITE_URL)
    args = parser.parse_args(argv)

    root = pathlib.Path(args.directory)
    if not root.is_dir():
        print(f"no directory at `{args.directory}`.", file=sys.stderr)
        print("  build the generated half first:", file=sys.stderr)
        print("        python3 -m mkdocs build --strict", file=sys.stderr)
        return 2

    origin = args.site_url.rstrip("/")

    pages = sorted(p for p in root.rglob("*.html") if p.is_file())
    checked = []
    exempt = []
    for page in pages:
        relative = page.relative_to(root).as_posix()
        if relative in EXEMPT:
            exempt.append(relative)
        else:
            checked.append((page, relative))

    if not checked:
        print(
            f"`{args.directory}` holds no page to check, so this run would "
            "report success without looking at anything.",
            file=sys.stderr,
        )
        print(
            f"  it holds {len(pages)} html file(s), {len(exempt)} of them exempt.",
            file=sys.stderr,
        )
        return 2

    absent = []
    repeated = []
    wrong = []
    for page, relative in checked:
        finder = CanonicalFinder()
        finder.feed(page.read_text(encoding="utf-8", errors="replace"))
        hrefs = [href for href in finder.hrefs if href]
        if not hrefs:
            absent.append(relative)
            continue
        if len(hrefs) > 1:
            repeated.append((relative, hrefs))
            continue
        expected = f"{origin}/{served_path(relative)}"
        if hrefs[0] != expected:
            wrong.append((relative, hrefs[0], expected))

    if absent:
        print("pages carrying no canonical URL:", file=sys.stderr)
        for relative in absent:
            print(f"  {relative}", file=sys.stderr)
    if repeated:
        print("pages carrying more than one canonical URL:", file=sys.stderr)
        for relative, hrefs in repeated:
            print(f"  {relative}: {', '.join(hrefs)}", file=sys.stderr)
    if wrong:
        print(
            "pages whose canonical URL is not the URL they are served at:",
            file=sys.stderr,
        )
        for relative, got, expected in wrong:
            print(f"  {relative}", file=sys.stderr)
            print(f"    it says   {got}", file=sys.stderr)
            print(f"    served at {expected}", file=sys.stderr)

    findings = len(absent) + len(repeated) + len(wrong)
    exempted = ", ".join(exempt) if exempt else "none"
    print(
        f"{len(checked) - findings} of {len(checked)} generated pages carry "
        f"one canonical URL naming the URL they are served at, against "
        f"`{origin}/`."
    )
    print(
        f"{len(absent)} with none, {len(repeated)} with more than one, "
        f"{len(wrong)} naming another URL. Exempt: {exempted}."
    )
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
