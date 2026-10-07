#!/usr/bin/env python3
"""check-site-links.py — read the served bytes and fail on every link to a path
the site does not serve.

WHY THIS EXISTS

  On 2026-09-22 Search Console reported 44 "Not found (404)" pages on
  headwater.tools, and `mkdocs build --strict` had exited 0 throughout. Three
  readers each missed the same population:

    `validation.links.not_found` was `info`, which `--strict` does not fail on,
    and MkDocs never resolves a root-absolute link at all.

    `tools/site/check-site-fragments.py` resolves only a link that carries a
    `#fragment`. A dead path with no fragment is counted by it and, without
    `--strict-paths`, not failed.

    Nothing read a link between a served page and the repository files it
    cites, or between a page that moved and the URL it left behind.

  This is the reader of the served directory for the whole population: every
  `href` and `src` of every served page, with or without a fragment.

WHY THIS READS THE SERVED DIRECTORY AND NOT THE SOURCE

  A checker of Markdown would share the assumption that produced the defect,
  that a relative path which resolves in a clone resolves on the site. This one
  asserts nothing about how a page was rendered. It reads the bytes a visitor's
  browser receives and asks whether each link lands on a file in the served
  root, the way Cloudflare Workers static assets would answer it.

WHAT IT CHECKS

  1. Every internal link resolves. An internal link is a relative path, a
     root-absolute path, or an absolute URL on this site's origin. It resolves
     when it names a served file, a directory that holds an `index.html`
     (with or without the trailing slash, which the host redirects), or a
     source in the `_redirects` file that sits in the served root. A link to a
     redirect source is not a failure, but its destination must itself
     resolve.
  2. No internal link is written with `http://`. The host redirects it, and
     Search Console reports the `http` URL as a duplicate.
  3. Every `<loc>` of `sitemap.xml` names a served page, not a redirect, and
     is written with `https://`.

WHAT IT DOES NOT CHECK

  A fragment, which `check-site-fragments.py` owns. A link to another origin.
  A link whose scheme is not http or https (`mailto:`, `data:`, `tel:`).

THE ASSUMPTION IT STATES, AND FAILS ON

  That there is a served directory with pages in it. An empty or missing root
  exits 2 rather than printing "0 dead links", which is the shape every other
  site gate uses: a suite that cannot tell "nothing is wrong" from "nothing
  ran" is not a gate.

USAGE

  python3 tools/site/check-site-links.py [ROOT] [--origin URL] [--quiet]

  ROOT defaults to `.headwater/site-deploy`. `--origin` names the origin of an
  absolute link that counts as internal, `https://headwater.tools` by default.
  Exit 0 when every test holds, 1 with each finding on standard error, 2 when
  the input cannot be read.
"""

import collections
import html.parser
import pathlib
import posixpath
import re
import sys
import urllib.parse

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
from sitemap import BASE  # noqa: E402

ROOT = HERE.parent.parent
DEFAULT_ROOT = ROOT / ".headwater" / "site-deploy"

# Attributes that name a resource of this site, by element.
LINK_ATTRS = {
    "a": ("href",),
    "area": ("href",),
    "link": ("href",),
    "img": ("src",),
    "script": ("src",),
    "source": ("src",),
    "iframe": ("src",),
    "audio": ("src",),
    "video": ("src", "poster"),
}


class Links(html.parser.HTMLParser):
    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.found = []

    def handle_starttag(self, tag, attrs):
        wanted = LINK_ATTRS.get(tag)
        if not wanted:
            return
        for name, value in attrs:
            if name in wanted and value:
                self.found.append((self.getpos()[0], tag, value.strip()))


def read_redirects(root):
    """The `_redirects` of the served root as `[(source, destination, status)]`.

    Static rules only are resolved. A rule with a splat or a placeholder is kept
    as a pattern, which is enough to say that a link under it is answered.
    """
    path = root / "_redirects"
    rules = []
    if not path.is_file():
        return rules
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        parts = line.split()
        if len(parts) < 2:
            continue
        rules.append((parts[0], parts[1], parts[2] if len(parts) > 2 else ""))
    return rules


def pattern(source):
    """A compiled regex for a `_redirects` source with `*` and `:name` parts."""
    out, i = "", 0
    while i < len(source):
        c = source[i]
        if c == "*":
            out += "(.*)"
        elif c == ":" and re.match(r":[A-Za-z]", source[i:]):
            m = re.match(r":[A-Za-z0-9_]+", source[i:])
            out += "([^/]+)"
            i += len(m.group(0)) - 1
        else:
            out += re.escape(c)
        i += 1
    return re.compile("^" + out + "$")


