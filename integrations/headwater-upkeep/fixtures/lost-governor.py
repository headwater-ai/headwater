#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
#
# Hold how `upkeep-report.py` reads what a deleted document governed, from a
# work directory built here from nothing, with no engine and no network. The
# end-to-end fixture (`governed-change.sh`) holds one literal edge. This
# fixture holds the shapes it does not reach.
#
# One deleted document, `explain-base/1.json`, declares these edges:
#
#   literal    `src/lit.rs`, in the shape v0.4.0 writes, with no
#              `reach.members[].paths`: the pattern is the path. It is on
#              the tree and the change also edits it, and route finds no edge
#              for it now, so Unmeasured defers to Touched
#   expanded   `src/g/*.rs`, with `paths`. `src/g/x.rs` is on the tree, and
#              another document governs it now. `src/g/gone.rs` is deleted
#              by the same change, so it is not named
#   missing    `src/missing.rs`, a literal that is not on the tree: not named
#   pattern    `src/p/**`, a pattern with no `paths`: Unmeasured names it
#              with its count, because this report cannot expand it
#   inbound    an inbound governs edge, a document target and an anchor edge
#              that governs nothing: none names a path
#
# A second path of gone.tsv, `src/not-a-document.rs`, has no explain report,
# as when `explain` refuses a path that is not a document.
#
# Usage: python3 lost-governor.py [<upkeep-report.py>]
# Exits 0 when every case holds. Writes only under a temporary directory it
# removes.
import json
import os
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))

DELETED = "docs/decisions/0001-deleted.md"
STILL = "docs/decisions/0002-still-here.md"


def write(path, text):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(text)


def anchor(targets, members=None, governs="source", inbound=False):
    edge = {"relation": "governs", "inbound": inbound, "target": ", ".join(targets), "targets": targets, "governs": governs}
    if members is not None:
        edge["reach"] = {"total": sum(m["matched"] for m in members), "members": members}
    return edge


EXPLAIN = {
    "version": "1.0",
    "path": DELETED,
    "id": "ACME-DR-0001",
    "kind": "decision",
    "summary": "The deleted decision.",
    "related": [
        anchor(["src/lit.rs"], [{"pattern": "src/lit.rs", "matched": 1}]),
        anchor(["src/g/*.rs"], [{"pattern": "src/g/*.rs", "matched": 2, "paths": ["src/g/gone.rs", "src/g/x.rs"]}]),
        anchor(["src/missing.rs"], [{"pattern": "src/missing.rs", "matched": 1}]),
        anchor(["src/p/**"], [{"pattern": "src/p/**", "matched": 3}]),
        anchor(["src/inbound.rs"], [{"pattern": "src/inbound.rs", "matched": 1}], inbound=True),
        anchor(["src/neither.rs"], [{"pattern": "src/neither.rs", "matched": 1}], governs="neither"),
        {
            "relation": "governs",
            "inbound": False,
            "target": "ACME-DR-0009",
            "targets": ["ACME-DR-0009"],
            "pointer": {"path": "docs/decisions/0009-far.md", "id": "ACME-DR-0009"},
            "governs": "source",
        },
    ],
}


def route_governed(path, doc, ident):
    return {
        "pointers": [
            {"path": doc, "id": ident, "summary": "s", "evidence": {"by": "anchor", "anchors": [path], "suspect": []}}
        ],
        "ungoverned": [],
    }


NO_EDGE = {"pointers": [], "ungoverned": []}


def part(text, name):
    head = f"## {name}\n"
    return text.split(head, 1)[1].split("\n## ", 1)[0] if head in text else ""


