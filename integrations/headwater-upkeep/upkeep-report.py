#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
#
# Write the four-part upkeep report of one change from what `upkeep.sh`
# collected, with no model and no network:
#
#   Touched     the changed documents, each changed path that a document
#               governs with that document, and each path the change deleted
#               or renamed away with the document that governed it at the base,
#               and each path still on the tree that a document the change
#               deleted governed at the base, whether or not the change
#               names that path
#   Stale      what the engine reports as suspect: `relation.target.suspect`
#               findings and the `evidence.suspect` edges of `route`
#   Owed        the other findings of `headwater check --change`, by rule and
#               document
#   Unmeasured  what this report cannot see. It is always present and never
#               empty, because a report that says nothing where it looked at
#               nothing reads as a clean bill.
#
# WHICH FINDINGS ARE LISTED. `check --change` reports over the whole corpus,
# so a corpus with old debt would bury the change under it. A finding is
# listed when the change introduced it, wherever it lands: no finding of the
# run over the base tree pairs with it. A base finding pairs with a current
# one when rule, path and message agree and its line, carried through the
# change's diff hunks, is the current line. A line inside a hunk the change
# deleted or rewrote maps to nothing, so a finding there is new. Same-key
# findings are common (one rule, one message, many sentences), so the line
# is what tells the one the change added from the one that stood. A deleted file breaks an edge on the document that governed it,
# and a deleted document breaks an edge on a document one relation away, and
# both are new findings. A finding that stood before the change is listed
# only when it is on a document the change reaches, and counted otherwise. A
# finding whose `escape` is not `none` (a suppression, a migration task) is a
# deviation the corpus already declared, so it is counted and never listed.
#
# The report proposes. It accepts nothing, and nothing it writes is a
# finding that a gate reads.
#
# Usage: upkeep-report.py <work-dir> <report-path>
#        upkeep-report.py --lost <work-dir> <root>
# <work-dir> holds what `upkeep.sh` wrote: changed.txt, gone.tsv,
# check.json, check.exit, base-check.json, diff.patch, routes.tsv, route/<n>.json,
# routes-base.tsv, route-base/<n>.json, explains-base.tsv, explain-base/<n>.json,
# lost.txt, routes-lost.tsv and route-lost/<n>.json. The `--lost` form prints
# lost.txt: each path a deleted document governed that is still under <root>.
import json
import os
import re
import sys
from collections import Counter, defaultdict

SUSPECT_RULES = {"relation.target.suspect", "relation.target.verification.suspect"}


def read_lines(path):
    if not os.path.exists(path):
        return []
    with open(path, encoding="utf-8") as f:
        return [line.rstrip("\n") for line in f if line.strip()]


def load_json(path):
    try:
        with open(path, encoding="utf-8") as f:
            return json.load(f)
    except (OSError, ValueError):
        return None


def load_routes(work, index, directory):
    routes = {}
    for line in read_lines(os.path.join(work, index)):
        n, path = line.split("\t", 1)
        routes[path] = load_json(os.path.join(work, directory, f"{n}.json"))
    return routes


def code(text):
    return "`" + str(text).replace("`", "'") + "`"


def ancestors(path):
    parts = path.split("/")
    return ["/".join(parts[:i]) for i in range(len(parts) - 1, 0, -1)]


def classify(path, route):
    """Return (kind, detail) for one path: `governed` with its exact
    pointers, `directory` with (directory, pointer) pairs, `in_scope`,
    `no_edge`, or `unread`. Also return the suspect edges route names."""
    if route is None:
        return "unread", None, []
    exact, by_dir, suspect = [], [], []
    for pointer in route.get("pointers") or []:
        evidence = pointer.get("evidence") or {}
        if evidence.get("by") != "anchor":
            continue
        anchors = evidence.get("anchors") or []
        for edge in evidence.get("suspect") or []:
            suspect.append((pointer.get("path"), edge.get("target"), edge.get("verified"), edge.get("current")))
        if path in anchors:
            exact.append(pointer)
        else:
            dirs = [a for a in anchors if a in ancestors(path)]
            if dirs:
                by_dir.append((dirs[0], pointer))
    if exact:
        return "governed", exact, suspect
    if by_dir:
        return "directory", by_dir, suspect
    if any(u.get("path") == path for u in route.get("ungoverned") or []):
        return "in_scope", None, suspect
    return "no_edge", None, suspect


