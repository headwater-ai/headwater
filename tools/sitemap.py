"""sitemap.py — write one sitemap from a directory of pages, and never a list
a person typed.

WHAT THIS READS, AND WHY A DIRECTORY IS THE INPUT

  A sitemap is a list of the pages a site serves. The only place that list is
  known is the directory that is served, so that directory is what this reads.
  Every `index.html` under it becomes one `<loc>`, derived from the path.
  Nothing here is typed, and a page added, renamed or removed moves the
  sitemap on the next run.

  `site/sitemap.xml` was a list of seven URLs typed by hand on 2026-09-06.
  `site/changelog/index.html` was already committed on that day and the list
  did not carry it, so the file was stale before it was pushed. That is the
  failure HW-DR-0037 forbids on a hand-built page, and #554 is the report of
  it.

WHY BOTH HALVES CALL THIS, AND WITH WHICH DIRECTORY

  `tools/refresh-crawler-files.sh` calls it with `site/`, and commits the
  result as `site/sitemap.xml`. That is what a reader gets while
  `wrangler.jsonc` still names `./site`, and it lists the hand-built pages.

  `tools/assemble-site.sh` calls it with `.headwater/site-deploy/`, after it
  has copied both halves there, and overwrites the copy it finds. The
  assembled directory is the only place the two halves exist together, so it
  is the only place a sitemap of the whole site can be derived. That is what a
  reader gets once `wrangler.jsonc` names the assembled directory, which
  HW-DR-0047 orders.

  One walk serves both, so there is no second copy of the rule to keep in
  step.

WHAT MKDOCS WRITES, AND WHY IT IS NOT ENOUGH

  `mkdocs build` writes a `sitemap.xml` of its own into the generated half. It
  knows only that half, which is why it is never enough whatever it carries.

  It carried zero URLs until #556: the MkDocs template emits a `<loc>` from
  `page.canonical_url`, `mkdocs.yml` set no `site_url`, and a page with no
  canonical URL contributes no line, so the generated sitemap was an empty
  `<urlset>` and serving it would have told a crawler this site has no pages at
  all. `mkdocs.yml` now declares `site_url`, so it carries one `<loc>` per
  generated page. The eight hand-built pages are still absent from it, and a
  sitemap that omits the page served at the site root is the wrong one to
  serve.

USAGE

  python3 tools/sitemap.py <directory>     print the sitemap for a directory
"""

import pathlib
import sys

BASE = "https://headwater.tools"


def urls(directory, base=BASE):
    """Every served URL under `directory`, sorted, one per `index.html`.

    A page is a directory holding an `index.html`, which is the shape both
    halves use: `mkdocs.yml` sets no `use_directory_urls`, so MkDocs keeps its
    default of directory URLs, and every hand-built page is a directory too.
    A file that is not an `index.html` — `404.html`, a stylesheet, the search
    index — is not a page and gets no line.
    """
    root = pathlib.Path(directory)
    found = []
    for path in root.glob("**/index.html"):
        rel = path.relative_to(root).parent
        tail = "" if rel == pathlib.Path(".") else str(rel) + "/"
        found.append(base + "/" + tail)
    # Sorted by URL rather than by path, so the root sorts first — it is a
    # prefix of every other line — and the file reads top-down.
    return sorted(found)


def sitemap(directory, base=BASE):
    """The XML text of the sitemap for `directory`.

    `<loc>` and nothing else. `<lastmod>` would need a date, and the only
    dates available are a file mtime, which a fresh clone resets, and a date a
    person types, which is what this module exists to refuse.
    """
    lines = [
        '<?xml version="1.0" encoding="UTF-8"?>',
        '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">',
    ]
    for url in urls(directory, base):
        lines.append("  <url>")
        lines.append("    <loc>%s</loc>" % url)
        lines.append("  </url>")
    lines.append("</urlset>")
    return "\n".join(lines) + "\n"


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit("usage: python3 tools/sitemap.py <directory>")
    target = pathlib.Path(sys.argv[1])
    if not target.is_dir():
        sys.exit("sitemap.py: not a directory: %s" % target)
    text = sitemap(target)
    if text.count("<loc>") == 0:
        sys.exit("sitemap.py: no `index.html` under %s, so the sitemap would "
                 "tell a crawler this site has no pages" % target)
    sys.stdout.write(text)
