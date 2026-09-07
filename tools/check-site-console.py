#!/usr/bin/env python3
"""Load every served page in a real browser and fail on anything it logs.

WHY A BROWSER, WHEN THREE OTHER READERS ALREADY WALK THIS DIRECTORY

`mkdocs build --strict` reads source documents. `tools/check-site-fragments.py`
parses the served bytes. Neither of them runs a line of JavaScript, and #532 is
what lives in that gap: on 13 of the served pages the theme's own `js/base.js`
threw twice on load, every specification part a visitor reads first was
affected, and every instrument this repository had over the site reported green
for as long as the defect stood. A page can be well-formed, well-linked and
correctly titled and still be broken for the person reading it.

So this reader is the browser. It serves the assembled directory on a loopback
port and loads each page with headless Chrome.

WHAT THIS ASSERTS PER PAGE, AND WHY IT IS THREE THINGS AND NOT ONE

An absence-of-errors check is the assertion that goes quiet the moment its own
mechanism breaks. A wrong URL, a server that never came up, a browser that
failed to launch and a capture flag Chrome renames all produce the same empty
console that a healthy page produces, and the check reports a pass over every
one of them. That failure was made on purpose while this issue was being
adjudicated: a three-page run reported three pages clean and two of them were
HTTP 404s.

Each page therefore has to clear three bars, and only the third is the one the
issue is about:

  1. The server answered 200 for it. A 404 body is a page too, and an empty
     console over one proves nothing.
  2. The probe ran. This tool injects one script immediately before `</body>`
     of every page it serves, and that script writes two attributes onto the
     root element. Their absence from the dumped DOM means the pipeline broke
     somewhere between the socket and the parser, whatever the console says.
  3. The page recorded no uncaught error, and — on a page that loads the
     theme's `js/base.js` — `keyCodes` was defined by the time the probe ran.
     That is the positive marker for #532 specifically: the binding is hoisted,
     so it exists and holds `undefined` on a page where evaluation aborted, and
     the probe reads which of the two happened.

The two injected scripts are the only bytes this tool adds to a page, they are
added at serve time and never on disk, and they read two globals without
touching one.

WHAT THE GATE FAILS ON, AND WHAT IT ONLY REPORTS

The fatal arm is what the *page* recorded: an `error` event or an
`unhandledrejection`, captured by a listener this tool installs in `<head>`
ahead of every script the page loads. That is a property of the document that
produced it, it is the same list on every run, and it is exactly the population
#532 is about.

Chrome's stderr is the second arm and it is advisory. It is read, printed and
counted on every run, and `--strict-console` makes it fatal. It is not fatal by
default because a record on it carries no attribution to the page whose load
was being captured: the MkDocs search worker logs `All search scripts loaded,
building Lunr index...` once per session from `search/worker.js`, and whichever
of the 311 concurrent loads happened to be capturing wore the failure. Three
runs reddened three different pages, one each, over a corpus that had not
moved.

**State plainly what stopped being fatal.** A bare `console.error(...)` or
`console.warn(...)` from page script no longer reddens a run on its own; it is
printed under `reported, not fatal` and counted. An uncaught exception still
does, by the first arm, and so does an unhandled promise rejection — which the
old stderr arm never separated out at all. `--strict-console` restores the old
severity for a run that wants it.

WHAT DOES NOT COUNT, AND WHY EACH ONE IS NAMED RATHER THAN FILTERED

These apply to the advisory arm, and they decide what is printed as a record
rather than what fails. A record whose
`source:` is a `chrome://` page came from the browser's own interface — its
omnibox announces a slow network on whichever page happened to be loading — so
it is not about the corpus under any reading, and it is scoped out rather than
allowed. A record this tool has read and decided not to fail on is an entry in
`ALLOWANCES` below, carrying its reason in the source the way an `allow=`
directive does. Today there is one such entry, for the CDN highlighter's
unknown-language warning, which HW-OBL-0160 owns. Every run prints how many
records each of the two absorbed, so neither goes quiet, and `--no-allowances`
runs with the second one off.

WHAT IT WRITES

Nothing inside the checkout. One temporary browser profile per worker, removed
on the way out.

USAGE

    python3 tools/check-site-console.py [ROOT] [--chrome PATH] [--jobs N]
                                       [--strict-console] [--no-allowances]

ROOT defaults to `.headwater/site-deploy`, which `tools/assemble-site.sh`
writes. `HEADWATER_CHROME` names a browser if `--chrome` does not.

EXIT STATUS

    0  every page loaded clean
    1  at least one page failed one of the three bars
    2  an assumption is unmet — no such directory, no page under it, an
       argument this tool does not know
    3  no browser to run, so nothing was checked. This is a skip and it is
       loud: it is a separate status precisely so that a caller cannot read it
       as a pass, and `.github/workflows/ci.yml` turns it into a visible
       annotation rather than a silent green step.
"""

