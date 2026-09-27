#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
#
# Write the four-part upkeep report of one change from what `upkeep.sh`
# collected, with no model and no network:
#
#   Touched     the changed documents, and each changed path that a document
#               governs, with the document that governs it
#   Stale       what the engine reports as suspect: `relation.target.suspect`
#               findings and the `evidence.suspect` edges of `route`
#   Owed        every other finding of `headwater check --change`, by rule
#               and document
#   Unmeasured  what this report cannot see. It is always present and never
#               empty, because a report that says nothing where it looked at
#               nothing reads as a clean bill.
#
# The report proposes. It accepts nothing, and nothing it writes is a
# finding that a gate reads.
#
# Usage: upkeep-report.py <work-dir> <report-path>
# <work-dir> holds what `upkeep.sh` wrote: changed.txt, deleted.txt,
# check.json, check.exit, check.err, routes.tsv and route/<n>.json.
import json
import os
import sys
from collections import defaultdict

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


def code(text):
    return "`" + text.replace("`", "'") + "`"


def ancestors(path):
    parts = path.split("/")
    return ["/".join(parts[:i]) for i in range(len(parts) - 1, 0, -1)]


def main():
    work, out = sys.argv[1], sys.argv[2]
    changed = read_lines(os.path.join(work, "changed.txt"))
    deleted = read_lines(os.path.join(work, "deleted.txt"))
    check = load_json(os.path.join(work, "check.json"))
    check_exit = "".join(read_lines(os.path.join(work, "check.exit"))) or "?"

    routes = {}
    for line in read_lines(os.path.join(work, "routes.tsv")):
        n, path = line.split("\t", 1)
        routes[path] = load_json(os.path.join(work, "route", f"{n}.json"))

    # A changed path that no census row holds is not a document of this
    # corpus. `check --change` names those in `change.unmatched`.
    unmatched = set()
    if check is not None:
        unmatched = set((check.get("change") or {}).get("unmatched") or [])
    documents = [p for p in changed if check is not None and p not in unmatched]
    others = [p for p in changed if p not in documents]

    governed = []  # (path, pointer)
    under_directory = []  # (path, directory, pointer)
    ungoverned_in_scope = []
    no_edge = []
    unread = []
    suspect_edges = []  # (document, target, verified, current)
    for path in others:
        route = routes.get(path)
        if route is None:
            unread.append(path)
            continue
        exact, by_dir = [], []
        for pointer in route.get("pointers") or []:
            evidence = pointer.get("evidence") or {}
            if evidence.get("by") != "anchor":
                continue
            anchors = evidence.get("anchors") or []
            for edge in evidence.get("suspect") or []:
                suspect_edges.append(
                    (pointer.get("path"), edge.get("target"), edge.get("verified"), edge.get("current"))
                )
            if path in anchors:
                exact.append(pointer)
            else:
                dirs = [a for a in anchors if a in ancestors(path)]
                if dirs:
                    by_dir.append((dirs[0], pointer))
        if exact:
            for pointer in exact:
                governed.append((path, pointer))
        elif by_dir:
            for directory, pointer in by_dir:
                under_directory.append((path, directory, pointer))
        elif any(u.get("path") == path for u in route.get("ungoverned") or []):
            ungoverned_in_scope.append(path)
        else:
            no_edge.append(path)

    # `check --change` reports over the whole corpus, and the change scopes
    # only the rules that read a transition. The report keeps the findings on
    # the documents this change reaches, and counts the rest in one line, so
    # a corpus with old debt does not bury the change under it.
    reached = set(documents) | {p.get("path") for _, p in governed} | {p.get("path") for _, _, p in under_directory}
    # A finding a directive suppressed, or that migration debt holds, has
    # `escape` other than `none`. It is a deviation the corpus already
    # declared, so it is counted and not listed.
    findings = (check or {}).get("findings") or []
    escaped = [f for f in findings if f.get("escape", "none") != "none"]
    findings = [f for f in findings if f.get("escape", "none") == "none"]
    elsewhere = [f for f in findings if f.get("path") not in reached]
    findings = [f for f in findings if f.get("path") in reached]
    suspect = [f for f in findings if f.get("rule") in SUSPECT_RULES]
    owed = [f for f in findings if f.get("rule") not in SUSPECT_RULES]

    lines = ["# Upkeep report", ""]
    lines.append(
        f"This change touches {len(changed)} path(s): {len(documents)} document(s) and "
        f"{len(others)} other path(s). A governs edge reaches {len({p for p, _ in governed})} "
        f"of the {len(others)} other path(s) by name."
    )
    lines.append("")

    lines.append("## Touched")
    lines.append("")
    if not documents and not governed:
        lines.append("No changed path is a document, and no governs edge names a changed path.")
    for path in documents:
        lines.append(f"- {code(path)}: a document of this corpus, changed")
    by_path = defaultdict(list)
    for path, pointer in governed:
        by_path[path].append(pointer)
    for path in [p for p in others if p in by_path]:
        pointers = by_path[path]
        lines.append(f"- {code(path)}: governed by {len(pointers)} document(s)")
        for pointer in pointers:
            ident = pointer.get("id") or pointer.get("kind") or "document"
            summary = pointer.get("summary") or ""
            lines.append(f"  - {code(pointer.get('path', '?'))} ({ident}). {summary}".rstrip())
    lines.append("")

    lines.append("## Stale")
    lines.append("")
    if not suspect and not suspect_edges:
        lines.append(
            "The engine reports no suspect edge. That is not a statement that nothing is stale: see Unmeasured."
        )
    for f in suspect:
        lines.append(f"- {code(f.get('path', '?'))}:{f.get('line', '?')} `{f.get('rule')}`: {f.get('message', '')}")
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
            f"`headwater check --change` reports no finding outside Stale on a document this change reaches (exit {check_exit})."
        )
    else:
        by_rule = defaultdict(lambda: defaultdict(list))
        for f in owed:
            by_rule[f.get("rule", "?")][f.get("path", "?")].append(f)
        for rule in sorted(by_rule):
            lines.append(f"- `{rule}`")
            for path in sorted(by_rule[rule]):
                for f in by_rule[rule][path]:
                    lines.append(
                        f"  - {code(path)}:{f.get('line', '?')} [{f.get('severity', '?')}]: {f.get('message', '')}"
                    )
    if elsewhere:
        lines.append("")
        lines.append(
            f"{len(elsewhere)} more finding(s) are on documents this change does not reach. `headwater check` lists them."
        )
    if escaped:
        lines.append("")
        lines.append(
            f"{len(escaped)} finding(s) are not listed because a suppression or a migration task in the corpus holds them."
        )
    lines.append("")

    lines.append("## Unmeasured")
    lines.append("")
    for path, directory, pointer in under_directory:
        lines.append(
            f"- {code(path)}: {code(pointer.get('path', '?'))} governs the directory {code(directory)}. "
            "The engine matches a literal directory edge by name, so it does not reach the files under it."
        )
    for path in ungoverned_in_scope:
        lines.append(f"- {code(path)}: in the governed scope this corpus declares, and no document governs it.")
    for path in no_edge:
        lines.append(f"- {code(path)}: no governs edge names it, so nothing here says which document it could stale.")
    for path in deleted:
        lines.append(f"- {code(path)}: deleted. `route` reads the tree, so this report does not say which edge named it.")
    for path in unread:
        lines.append(f"- {code(path)}: `route` wrote no report for it.")
    if check is None:
        lines.append("- Every finding: `headwater check --change` wrote no report.")
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

    report = "\n".join(lines)
    with open(out, "w", encoding="utf-8") as f:
        f.write(report)


if __name__ == "__main__":
    main()
