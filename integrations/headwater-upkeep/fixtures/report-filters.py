#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
#
# Hold `upkeep-report.py`'s finding filters against a work directory built
# here from nothing, with no engine and no network. The end-to-end fixture
# (`governed-change.sh`) runs a clean corpus, so no finding ever reaches a
# filter there, and a filter replaced by "keep everything" stays green. These
# cases each give a filter a finding it must treat one way and not the other.
#
#   escape   a finding a suppression holds, on a document the change reaches:
#            counted, never listed
#   reach    a finding that stood before the change, on a document the change
#            does not reach: counted, never listed
#   kept     a finding that stood before, on a document the change reaches:
#            listed, and not marked new
#   gone     a finding that stood before, on the document that governed a
#            path the change deleted: listed, because that document is
#            reached through the base tree and through nothing else
#   new      a finding the change introduced, on a document the change does
#            not reach: listed, and marked new. It shares its rule and its
#            path with an old finding, and its message with another old
#            finding on another rule and path, so a key of path alone, rule
#            alone, message alone, or rule and path calls it old
#   twice    one finding raised twice, where the base raised it once: one
#            copy is new and listed, and one is old and counted, so a key
#            compared as a set rather than a multiset calls both old
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
GONE_GOVERNING = "docs/decisions/0002-governed-the-deleted-file.md"
FAR = "docs/decisions/0009-far-away.md"
OTHER = "docs/decisions/0010-other.md"
TWICE_DOC = "docs/decisions/0011-twice.md"


def finding(rule, path, message, escape="none", severity="error"):
    return {"rule": rule, "path": path, "line": 5, "message": message, "severity": severity, "escape": escape}


ESCAPED = finding("facet.value.not_permitted", GOVERNING, "ESCAPED-ON-REACHED", escape="suppression")
OLD_FAR = finding("facet.value.not_permitted", FAR, "OLD-ON-UNREACHED")
OLD_KEPT = finding("facet.required.missing", GOVERNING, "OLD-ON-REACHED")
OLD_GONE = finding("link.path.unresolved", GONE_GOVERNING, "OLD-ON-GONE-GOVERNOR")
# Same message as NEW_FAR, on another rule and another path. Old, elsewhere.
OLD_SHARED = finding("section.required.missing", OTHER, "SHARED-MESSAGE")
# Same rule and path as OLD_FAR, and the message of OLD_SHARED. New.
NEW_FAR = finding("facet.value.not_permitted", FAR, "SHARED-MESSAGE")
TWICE = finding("identifier.claim.missing", TWICE_DOC, "RAISED-TWICE")


def write(path, text):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(text)


def route(path, doc, ident):
    return {
        "pointers": [
            {
                "path": doc,
                "id": ident,
                "summary": "s",
                "evidence": {"by": "anchor", "anchors": [path], "suspect": []},
            }
        ],
        "ungoverned": [],
    }


def main():
    # NEW_FAR comes before every old finding it collides with. A partial key
    # still counts the right number of new findings, because the multiset
    # decrements, but it spends the base's copy on NEW_FAR and calls the old
    # finding after it new. So the order is what makes each partial key red.
    current = [NEW_FAR, ESCAPED, OLD_FAR, OLD_KEPT, OLD_GONE, OLD_SHARED, TWICE, dict(TWICE)]
    base = [ESCAPED, OLD_FAR, OLD_KEPT, OLD_GONE, OLD_SHARED, TWICE]
    with tempfile.TemporaryDirectory() as work:
        write(os.path.join(work, "changed.txt"), "src/a.rs\n")
        write(os.path.join(work, "gone.tsv"), "D\tsrc/gone.rs\n")
        write(os.path.join(work, "check.exit"), "1\n")
        write(os.path.join(work, "check.json"), json.dumps({"change": {"unmatched": ["src/a.rs"]}, "findings": current}))
        write(os.path.join(work, "base-check.json"), json.dumps({"findings": base}))
        write(os.path.join(work, "routes.tsv"), "1\tsrc/a.rs\n")
        write(os.path.join(work, "route", "1.json"), json.dumps(route("src/a.rs", GOVERNING, "ACME-DR-0001")))
        write(os.path.join(work, "routes-base.tsv"), "1\tsrc/gone.rs\n")
        write(os.path.join(work, "route-base", "1.json"), json.dumps(route("src/gone.rs", GONE_GOVERNING, "ACME-DR-0002")))
        out = os.path.join(work, "report.md")
        subprocess.run([sys.executable, REPORT, work, out], check=True)
        text = open(out, encoding="utf-8").read()

    owed = text.split("## Owed\n", 1)[1].split("\n## ", 1)[0]
    listed = [l for l in owed.splitlines() if l.lstrip().startswith("- `docs/")]

    def lines_of(marker):
        return [l for l in listed if marker in l]

    failed = []
    if lines_of("ESCAPED-ON-REACHED"):
        failed.append("escape: a suppressed finding is listed in Owed")
    if "1 finding(s) are not listed because a suppression" not in owed:
        failed.append("escape: the suppressed finding is not counted once")
    if lines_of("OLD-ON-UNREACHED"):
        failed.append("reach: an old finding on a document the change does not reach is listed in Owed")
    if lines_of("SHARED-MESSAGE") and any(OTHER in l for l in lines_of("SHARED-MESSAGE")):
        failed.append("reach: the old finding on another document that shares a message is listed")
    kept = lines_of("OLD-ON-REACHED")
    if not kept:
        failed.append("kept: an old finding on a document the change reaches is not listed")
    elif "(new with this change)" in kept[0]:
        failed.append("kept: an old finding is marked new")
    gone = lines_of("OLD-ON-GONE-GOVERNOR")
    if not gone:
        failed.append("gone: an old finding on the document that governed a deleted path is not listed")
    elif "(new with this change)" in gone[0]:
        failed.append("gone: an old finding on the governor of a deleted path is marked new")
    new = [l for l in lines_of("SHARED-MESSAGE") if FAR in l]
    if len(new) != 1:
        failed.append(f"new: the finding the change introduced is listed {len(new)} time(s), not once")
    elif "(new with this change)" not in new[0]:
        failed.append("new: the introduced finding is not marked new")
    twice = lines_of("RAISED-TWICE")
    if len(twice) != 1:
        failed.append(f"twice: a finding raised twice where the base raised it once is listed {len(twice)} time(s), not once")
    elif "(new with this change)" not in twice[0]:
        failed.append("twice: the second copy is not marked new")
    # Old and elsewhere: OLD_FAR, OLD_SHARED and one copy of TWICE.
    if "3 more finding(s) stood before this change" not in owed:
        failed.append("count: the old findings elsewhere are not counted as 3")

    for f in failed:
        print(f"FAIL {f}", file=sys.stderr)
    if failed:
        print(text, file=sys.stderr)
        return 1
    print("report-filters: escape, reach, kept, gone, new and twice each hold")
    return 0


if __name__ == "__main__":
    sys.exit(main())
