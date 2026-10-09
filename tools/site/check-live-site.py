#!/usr/bin/env python3
"""check-live-site.py — ask the deployed site the questions the assembled
directory cannot answer.

WHY THIS EXISTS

  `check-site-links.py` and `check-site-redirects.py` read the directory
  `wrangler` uploads. They cannot say that the host serves what was uploaded,
  and three things about headwater.tools are the host's to do and nothing in the
  repository can observe before a deploy: that Cloudflare Workers static assets
  reads `site/_redirects` and answers each rule with the status written there,
  that `http://` is redirected to `https://`, and that a link on a deployed page
  lands on a page that deployed. Search Console found a lost URL weeks after it
  was lost, because nothing asked.

WHAT IT ASKS

  1. Every URL in the live `sitemap.xml` answers 200 without a redirect.
  2. Every internal link on those pages lands, within three redirects, on a 200.
  3. Every static rule of `_redirects` answers its status, 301 or 308, with a
     `Location` that names the destination the file writes.
  4. `http://` answers a permanent redirect to `https://` for the origin and for
     one page.

  `--settle SECONDS` first waits, at most that long, until the host answers the
  first static rule of `_redirects` with its status. `wrangler deploy` returns
  before every edge serves the new version, and a 404 is an answer and not a
  failure to retry, so a check that starts in the second the deploy ends reads
  the previous site (it did, on the first deploy that ran it). A host that never
  settles is not waited on past SECONDS: the questions are asked anyway and the
  findings say what the host answered. The same deadline covers a sitemap URL
  that answers 404, because a page the deploy added reaches an edge after the
  rule does: the URL is asked again every few seconds until it answers or the
  deadline passes, and a page that never appears is still a finding.

  A failing request is retried, `--retries` times a few seconds apart, because
  a deploy takes a moment to reach every edge. A finding that survives the
  retries is a finding.

  It is run by `deploy-site.yml` after `wrangler deploy`. The deploy has
  already happened, so a failure here is an alert and not a rollback: the job
  turns red and the log names each URL.

USAGE

  python3 tools/site/check-live-site.py [ORIGIN] [--redirects FILE]
      [--retries N] [--workers N] [--settle SECONDS] [--skip-http-check] [--quiet]

  ORIGIN defaults to `https://headwater.tools`. Exit 0 when every question is
  answered well, 1 with each finding on standard error, 2 when the sitemap
  cannot be read, because a run that asked nothing is not a pass.
"""

import concurrent.futures
import pathlib
import re
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from importlib import util as _util

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parent.parent

_spec = _util.spec_from_file_location("check_site_links", HERE / "check-site-links.py")
links = _util.module_from_spec(_spec)
_spec.loader.exec_module(links)


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, *args, **kwargs):
        return None


OPENER = urllib.request.build_opener(NoRedirect)
UA = "headwater-site-smoke/1 (+https://headwater.tools/)"
DELAY = 4


def fetch(url, retries, want_body=False):
    """(status, location, body) of one GET that follows no redirect. Status 0 on a
    transport failure. Retries a 5xx, a timeout and a transport failure."""
    status, location, body = 0, "", b""
    for attempt in range(retries + 1):
        request = urllib.request.Request(url, headers={"User-Agent": UA})
        try:
            with OPENER.open(request, timeout=20) as response:
                status = response.status
                location = response.headers.get("Location", "")
                body = response.read() if want_body else b""
        except urllib.error.HTTPError as err:
            status = err.code
            location = err.headers.get("Location", "") if err.headers else ""
        except (urllib.error.URLError, OSError, ValueError):
            status = 0
        if status and status < 500:
            break
        if attempt < retries:
            time.sleep(DELAY)
    return status, location, body


def land(url, retries, hops=3):
    """The final status after at most HOPS redirects, and the chain written."""
    chain = []
    for _ in range(hops + 1):
        status, location, _ = fetch(url, retries)
        if status in (301, 302, 303, 307, 308) and location:
            chain.append(status)
            url = urllib.parse.urljoin(url, location)
            continue
        return status, url
    return status, url


