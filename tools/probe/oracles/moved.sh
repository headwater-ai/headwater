#!/bin/sh
# Generic oracle: a file moved and every link to it was repaired (#1874).
# Row 7 of the grid in HW-DR-0112, "move or relocate a document or a shelf".
#
#     sh tools/probe/oracles/moved.sh [--check-clean] <old-path> <new-path>
#
# Each path is relative to the root. It passes when all three hold:
#
# 1. nothing is at <old-path>;
# 2. a file is at <new-path>;
# 3. no Markdown link in a `.md` file of the tree resolves to <old-path>. A
#    link resolves against the directory of the file that holds it, or
#    against the root when it opens with `/`, and its `#fragment` is dropped.
#
# For a shelf, name one document of the shelf, because a move of the shelf
# moves it. A link by identifier does not depend on the path, so it needs no
# repair and is not read. A probe of row 7 passes `--check-clean`.
#
# With `--check-clean`, the strict errors counted are every error on
# <new-path>, and each error that names the file name of either path on a
# `.md` file that holds a link whose last path segment is that file name,
# wherever the link resolves (#1874.3). So a link repaired to a path where
# nothing stands is read, and so is a broken relative link inside the moved
# file. The errors that a sealed tree holds elsewhere are not, and neither
# are the errors the seal made in a linking file, about its other links.

set -u
. "${0%/*}/lib.sh"
[ $# -eq 2 ] || hw_oracle_usage "moved.sh [--check-clean] <old-path> <new-path>"
old=${1#./} new=${2#./}
[ ! -e "$old" ] || hw_oracle_fail "$old is still there"
[ -f "$new" ] || hw_oracle_fail "nothing is at $new"
command -v python3 > /dev/null 2>&1 || hw_oracle_usage "moved.sh needs python3 to resolve links"
hw_oracle_tmp
python3 -I - "$old" "$new" > "$hw_oracle_scratch/links" << 'PY'
import os
import re
import sys

old = os.path.normpath(sys.argv[1])
names = {os.path.basename(old), os.path.basename(os.path.normpath(sys.argv[2]))}
link = re.compile(r"\]\(\s*<?([^)\s>]+)>?(?:\s+\"[^\"]*\")?\s*\)")
for top, dirs, files in os.walk("."):
    dirs[:] = [d for d in dirs if d not in (".git", "target", "node_modules")]
    for name in files:
        if not name.endswith(".md"):
            continue
        path = os.path.normpath(os.path.join(top, name))
        try:
            text = open(path, encoding="utf-8").read()
        except (OSError, UnicodeDecodeError):
            continue
        stale = near = False
        for match in link.finditer(text):
            target = match.group(1).split("#", 1)[0]
            if not target or re.match(r"^[a-z][a-z0-9+.-]*:", target):
                continue
            if target.startswith("/"):
                resolved = os.path.normpath(target.lstrip("/"))
            else:
                resolved = os.path.normpath(os.path.join(os.path.dirname(path), target))
            stale = stale or resolved == old
            near = near or os.path.basename(target) in names
        if stale:
            print("stale\t" + path)
        if near:
            print("near\t" + path)
PY
sed -n 's/^stale\t//p' "$hw_oracle_scratch/links" > "$hw_oracle_scratch/stale"
stale=$(sort -u "$hw_oracle_scratch/stale" | wc -l | tr -d ' ')
[ "$stale" -eq 0 ] || hw_oracle_fail "$old moved to $new, and $stale file(s) still link to $old, the first $(sort -u "$hw_oracle_scratch/stale" | sed -n '1p')"
hw_oracle_subjects=$new
hw_oracle_linkers=$(sed -n 's/^near\t//p' "$hw_oracle_scratch/links" | sort -u | tr '\n' ' ')
hw_oracle_mentions="${old##*/} ${new##*/}"
hw_oracle_pass_clean "$old moved to $new, and no link resolves to $old"
