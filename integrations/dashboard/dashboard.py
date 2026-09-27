#!/usr/bin/env python3
"""Render a read-only dashboard page from a Headwater `graph_export` JSON file.

    python3 integrations/dashboard/dashboard.py \\
        --export export.json [--tree <checkout>] [--out dashboard.html]

The input is the `json` target of a `graph_export` profile at
`export_version` 1.1 or later. The output is one self-contained HTML page with
three views: staleness (every document by `facets.last_verified`, oldest
first), warrant (every document by its top-level `warrant`, with an absent key
shown as "none stated"), and coverage (every `code_path` anchor that a
`governs` edge reaches, and, given `--tree`, the share of that tree's files the
anchors cover).

The script reads the export and, with `--tree`, lists the tree's files. It
runs no Headwater verb, writes nothing but the page, and commits nothing.

Every row is keyed by `(corpus_identity, id)`, as HW-DR-0080 rules. A
self-hosted page holds one corpus identity and never displays it.
"""

import argparse
import datetime
import html
import os
import re
import subprocess
import sys

NONE_STATED = "none stated"
NEVER_VERIFIED = "never verified"
DATE_SHAPE = re.compile(r"[0-9]{4}-[0-9]{2}-[0-9]{2}")
EMPTY_CORPUS = (
    "This export holds no documents, so every view below is empty. "
    "The corpus it was generated from has no governed document yet."
)
MINIMUM_EXPORT_VERSION = (1, 1)
WARRANT_ORDER = ["asserted", NONE_STATED, "proposed", "accepted"]


class ExportRefused(Exception):
    """The input is not a graph_export this page can read."""


class Model:
    """One corpus's rows. Every row carries `key` = (corpus_identity, id)."""

    def __init__(self, corpus_identity, documents, anchors, profile):
        self.corpus_identity = corpus_identity
        self.documents = documents
        self.anchors = anchors
        self.profile = profile


class Coverage:
    def __init__(self, rows, covered, total):
        self.rows = rows
        self.covered = covered
        self.total = total


def _version(text):
    try:
        return tuple(int(part) for part in str(text).split("."))
    except ValueError:
        return None


def _refuse(where, field, problem):
    raise ExportRefused("%s: `%s` %s" % (where, field, problem))


def _list(graph, name):
    value = graph.get(name)
    if not isinstance(value, list):
        _refuse("graph", name, "is missing or is not a list")
    return value


def _optional_string(value, where, field):
    if value is not None and not isinstance(value, str):
        _refuse(where, field, "is %s, not a string" % type(value).__name__)
    return value


def _date(value, where, field):
    """A calendar date, returned in the form YYYY-MM-DD so that text order is date order.

    A string that is not a real date is refused, because the page sorts on
    this value: `2026-02-30` or "last tuesday" would otherwise take a place
    in the staleness order that no date holds.
    """
    value = _optional_string(value, where, field)
    if value is None:
        return None
    problem = "is %r, which is not a calendar date in the form YYYY-MM-DD" % value
    # The shape is checked first, so every Python accepts the same set:
    # 3.11 and later also parse `20260115` and `2026-W03-4`. The pattern uses
    # [0-9] rather than \d, which would admit other scripts' digits, and
    # fullmatch rather than `$`, which would admit a trailing newline.
    if not DATE_SHAPE.fullmatch(value):
        _refuse(where, field, problem)
    try:
        return datetime.date.fromisoformat(value).isoformat()
    except ValueError:
        _refuse(where, field, problem)


