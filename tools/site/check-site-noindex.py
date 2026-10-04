"""check-site-noindex.py — hold the served crawler files against the pages that
ask a crawler not to index them (#1681).

WHAT IT HOLDS

  A `site_nav` section that declares `noindex: true` writes the directory of
  each of its shelves under `extra.headwater_noindex` in `.headwater/nav.yml`,
  and `mkdocs/overrides/main.html` gives every page under one of those
  directories `<meta name="robots" content="noindex">`. Four things can then
  go wrong, and this refuses each one:

    1. `sitemap.xml` lists a page that carries the meta.
    2. `llms.txt` lists a page that carries the meta.
    3. A page under a declared prefix does not carry the meta, which is the
       template dropping the line or the nav losing the list.
    4. The nav declares a prefix that no served page sits under, so the
       third test read nothing for it. That is refused rather than passed,
       one prefix at a time. A directory prefix covers every page served
       under it, and a file prefix covers that one page, as the template's
       match over the source path does.

WHY IT READS THE ASSEMBLED DIRECTORY

  `wrangler.jsonc` serves `.headwater/site-deploy`, and
  `tools/site/assemble-site.sh` writes that directory's `sitemap.xml` from the
  whole site. `site/sitemap.xml` and `site/llms.txt` list hand-built pages
  only, so a check that read those two alone listed no process page before
  this change and caught nothing. The assembled copies are what a crawler
  reads, so they are what this reads.

  The rule for "this page carries noindex" is the one `tools/site/sitemap.py`
  applies when it leaves a page out, imported rather than restated.

USAGE

  python3 tools/site/check-site-noindex.py <assembled-dir> [<nav.yml>]

  Exit 0 when every test holds, 1 with each finding on standard error, and 2
  when the input cannot be read: no directory, no page, or no `sitemap.xml`.
"""

import pathlib
import re
import sys

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
from sitemap import BASE, noindexed  # noqa: E402

ROOT = HERE.parent.parent


def prefixes(nav):
    """The `extra.headwater_noindex` list of a generated nav, or `[]`.

    Read by line rather than through a YAML parser, because the file is
    generated and its shape is fixed: `extra:`, then `headwater_noindex:`,
    then one double-quoted item per line. A parser would add a dependency to
    a gate that runs before any is installed.
    """
    out = []
    inside = False
    for line in nav.read_text(encoding="utf-8").splitlines():
        if line.strip() == "headwater_noindex:":
            inside = True
            continue
        if inside:
            match = re.match(r'^\s+- "(.*)"$', line)
            if not match:
                break
            out.append(match.group(1).replace('\\"', '"').replace("\\\\", "\\"))
    return out


def served(prefix):
    """The served page set of a source-path prefix, as `(path, exact)`.

    The template marks a page whose source path starts with the prefix. A
    directory prefix (`a/`) therefore covers every page served under `a/`. A file
    prefix covers that one file, which MkDocs serves at `a/b/` for `a/b.md` and
    at `a/` for `a/README.md`, so it matches that served directory exactly and
    never the pages beneath it.
    """
    if prefix == "" or prefix.endswith("/"):
        return prefix, False
    stem = prefix[:-3] if prefix.endswith(".md") else prefix
    parent, _, name = stem.rpartition("/")
    if name in ("README", "index"):
        return (parent + "/" if parent else ""), True
    return stem + "/", True


def under(rel, entry):
    path, exact = entry
    return rel == path if exact else rel.startswith(path)


def page_of(url, root):
    """The `index.html` a served URL of this site names, or None."""
    if not url.startswith(BASE + "/"):
        return None
    tail = url[len(BASE) + 1:].split("#")[0].split("?")[0]
    if tail and not tail.endswith("/"):
        return None
    path = root / tail / "index.html"
    return path if path.is_file() else None


def main(argv):
    if len(argv) not in (2, 3):
        print("usage: python3 tools/site/check-site-noindex.py <assembled-dir> [<nav.yml>]",
              file=sys.stderr)
        return 2
    root = pathlib.Path(argv[1])
    nav = pathlib.Path(argv[2]) if len(argv) == 3 else ROOT / ".headwater" / "nav.yml"
    if not root.is_dir():
        print("check-site-noindex: not a directory: %s" % root, file=sys.stderr)
        return 2
    pages = sorted(root.glob("**/index.html"))
    if not pages:
        print("check-site-noindex: no `index.html` under %s, so nothing was checked" % root,
              file=sys.stderr)
        return 2
    sitemap = root / "sitemap.xml"
    if not sitemap.is_file():
        print("check-site-noindex: no `sitemap.xml` under %s" % root, file=sys.stderr)
        return 2
    if not nav.is_file():
        print("check-site-noindex: no generated nav at %s" % nav, file=sys.stderr)
        return 2

    findings = []
    marked = {path for path in pages if noindexed(path)}

    # 1 and 2: a crawler file lists a page that asks not to be indexed.
    listed = {}
    locs = re.findall(r"<loc>([^<]+)</loc>", sitemap.read_text(encoding="utf-8"))
    listed["sitemap.xml"] = locs
    llms = root / "llms.txt"
    listed["llms.txt"] = (
        re.findall(r"\((https?://[^)\s]+)\)", llms.read_text(encoding="utf-8"))
        if llms.is_file() else []
    )
    for name, urls in listed.items():
        for url in urls:
            page = page_of(url, root)
            if page is not None and page in marked:
                findings.append("`%s` lists %s, and that page carries `noindex`" % (name, url))

    # 3 and 4: every page under a declared prefix carries the meta.
    raw = prefixes(nav)
    declared = [served(prefix) for prefix in raw]
    reach = [0] * len(declared)
    covered = 0
    for path in pages:
        rel = path.relative_to(root).parent.as_posix()
        rel = "" if rel == "." else rel + "/"
        hits = [i for i, entry in enumerate(declared) if under(rel, entry)]
        for i in hits:
            reach[i] += 1
        if hits:
            covered += 1
            if path not in marked:
                findings.append("`%s` is under a `noindex` section of the nav and carries no "
                                "robots `noindex` meta" % path.relative_to(root).as_posix())
    for prefix, count in zip(raw, reach):
        if count == 0:
            findings.append("the nav declares the `noindex` prefix `%s` and no served page sits "
                            "under it, so nothing was checked for it" % prefix)

    print("%d of %d pages carry `noindex`; %d sit under the %d declared prefixes; "
          "%d URLs in sitemap.xml and %d in llms.txt were read"
          % (len(marked), len(pages), covered, len(declared),
             len(listed["sitemap.xml"]), len(listed["llms.txt"])))
    for finding in findings:
        print("check-site-noindex: " + finding, file=sys.stderr)
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
