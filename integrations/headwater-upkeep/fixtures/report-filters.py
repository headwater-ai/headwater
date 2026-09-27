#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
#
# Hold `upkeep-report.py`'s finding filters and its new-or-old matcher
# against a work directory built here from nothing, with no engine and no
# network. The end-to-end fixture (`governed-change.sh`) runs a clean corpus,
# so no finding ever reaches a filter there.
#
# Both finding lists are fed in the order the engine emits them: sorted by
# path, line, column, rule and message (`engine/crates/check/src/finding.rs`).
# No case depends on an order chosen by hand.
#
# The matcher pairs a current finding with a base finding when rule, path,
# message AND line agree, after the base finding's line is carried through
# the change's diff hunks (`diff.patch`, from `git diff -U0 -M`). A base line
# inside a hunk that the change deleted or rewrote maps to nothing. A current
# finding that pairs with nothing is new.
#
#   escape   a suppressed finding on a reached document: counted, not listed
#   reach    an old finding on an unreached document: counted, not listed
#   kept     an old finding on a reached document: listed, not marked new
#   gone     an old finding on the document that governed a deleted path:
#            listed, because only the base route reaches that document
#   collide  a new finding that shares rule and path with an old one, and its
#            message with another old one on another rule and path. Its
#            message sorts first, so in engine order it meets the base's copy
#            before the old finding does
#   dup      same key, base line 40, current lines 12 and 40, with line 12
#            rewritten: 12 is new and listed, 40 is old and counted
#   moved    same key, base line 10, current line 13, three lines inserted
#            above it: old and counted, never new
#   edited   same key, base line 20, current line 20, line 20 rewritten: the
#            base line maps to nothing, so the current finding is new
#   renamed  base file renamed with no edit: base line 7 pairs with the
#            renamed file's line 7 (old), and a same-key finding at line 31,
#            below the line the change inserted at 30, is new
#   twice    one finding raised twice at one position (the engine emits such
#            pairs: two identical code spans on one line), where the base
#            raised it once: one copy is new and listed, and one is old and
#            counted, so a match compared as a set calls both old
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
DUP = "docs/decisions/0020-dup.md"
MOVED = "docs/decisions/0021-moved.md"
EDITED = "docs/decisions/0022-edited.md"
RENAMED_OLD = "docs/decisions/0023-before.md"
RENAMED_NEW = "docs/decisions/0024-after.md"
TWICE_DOC = "docs/decisions/0025-twice.md"


def finding(rule, path, message, line=5, escape="none", severity="error"):
    return {
        "rule": rule,
        "path": path,
        "line": line,
        "column": 1,
        "message": message,
        "severity": severity,
        "escape": escape,
    }


def engine_order(findings):
    return sorted(findings, key=lambda f: (f["path"], f["line"], f["column"], f["rule"], f["message"]))


ESCAPED = finding("facet.value.not_permitted", GOVERNING, "ESCAPED-ON-REACHED", escape="suppression")
OLD_KEPT = finding("facet.required.missing", GOVERNING, "OLD-ON-REACHED")
OLD_GONE = finding("link.path.unresolved", GONE_GOVERNING, "OLD-ON-GONE-GOVERNOR")
OLD_FAR = finding("facet.value.not_permitted", FAR, "OLD-ON-UNREACHED")
NEW_FAR = finding("facet.value.not_permitted", FAR, "A-SHARED-MESSAGE")
OLD_SHARED = finding("section.required.missing", OTHER, "A-SHARED-MESSAGE")

K = "language.controlled.not_met"
DUP_BASE_40 = finding(K, DUP, "DUP-KEY", line=40)
DUP_NEW_12 = finding(K, DUP, "DUP-KEY", line=12)
DUP_OLD_40 = finding(K, DUP, "DUP-KEY", line=40)
MOVED_BASE = finding(K, MOVED, "MOVED-KEY", line=10)
MOVED_NOW = finding(K, MOVED, "MOVED-KEY", line=13)
EDITED_BASE = finding(K, EDITED, "EDITED-KEY", line=20)
EDITED_NOW = finding(K, EDITED, "EDITED-KEY", line=20)
RENAMED_BASE = finding(K, RENAMED_OLD, "RENAMED-KEY", line=7)
RENAMED_OLD_NOW = finding(K, RENAMED_NEW, "RENAMED-KEY", line=7)
RENAMED_NEW_NOW = finding(K, RENAMED_NEW, "RENAMED-KEY", line=31)
TWICE = finding("surface.local_path.instructed", TWICE_DOC, "RAISED-TWICE", line=46)