import base64
import binascii
import functools
import http.server
import io
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import threading
import urllib.error
import urllib.request
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

DEFAULT_ROOT = Path(".headwater/site-deploy")

# The browsers this looks for, in order. `chromium` covers a Debian or Fedora
# host; `google-chrome` is what this project's self-hosted runner carries.
BROWSERS = (
    "google-chrome",
    "google-chrome-stable",
    "chromium",
    "chromium-browser",
)

# Chrome writes an uncaught exception to stderr as an `INFO:CONSOLE:<line>`
# record under `--enable-logging=stderr --v=1`. The severity is matched loosely
# on purpose: a future release that raises it to `ERROR` must not silence this.
CONSOLE_LINE = re.compile(r":CONSOLE:\d+\]")

# `... "MESSAGE", source: URL (LINE)`. The source is what says whether a record
# is about the page at all.
CONSOLE_SOURCE = re.compile(r", source: (\S+) \(\d+\)\s*$")

# A record whose source is one of these came from a page of the browser's own
# interface, not from the page under test. Chrome's omnibox popup announces a
# slow network and a font fallback on this loopback server, on whichever page
# happened to be loading when it did, so counting one as a finding would fail a
# page at random and pass the same page on a retry. This is a scoping rule and
# not an allowance: `--no-allowances` does not lift it, because there is no
# reading of these records under which they are about the corpus.
BROWSER_SCHEMES = ("chrome://", "chrome-extension://", "devtools://")

# A record this tool has read, named, and decided not to fail on. Each entry
# carries the reason in the source, the way an `allow=` directive does, and the
# report counts what each one absorbed on every run. `--no-allowances` runs with
# the list off, which is how the population behind it stays visible.
ALLOWANCES = (
    (
        re.compile(
            r'"WARN: (Could not find the language|'
            r'Falling back to no-highlight mode)'
        ),
        "highlight.js is loaded from a CDN bundle carrying a subset of "
        "languages, and this corpus fences `turtle` and `abnf` among others. "
        "The highlighter is HW-OBL-0160's, and #532 is about an exception the "
        "theme's own script throws.",
    ),
)

# THE FATAL ARM, AND WHY IT IS THE PAGE AND NOT THE BROWSER'S LOG
#
# This goes into `<head>`, before any script the page loads, and it records
# every uncaught error and unhandled rejection the *document* produces, writing
# the list back onto the root element as it goes. It is written on every error
# rather than once at the end, because the second of #532's two exceptions
# arrives from a `DOMContentLoaded` handler, which is after any script that
# sits in the body has run.
#
# It replaces Chrome's stderr as the thing this gate fails on, and the reason
# is measured rather than stylistic. A record on that stderr carries no
# attribution to the page whose load was being captured: the MkDocs search
# worker logs `All search scripts loaded, building Lunr index...` once, from
# `search/worker.js`, and whichever of the 311 concurrent loads happened to be
# capturing at that moment wore it. Three runs, three different pages, one
# failure each — a verdict that moves while the corpus stands still. A required
# check that fails at random is read as noise within a day, which is the same
# family of defect as the one this whole tool exists to catch.
#
# An `error` event is a property of the document that produced it. It cannot be
# attributed to another page, it cannot arrive from a worker's `console.log`,
# and it is exactly the population #532 is about.
HEAD_PROBE = b"""<script>
(function () {
    var seen = [];
    function record(entry) {
        seen.push(entry);
        try {
            document.documentElement.setAttribute(
                'data-headwater-errors',
                btoa(unescape(encodeURIComponent(JSON.stringify(seen)))));
        } catch (ignored) {
            document.documentElement.setAttribute(
                'data-headwater-errors-broken', String(seen.length));
        }
    }
    window.addEventListener('error', function (event) {
        record({
            kind: 'uncaught',
            message: String(event.message || (event.error && event.error.message) || event.type),
            source: String(event.filename || ''),
            line: event.lineno || 0
        });
    });
    window.addEventListener('unhandledrejection', function (event) {
        record({
            kind: 'unhandled-rejection',
            message: String((event.reason && event.reason.message) || event.reason),
            source: '',
            line: 0
        });
    });
}());
</script>
"""