def main():
    report = sys.argv[1] if len(sys.argv) > 1 else os.path.join(HERE, "..", "upkeep-report.py")
    failed = []
    with tempfile.TemporaryDirectory() as tmp:
        root, work = os.path.join(tmp, "root"), os.path.join(tmp, "work")
        for path in ("src/lit.rs", "src/g/x.rs", "src/inbound.rs", "src/neither.rs"):
            write(os.path.join(root, path), "x\n")
        write(os.path.join(work, "gone.tsv"), f"D\t{DELETED}\nD\tsrc/g/gone.rs\nD\tsrc/not-a-document.rs\n")
        write(os.path.join(work, "explains-base.tsv"), f"1\t{DELETED}\n")
        write(os.path.join(work, "explain-base", "1.json"), json.dumps(EXPLAIN))
        lost = subprocess.run(
            [sys.executable, report, "--lost", work, root], check=True, capture_output=True, text=True
        ).stdout.splitlines()
        if lost != ["src/g/x.rs", "src/lit.rs"]:
            failed.append(f"--lost: got {lost}, want ['src/g/x.rs', 'src/lit.rs']")

        write(os.path.join(work, "lost.txt"), "\n".join(lost) + "\n")
        write(os.path.join(work, "routes-lost.tsv"), "1\tsrc/g/x.rs\n2\tsrc/lit.rs\n")
        write(os.path.join(work, "route-lost", "1.json"), json.dumps(route_governed("src/g/x.rs", STILL, "ACME-DR-0002")))
        write(os.path.join(work, "route-lost", "2.json"), json.dumps(NO_EDGE))
        write(os.path.join(work, "changed.txt"), "src/lit.rs\n")
        write(os.path.join(work, "routes.tsv"), "1\tsrc/lit.rs\n")
        write(os.path.join(work, "route", "1.json"), json.dumps(NO_EDGE))
        write(os.path.join(work, "check.exit"), "0\n")
        write(os.path.join(work, "check.json"), json.dumps({"change": {"unmatched": ["src/lit.rs"]}, "findings": []}))
        write(os.path.join(work, "base-check.json"), json.dumps({"findings": []}))
        out = os.path.join(tmp, "report.md")
        subprocess.run([sys.executable, report, work, out], check=True)
        text = open(out, encoding="utf-8").read()

    touched, unmeasured = part(text, "Touched"), part(text, "Unmeasured")
    lit = [l for l in touched.splitlines() if l.startswith("- `src/lit.rs`: this change deleted")]
    if len(lit) != 1 or "no document governs it now" not in lit[0]:
        failed.append("literal: Touched does not name src/lit.rs once as a path no document governs now")
    x = [l for l in touched.splitlines() if l.startswith("- `src/g/x.rs`: this change deleted")]
    if len(x) != 1 or "1 other document(s) govern it now" not in x[0]:
        failed.append("expanded: Touched does not name src/g/x.rs once, with the one document that governs it now")
    if f"`{DELETED}` (ACME-DR-0001)" not in touched:
        failed.append("Touched does not name the deleted document under the paths it governed")
    for path in ("src/g/gone.rs", "src/missing.rs", "src/inbound.rs", "src/neither.rs", "src/not-a-document.rs"):
        if any(l.startswith(f"- `{path}`: this change deleted") for l in touched.splitlines()):
            failed.append(f"Touched names {path} as a path that lost its governor, and it is not one")
    if any(l.startswith("- `src/lit.rs`: no governs edge names it, so") for l in unmeasured.splitlines()):
        failed.append("literal: Unmeasured says only that no edge names src/lit.rs, and Touched names its lost governor")
    if not any("`src/lit.rs`: no governs edge names it now. Touched names" in l for l in unmeasured.splitlines()):
        failed.append("literal: Unmeasured does not defer to Touched for src/lit.rs")
    pattern = [l for l in unmeasured.splitlines() if l.startswith("- `src/p/**`:")]
    if len(pattern) != 1 or DELETED not in pattern[0] or "the 3 path(s)" not in pattern[0]:
        failed.append("pattern: Unmeasured does not name src/p/** once, with its document and its 3 paths")
    if "It deletes a document that governed 2 path(s) still on the tree." not in text:
        failed.append("summary: the opening sentence does not count the 2 paths that lost a governor")

    for f in failed:
        print(f"FAIL {f}", file=sys.stderr)
    if failed:
        print(text, file=sys.stderr)
        return 1
    print("lost-governor: literal, expanded, missing, pattern and inbound each hold")
    return 0


if __name__ == "__main__":
    sys.exit(main())