def load(export, corpus_identity):
    """Read one export as one corpus's data, keyed by the pair on every row.

    A malformed input is refused with ExportRefused, which names the document
    and the field, and is never rendered as a partial page.
    """
    if not corpus_identity:
        raise ExportRefused("a corpus identity is required; every row is keyed by it")
    if not isinstance(export, dict):
        raise ExportRefused("the export is not a JSON object")
    version = _version(export.get("export_version"))
    if version is None or version < MINIMUM_EXPORT_VERSION:
        raise ExportRefused(
            "export_version %r is below 1.1; regenerate the export with a current engine"
            % export.get("export_version")
        )
    graph = export.get("graph")
    if not isinstance(graph, dict):
        raise ExportRefused("the export has no `graph` object")
    raw_documents = _list(graph, "documents")
    raw_anchors = _list(graph, "anchors")
    raw_edges = _list(graph, "edges")

    documents = []
    by_path = {}
    seen = set()
    for index, document in enumerate(raw_documents):
        where = "graph.documents[%d]" % index
        if not isinstance(document, dict):
            _refuse(where, "document", "is not an object")
        path = _optional_string(document.get("path"), where, "path")
        identifier = _optional_string(document.get("id"), where, "id") or path
        if not identifier:
            _refuse(where, "id", "and `path` are both missing, so the row has no key")
        where = "%s (%s)" % (where, identifier)
        facets = document.get("facets", {})
        if not isinstance(facets, dict):
            _refuse(where, "facets", "is not an object")
        # The export carries this field under `facets`, never at the top level.
        last_verified = _date(facets.get("last_verified"), where, "facets.last_verified")
        # An absent key stays absent. It is never read as `asserted`.
        warrant = _optional_string(document.get("warrant"), where, "warrant")
        if (corpus_identity, identifier) in seen:
            _refuse(where, "id", "appears on more than one document")
        seen.add((corpus_identity, identifier))
        title = facets.get("title")
        row = {
            "corpus_identity": corpus_identity,
            "id": identifier,
            "key": (corpus_identity, identifier),
            "path": path,
            "kind": document.get("kind"),
            "title": title if isinstance(title, str) and title else path,
            "last_verified": last_verified,
            "warrant": warrant,
        }
        documents.append(row)
        if path:
            by_path[path] = row

    code_paths = set()
    for index, anchor in enumerate(raw_anchors):
        where = "graph.anchors[%d]" % index
        if not isinstance(anchor, dict):
            _refuse(where, "anchor", "is not an object")
        if anchor.get("anchor_kind") == "code_path":
            code_paths.add(_optional_string(anchor.get("id"), where, "id"))

    governed_by = {}
    for index, edge in enumerate(raw_edges):
        where = "graph.edges[%d]" % index
        if not isinstance(edge, dict):
            _refuse(where, "edge", "is not an object")
        target = edge.get("target")
        if not isinstance(target, dict):
            _refuse(where, "target", "is missing or is not an object")
        if edge.get("relation") != "governs" or target.get("bound") != "anchor":
            continue
        if target.get("anchor_kind") != "code_path":
            continue
        anchor_id = _optional_string(target.get("id"), where, "target.id")
        if not anchor_id:
            _refuse(where, "target.id", "is missing on a governs edge")
        source_path = _optional_string(edge.get("source"), where, "source")
        source = by_path.get(source_path)
        governor = source["id"] if source else source_path
        if not governor:
            _refuse(where, "source", "is missing on a governs edge")
        governed_by.setdefault(anchor_id, [])
        if governor not in governed_by[anchor_id]:
            governed_by[anchor_id].append(governor)

    anchors = []
    for identifier in sorted(governed_by):
        anchors.append(
            {
                "corpus_identity": corpus_identity,
                "id": identifier,
                "key": (corpus_identity, identifier),
                "declared": identifier in code_paths,
                "governed_by": sorted(governed_by[identifier]),
            }
        )
    return Model(corpus_identity, documents, anchors, export.get("profile") or {})


def staleness_view(model):
    """Every document, oldest `last_verified` first. A missing date sorts first."""
    return sorted(
        model.documents,
        key=lambda row: (row["last_verified"] is not None, row["last_verified"] or "", row["key"]),
    )


def warrant_view(model):
    """Every document grouped by warrant, as (label, rows) pairs in a fixed order."""
    groups = {}
    for row in model.documents:
        label = row["warrant"] if row["warrant"] is not None else NONE_STATED
        groups.setdefault(label, []).append(row)
    order = [label for label in WARRANT_ORDER if label in groups]
    order += sorted(label for label in groups if label not in WARRANT_ORDER)
    return [(label, sorted(groups[label], key=lambda row: row["key"])) for label in order]


def _pattern(anchor):
    """A matcher for one code_path anchor: an exact file, a directory, or a glob."""
    anchor = anchor.strip("/")
    if not any(char in anchor for char in "*?["):
        return lambda path: path == anchor or path.startswith(anchor + "/")
    regex = ""
    index = 0
    while index < len(anchor):
        if anchor.startswith("**/", index):
            regex += "(?:.*/)?"
            index += 3
        elif anchor.startswith("**", index):
            regex += ".*"
            index += 2
        elif anchor[index] == "*":
            regex += "[^/]*"
            index += 1
        elif anchor[index] == "?":
            regex += "[^/]"
            index += 1
        else:
            regex += re.escape(anchor[index])
            index += 1
    compiled = re.compile(regex + r"\Z")
    return lambda path: compiled.match(path) is not None


def tree_files(tree):
    """The files of a checkout: `git ls-files` in a git top level, else a walk that skips `.git`."""
    if os.path.exists(os.path.join(tree, ".git")):
        listed = subprocess.run(
            ["git", "-C", tree, "ls-files", "-z"], capture_output=True, text=True, check=True
        ).stdout
        return sorted(path for path in listed.split("\0") if path)
    found = []
    for directory, subdirectories, files in os.walk(tree):
        subdirectories[:] = sorted(name for name in subdirectories if name != ".git")
        for name in files:
            found.append(os.path.relpath(os.path.join(directory, name), tree).replace(os.sep, "/"))
    return sorted(found)


def coverage_view(model, tree=None):
    """Each reached anchor with its matched-file count, and the tree's covered share.

    Without a tree, counts and the share are None: the export alone has no
    denominator.
    """
    if tree is None:
        return Coverage([(row, None) for row in model.anchors], None, None)
    files = tree_files(tree)
    covered = set()
    rows = []
    for row in model.anchors:
        matches = _pattern(row["id"])
        hit = [path for path in files if matches(path)]
        covered.update(hit)
        rows.append((row, len(hit)))
    return Coverage(rows, len(covered), len(files))


