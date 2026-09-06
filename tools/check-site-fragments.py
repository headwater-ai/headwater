#!/usr/bin/env python3
"""check-site-fragments.py — read the served bytes for two properties: every
in-site fragment link resolves against the `id` attributes its target page
carries, and every served page carries a `<title>` that no other page carries.

WHY ONE SCRIPT READS TWO PROPERTIES

  #567 needed an instrument over the served `<title>` of every page. That is
  the same walk, the same parser, the same empty-root refusal and the same
  denominator this file already derives, so a second script would be a second
  copy of all four, and a second CI step to keep beside this one. The two
  properties are independent — a page can carry a correct set of anchors and
  an indistinguishable title — so each one reports on its own line and each
  one fails the run on its own.

WHY THIS READS THE SERVED DIRECTORY AND NOT THE SOURCE

  #431 existed for three weeks with a green build. The only instrument over
  it was `mkdocs build --strict`, which reports a link to a missing anchor at
  `info` and exits 0, and until the generated half was served nobody followed
  one. A checker that reads Markdown would share the assumption that produced
  the defect: that the anchor a renderer derives is the anchor the prose
  cites. This one asserts nothing about how an anchor is derived. It reads
  the bytes a visitor's browser receives, collects the `id` attributes that
  are present, and resolves each `href` against them.

  That makes it the reader of the last step. `mkdocs build` validates links
  between source documents; this validates links between served pages, which
  is a larger population — the theme's own table of contents, the navigation,
  and anything the hand-built half writes are all in it, and none of them
  reaches MkDocs's validator.

WHAT IT WALKS, AND WHY NOTHING IS LISTED HERE

  Every `.html` file under the served root, found by walking it.
  `tools/assemble-site.sh` composes that root from two halves whose contents
  change on every merge, so a hard-coded list of pages would be a second copy
  of the corpus and would go stale the first time a document is added. The
  denominator this prints is derived the same way.

WHAT COUNTS AS A LINK IT CHECKS

  An `href` with a fragment, whose path resolves inside the served root. A
  fragment on an absolute URL is somebody else's page and is not checked. A
  bare `#` carries no fragment and is not a link to anywhere. A link whose
  path does not resolve to a file in the served root is a dead *path*, which
  is a different defect from a dead *fragment*; it is counted and reported
  separately so that neither hides inside the other's number.

WHAT THE TITLE PASS ASSERTS, AND WHY IT IS NOT A STRING MATCH

  Every served page carries a `<title>`, and no title is carried by more than
  one page. It names no expected string. A checker that asserted a particular
  title would have to take that string from `.headwater/nav.yml`, which is
  the artifact the emitter under test writes, so an emitter that wrote the
  wrong label would agree with its own expectation and pass. Distinctness
  reads the served bytes and takes its expectation from nothing. That is also
  why #538 renaming every shelf, #556 adding a canonical URL and #566
  replacing the theme each leave this pass green with no edit here.

  A repeated title cannot be reported unless both colliding pages were read,
  so a failure here is positive evidence that the walk reached each page it
  names. An assertion of the form "no page carries the title X" has no such
  evidence behind it: a walk that reached nothing satisfies it too.

  The element it reads is the first `<title>` outside any `<svg>`. An inline
  icon may carry a `<title>` of its own as its accessible name, and a reader
  that took the first one in document order would compare icon labels on the
  pages that have one and page titles on the pages that do not.

WHAT FAILS THE RUN, AND WHAT ONLY REPORTS

  A dead fragment fails. A dead path reports and does not, and the reason is
  a trade-off `mkdocs.yml` already made and stated: it holds `not_found` at
  `info` because 18 citations point out of `docs_dir` at files that are
  correct in the repository and unreachable from a site rooted at `docs/`,
  and Q31 leaves this repository private so a rewrite to absolute URLs would
  404. Measured on the tree that introduced this file, every dead path is
  that class or a link into `taxonomies/`, which `exclude_docs` removes from
  the site on purpose. Failing on them here would contradict a decision
  recorded 20 lines above them. #406 is the issue that owns the path half,
  and it is where that severity rises.

WHAT IT REPORTS

  The served page, the line in it, the href as written, the target page, and
  the fragment that page does not carry. Where the served page was rendered
  from a Markdown source that is still on disk, it also names that file and
  the line in it, because that is the file a person edits to fix the finding.

THE ASSUMPTION IT STATES, AND FAILS ON

  That there is a served directory with pages in it. A run that walks an
  empty or missing root exits 2 rather than printing "0 dead fragments",
  which is the shape `.githooks/fixtures.sh` uses: a suite that cannot tell
  "nothing is wrong" from "nothing ran" is not a gate.

USAGE

  python3 tools/check-site-fragments.py [ROOT] [--source-dir DIR] [--quiet]

  ROOT defaults to `.headwater/site-deploy`. Exit 0 when every in-site
  fragment resolves and every page carries a title no other page carries, 1
  when one does not, 2 when the assumption above fails. `--strict-paths` also
  fails on a dead path.
"""