def main(argv):
    origin = "https://headwater.tools"
    redirects = ROOT / "site" / "_redirects"
    retries, workers = 3, 8
    settle = 0
    http_check = True
    quiet = False
    args = list(argv)
    while args:
        arg = args.pop(0)
        if arg == "--redirects" and args:
            redirects = pathlib.Path(args.pop(0))
        elif arg == "--retries" and args:
            retries = int(args.pop(0))
        elif arg == "--workers" and args:
            workers = int(args.pop(0))
        elif arg == "--settle" and args:
            settle = int(args.pop(0))
        elif arg == "--skip-http-check":
            http_check = False
        elif arg == "--quiet":
            quiet = True
        elif arg.startswith("-"):
            sys.stderr.write("usage: python3 tools/site/check-live-site.py [ORIGIN] "
                             "[--redirects FILE] [--retries N] [--workers N] "
                             "[--settle SECONDS] [--skip-http-check] [--quiet]\n")
            return 2
        else:
            origin = arg.rstrip("/")

    deadline = time.monotonic() + settle if settle else 0.0
    if settle and redirects.is_file():
        first = next(((p[0], p[2] if len(p) > 2 else "302")
                      for p in (l.split() for l in redirects.read_text(encoding="utf-8").splitlines()
                                if l.strip() and not l.lstrip().startswith("#"))
                      if len(p) >= 2 and "*" not in p[0] and ":" not in p[0]), None)
        while first and time.monotonic() < deadline:
            got, _, _ = fetch(origin + first[0], 0)
            if str(got) == first[1]:
                break
            time.sleep(DELAY)

    status, _, body = fetch(origin + "/sitemap.xml", retries, want_body=True)
    locs = re.findall(r"<loc>([^<]+)</loc>", body.decode("utf-8", "replace"))
    if status != 200 or not locs:
        sys.stderr.write("check-live-site: %s/sitemap.xml answered %s with %d URLs, so "
                         "nothing was checked\n" % (origin, status or "no response", len(locs)))
        return 2

    findings = []
    pool = concurrent.futures.ThreadPoolExecutor(max_workers=workers)

    # 1. Every sitemap URL answers 200, and 2. collect the links of each page.
    def page(url):
        got, location, html = fetch(url, retries, want_body=True)
        # A page this deploy added reaches an edge after the first `_redirects`
        # rule does, because that rule was already served by the version before.
        # So a 404 on a sitemap URL is polled until the same `--settle` deadline
        # (2026-10-09: five new campaign pages answered 404 seven seconds after
        # `wrangler deploy` and 200 when asked again about a minute later).
        while got == 404 and time.monotonic() < deadline:
            time.sleep(DELAY)
            got, location, html = fetch(url, retries, want_body=True)
        return url, got, location, html

    targets = {}
    on_site = set()
    for url, got, location, html in pool.map(page, locs):
        if got != 200:
            findings.append("sitemap URL %s answered %s%s"
                            % (url, got or "no response",
                               " to %s" % location if location else ""))
            continue
        parser = links.Links()
        parser.feed(html.decode("utf-8", "replace"))
        base = urllib.parse.urlsplit(url).path or "/"
        on_site.add(url)
        for _line, tag, href in parser.found:
            if href.startswith("#") or tag in ("link", "script", "img", "source"):
                continue
            split = urllib.parse.urlsplit(urllib.parse.urljoin(origin + base, href))
            if (split.scheme + "://" + split.netloc) != origin:
                continue
            target = origin + (split.path or "/")
            targets.setdefault(target, url)

    # 2. Every internal link lands on a 200.
    todo = [t for t in targets if t not in on_site]

    def check(target):
        return target, land(target, retries)

    for target, (got, final) in pool.map(check, todo):
        if got != 200:
            findings.append("%s links to %s, which answered %s%s"
                            % (targets[target], target, got or "no response",
                               "" if final == target else " after redirecting to %s" % final))

    # 3. Every static rule of `_redirects` is honoured with its own status.
    rules = []
    if redirects.is_file():
        for line in redirects.read_text(encoding="utf-8").splitlines():
            parts = line.split()
            if len(parts) >= 2 and not line.lstrip().startswith("#") \
                    and "*" not in parts[0] and ":" not in parts[0]:
                rules.append((parts[0], parts[1], parts[2] if len(parts) > 2 else "302"))

    def rule(item):
        source, dest, want = item
        got, location, _ = fetch(origin + source, retries)
        return item, got, location

    for (source, dest, want), got, location in pool.map(rule, rules):
        where = urllib.parse.urlsplit(urllib.parse.urljoin(origin + source, location)).path
        wanted = urllib.parse.urlsplit(dest).path or "/"
        if str(got) != want or (location and where != wanted):
            findings.append("redirect %s should answer %s to %s; the host answered %s%s"
                            % (source, want, dest, got or "no response",
                               " to %s" % location if location else ""))

    # 4. http:// is redirected to https://.
    if http_check:
        plain = "http://" + origin.split("://", 1)[1]
        for path in ("/", urllib.parse.urlsplit(locs[len(locs) // 2]).path):
            got, location, _ = fetch(plain + path, retries)
            if got not in (301, 308) or not location.startswith("https://"):
                findings.append("%s%s should answer a permanent redirect to https; the host "
                                "answered %s%s" % (plain, path, got or "no response",
                                                   " to %s" % location if location else ""))
    pool.shutdown()

    if not quiet:
        print("%d sitemap URLs, %d further link targets and %d redirect rules asked of %s; "
              "%d finding%s"
              % (len(locs), len(todo), len(rules), origin, len(findings),
                 "" if len(findings) == 1 else "s"))
    for finding in findings:
        sys.stderr.write("check-live-site: %s\n" % finding)
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