GLOB_BYTES = set("*?[{")


def lost_governors(work, keep):
    """Read what each document the change deleted governed at the base, from
    `explain-base/<n>.json`. Return (lost, unexpanded). `lost` maps each
    governed path for which `keep(path)` holds to the deleted documents that
    governed it. `unexpanded` holds (pattern, document, matched count) for a
    pattern the engine did not expand: an engine before `reach.members[].paths`
    names the paths only of a literal anchor."""
    lost, unexpanded = defaultdict(list), []
    for line in read_lines(os.path.join(work, "explains-base.tsv")):
        n, doc = line.split("\t", 1)
        data = load_json(os.path.join(work, "explain-base", f"{n}.json"))
        if data is None:
            continue
        document = {
            "path": data.get("path") or doc,
            "id": data.get("id"),
            "kind": data.get("kind"),
            "summary": data.get("summary"),
        }
        for edge in data.get("related") or []:
            # Only an edge that this document declared, with this document as
            # the governing end, names a path the document governed. A
            # document target names an identifier, which is no path on the
            # tree, so the existence test in `keep` drops it.
            if edge.get("inbound") or edge.get("governs") != "source":
                continue
            members = (edge.get("reach") or {}).get("members")
            if members is None:
                members = [{"pattern": t} for t in edge.get("targets") or []]
            for member in members:
                pattern = member.get("pattern") or ""
                if "paths" in member:
                    paths = member.get("paths") or []
                elif pattern and not set(pattern) & GLOB_BYTES:
                    paths = [pattern]
                else:
                    unexpanded.append((pattern, document, member.get("matched")))
                    continue
                for path in paths:
                    if keep(path) and document not in lost[path]:
                        lost[path].append(document)
    return lost, unexpanded


def pointer_lines(pointers):
    out = []
    for pointer in pointers:
        ident = pointer.get("id") or pointer.get("kind") or "document"
        summary = pointer.get("summary") or ""
        out.append(f"  - {code(pointer.get('path', '?'))} ({ident}). {summary}".rstrip())
    return out


HUNK = re.compile(r"^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@")


C_ESCAPES = {"a": 7, "b": 8, "t": 9, "n": 10, "v": 11, "f": 12, "r": 13, '"': 34, "\\": 92}


def unquote(name):
    """Undo git's C-style quoting of a path. Git quotes a path that holds a
    double quote, a backslash, a control byte or (with the default
    `core.quotePath`) a byte above 0x7f. It writes each such byte as a
    backslash escape or as three octal digits. An unquoted path is returned
    as it is."""
    if len(name) < 2 or not (name.startswith('"') and name.endswith('"')):
        return name
    body, out, i = name[1:-1], bytearray(), 0
    while i < len(body):
        ch = body[i]
        if ch == "\\" and i + 1 < len(body):
            nxt = body[i + 1]
            if nxt in C_ESCAPES:
                out.append(C_ESCAPES[nxt])
                i += 2
                continue
            octal = body[i + 1:i + 4]
            if len(octal) == 3 and all(c in "01234567" for c in octal):
                out.append(int(octal, 8))
                i += 4
                continue
        out.extend(ch.encode("utf-8"))
        i += 1
    return out.decode("utf-8", errors="replace")


def side_path(field, prefix):
    """Read the name on a `---` or `+++` line. Git ends a name that holds a
    space with a TAB, so that a patch tool can find where the name ends, and
    it does so whether or not it also quotes the name. A name with a TAB of
    its own is always quoted, and git writes that TAB as `\\t` inside the
    quotes, so a trailing TAB is git's and never the file's. Return None for
    /dev/null."""
    if field == "/dev/null":
        return None
    if field.endswith("\t"):
        field = field[:-1]
    field = unquote(field)
    return field[len(prefix):] if field.startswith(prefix) else field


