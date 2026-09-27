#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
#
# Hold `upkeep-report.py`'s two finding filters against a work directory
# built here from nothing, with no engine and no network. The end-to-end
# fixture (`governed-change.sh`) runs a clean corpus, so no finding ever
# reaches either filter there, and a filter replaced by "keep everything"
# stays green. These cases each give a filter one finding it must drop.
#
#   escape   a finding a suppression holds, on a document the change reaches:
#            counted, never listed
#   reach    a finding that stood before the change, on a document the change
#            does not reach: counted, never listed
#   new      a finding the change introduced, on a document the change does
#            not reach: listed, and marked new
#   kept     a finding that stood before, on a document the change reaches:
#            listed, and not marked new
#
# Usage: python3 report-filters.py
# Exits 0 when every case holds, 1 otherwise. Writes only under a temporary
# directory it removes.
import json
import os
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
REPORT = os.path.join(HERE, "..", "upkeep-report.py")

GOVERNING = "docs/decisions/0001-governs-a.md"
FAR = "docs/decisions/0009-far-away.md"


def finding(rule, path, message, escape="none", severity="error"):
    return {"rule": rule, "path": path, "line": 5, "message": message, "severity": severity, "escape": escape}


ESCAPED = finding("facet.value.not_permitted", GOVERNING, "ESCAPED-ON-REACHED", escape="suppression")
OLD_FAR = finding("facet.value.not_permitted", FAR, "OLD-ON-UNREACHED")
NEW_FAR = finding("relation.target.unresolved", FAR, "NEW-ON-UNREACHED")
OLD_KEPT = finding("facet.required.missing", GOVERNING, "OLD-ON-REACHED")


def write(path, text):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(text)


def main():
    with tempfile.TemporaryDirectory() as work:
        write(os.path.join(work, "changed.txt"), "src/a.rs\n")
        write(os.path.join(work, "gone.tsv"), "")
        write(os.path.join(work, "check.exit"), "1\n")
        write(
            os.path.join(work, "check.json"),
            json.dumps(
                {"change": {"unmatched": ["src/a.rs"]}, "findings": [ESCAPED, OLD_FAR, NEW_FAR, OLD_KEPT]}
            ),
        )
        write(os.path.join(work, "base-check.json"), json.dumps({"findings": [ESCAPED, OLD_FAR, OLD_KEPT]}))
        write(os.path.join(work, "routes.tsv"), "1\tsrc/a.rs\n")
        write(
            os.path.join(work, "route", "1.json"),
            json.dumps(
                {
                    "pointers": [
                        {
                            "path": GOVERNING,
                            "id": "ACME-DR-0001",
                            "summary": "s",
                            "evidence": {"by": "anchor", "anchors": ["src/a.rs"], "suspect": []},
                        }
                    ],
                    "ungoverned": [],
                }
            ),
        )
        out = os.path.join(work, "report.md")
        subprocess.run([sys.executable, REPORT, work, out], check=True)
        text = open(out, encoding="utf-8").read()

    owed = text.split("## Owed\n", 1)[1].split("\n## ", 1)[0]
    listed = [l for l in owed.splitlines() if l.lstrip().startswith("- `docs/")]

    def line_of(marker):
        hits = [l for l in listed if marker in l]
        return hits[0] if hits else None

    failed = []
    if line_of("ESCAPED-ON-REACHED"):
        failed.append("escape: a suppressed finding is listed in Owed")
    if "1 finding(s) are not listed because a suppression" not in owed:
        failed.append("escape: the suppressed finding is not counted")
    if line_of("OLD-ON-UNREACHED"):
        failed.append("reach: an old finding on a document the change does not reach is listed in Owed")
    if "1 more finding(s) stood before this change" not in owed:
        failed.append("reach: the old finding elsewhere is not counted")
    new = line_of("NEW-ON-UNREACHED")
    if not new:
        failed.append("new: a finding the change introduced, on a document it does not reach, is not listed")
    elif "(new with this change)" not in new:
        failed.append("new: the introduced finding is not marked new")
    kept = line_of("OLD-ON-REACHED")
    if not kept:
        failed.append("kept: an old finding on a document the change reaches is not listed")
    elif "(new with this change)" in kept:
        failed.append("kept: an old finding is marked new")

    for f in failed:
        print(f"FAIL {f}", file=sys.stderr)
    if failed:
        print(text, file=sys.stderr)
        return 1
    print("report-filters: escape, reach, new and kept each hold")
    return 0


if __name__ == "__main__":
    sys.exit(main())