import collections
import html.parser
import os
import posixpath
import sys
import urllib.parse

DEFAULT_ROOT = os.path.join(".headwater", "site-deploy")
DEFAULT_SOURCE_DIR = "docs"


class Page(html.parser.HTMLParser):
    """One served page: its title, the ids it carries, the hrefs it writes."""

    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.ids = set()
        self.links = []  # (href, line)
        self.title = None
        self._svg_depth = 0
        self._in_title = False
        self._title_text = []

    def handle_starttag(self, tag, attrs):
        a = dict(attrs)
        if tag == "svg":
            self._svg_depth += 1
        elif tag == "title" and self.title is None and self._svg_depth == 0:
            self._in_title = True
        ident = a.get("id")
        if ident:
            self.ids.add(ident)
        if tag == "a":
            # `<a name="x">` is the pre-HTML5 anchor form. Browsers still
            # honour it as a fragment target, so a checker that read only
            # `id` would report a link that in fact resolves.
            name = a.get("name")
            if name:
                self.ids.add(name)
            href = a.get("href")
            if href is not None:
                self.links.append((href, self.getpos()[0]))

    def handle_endtag(self, tag):
        if tag == "svg" and self._svg_depth > 0:
            self._svg_depth -= 1
        elif tag == "title" and self._in_title:
            self._in_title = False
            self.title = "".join(self._title_text)

    def handle_data(self, data):
        if self._in_title:
            self._title_text.append(data)


def parse(path):
    with open(path, "r", encoding="utf-8", errors="replace") as fh:
        text = fh.read()
    page = Page()
    page.feed(text)
    page.close()
    return page


def walk_html(root):
    out = []
    for dirpath, _dirnames, filenames in os.walk(root):
        for name in sorted(filenames):
            if name.endswith(".html"):
                out.append(os.path.join(dirpath, name))
    return sorted(out)


def rel(root, path):
    return os.path.relpath(path, root).replace(os.sep, "/")


def resolve_target(root, page_rel, link_path):
    """The served file an href's path part names, or None if it names none.

    A directory URL, and a URL ending in `/`, serve that directory's
    `index.html`, which is what a static host does and therefore what a
    visitor gets.
    """
    if link_path == "":
        return page_rel
    if link_path.startswith("/"):
        candidate = link_path.lstrip("/")
    else:
        candidate = posixpath.normpath(
            posixpath.join(posixpath.dirname(page_rel), link_path)
        )
    if candidate.startswith(".."):
        return None
    if candidate in (".", ""):
        candidate = "index.html"
    disk = os.path.join(root, candidate.replace("/", os.sep))
    if os.path.isdir(disk):
        candidate = posixpath.join(candidate, "index.html")
        disk = os.path.join(root, candidate.replace("/", os.sep))
    elif candidate.endswith("/"):
        candidate = candidate + "index.html"
        disk = os.path.join(root, candidate.replace("/", os.sep))
    if os.path.isfile(disk):
        return candidate
    return None


def source_of(source_dir, page_rel):
    """The Markdown file a served page was rendered from, if it is on disk.

    MkDocs writes `X.md` to `X/index.html` under `use_directory_urls`, and
    `index.md` to `index.html`. Nothing here depends on that being right: a
    guess that misses simply leaves the source location out of the report.
    """
    if not source_dir:
        return None
    if page_rel.endswith("/index.html"):
        stem = page_rel[: -len("/index.html")]
    elif page_rel == "index.html":
        stem = "index"
    elif page_rel.endswith(".html"):
        stem = page_rel[: -len(".html")]
    else:
        return None
    candidate = os.path.join(source_dir, *(stem.split("/"))) + ".md"
    return candidate if os.path.isfile(candidate) else None


def source_line(source_path, fragment):
    """The first line of the source that writes this fragment, or None."""
    try:
        with open(source_path, "r", encoding="utf-8", errors="replace") as fh:
            for number, line in enumerate(fh, start=1):
                if "#" + fragment in line:
                    return number
    except OSError:
        return None
    return None


