#!/usr/bin/env python3
"""check-site-redirects.py — hold `site/_redirects` against the pages it
redirects to, and hold a renamed document against the URL it left behind.

WHY THIS EXISTS

  A document that moves to a process shelf (HW-PD-0024), or whose title and so
  its slug changes, keeps its identifier and loses its URL. Google keeps the old
  URL and reports it as "Not found (404)" in Search Console until something
  answers it. 44 such URLs were reported on 2026-09-22, and nothing in the build
  said a URL had been lost, because a build reads the pages that exist and a lost
  URL is a page that does not.

  `site/_redirects` is the answer, and it is served by Cloudflare Workers static
  assets from the root of the assembled directory (HW-DR-0047). A redirect file
  is only worth having while it is true, so this holds it.

WHAT IT CHECKS ON THE FILE

  1. Every rule has a source and a destination that start with `/` (or a
     destination on another origin), and a permanent status, 301 or 308. A rule
     with no status is a 302, which tells a crawler the old URL is still the
     right one.
  2. No source is listed twice.
  3. No source is also a served page. The host would redirect a live page away.
  4. Every internal destination is served, directly or through one more rule.
     A destination that is itself a source is a chain, and is refused, because
     the first rule should name the last page. A loop is refused.
  5. The file holds at most 2000 static and 100 dynamic rules, the limits of
     the host.

WHAT IT CHECKS ON A CHANGE (`--base REF`)

  A document renamed or moved under `docs/` between REF and HEAD, whose served
  URL changed, must leave that old URL answered: still served, or a source in
  the file. This is the check that would have caught the move of the process
  shelves and the retitle of HW-OBL-0125. A document deleted outright is
  reported and does not fail, because a 404 is the right answer for a page that
  has no successor. A document `exclude_docs` removes from the site was never
  served, so its rename costs nothing.

  CI passes the merge base with main. On a push to main that is HEAD, the
  change is empty, and the pass reads nothing, which it says.

THE ASSUMPTION IT STATES, AND FAILS ON

  That there is a served directory with pages in it. An empty or missing root
  exits 2 rather than passing over nothing.

USAGE

  python3 tools/site/check-site-redirects.py [ROOT] [--base REF] [--repo DIR] [--quiet]

  ROOT defaults to `.headwater/site-deploy`. `--repo` names the repository `--base`
  is read in, this one by default; the fixtures point it at a scratch repository. Exit 0 when every test holds, 1
  with each finding on standard error, 2 when the input cannot be read.
"""

import fnmatch
import pathlib
import subprocess
import sys
import urllib.parse

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
from sitemap import BASE  # noqa: E402
from importlib import util as _util  # noqa: E402

_spec = _util.spec_from_file_location("check_site_links", HERE / "check-site-links.py")
links = _util.module_from_spec(_spec)
_spec.loader.exec_module(links)

ROOT = HERE.parent.parent
DEFAULT_ROOT = ROOT / ".headwater" / "site-deploy"
PERMANENT = {"301", "308"}
STATIC_LIMIT = 2000
DYNAMIC_LIMIT = 100

# `exclude_docs` of mkdocs.yml, read as the globs it states. A document these
# remove is never served, so moving one loses no URL.
EXCLUDED = ["taxonomies/*/fixtures/*", "taxonomies/*/templates/*", "taxonomies/*/bundle.yml",
            "doctrine/*", "w3id/*", "LICENSE"]


def exclusions(repo):
    """The `exclude_docs` globs of the repository's `mkdocs.yml`, or the copy above."""
    path = repo / "mkdocs.yml"
    out, inside = [], False
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except OSError:
        return EXCLUDED
    for line in lines:
        if line.startswith("exclude_docs:"):
            inside = True
            continue
        if inside:
            if line.startswith("  ") and line.strip():
                glob = line.strip()
                out.append(glob + "*" if glob.endswith("/") else glob)
            else:
                break
    return out or EXCLUDED


def doc_url(path):
    """The URL path MkDocs serves `docs/<path>.md` at, or None if it is no page."""
    if not path.startswith("docs/") or not path.endswith(".md"):
        return None
    rel = path[len("docs/"):-len(".md")]
    if rel in ("README", "index"):
        return "/"
    for tail in ("/README", "/index"):
        if rel.endswith(tail):
            return "/" + rel[: -len(tail)] + "/"
    return "/" + rel + "/"


def changed(base, repo):
    """([(old, new)] renames, [path] deletions) of `docs/` since BASE, or None."""
    try:
        out = subprocess.run(
            ["git", "diff", "-M", "--name-status", base, "--", "docs"],
            cwd=repo, capture_output=True, text=True, check=True).stdout
    except (OSError, subprocess.CalledProcessError) as err:
        sys.stderr.write("check-site-redirects: cannot read the change since %s: %s\n"
                         % (base, getattr(err, "stderr", err)))
        return None
    renames, deletions = [], []
    for line in out.splitlines():
        parts = line.split("\t")
        if parts[0].startswith("R") and len(parts) == 3:
            renames.append((parts[1], parts[2]))
        elif parts[0] == "D" and len(parts) == 2:
            deletions.append(parts[1])
    return renames, deletions


