#!/usr/bin/env python3
"""Read every served page in a headless browser the way a person would look at it.

WHAT THIS READS THAT NOTHING ELSE DOES

`tools/site/check-site-console.py` loads every page in headless Chrome and
fails on an uncaught error. It reads one window size, one color scheme and the
console. A page that is ten centimeters wider than a phone has an empty console
and passes it. So does a page whose dark theme hides its text, a link whose
focus ring was styled away, and a navigation bar that lost a section. This tool
is the reader for those, and #1489 is the issue that asked for it.

For each page it takes these readings, one browser load each:

  * at 1280x800 and at 390x844, in the light scheme, always;
  * in the dark scheme at both sizes, for a page that `--shoot` matches or
    under `--dark-all`;
  * a screenshot of the first screen of each reading, for a page that
    `--shoot` matches, into `--shots`.

On each load an injected probe writes four things onto the root element, and
`--dump-dom` returns them:

  1. Horizontal overflow: `document.documentElement.scrollWidth` against its
     `clientWidth`, and the outermost elements past the right edge. A page
     that scrolls sideways at phone width is flagged.
  2. The color scheme the page saw, from `matchMedia`. A dark reading that the
     page did not see as dark is flagged as a broken reading and never passed.
     Blink's `preferredColorScheme` setting is 0 for dark and 1 for light,
     measured on Chrome for Testing 153 on 2026-10-15. `--force-dark-mode` did
     nothing in the headless shell on the same day.
  3. Keyboard focus: the first focusable element is focused, and its computed
     `outline-style`, `outline-width` and `box-shadow` are read. No outline and
     no shadow is flagged. A focus that did not take is reported as unread,
     never as a pass.
  4. The navigation the reader sees, on a page that carries the generated
     site's `nav[aria-label="Shelves"]`: every link in it must resolve to a
     served page, and every top-level and second-level section of
     `.headwater/nav.yml` must appear in it by name.

WHAT IT DOES NOT JUDGE

Whether a page reads well, whether a color pair has enough contrast to a
person with low vision, and whether a screenshot looks right. Those are a
person's, and the screenshots are what that person reads.

WHERE THE PAGES COME FROM

A directory (default `site`), served on a loopback port as
`check-site-console.py` serves one. Or `--base URL`, which reads
`<URL>/sitemap.xml`, fetches each page from that host, and serves it on the
loopback port with a `<base href>` pointing back at the host, so that the
page's own styles and scripts load from where a reader gets them. The second
form is how the generated half is read on a host that cannot build it.

WHAT IT WRITES

Screenshots, into `--shots` only, and a JSON file of every reading under
`--json`. Nothing inside the checkout unless one of those names it. One
temporary browser profile per worker, removed on the way out.

USAGE

    python3 tools/site/review-site.py [ROOT | --base URL] [--chrome PATH]
        [--jobs N] [--timeout S] [--nav PATH] [--dark-all]
        [--shots DIR] [--shoot REGEX]... [--json FILE]

EXIT STATUS

    0  no page flagged
    1  at least one page flagged, or a reading that did not take
    2  an assumption is unmet: no such directory, no page under it, an
       argument this tool does not know
    3  no browser to run, so nothing was read. A caller must not read it as a
       pass, as for `check-site-console.py`.
"""

import functools
import glob
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
import urllib.parse
import urllib.request
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

DEFAULT_ROOT = Path("site")
DEFAULT_NAV = Path(".headwater/nav.yml")

SIZES = ((1280, 800), (390, 844))
SCHEMES = ("light", "dark")

# Blink's `preferredColorScheme`: 0 is dark and 1 is light. Measured, not read
# from documentation, because the numbering is the reverse of what the name
# suggests.
SCHEME_SETTING = {"dark": "0", "light": "1"}

BROWSERS = (
    "google-chrome",
    "google-chrome-stable",
    "chromium",
    "chromium-browser",
)

# The headless shell that Playwright installs. On a host with no system
# browser it is the only one, and `check-site-console.py` does not look here.
PLAYWRIGHT_SHELL = os.path.expanduser(
    "~/.cache/ms-playwright/chromium_headless_shell-*/"
    "chrome-headless-shell-linux64/chrome-headless-shell"
)