STYLE = """
body{font-family:system-ui,sans-serif;margin:2rem;max-width:72rem;color:#1b1b1b}
h1{font-size:1.5rem}h2{font-size:1.2rem;margin-top:2rem}
table{border-collapse:collapse;width:100%;font-size:.9rem}
th,td{text-align:left;padding:.25rem .5rem;border-bottom:1px solid #ddd;vertical-align:top}
th{background:#f4f4f4}code{font-size:.85rem}.muted{color:#666}
nav a{margin-right:1rem}
"""


def _cell(text):
    return "<td>%s</td>" % html.escape("" if text is None else str(text))


def render(model, coverage=None):
    """One self-contained HTML page. The corpus identity is never displayed."""
    if coverage is None:
        coverage = coverage_view(model, tree=None)
    stale = staleness_view(model)
    groups = warrant_view(model)
    out = []
    out.append("<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\">")
    out.append("<title>Headwater corpus dashboard</title><style>%s</style></head><body>" % STYLE)
    out.append("<h1>Headwater corpus dashboard</h1>")
    out.append(
        "<p class=\"muted\">Read-only. Rendered from a <code>graph_export</code> file (profile %s). "
        "%d documents, %d governed code paths.</p>"
        % (html.escape(str(model.profile.get("name", "unnamed"))), len(model.documents), len(model.anchors))
    )
    out.append("<nav><a href=\"#staleness\">Staleness</a><a href=\"#warrant\">Warrant</a>"
               "<a href=\"#coverage\">Coverage</a></nav>")
    if not model.documents:
        out.append("<p id=\"empty-corpus\"><strong>%s</strong></p>" % html.escape(EMPTY_CORPUS))

    out.append("<h2 id=\"staleness\">Staleness: oldest verification first</h2>")
    out.append("<table><tr><th>Last verified</th><th>Identifier</th><th>Kind</th><th>Title</th></tr>")
    for row in stale:
        out.append(
            "<tr>%s%s%s%s</tr>"
            % (_cell(row["last_verified"] or NEVER_VERIFIED), _cell(row["id"]), _cell(row["kind"]), _cell(row["title"]))
        )
    out.append("</table>")

    out.append("<h2 id=\"warrant\">Warrant: how far each document is established</h2>")
    out.append("<table><tr>%s</tr><tr>%s</tr></table>" % (
        "".join("<th>%s</th>" % html.escape(label) for label, _ in groups),
        "".join(_cell(len(rows)) for _, rows in groups),
    ))
    for label, rows in groups:
        out.append("<h3>%s (%d)</h3><table><tr><th>Identifier</th><th>Kind</th><th>Title</th></tr>"
                   % (html.escape(label), len(rows)))
        for row in rows:
            out.append("<tr>%s%s%s</tr>" % (_cell(row["id"]), _cell(row["kind"]), _cell(row["title"])))
        out.append("</table>")

    out.append("<h2 id=\"coverage\">Coverage: code paths a governs edge reaches</h2>")
    if coverage.total is None:
        out.append("<p class=\"muted\">No tree was given, so no share of files is stated.</p>")
    else:
        share = (100.0 * coverage.covered / coverage.total) if coverage.total else 0.0
        out.append("<p>%d of %d files in the tree are covered (%.1f%%).</p>" % (coverage.covered, coverage.total, share))
    out.append("<table><tr><th>Code path</th><th>Files matched</th><th>Governed by</th></tr>")
    for row, count in coverage.rows:
        out.append("<tr><td><code>%s</code></td>%s%s</tr>" % (
            html.escape(row["id"]), _cell("" if count is None else count), _cell(", ".join(row["governed_by"]))
        ))
    out.append("</table></body></html>")
    return "\n".join(out) + "\n"


def main(argv=None):
    import json

    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--export", required=True, help="a graph_export json file, export_version 1.1 or later")
    parser.add_argument("--tree", help="a checkout of the commit the export was generated from")
    parser.add_argument("--out", default="dashboard.html", help="the page to write (default: dashboard.html)")
    parser.add_argument(
        "--corpus-identity",
        default="local",
        help="the corpus this export belongs to; keys every row and is never displayed (default: local)",
    )
    arguments = parser.parse_args(argv)
    try:
        with open(arguments.export, encoding="utf-8") as handle:
            export = json.load(handle)
        model = load(export, arguments.corpus_identity)
    except (OSError, ValueError, ExportRefused) as error:
        print("dashboard: %s" % error, file=sys.stderr)
        return 2
    coverage = coverage_view(model, tree=arguments.tree)
    with open(arguments.out, "w", encoding="utf-8") as handle:
        handle.write(render(model, coverage))
    print("dashboard: wrote %s" % arguments.out, file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