class Site:
    def __init__(self, root, origin):
        self.root = root
        self.origin = origin
        self.rules = read_redirects(root)
        self.static = {s: d for s, d, _ in self.rules if "*" not in s and ":" not in s}
        self.dynamic = [(pattern(s), d) for s, d, _ in self.rules if "*" in s or ":" in s]

    def served(self, path):
        """True when the host answers 200 for this URL path, redirects aside."""
        rel = urllib.parse.unquote(path.lstrip("/"))
        target = self.root / rel
        try:
            if target.is_file():
                return True
            if target.is_dir():
                return (target / "index.html").is_file()
            # `/page` is answered by `page.html` as well as `page/index.html`.
            return (self.root / (rel + ".html")).is_file()
        except OSError:
            return False

    def redirected(self, path):
        """The destination a `_redirects` rule gives this path, or None."""
        for variants in (path, path.rstrip("/"), path.rstrip("/") + "/"):
            if variants in self.static:
                return self.static[variants]
        for rx, dest in self.dynamic:
            if rx.match(path):
                return dest
        return None

    def resolves(self, path, seen=()):
        """(True, '') when the path is answered, else (False, why)."""
        if self.served(path):
            return True, ""
        dest = self.redirected(path)
        if dest is None:
            return False, "no file, directory or redirect answers it"
        if path in seen:
            return False, "the redirect rules loop"
        split = urllib.parse.urlsplit(dest)
        if split.netloc and not (split.scheme + "://" + split.netloc).startswith(self.origin):
            return True, ""
        ok, why = self.resolves(split.path or "/", seen + (path,))
        if not ok:
            return False, "redirected to %s, which %s" % (dest, why[0].lower() + why[1:])
        return True, ""


def page_url(rel):
    """The URL path a served file `rel` (posix, under the root) answers."""
    if rel == "index.html":
        return "/"
    if rel.endswith("/index.html"):
        return "/" + rel[: -len("index.html")]
    return "/" + rel


def main(argv):
    root = DEFAULT_ROOT
    origin = BASE
    quiet = False
    args = list(argv)
    positional = []
    while args:
        arg = args.pop(0)
        if arg == "--origin" and args:
            origin = args.pop(0).rstrip("/")
        elif arg == "--quiet":
            quiet = True
        elif arg.startswith("-"):
            sys.stderr.write("usage: python3 tools/site/check-site-links.py "
                             "[ROOT] [--origin URL] [--quiet]\n")
            return 2
        else:
            positional.append(arg)
    if positional:
        root = pathlib.Path(positional[0])
    if not root.is_dir():
        sys.stderr.write("check-site-links: not a directory: %s\n" % root)
        return 2
    pages = sorted(root.glob("**/*.html"))
    if not pages:
        sys.stderr.write("check-site-links: no `.html` under %s, so nothing was checked\n" % root)
        sys.stderr.write("  Build it first: python3 -m mkdocs build --strict "
                         "&& sh tools/site/assemble-site.sh\n")
        return 2

    site = Site(root, origin)
    http_origin = "http://" + origin.split("://", 1)[1]
    dead = collections.defaultdict(list)   # url path -> [(page, line, href, why)]
    insecure = []
    checked = 0
    redirected = 0

    for page in pages:
        rel = page.relative_to(root).as_posix()
        base = page_url(rel)
        parser = Links()
        parser.feed(page.read_text(encoding="utf-8", errors="replace"))
        for line, _tag, href in parser.found:
            if href.startswith("#"):
                continue
            if href.startswith(http_origin + "/") or href == http_origin:
                insecure.append((rel, line, href))
                href = origin + href[len(http_origin):]
            split = urllib.parse.urlsplit(href)
            if split.scheme or split.netloc:
                if (split.scheme + "://" + split.netloc) != origin:
                    continue
                path = split.path or "/"
            else:
                joined = urllib.parse.urljoin("https://x" + base, href)
                path = urllib.parse.urlsplit(joined).path
            path = posixpath.normpath(path) + ("/" if path.endswith("/") and path != "/" else "")
            if not path.startswith("/"):
                path = "/" + path
            checked += 1
            ok, why = site.resolves(path)
            if ok and not site.served(path):
                redirected += 1
            if not ok:
                dead[path].append((rel, line, href, why))

    findings = []
    for path in sorted(dead):
        hits = dead[path]
        page, line, href, why = hits[0]
        extra = "" if len(hits) == 1 else " (and %d more page%s)" % (
            len(hits) - 1, "" if len(hits) == 2 else "s")
        findings.append("dead link %s: %s — %s:%d writes `%s`%s"
                        % (path, why, page, line, href, extra))
    for page, line, href in insecure:
        findings.append("%s:%d links to %s with `http://`; this site is https only"
                        % (page, line, href))

    # 3. The sitemap names served pages, written with https.
    sitemap = root / "sitemap.xml"
    locs = []
    if sitemap.is_file():
        locs = re.findall(r"<loc>([^<]+)</loc>", sitemap.read_text(encoding="utf-8"))
    for loc in locs:
        if loc.startswith(http_origin):
            findings.append("`sitemap.xml` lists %s with `http://`" % loc)
            continue
        if not loc.startswith(origin):
            continue
        path = urllib.parse.urlsplit(loc).path or "/"
        if not site.served(path):
            why = "redirected" if site.redirected(path) else "not served"
            findings.append("`sitemap.xml` lists %s, which is %s" % (loc, why))

    if not quiet:
        print("%d internal links on %d pages checked, %d answered by a redirect; "
              "%d dead path%s, %d `http://` link%s; %d sitemap URLs read"
              % (checked, len(pages), redirected,
                 len(dead), "" if len(dead) == 1 else "s",
                 len(insecure), "" if len(insecure) == 1 else "s", len(locs)))
    for finding in findings:
        sys.stderr.write("check-site-links: %s\n" % finding)
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