PROBE = b"""<script>
(function () {
    function describe(el) {
        var name = el.tagName.toLowerCase();
        if (el.id) { name += '#' + el.id; }
        if (typeof el.className === 'string' && el.className.trim()) {
            name += '.' + el.className.trim().split(/\\s+/).join('.');
        }
        return name;
    }
    function read() {
        var root = document.documentElement;
        var out = {
            probe: 'ran',
            dark: window.matchMedia('(prefers-color-scheme: dark)').matches,
            scrollWidth: root.scrollWidth,
            clientWidth: root.clientWidth,
            innerWidth: window.innerWidth,
            culprits: [],
            focus: null,
            nav: null
        };
        if (out.scrollWidth > out.clientWidth && document.body) {
            var edge = out.clientWidth + 1;
            var all = document.body.querySelectorAll('*');
            for (var i = 0; i < all.length && out.culprits.length < 3; i++) {
                var el = all[i];
                var right = el.getBoundingClientRect().right + window.scrollX;
                var parent = el.parentElement;
                var parentRight = parent ? parent.getBoundingClientRect().right + window.scrollX : 0;
                if (right > edge && (!parent || parent === document.body || parentRight <= edge)) {
                    out.culprits.push(describe(el) + ' to ' + Math.round(right));
                }
            }
        }
        var focusable = document.querySelector(
            'a[href], button, input, select, textarea, [tabindex]:not([tabindex="-1"])');
        if (focusable) {
            focusable.focus();
            var style = window.getComputedStyle(focusable);
            out.focus = {
                element: describe(focusable),
                took: document.activeElement === focusable,
                visible: focusable.matches(':focus-visible'),
                outlineStyle: style.outlineStyle,
                outlineWidth: style.outlineWidth,
                boxShadow: style.boxShadow
            };
            focusable.blur();
        }
        var shelves = document.querySelector('nav[aria-label="Shelves"]');
        if (shelves) {
            out.nav = {
                shown: shelves.getBoundingClientRect().height > 0,
                groups: Array.prototype.map.call(
                    shelves.querySelectorAll('.hw-grp'),
                    function (g) { return g.textContent.trim(); }),
                links: Array.prototype.map.call(
                    shelves.querySelectorAll('a[href]'),
                    function (a) { return [a.textContent.trim(), a.href]; })
            };
        }
        root.setAttribute('data-headwater-review',
            btoa(unescape(encodeURIComponent(JSON.stringify(out)))));
    }
    read();
    window.addEventListener('load', function () { setTimeout(read, 50); });
}());
</script>
"""

READING = re.compile(r'data-headwater-review="([A-Za-z0-9+/=]*)"')


class Report(Exception):
    """An assumption this tool states, and stops on rather than reporting zero."""

    def __init__(self, lines, status=2):
        super().__init__(lines[0])
        self.lines = lines
        self.status = status


def inject(body, base_href=None):
    """Put the probe last in the body, and a `<base>` first in the head."""
    if base_href:
        tag = b'<base href="' + base_href.encode("utf-8") + b'">'
        head = body.lower().find(b"<head>")
        if head != -1:
            cut = head + len(b"<head>")
            body = body[:cut] + tag + body[cut:]
        else:
            body = tag + body
    marker = body.lower().rfind(b"</body>")
    if marker == -1:
        return body + PROBE
    return body[:marker] + PROBE + body[marker:]


class Source:
    """Where the HTML of a page path comes from."""

    def __init__(self, root=None, base=None):
        self.root = root
        self.base = base
        self.cache = {}
        self.lock = threading.Lock()

    def html(self, path):
        """Return (status, bytes) for a page path such as `spec/` or `a.html`."""
        if self.root is not None:
            local = self.root / path
            if path == "" or path.endswith("/"):
                local = local / "index.html"
            if local.is_file():
                return 200, inject(local.read_bytes())
            return 404, b""
        with self.lock:
            if path in self.cache:
                return self.cache[path]
        url = self.base + path
        try:
            with urllib.request.urlopen(url, timeout=60) as response:
                answer = response.status, inject(response.read(), base_href=url)
        except urllib.error.HTTPError as error:
            answer = error.code, b""
        except OSError:
            answer = 0, b""
        with self.lock:
            self.cache[path] = answer
        return answer