PROBE = b"""<script>
(function () {
    var root = document.documentElement;
    root.setAttribute('data-headwater-probe', 'ran');
    root.setAttribute(
        'data-headwater-keycodes',
        (typeof keyCodes === 'undefined') ? 'undefined' : 'defined');
}());
</script>
"""

ERRORS_ATTRIBUTE = re.compile(r'data-headwater-errors="([A-Za-z0-9+/=]*)"')

# A page that names this loads the theme script #532 is about, and only such a
# page owes the `keyCodes` marker. The hand-built half of the site loads no
# script at all and is held to the other two bars.
BASE_JS = b"js/base.js"


class Report(Exception):
    """An assumption this tool states, and stops on rather than reporting zero."""

    def __init__(self, lines, status=2):
        super().__init__(lines[0])
        self.lines = lines
        self.status = status


def inject(body):
    """Put the error listener first and the marker probe last.

    The listener has to precede every script the page loads, or an exception
    thrown before it installs would go unrecorded. The marker probe has to
    follow them, because what it reads is what they defined.
    """
    lowered = body.lower()
    head = lowered.find(b"<head>")
    if head != -1:
        cut = head + len(b"<head>")
        body = body[:cut] + HEAD_PROBE + body[cut:]
    else:
        body = HEAD_PROBE + body
    marker = body.lower().rfind(b"</body>")
    if marker == -1:
        return body + PROBE
    return body[:marker] + PROBE + body[marker:]


def page_errors(dom):
    """The uncaught errors the page itself recorded, from the dumped DOM."""
    found = ERRORS_ATTRIBUTE.search(dom)
    if not found or not found.group(1):
        return []
    try:
        raw = base64.b64decode(found.group(1)).decode("utf-8", "replace")
        entries = json.loads(raw)
    except (ValueError, binascii.Error):
        return [{"kind": "unreadable", "message": found.group(1)[:120], "source": "", "line": 0}]
    return entries if isinstance(entries, list) else []


