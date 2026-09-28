#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
#
# Hold the line map of `upkeep-report.py` against diffs that git itself
# writes, and not against a patch written by hand. `report-filters.py` holds
# the filters with a synthetic patch. This fixture holds the three places a
# hand-written patch left open: where a hunk ends, whether an insertion moves
# the line it follows, and how far a rewrite with a different line count
# moves the lines below it. It also holds the names git writes in a way the
# parser must undo: a name with a space, which git ends with a TAB on the
# `---` and `+++` lines, a name that git quotes, and a name that git both
# quotes and ends with a TAB. It also holds a pure rename, which git writes
# with no `---` or `+++` line at all.
#
# It builds a temporary repository, one file per case, commits a base, makes
# the edits, commits them, and runs `git diff` with the flags `upkeep.sh`
# uses (read from `upkeep.sh` itself, so the two cannot drift). Every file
# holds the same rule and message at each line named, so only the line map
# can tell a finding that moved from one the change added.
#
# Each case names the base lines, the current lines, and the verdict on each
# current line: `old` when a base finding carried through the hunks lands on
# it, `new` when none does.
#
# Usage: python3 line-map.py [<upkeep-report.py>]
# The argument names the script to hold, and defaults to the one beside this
# directory. Exits 0 when every case holds. Writes only under a temporary
# directory it removes.
import importlib.util
import os
import re
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
UPKEEP = os.path.join(HERE, "..", "upkeep.sh")


def lines(n, tag="L"):
    return [f"{tag}{i}" for i in range(1, n + 1)]


# (name, base file, edit, base lines, current lines, verdicts)
# An edit is (first, last, replacement) over 1-based lines, as a splice: the
# lines first..last are replaced, and first..first-1 is an insertion before
# `first`.
CASES = [
    # -8,3 +8,4: 7 stands before the hunk, 8 and 10 are inside it, and 11
    # stands after it and moves down by 4 - 3 = 1.
    ("boundary", lines(20), (8, 10, ["x", "y", "z", "w"]), [7, 8, 10, 11], [7, 8, 11, 12],
     {7: "old", 8: "new", 11: "new", 12: "old"}),
    # -8,0 +9,2: an insertion after line 8 leaves 8 where it is and moves 9.
    ("ins8", lines(20), (9, 8, ["i1", "i2"]), [8, 9], [8, 11], {8: "old", 11: "old"}),
    # A rewrite of the last line.
    ("last", lines(20), (20, 20, ["zz"]), [19, 20], [19, 20], {19: "old", 20: "new"}),
    # -0,0 +1,2: an insertion before line 1.
    ("ins1", lines(20), (1, 0, ["t1", "t2"]), [1, 5], [3, 7], {3: "old", 7: "old"}),
    ("ins1stale", lines(20), (1, 0, ["t1"]), [1], [1], {1: "new"}),
    # -5 +4,0: a deletion above moves 10 up by one.
    ("delabove", lines(20), (5, 5, []), [10], [9], {9: "old"}),
    # -10,2 +10,2: line 11 is rewritten in place, so a finding there is new.
    ("reappear", lines(20), (10, 11, ["L11", "bad"]), [10], [11], {11: "new"}),
    # -10 +9,0: line 10 goes, 11 moves up to 10.
    ("dupbelow", lines(20), (10, 10, []), [10, 11], [10], {10: "old"}),
    # A file nothing edits keeps every line.
    ("untouched", lines(20), None, [3], [3, 15], {3: "old", 15: "new"}),
    # A name with a space: git writes `--- a/docs/with space.md<TAB>`.
    ("with space", lines(20), (1, 0, ["t"]), [5], [6], {6: "old"}),
    # A name git quotes: `"docs/\303\274ber.md"`.
    ("über", lines(20), (1, 0, ["t"]), [5], [6], {6: "old"}),
    # A name git both quotes and ends with a TAB, because it holds a space
    # and a byte above 0x7f: `--- "a/docs/esp\303\251ce x.md"<TAB>`.
    ("espéce x", lines(20), (1, 0, ["t"]), [5], [6], {6: "old"}),
]

# A rename with an insertion at 3 and a rewrite of old line 10: 2 stays, 5
# moves to 6, and 10 is rewritten, so a finding at its new place 11 is new.
RENAME_FROM, RENAME_TO = "docs/ren_a.md", "docs/ren_b.md"
RENAME = ([2, 5, 10], [2, 6, 11], {2: "old", 6: "old", 11: "new"})