class ProbeHandler(http.server.SimpleHTTPRequestHandler):
    source = None

    def log_message(self, fmt, *args):
        return

    def handle_one_request(self):
        try:
            super().handle_one_request()
        except (BrokenPipeError, ConnectionResetError):
            self.close_connection = True

    def send_head(self):
        path = urllib.parse.unquote(self.path.split("?", 1)[0]).lstrip("/")
        is_page = path == "" or path.endswith("/") or path.endswith(".html")
        if self.source.root is not None and not is_page:
            return super().send_head()
        if self.source.root is not None and os.path.isdir(self.source.root / path) and path and not path.endswith("/"):
            return super().send_head()
        status, body = self.source.html(path)
        if status != 200:
            self.send_error(status or 502)
            return None
        self.send_response(200)
        self.send_header("Content-Type", "text/html; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        return io.BytesIO(body)


class Server(http.server.ThreadingHTTPServer):
    daemon_threads = True
    allow_reuse_address = True
    # `check-site-console.py` records why the default backlog of five drops
    # connections under eight browsers.
    request_queue_size = 256


def serve(source):
    directory = str(source.root) if source.root is not None else tempfile.gettempdir()
    handler = functools.partial(ProbeHandler, directory=directory)
    handler.func.source = source
    server = Server(("127.0.0.1", 0), handler)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    return server


PROFILES = threading.local()


def profile_dir(parent):
    existing = getattr(PROFILES, "path", None)
    if existing is None:
        existing = tempfile.mkdtemp(dir=parent, prefix="profile-")
        PROFILES.path = existing
    return existing


def browser_argv(browser, profile, size, scheme):
    return [
        browser,
        "--headless=new",
        "--disable-gpu",
        "--no-sandbox",
        "--disable-dev-shm-usage",
        "--virtual-time-budget=8000",
        "--window-size=%d,%d" % size,
        "--blink-settings=preferredColorScheme=" + SCHEME_SETTING[scheme],
        "--user-data-dir=" + profile,
    ]


def decode(dom):
    found = READING.search(dom)
    if not found:
        return None
    import base64

    try:
        return json.loads(base64.b64decode(found.group(1)).decode("utf-8", "replace"))
    except ValueError:
        return None


def label(size, scheme):
    return "%dx%d, %s" % (size[0], size[1], scheme)


def judge(reading, size, scheme):
    """The flags one reading raises."""
    flags = []
    where = label(size, scheme)
    if reading.get("dark") != (scheme == "dark"):
        flags.append(
            "the %s reading did not take: the page saw prefers-color-scheme %s"
            % (where, "dark" if reading.get("dark") else "light")
        )
        return flags
    if reading["scrollWidth"] > reading["clientWidth"]:
        culprits = reading.get("culprits") or []
        flags.append(
            "horizontal overflow at %s: scrollWidth %d > %d%s"
            % (
                where,
                reading["scrollWidth"],
                reading["clientWidth"],
                (" (" + "; ".join(culprits) + ")") if culprits else "",
            )
        )
    focus = reading.get("focus")
    if focus and focus.get("took"):
        no_outline = focus["outlineStyle"] == "none" or focus["outlineWidth"] in ("0px", "")
        if no_outline and focus["boxShadow"] in ("none", ""):
            flags.append(
                "no visible focus indicator on <%s> at %s: outline %s %s, box-shadow %s"
                % (
                    focus["element"],
                    where,
                    focus["outlineStyle"],
                    focus["outlineWidth"],
                    focus["boxShadow"],
                )
            )
    elif focus:
        flags.append("focus on <%s> at %s did not take, so it was not read" % (focus["element"], where))
    return flags


def nav_sections(path):
    """The top-level and second-level section names of a nav file."""
    if path is None or not path.is_file():
        return None
    import yaml

    data = yaml.safe_load(path.read_text(encoding="utf-8")) or {}
    top, second = [], []
    for entry in data.get("nav") or []:
        if not isinstance(entry, dict):
            continue
        for name, children in entry.items():
            top.append(name)
            for child in children if isinstance(children, list) else []:
                if isinstance(child, dict):
                    second.extend(child.keys())
    return top, second


def judge_nav(nav, sections, served, base_url, source, unlisted):
    """The flags the shelves navigation of one page raises.

    A link to a page the page list does not name is fetched once. One that
    answers 200 is served and is not this page's defect: it goes into
    `unlisted`, which the run reports once, because what it says is that the
    page list (on the live host, the sitemap) left out a page a reader can
    reach. One that does not answer 200 is this page's defect.
    """
    flags = []
    if nav is None:
        return flags
    if sections is not None:
        top, second = sections
        groups = set(nav["groups"])
        texts = {text for text, _ in nav["links"]}
        for name in top:
            if name not in groups:
                flags.append("the navigation does not show the section `%s` of nav.yml" % name)
        for name in second:
            if name not in texts:
                flags.append("the navigation does not show the section `%s` of nav.yml" % name)
    seen = set()
    for _, href in nav["links"]:
        target = page_path(href, base_url)
        if target is None or target in seen:
            continue
        seen.add(target)
        if target in served:
            continue
        status, _ = source.html(target)
        if status == 200:
            unlisted.add(target)
        else:
            flags.append("a navigation link resolves to no served page: %s" % (target or "/"))
    return flags


def page_path(href, base_url):
    """The served path a link names, or None for a link off this site."""
    href = href.split("#", 1)[0].split("?", 1)[0]
    if not href.startswith(base_url):
        return None
    path = href[len(base_url):]
    if path.endswith("index.html"):
        path = path[: -len("index.html")]
    return path


def slug(path):
    name = path.rstrip("/")
    if name.endswith(".html"):
        name = name[: -len(".html")]
    return (name or "home").replace("/", "_")


def review_page(
    path, base, browser, profiles, timeout, readings, shots, sections, served, base_url, source, unlisted
):
    """Return (path, flags, readings-taken, records)."""
    flags = []
    records = []
    taken = 0
    url = base + path
    for size, scheme, shoot in readings:
        argv = browser_argv(browser, profile_dir(profiles), size, scheme)
        taken += 1
        try:
            result = subprocess.run(
                argv + ["--dump-dom", url],
                stdout=subprocess.PIPE,
                stderr=subprocess.DEVNULL,
                timeout=timeout,
                check=False,
            )
        except subprocess.TimeoutExpired:
            flags.append("the browser did not finish %s in %ds" % (label(size, scheme), timeout))
            continue
        reading = decode(result.stdout.decode("utf-8", "replace"))
        if result.returncode != 0:
            flags.append("the browser exited %d at %s" % (result.returncode, label(size, scheme)))
        if reading is None:
            flags.append(
                "the probe this tool injects never ran at %s, so nothing here read the page"
                % label(size, scheme)
            )
            continue
        reading.update({"width": size[0], "height": size[1], "scheme": scheme})
        records.append(reading)
        flags.extend(judge(reading, size, scheme))
        if size == SIZES[0] and scheme == "light":
            flags.extend(
                judge_nav(reading.get("nav"), sections, served, base_url, source, unlisted)
            )
        if shoot and shots is not None:
            target = shots / ("%s--%dx%d-%s.png" % (slug(path), size[0], size[1], scheme))
            try:
                subprocess.run(
                    argv + ["--screenshot=" + str(target), url],
                    stdout=subprocess.DEVNULL,
                    stderr=subprocess.DEVNULL,
                    timeout=timeout,
                    check=False,
                )
            except subprocess.TimeoutExpired:
                pass
            if not target.is_file() or target.stat().st_size == 0:
                flags.append("no screenshot was written for %s" % label(size, scheme))
    # A flag that every reading raised is one finding, not four.
    unique = []
    for flag in flags:
        if flag not in unique:
            unique.append(flag)
    return path, unique, taken, records


def parse(argv):
    options = {
        "root": None,
        "base": None,
        "browser": os.environ.get("HEADWATER_CHROME") or None,
        "jobs": min(8, os.cpu_count() or 2),
        "timeout": 90,
        "nav": DEFAULT_NAV,
        "dark_all": False,
        "shots": None,
        "shoot": [],
        "json": None,
    }
    rest = list(argv)
    valued = {
        "--base": "base",
        "--chrome": "browser",
        "--jobs": "jobs",
        "--timeout": "timeout",
        "--nav": "nav",
        "--shots": "shots",
        "--json": "json",
    }
    while rest:
        item = rest.pop(0)
        if item in valued or item == "--shoot":
            if not rest:
                raise Report(["`%s` needs a value." % item])
            value = rest.pop(0)
            if item == "--shoot":
                options["shoot"].append(re.compile(value))
            else:
                options[valued[item]] = value
        elif item == "--dark-all":
            options["dark_all"] = True
        elif item.startswith("--"):
            raise Report(
                [
                    "unknown argument `%s`." % item,
                    "usage: review-site.py [ROOT | --base URL] [--chrome PATH] [--jobs N] "
                    "[--timeout S] [--nav PATH] [--dark-all] [--shots DIR] "
                    "[--shoot REGEX]... [--json FILE]",
                ]
            )
        elif options["root"] is None:
            options["root"] = Path(item)
        else:
            raise Report(["more than one root named: `%s` and `%s`." % (options["root"], item)])
    if options["root"] is not None and options["base"]:
        raise Report(["name a directory or `--base`, not both."])
    if options["base"] is None and options["root"] is None:
        options["root"] = DEFAULT_ROOT
    if options["base"] and not options["base"].endswith("/"):
        options["base"] += "/"
    options["jobs"] = max(1, int(options["jobs"]))
    options["timeout"] = int(options["timeout"])
    options["nav"] = Path(options["nav"]) if options["nav"] else None
    options["shots"] = Path(options["shots"]) if options["shots"] else None
    return options


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
    shells = sorted(glob.glob(PLAYWRIGHT_SHELL))
    if shells:
        return shells[-1]
    raise Report(
        [
            "SKIPPED: no browser on this host, so no page was read and this",
            "  run reviewed nothing. Looked for: %s, and Playwright's headless shell."
            % ", ".join(BROWSERS),
            "  Name one with `--chrome PATH` or `HEADWATER_CHROME`.",
        ],
        status=3,
    )


def collect_directory(root):
    if not root.is_dir():
        raise Report(["no directory at `%s`." % root])
    pages = []
    for page in sorted(root.rglob("*.html")):
        relative = page.relative_to(root).as_posix()
        if relative == "index.html":
            relative = ""
        elif relative.endswith("/index.html"):
            relative = relative[: -len("index.html")]
        pages.append(relative)
    if not pages:
        raise Report(
            [
                "no `.html` page under `%s`, so this run reviewed nothing." % root,
                "  An empty directory is not a pass, and this status says so.",
            ]
        )
    return pages


def collect_sitemap(base):
    try:
        with urllib.request.urlopen(base + "sitemap.xml", timeout=60) as response:
            text = response.read().decode("utf-8", "replace")
    except OSError as error:
        raise Report(["could not read `%ssitemap.xml`: %s" % (base, error)])
    pages = []
    for loc in re.findall(r"<loc>\s*([^<\s]+)\s*</loc>", text):
        path = urllib.parse.urlparse(loc).path.lstrip("/")
        if path.endswith("index.html"):
            path = path[: -len("index.html")]
        if path not in pages:
            pages.append(path)
    if not pages:
        raise Report(["`%ssitemap.xml` lists no page, so this run reviewed nothing." % base])
    return pages


def run(argv):
    options = parse(argv)
    browser = resolve_browser(options["browser"])
    if options["base"]:
        pages = collect_sitemap(options["base"])
        source = Source(base=options["base"])
    else:
        pages = collect_directory(options["root"])
        source = Source(root=options["root"])
    sections = nav_sections(options["nav"])
    served = set(pages)
    if options["shots"] is not None:
        options["shots"].mkdir(parents=True, exist_ok=True)

    def plan(path):
        shoot = any(pattern.search(path or "index.html") for pattern in options["shoot"])
        schemes = SCHEMES if (shoot or options["dark_all"]) else ("light",)
        return [(size, scheme, shoot) for scheme in schemes for size in SIZES]

    # Pages the navigation reaches and the page list does not name. They are
    # read in a second round, so that a section the sitemap leaves out is still
    # reviewed, and the run says how many there were.
    unlisted = set()
    profiles = tempfile.mkdtemp(prefix="headwater-review-")
    try:
        server = serve(source)
        base = "http://127.0.0.1:%d/" % server.server_address[1]

        def work(path):
            return review_page(
                path,
                base,
                browser,
                profiles,
                options["timeout"],
                plan(path),
                options["shots"],
                sections,
                served,
                # A link on a fetched page resolves against its `<base href>`,
                # which names the host and not the loopback port.
                options["base"] or base,
                source,
                unlisted,
            )

        try:
            with ThreadPoolExecutor(max_workers=options["jobs"]) as pool:
                results = list(pool.map(work, pages))
                extra = sorted(unlisted)
                served.update(extra)
                results.extend(pool.map(work, extra))
        finally:
            server.shutdown()
            server.server_close()
    finally:
        shutil.rmtree(profiles, ignore_errors=True)

    if extra:
        print(
            "%d page%s the navigation links answer 200 and the page list does not "
            "name them, so they were read in a second round: %s"
            % (len(extra), "" if len(extra) == 1 else "s", ", ".join(extra))
        )
        print("")
    flagged = [(path, flags) for path, flags, _, _ in results if flags]
    readings = sum(taken for _, _, taken, _ in results)
    for path, flags in flagged:
        for flag in flags:
            print("%s: %s" % (path or "index.html", flag))
    if flagged:
        print("")
    if options["json"]:
        with open(options["json"], "w", encoding="utf-8") as handle:
            json.dump(
                [
                    {"page": path or "index.html", "flags": flags, "readings": records}
                    for path, flags, _, records in results
                ],
                handle,
                indent=1,
            )
    print(
        "%d page%s flagged, out of %d served pages, over %d readings in %s"
        % (
            len(flagged),
            "" if len(flagged) == 1 else "s",
            len(results),
            readings,
            os.path.basename(browser),
        )
    )
    if sections is None:
        print("no nav file at `%s`, so no page was held to its sections" % options["nav"])
    return 1 if flagged else 0


def main(argv):
    try:
        return run(argv)
    except Report as report:
        for line in report.lines:
            print(line, file=sys.stderr)
        return report.status


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