def parse_diff(text):
    """Read `git diff -U0 -M` into {old path: (new path or None, hunks)}.
    Each hunk is (old start, old count, new start, new count). A file the
    diff does not name did not change, and maps to itself line for line."""
    files = {}
    old = new = None
    hunks = None
    for line in text.splitlines():
        if line.startswith("diff --git "):
            old = new = None
            hunks = []
        elif hunks is None:
            continue
        elif line.startswith("rename from "):
            old = unquote(line[len("rename from "):])
        elif line.startswith("rename to "):
            new = unquote(line[len("rename to "):])
            # A pure rename carries no `---`/`+++` pair, so it is recorded
            # here. A later `+++` line replaces it with the hunks.
            if old is not None:
                files.setdefault(old, (new, hunks))
        elif line.startswith("--- "):
            old = side_path(line[4:], "a/")
        elif line.startswith("+++ "):
            new = side_path(line[4:], "b/")
            if old is not None:
                files[old] = (new, hunks)
        else:
            m = HUNK.match(line)
            if m:
                a, b, c, d = m.groups()
                hunks.append((int(a), 1 if b is None else int(b), int(c), 1 if d is None else int(d)))
    return files


def map_line(path, line, diff):
    """Carry a base finding's (path, line) to where it stands after the
    change, or return None when the change deleted the file or deleted or
    rewrote that line. A finding with no line number keeps it."""
    if path not in diff:
        return path, line
    new_path, hunks = diff[path]
    if new_path is None:
        return None
    if not isinstance(line, int) or line <= 0:
        return new_path, line
    shift = 0
    for a, b, c, d in hunks:
        if b == 0:
            # An insertion after old line `a`: it moves every line below it.
            if line > a:
                shift += d
        elif a <= line < a + b:
            return None
        elif line >= a + b:
            shift += d - b
    return new_path, line + shift


def finding_key(f):
    return (f.get("rule"), f.get("path"), f.get("message"))


def split_findings(findings, base_findings, reached, diff):
    """Return (listed, elsewhere, escaped). `listed` holds (finding, is_new)
    pairs. With no base run, nothing can be called new.

    A current finding is old when a base finding has the same rule, path and
    message and its line, carried through the diff, is the current line. Each
    base finding pairs once. A current finding that pairs with nothing is
    new."""
    base = Counter()
    for f in base_findings or []:
        moved = map_line(f.get("path"), f.get("line"), diff)
        if moved is None:
            continue
        rule, _, message = finding_key(f)
        base[(rule, moved[0], message, moved[1])] += 1
    listed, elsewhere, escaped = [], [], []
    for f in findings:
        key = finding_key(f) + (f.get("line"),)
        is_new = base_findings is not None and base[key] == 0
        if base[key] > 0:
            base[key] -= 1
        if f.get("escape", "none") != "none":
            escaped.append(f)
        elif is_new or f.get("path") in reached:
            listed.append((f, is_new))
        else:
            elsewhere.append(f)
    return listed, elsewhere, escaped


def print_lost(work, root):
    """`--lost`: print each path a deleted document governed at the base that
    is still on the tree under <root> and is not itself deleted or renamed
    away, one per line, for `upkeep.sh` to route."""
    gone_paths = {line.split("\t")[1] for line in read_lines(os.path.join(work, "gone.tsv"))}
    lost, _ = lost_governors(
        work, lambda path: path not in gone_paths and os.path.exists(os.path.join(root, path))
    )
    for path in sorted(lost):
        print(path)