DIFF = f"""diff --git a/{DUP} b/{DUP}
--- a/{DUP}
+++ b/{DUP}
@@ -12 +12 @@
-old line twelve
+new line twelve
diff --git a/{MOVED} b/{MOVED}
--- a/{MOVED}
+++ b/{MOVED}
@@ -3,0 +4,3 @@
+one
+two
+three
diff --git a/{EDITED} b/{EDITED}
--- a/{EDITED}
+++ b/{EDITED}
@@ -20 +20 @@
-old line twenty
+new line twenty
diff --git a/{RENAMED_OLD} b/{RENAMED_NEW}
similarity index 97%
rename from {RENAMED_OLD}
rename to {RENAMED_NEW}
--- a/{RENAMED_OLD}
+++ b/{RENAMED_NEW}
@@ -29,0 +30,1 @@
+an inserted line
"""


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
    current = engine_order(
        [ESCAPED, OLD_KEPT, OLD_GONE, OLD_FAR, NEW_FAR, OLD_SHARED, DUP_NEW_12, DUP_OLD_40, MOVED_NOW, EDITED_NOW,
         RENAMED_OLD_NOW, RENAMED_NEW_NOW, TWICE, dict(TWICE)]
    )
    base = engine_order(
        [ESCAPED, OLD_KEPT, OLD_GONE, OLD_FAR, OLD_SHARED, DUP_BASE_40, MOVED_BASE, EDITED_BASE, RENAMED_BASE, TWICE]
    )
    with tempfile.TemporaryDirectory() as work:
        write(os.path.join(work, "changed.txt"), "src/a.rs\n")
        write(os.path.join(work, "gone.tsv"), "D\tsrc/gone.rs\n")
        write(os.path.join(work, "check.exit"), "1\n")
        write(os.path.join(work, "check.json"), json.dumps({"change": {"unmatched": ["src/a.rs"]}, "findings": current}))
        write(os.path.join(work, "base-check.json"), json.dumps({"findings": base}))
        write(os.path.join(work, "diff.patch"), DIFF)
        write(os.path.join(work, "routes.tsv"), "1\tsrc/a.rs\n")
        write(os.path.join(work, "route", "1.json"), json.dumps(route("src/a.rs", GOVERNING, "ACME-DR-0001")))
        write(os.path.join(work, "routes-base.tsv"), "1\tsrc/gone.rs\n")
        write(os.path.join(work, "route-base", "1.json"), json.dumps(route("src/gone.rs", GONE_GOVERNING, "ACME-DR-0002")))
        out = os.path.join(work, "report.md")
        subprocess.run([sys.executable, REPORT, work, out], check=True)
        text = open(out, encoding="utf-8").read()

    owed = text.split("## Owed\n", 1)[1].split("\n## ", 1)[0]
    listed = [l for l in owed.splitlines() if l.lstrip().startswith("- `docs/")]

    def at(path, line):
        return [l for l in listed if f"`{path}`:{line} " in l]

    def with_msg(marker):
        return [l for l in listed if marker in l]

    failed = []

    def expect_listed(case, path, line, new):
        hits = at(path, line)
        if len(hits) != 1:
            failed.append(f"{case}: {path}:{line} is listed {len(hits)} time(s), not once")
        elif new and "(new with this change)" not in hits[0]:
            failed.append(f"{case}: {path}:{line} is not marked new")
        elif not new and "(new with this change)" in hits[0]:
            failed.append(f"{case}: {path}:{line} is marked new, and it stood before the change")

    def expect_unlisted(case, path, line):
        if at(path, line):
            failed.append(f"{case}: {path}:{line} is listed, and it is an old finding on an unreached document")

    if with_msg("ESCAPED-ON-REACHED"):
        failed.append("escape: a suppressed finding is listed in Owed")
    if "1 finding(s) are not listed because a suppression" not in owed:
        failed.append("escape: the suppressed finding is not counted once")
    if with_msg("OLD-ON-UNREACHED"):
        failed.append("reach: an old finding on a document the change does not reach is listed")
    if [l for l in with_msg("A-SHARED-MESSAGE") if OTHER in l]:
        failed.append("collide: the old finding on another document that shares a message is listed")
    expect_listed("kept", GOVERNING, 5, new=False)
    expect_listed("gone", GONE_GOVERNING, 5, new=False)
    new_far = [l for l in with_msg("A-SHARED-MESSAGE") if FAR in l]
    if len(new_far) != 1 or "(new with this change)" not in new_far[0]:
        failed.append("collide: the finding the change introduced on the far document is not listed once, marked new")
    expect_listed("dup", DUP, 12, new=True)
    expect_unlisted("dup", DUP, 40)
    expect_unlisted("moved", MOVED, 13)
    expect_listed("edited", EDITED, 20, new=True)
    expect_unlisted("renamed", RENAMED_NEW, 7)
    expect_listed("renamed", RENAMED_NEW, 31, new=True)
    expect_listed("twice", TWICE_DOC, 46, new=True)
    # Old and elsewhere: OLD_FAR, OLD_SHARED, DUP 40, MOVED 13, RENAMED 7,
    # and one copy of TWICE.
    if "6 more finding(s) stood before this change" not in owed:
        failed.append("count: the old findings elsewhere are not counted as 6")

    for f in failed:
        print(f"FAIL {f}", file=sys.stderr)
    if failed:
        print(text, file=sys.stderr)
        return 1
    print("report-filters: escape, reach, kept, gone, collide, dup, moved, edited, renamed and twice each hold")
    return 0


if __name__ == "__main__":
    sys.exit(main())