# A pure rename, with no edit. Git writes only `rename from` and `rename to`
# for it, with no `---` or `+++` line and no hunk, so the parser must take
# the new name from `rename to`. Every line stands where it stood.
PURE_FROM, PURE_TO = "docs/pure_a.md", "docs/pure_b.md"
PURE = ([4, 9], [4, 9], {4: "old", 9: "old"})


def diff_flags():
    """The `git diff` arguments `upkeep.sh` passes when it writes diff.patch."""
    text = open(UPKEEP, encoding="utf-8").read()
    m = re.search(r'git diff (-U0 [^"]*?)"\$base"\s*\)\s*>"\$work/diff\.patch"', text)
    if not m:
        sys.exit("line-map: upkeep.sh no longer writes diff.patch with `git diff -U0 ...`; update this fixture")
    return m.group(1).split()


def git(repo, *args):
    subprocess.run(["git", "-C", repo, *args], check=True, capture_output=True)


def write(repo, path, content):
    full = os.path.join(repo, path)
    os.makedirs(os.path.dirname(full), exist_ok=True)
    with open(full, "w", encoding="utf-8") as f:
        f.write("\n".join(content) + "\n")


def load(report):
    # Loading a module by path writes `__pycache__` beside it unless told
    # not to, and this fixture writes nothing outside its temporary directory.
    sys.dont_write_bytecode = True
    spec = importlib.util.spec_from_file_location("upkeep_report", report)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def finding(path, line):
    return {"rule": "r", "path": path, "line": line, "column": 1, "message": "M", "escape": "none"}


def main():
    report = sys.argv[1] if len(sys.argv) > 1 else os.path.join(HERE, "..", "upkeep-report.py")
    module = load(report)
    with tempfile.TemporaryDirectory() as repo:
        git(repo, "init", "-q")
        for name, base, _, _, _, _ in CASES:
            write(repo, f"docs/{name}.md", base)
        write(repo, RENAME_FROM, lines(30))
        write(repo, PURE_FROM, lines(12, "P"))
        git(repo, "add", "-A")
        git(repo, "-c", "user.name=f", "-c", "user.email=f@x", "commit", "-q", "-m", "base")
        for name, base, edit, _, _, _ in CASES:
            if edit is None:
                continue
            first, last, replacement = edit
            content = list(base)
            content[first - 1:last] = replacement
            write(repo, f"docs/{name}.md", content)
        moved = lines(30)
        moved[9] = "rewritten"
        moved.insert(2, "ins")
        os.remove(os.path.join(repo, RENAME_FROM))
        write(repo, RENAME_TO, moved)
        git(repo, "mv", PURE_FROM, PURE_TO)
        git(repo, "add", "-A")
        git(repo, "-c", "user.name=f", "-c", "user.email=f@x", "commit", "-q", "-m", "edit")
        patch = subprocess.run(
            ["git", "-C", repo, "diff", *diff_flags(), "HEAD~1"], check=True, capture_output=True, text=True
        ).stdout
    diff = module.parse_diff(patch)

    failed = []

    def hold(label, base_path, cur_path, base_lines, cur_lines, want):
        base = [finding(base_path, l) for l in base_lines]
        current = [finding(cur_path, l) for l in cur_lines]
        listed, elsewhere, _ = module.split_findings(current, base, set(), diff)
        got = {f["line"]: ("new" if is_new else "old") for f, is_new in listed}
        got.update({f["line"]: "old" for f in elsewhere})
        if got != want:
            failed.append(f"{label}: got {sorted(got.items())}, want {sorted(want.items())}")

    for name, _, _, base_lines, cur_lines, want in CASES:
        path = f"docs/{name}.md"
        hold(name, path, path, base_lines, cur_lines, want)
    hold("rename", RENAME_FROM, RENAME_TO, *RENAME)
    if f"rename from {PURE_FROM}" not in patch or f"+++ b/{PURE_TO}" in patch:
        failed.append(f"pure rename: git did not write {PURE_FROM} as a rename with no `+++` line")
    hold("pure rename", PURE_FROM, PURE_TO, *PURE)

    for f in failed:
        print(f"FAIL {f}", file=sys.stderr)
    if failed:
        print("parsed paths: " + ", ".join(repr(k) for k in sorted(diff)), file=sys.stderr)
        return 1
    print(f"line-map: {len(CASES) + 2} cases hold against diffs git wrote")
    return 0


if __name__ == "__main__":
    sys.exit(main())