def main():
    if sys.argv[1] == "--lost":
        print_lost(sys.argv[2], sys.argv[3])
        return
    work, out = sys.argv[1], sys.argv[2]
    changed = read_lines(os.path.join(work, "changed.txt"))
    gone = [line.split("\t") for line in read_lines(os.path.join(work, "gone.tsv"))]
    lost_paths = set(read_lines(os.path.join(work, "lost.txt")))
    lost, unexpanded = lost_governors(work, lambda path: path in lost_paths)
    lost_routes = load_routes(work, "routes-lost.tsv", "route-lost")
    check = load_json(os.path.join(work, "check.json"))
    base_check = load_json(os.path.join(work, "base-check.json"))
    check_exit = "".join(read_lines(os.path.join(work, "check.exit"))) or "?"
    routes = load_routes(work, "routes.tsv", "route")
    base_routes = load_routes(work, "routes-base.tsv", "route-base")

    # A changed path that no census row holds is not a document of this
    # corpus. `check --change` names those in `change.unmatched`.
    unmatched = set()
    if check is not None:
        unmatched = set((check.get("change") or {}).get("unmatched") or [])
    documents = [p for p in changed if check is not None and p not in unmatched]
    others = [p for p in changed if p not in documents]

    classes = {}
    suspect_edges = []
    for path in others:
        kind, detail, suspect = classify(path, routes.get(path))
        classes[path] = (kind, detail)
        suspect_edges.extend(suspect)
    # A path the change deleted or renamed away is routed against the base
    # tree, because the edge that named it is on the base.
    gone_classes = {}
    for entry in gone:
        old = entry[1]
        kind, detail, _ = classify(old, base_routes.get(old))
        gone_classes[old] = (kind, detail)

    reached = set(documents)
    for kind, detail in list(classes.values()) + list(gone_classes.values()):
        if kind == "governed":
            reached |= {p.get("path") for p in detail}
        elif kind == "directory":
            reached |= {p.get("path") for _, p in detail}

    findings = (check or {}).get("findings") or []
    base_findings = None if base_check is None else (base_check.get("findings") or [])
    diff = parse_diff("\n".join(read_lines(os.path.join(work, "diff.patch"))))
    listed, elsewhere, escaped = split_findings(findings, base_findings, reached, diff)
    suspect = [(f, n) for f, n in listed if f.get("rule") in SUSPECT_RULES]
    owed = [(f, n) for f, n in listed if f.get("rule") not in SUSPECT_RULES]

    governed_count = sum(1 for p in others if classes[p][0] == "governed")
    lines = ["# Upkeep report", ""]
    lines.append(
        f"This change touches {len(changed)} path(s): {len(documents)} document(s) and "
        f"{len(others)} other path(s). A governs edge reaches {governed_count} "
        f"of the {len(others)} other path(s) by name. It deletes or renames away {len(gone)} path(s)."
        + (
            f" It deletes a document that governed {len(lost)} path(s) still on the tree."
            if lost
            else ""
        )
    )
    lines.append("")

    lines.append("## Touched")
    lines.append("")
    before = len(lines)
    for path in documents:
        lines.append(f"- {code(path)}: a document of this corpus, changed")
    for path in others:
        kind, detail = classes[path]
        if kind == "governed":
            lines.append(f"- {code(path)}: governed by {len(detail)} document(s)")
            lines.extend(pointer_lines(detail))
    for entry in gone:
        old = entry[1]
        what = f"renamed to {code(entry[2])}" if entry[0] == "R" else "deleted"
        kind, detail = gone_classes[old]
        if kind == "governed":
            lines.append(
                f"- {code(old)}: {what}. At the base, {len(detail)} document(s) governed it, and the edge now names nothing"
            )
            lines.extend(pointer_lines(detail))
        else:
            lines.append(f"- {code(old)}: {what}")
    # A path the change may not name at all: the document that governed it
    # is gone, and the path is still here.
    for path in sorted(lost):
        kind, detail, _ = classify(path, lost_routes.get(path))
        if kind == "governed":
            now = f"{len(detail)} other document(s) govern it now"
        elif kind == "unread":
            now = "`route` wrote no report for it, so whether a document governs it now is not measured"
        else:
            now = "no document governs it now"
        lines.append(
            f"- {code(path)}: this change deleted {len(lost[path])} document(s) that governed it at the base, and {now}"
        )
        lines.extend(pointer_lines(lost[path]))
    if len(lines) == before:
        lines.append("No changed path is a document, and no governs edge names a changed path.")
    lines.append("")

    def finding_line(f, is_new, indent=""):
        mark = " (new with this change)" if is_new else ""
        return (
            f"{indent}- {code(f.get('path', '?'))}:{f.get('line', '?')}{mark} "
            f"[{f.get('severity', '?')}] `{f.get('rule', '?')}`: {f.get('message', '')}"
        )

    lines.append("## Stale")
    lines.append("")
    if not suspect and not suspect_edges:
        lines.append(
            "The engine reports no suspect edge. That is not a statement that nothing is stale: see Unmeasured."
        )
    for f, is_new in suspect:
        lines.append(finding_line(f, is_new))
    for document, target, verified, current in suspect_edges:
        lines.append(
            f"- {code(document or '?')} governs {code(target or '?')}, verified at {verified}, and it is now {current}"
        )
    lines.append("")

    lines.append("## Owed")
    lines.append("")
    if check is None:
        lines.append(f"`headwater check --change` wrote no report (exit {check_exit}). Its standard error is in the job log.")
    elif not owed:
        lines.append(
            "`headwater check --change` reports no finding outside Stale that this change introduced "
            f"or that is on a document it reaches (exit {check_exit})."
        )
    else:
        by_rule = defaultdict(list)
        for f, is_new in owed:
            by_rule[f.get("rule", "?")].append((f, is_new))
        for rule in sorted(by_rule):
            lines.append(f"- `{rule}`")
            for f, is_new in sorted(by_rule[rule], key=lambda p: (p[0].get("path") or "", p[0].get("line") or 0)):
                mark = " (new with this change)" if is_new else ""
                lines.append(
                    f"  - {code(f.get('path', '?'))}:{f.get('line', '?')}{mark} [{f.get('severity', '?')}]: {f.get('message', '')}"
                )
    if elsewhere:
        lines.append("")
        lines.append(
            f"{len(elsewhere)} more finding(s) stood before this change, on documents it does not reach. "
            "`headwater check` lists them."
        )
    if escaped:
        lines.append("")
        lines.append(
            f"{len(escaped)} finding(s) are not listed because a suppression or a migration task in the corpus holds them."
        )
    lines.append("")

    lines.append("## Unmeasured")
    lines.append("")
    for path in others:
        kind, detail = classes[path]
        if kind == "directory":
            for directory, pointer in detail:
                lines.append(
                    f"- {code(path)}: {code(pointer.get('path', '?'))} governs the directory {code(directory)}. "
                    "The engine matches a literal directory edge by name, so it does not reach the files under it."
                )
        elif kind == "in_scope":
            lines.append(f"- {code(path)}: in the governed scope this corpus declares, and no document governs it.")
        elif kind == "no_edge" and path in lost:
            lines.append(
                f"- {code(path)}: no governs edge names it now. Touched names the document that governed it "
                "at the base, which this change deleted."
            )
        elif kind == "no_edge":
            lines.append(f"- {code(path)}: no governs edge names it, so nothing here says which document it could stale.")
        elif kind == "unread":
            lines.append(f"- {code(path)}: `route` wrote no report for it.")
    for entry in gone:
        old = entry[1]
        kind, detail = gone_classes[old]
        if kind == "directory":
            for directory, pointer in detail:
                lines.append(
                    f"- {code(old)}: at the base, {code(pointer.get('path', '?'))} governed the directory {code(directory)}, "
                    "which a literal directory edge does not carry to the files under it."
                )
        elif kind == "unread":
            lines.append(f"- {code(old)}: `route` over the base tree wrote no report for it.")
        elif kind != "governed":
            lines.append(f"- {code(old)}: no governs edge named it at the base.")
    for pattern, document, matched in unexpanded:
        what = "the paths it matched there" if matched is None else f"the {matched} path(s) it matched there"
        lines.append(
            f"- {code(pattern)}: at the base, {code(document.get('path', '?'))} governed this pattern, and this change "
            f"deleted that document. The engine that ran does not list the paths a pattern matches, so this report "
            f"does not name {what}."
        )
    if check is None:
        lines.append("- Every finding: `headwater check --change` wrote no report.")
    elif base_findings is None:
        lines.append(
            "- New or old: the check over the base tree wrote no report, so no finding is marked new, "
            "and a finding on a document this change does not reach is counted rather than listed."
        )
    if not suspect and not suspect_edges:
        lines.append(
            "- Staleness by age: an edge goes suspect only when it records a `verified_revision`. "
            "An edge that records none never goes suspect, so an empty Stale part says nothing about it."
        )
    lines.append(
        "- Prose: no reader checked any sentence of any document against this change. "
        "This action calls no model, so a sentence that the change made false is not reported here."
    )
    lines.append("")

    with open(out, "w", encoding="utf-8") as f:
        f.write("\n".join(lines))


if __name__ == "__main__":
    main()