def main(argv):
    root = DEFAULT_ROOT
    base = None
    repo = ROOT
    quiet = False
    args = list(argv)
    positional = []
    while args:
        arg = args.pop(0)
        if arg == "--base" and args:
            base = args.pop(0)
        elif arg == "--repo" and args:
            repo = pathlib.Path(args.pop(0))
        elif arg == "--quiet":
            quiet = True
        elif arg.startswith("-"):
            sys.stderr.write("usage: python3 tools/site/check-site-redirects.py "
                             "[ROOT] [--base REF] [--repo DIR] [--quiet]\n")
            return 2
        else:
            positional.append(arg)
    if positional:
        root = pathlib.Path(positional[0])
    if not root.is_dir() or not list(root.glob("**/index.html")):
        sys.stderr.write("check-site-redirects: no served pages under %s, so nothing was "
                         "checked\n" % root)
        return 2

    site = links.Site(root, BASE)
    findings = []
    seen = {}
    static = dynamic = 0

    for source, dest, status in site.rules:
        is_dynamic = "*" in source or ":" in source
        static, dynamic = (static, dynamic + 1) if is_dynamic else (static + 1, dynamic)
        if not source.startswith("/"):
            findings.append("`%s %s`: the source must start with `/`" % (source, dest))
            continue
        split = urllib.parse.urlsplit(dest)
        external = bool(split.netloc) and (split.scheme + "://" + split.netloc) != BASE
        if not external and not (dest.startswith("/") or split.netloc):
            findings.append("`%s`: the destination `%s` must start with `/` or name an origin"
                            % (source, dest))
            continue
        if status not in PERMANENT:
            findings.append("`%s`: status %s; a moved page needs 301 or 308, because a 302 "
                            "tells a crawler to keep the old URL"
                            % (source, repr(status) if status else "is missing, which is a 302"))
        if source in seen:
            findings.append("`%s` is the source of two rules (lines %d and %d of the file)"
                            % (source, seen[source], len(seen) + 1))
        seen[source] = len(seen) + 1
        if is_dynamic or external:
            continue
        if site.served(source):
            findings.append("`%s` is a redirect source and a served page; the host would "
                            "redirect a live page away" % source)
        path = split.path or "/"
        if path in site.static or path.rstrip("/") in site.static:
            target = site.static.get(path) or site.static.get(path.rstrip("/"))
            findings.append("`%s` redirects to `%s`, which is itself redirected to `%s`; name "
                            "the last page in the first rule" % (source, path, target))
            continue
        if not site.served(path):
            findings.append("`%s` redirects to `%s`, which no served file answers"
                            % (source, path))
    if static > STATIC_LIMIT:
        findings.append("%d static rules; the host allows %d" % (static, STATIC_LIMIT))
    if dynamic > DYNAMIC_LIMIT:
        findings.append("%d dynamic rules; the host allows %d" % (dynamic, DYNAMIC_LIMIT))

    renamed = deleted = unanswered = 0
    note = "no `--base`, so no rename was read"
    if base:
        result = changed(base, repo)
        if result is None:
            return 2
        renames, deletions = result
        excl = exclusions(repo)

        def built(path):
            rel = path[len("docs/"):]
            return not any(fnmatch.fnmatch(rel, glob) for glob in excl)

        for old, new in renames:
            old_url, new_url = doc_url(old), doc_url(new)
            if old_url is None or new_url is None or old_url == new_url or not built(old):
                continue
            renamed += 1
            if site.served(old_url):
                continue
            if site.redirected(old_url) is None:
                unanswered += 1
                findings.append("`%s` was renamed to `%s` and its old URL %s answers nothing; "
                                "add `%s %s 301` to `site/_redirects`"
                                % (old, new, old_url, old_url, new_url))
        for path in deletions:
            url = doc_url(path)
            if url is not None and built(path) and not site.served(url) \
                    and site.redirected(url) is None:
                deleted += 1
                sys.stderr.write("check-site-redirects: note: `%s` was deleted and %s is a 404; "
                                 "redirect it if the page has a successor\n" % (path, url))
        note = "%d renamed document%s read since %s, %d deleted without a redirect" % (
            renamed, "" if renamed == 1 else "s", base[:12], deleted)

    if not quiet:
        print("%d static and %d dynamic redirect rule%s read; %s"
              % (static, dynamic, "" if static + dynamic == 1 else "s", note))
    for finding in findings:
        sys.stderr.write("check-site-redirects: %s\n" % finding)
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