def main(argv):
    root = None
    source_dir = DEFAULT_SOURCE_DIR
    quiet = False
    strict_paths = False
    rest = list(argv)
    while rest:
        arg = rest.pop(0)
        if arg == "--strict-paths":
            strict_paths = True
        elif arg == "--source-dir":
            if not rest:
                sys.stderr.write("check-site-fragments: --source-dir needs a value\n")
                return 2
            source_dir = rest.pop(0)
        elif arg == "--quiet":
            quiet = True
        elif arg.startswith("-"):
            sys.stderr.write("check-site-fragments: unknown argument: %s\n" % arg)
            sys.stderr.write(
                "  usage: python3 tools/check-site-fragments.py "
                "[ROOT] [--source-dir DIR] [--strict-paths] [--quiet]\n"
            )
            return 2
        elif root is None:
            root = arg
        else:
            sys.stderr.write("check-site-fragments: unexpected argument: %s\n" % arg)
            return 2
    if root is None:
        root = DEFAULT_ROOT
    if source_dir and not os.path.isdir(source_dir):
        source_dir = None

    if not os.path.isdir(root):
        sys.stderr.write("check-site-fragments: no served directory at `%s`.\n" % root)
        sys.stderr.write("  `sh tools/assemble-site.sh` writes it, after a build:\n")
        sys.stderr.write("        python3 -m mkdocs build --strict\n")
        sys.stderr.write("        sh tools/assemble-site.sh\n")
        return 2

    pages = walk_html(root)
    if not pages:
        sys.stderr.write(
            "check-site-fragments: `%s` holds no `.html` file, so this run "
            "checked nothing.\n" % root
        )
        sys.stderr.write(
            "  Reporting zero dead fragments over zero pages would be a pass "
            "this suite has not earned.\n"
        )
        return 2

    ids = {}
    parsed = {}
    for path in pages:
        page = parse(path)
        page_rel = rel(root, path)
        ids[page_rel] = page.ids
        parsed[page_rel] = page

    # The title pass. It reads the same parse of the same walk. A page whose
    # `<title>` is absent, empty or whitespace is untitled: an empty element
    # is the same defect as a missing one to every reader of it.
    untitled = []
    by_title = collections.defaultdict(list)
    for page_rel in sorted(parsed):
        title = parsed[page_rel].title
        if title is None or not title.strip():
            untitled.append(page_rel)
        else:
            by_title[title.strip()].append(page_rel)
    repeated = sorted(
        (title, paths) for title, paths in by_title.items() if len(paths) > 1
    )

    checked = 0
    dead_fragments = []
    dead_paths = []
    external = 0

    for page_rel in sorted(parsed):
        for href, line in parsed[page_rel].links:
            split = urllib.parse.urlsplit(href)
            if split.scheme or split.netloc:
                external += 1
                continue
            fragment = split.fragment
            if not fragment:
                continue
            fragment = urllib.parse.unquote(fragment)
            target = resolve_target(root, page_rel, split.path)
            if target is None:
                dead_paths.append((page_rel, line, href))
                continue
            checked += 1
            if fragment not in ids[target]:
                dead_fragments.append((page_rel, line, href, target, fragment))

    if not quiet:
        for page_rel, line, href, target, fragment in dead_fragments:
            print("%s:%d: `%s`" % (page_rel, line, href))
            print(
                "    `%s` carries no id `%s`" % (target, fragment)
            )
            source = source_of(source_dir, page_rel)
            if source:
                number = source_line(source, fragment)
                if number:
                    print("    written at %s:%d" % (source, number))
                else:
                    print("    rendered from %s" % source)
        for page_rel, line, href in dead_paths:
            print("%s:%d: `%s`" % (page_rel, line, href))
            print(
                "    resolves to no file in `%s`%s"
                % (root, "" if strict_paths else " (reported, not fatal: #406)")
            )
        for page_rel in untitled:
            print("%s: carries no `<title>`" % page_rel)
        for title, paths in repeated:
            print("`%s` is the `<title>` of %d served pages:" % (title, len(paths)))
            for page_rel in paths:
                print("    %s" % page_rel)

    print(
        "%d dead fragment%s and %d dead path%s, "
        "out of %d in-site fragment link%s across %d served page%s "
        "(%d external link%s not checked)"
        % (
            len(dead_fragments),
            "" if len(dead_fragments) == 1 else "s",
            len(dead_paths),
            "" if len(dead_paths) == 1 else "s",
            checked,
            "" if checked == 1 else "s",
            len(pages),
            "" if len(pages) == 1 else "s",
            external,
            "" if external == 1 else "s",
        )
    )
    print(
        "%d repeated title%s and %d page%s with no title, "
        "out of %d distinct title%s across %d served page%s"
        % (
            len(repeated),
            "" if len(repeated) == 1 else "s",
            len(untitled),
            "" if len(untitled) == 1 else "s",
            len(by_title),
            "" if len(by_title) == 1 else "s",
            len(pages),
            "" if len(pages) == 1 else "s",
        )
    )

    if dead_fragments:
        return 1
    if repeated or untitled:
        return 1
    if dead_paths and strict_paths:
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