class ProbeHandler(http.server.SimpleHTTPRequestHandler):
    """A static file server that injects the probe into every HTML response."""

    def log_message(self, fmt, *args):
        return

    def handle_one_request(self):
        # The browser closes the socket the moment it has what it needs, and
        # the default handler prints a `BrokenPipeError` traceback when it
        # does. A traceback on stderr from the harness reads as a defect in the
        # page.
        try:
            super().handle_one_request()
        except (BrokenPipeError, ConnectionResetError):
            self.close_connection = True

    def send_head(self):
        path = self.translate_path(self.path)
        if os.path.isdir(path):
            if not self.path.endswith("/"):
                return super().send_head()
            path = os.path.join(path, "index.html")
        if not path.endswith(".html") or not os.path.isfile(path):
            return super().send_head()
        with open(path, "rb") as handle:
            body = inject(handle.read())
        self.send_response(200)
        self.send_header("Content-Type", "text/html; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        return io.BytesIO(body)


class Server(http.server.ThreadingHTTPServer):
    daemon_threads = True
    allow_reuse_address = True


def serve(root):
    handler = functools.partial(ProbeHandler, directory=str(root))
    server = Server(("127.0.0.1", 0), handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    return server


def url_for(root, page):
    relative = page.relative_to(root).as_posix()
    if relative == "index.html":
        return ""
    if relative.endswith("/index.html"):
        return relative[: -len("index.html")]
    return relative


PROFILES = threading.local()


def profile_dir(parent):
    """One browser profile per worker thread, reused across that worker's pages."""
    existing = getattr(PROFILES, "path", None)
    if existing is None:
        existing = tempfile.mkdtemp(dir=parent, prefix="profile-")
        PROFILES.path = existing
    return existing


def load(browser, url, profile, timeout):
    argv = [
        browser,
        "--headless=new",
        "--disable-gpu",
        "--no-sandbox",
        "--disable-dev-shm-usage",
        "--enable-logging=stderr",
        "--v=1",
        "--virtual-time-budget=8000",
        "--user-data-dir=" + profile,
        "--dump-dom",
        url,
    ]
    return subprocess.run(
        argv,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=timeout,
        check=False,
    )


def classify(record, allowances):
    """`finding`, `allowed:<reason>` or `noise` for one console record."""
    match = CONSOLE_SOURCE.search(record)
    origin = match.group(1) if match else ""
    if origin.startswith(BROWSER_SCHEMES):
        return "noise", None
    if allowances:
        for pattern, reason in ALLOWANCES:
            if pattern.search(record):
                return "allowed", reason
    return "finding", None


def check_page(root, page, base, browser, profiles, timeout, allowances, strict_console):
    """Return (path, findings, loads_base, allowed, noise, reported)."""
    relative = page.relative_to(root).as_posix()
    source = page.read_bytes()
    loads_base = BASE_JS in source
    findings = []
    allowed = []
    noise = 0
    reported = []
    url = base + url_for(root, page)

    try:
        with urllib.request.urlopen(url, timeout=timeout) as response:
            status = response.status
            served = len(response.read())
    except urllib.error.HTTPError as error:
        status, served = error.code, 0
    except OSError as error:
        findings.append("the server did not answer for it: %s" % error)
        return relative, findings, loads_base, allowed, noise, reported

    if status != 200:
        findings.append("the server answered %d for it, not 200" % status)
    if served == 0:
        findings.append("the server answered with an empty body")

    try:
        result = load(browser, url, profile_dir(profiles), timeout)
    except subprocess.TimeoutExpired:
        findings.append("the browser did not finish in %ds" % timeout)
        return relative, findings, loads_base, allowed, noise, reported

    dom = result.stdout.decode("utf-8", "replace")
    log = result.stderr.decode("utf-8", "replace")

    if result.returncode != 0:
        findings.append("the browser exited %d on it" % result.returncode)
    if 'data-headwater-probe="ran"' not in dom:
        findings.append(
            "the probe this tool injects never ran, so nothing here read the page"
        )
    elif loads_base and 'data-headwater-keycodes="defined"' not in dom:
        findings.append(
            "`js/base.js` did not run to the end: `keyCodes` was still undefined"
        )

    for entry in page_errors(dom):
        findings.append(
            "the page threw and nothing caught it: %s (%s:%s)"
            % (entry.get("message"), entry.get("source") or "inline", entry.get("line"))
        )

    for line in log.splitlines():
        if not CONSOLE_LINE.search(line):
            continue
        record = line.strip()
        verdict, reason = classify(record, allowances)
        if verdict == "noise":
            noise += 1
        elif verdict == "allowed":
            allowed.append(reason)
        elif strict_console:
            findings.append("the console said: " + record)
        else:
            reported.append("the console said: " + record)

    return relative, findings, loads_base, allowed, noise, reported


def parse(argv):
    root = None
    browser = os.environ.get("HEADWATER_CHROME") or None
    jobs = min(8, (os.cpu_count() or 2))
    timeout = 90
    allowances = True
    strict_console = False
    rest = list(argv)
    while rest:
        item = rest.pop(0)
        if item == "--chrome":
            browser = rest.pop(0) if rest else None
        elif item == "--jobs":
            jobs = int(rest.pop(0)) if rest else jobs
        elif item == "--timeout":
            timeout = int(rest.pop(0)) if rest else timeout
        elif item == "--no-allowances":
            allowances = False
        elif item == "--strict-console":
            strict_console = True
        elif item.startswith("--"):
            raise Report(
                [
                    "unknown argument `%s`." % item,
                    "usage: check-site-console.py [ROOT] [--chrome PATH] "
                    "[--jobs N] [--timeout S] [--no-allowances] [--strict-console]",
                ]
            )
        elif root is None:
            root = Path(item)
        else:
            raise Report(["more than one root named: `%s` and `%s`." % (root, item)])
    return (root or DEFAULT_ROOT), browser, max(1, jobs), timeout, allowances, strict_console


def resolve_browser(named):
    if named:
        found = shutil.which(named) or (named if os.path.isfile(named) else None)
        if found:
            return found
        raise Report(
            ["no browser at `%s`, which `--chrome` or `HEADWATER_CHROME` named." % named],
            status=3,
        )
    for candidate in BROWSERS:
        found = shutil.which(candidate)
        if found:
            return found
    raise Report(
        [
            "SKIPPED: no browser on this host, so no page was loaded and this",
            "  run checked nothing. Looked for: %s." % ", ".join(BROWSERS),
            "  Name one with `--chrome PATH` or `HEADWATER_CHROME`.",
        ],
        status=3,
    )


def collect(root):
    if not root.is_dir():
        raise Report(
            [
                "no directory at `%s`." % root,
                "  `sh tools/assemble-site.sh` writes the one this reads, after",
                "  `python3 -m mkdocs build --strict`.",
            ]
        )
    pages = sorted(root.rglob("*.html"))
    if not pages:
        raise Report(
            [
                "no `.html` page under `%s`, so this run checked nothing." % root,
                "  An empty directory is not a pass, and this status says so.",
            ]
        )
    return pages


def run(argv):
    root, named, jobs, timeout, allowances, strict_console = parse(argv)
    browser = resolve_browser(named)
    pages = collect(root)

    profiles = tempfile.mkdtemp(prefix="headwater-console-")
    try:
        server = serve(root)
        base = "http://127.0.0.1:%d/" % server.server_address[1]
        work = functools.partial(
            check_page,
            root,
            base=base,
            browser=browser,
            profiles=profiles,
            timeout=timeout,
            allowances=allowances,
            strict_console=strict_console,
        )
        try:
            with ThreadPoolExecutor(max_workers=jobs) as pool:
                results = list(pool.map(work, pages))
        finally:
            server.shutdown()
            server.server_close()
    finally:
        shutil.rmtree(profiles, ignore_errors=True)

    failed = [(name, f) for name, f, _, _, _, _ in results if f]
    with_base = sum(1 for _, _, loads, _, _, _ in results if loads)
    allowed = [r for _, _, _, rs, _, _ in results for r in rs]
    allowed_pages = sum(1 for _, _, _, rs, _, _ in results if rs)
    noise = sum(c for _, _, _, _, c, _ in results)
    reported = [(name, r) for name, _, _, _, _, rs in results for r in rs]

    for name, findings in failed:
        for finding in findings:
            print("%s: %s" % (name, finding))
        print("")

    for name, record in reported:
        print("reported, not fatal — %s: %s" % (name, record))
    if reported:
        print("")

    print(
        "%d page%s with a finding, out of %d served pages loaded in %s"
        % (
            len(failed),
            "" if len(failed) == 1 else "s",
            len(results),
            os.path.basename(browser),
        )
    )
    print(
        "%d of those pages load the theme's `js/base.js` and were held to "
        "`keyCodes` being defined" % with_base
    )
    print(
        "%d console record%s reported and not fatal%s"
        % (
            len(reported),
            "" if len(reported) == 1 else "s",
            "" if strict_console else " (`--strict-console` makes them fatal)",
        )
    )
    if allowances:
        print(
            "%d console record%s on %d page%s allowed by name, and %d from the "
            "browser's own interface"
            % (
                len(allowed),
                "" if len(allowed) == 1 else "s",
                allowed_pages,
                "" if allowed_pages == 1 else "s",
                noise,
            )
        )
        for reason in sorted(set(allowed)):
            print("  allowed because " + reason)
    else:
        print(
            "no allowance was applied (`--no-allowances`), and %d record%s from "
            "the browser's own interface still did not count"
            % (noise, "" if noise == 1 else "s")
        )
    return 1 if failed else 0


def main(argv):
    try:
        return run(argv)
    except Report as report:
        for line in report.lines:
            print(line, file=sys.stderr)
        return report.status


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
