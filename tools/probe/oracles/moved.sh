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

set -u
. "${0%/*}/lib.sh"
[ $# -eq 2 ] || hw_oracle_usage "moved.sh [--check-clean] <old-path> <new-path>"
old=${1#./} new=${2#./}
[ ! -e "$old" ] || hw_oracle_fail "$old is still there"
[ -f "$new" ] || hw_oracle_fail "nothing is at $new"
command -v python3 > /dev/null 2>&1 || hw_oracle_usage "moved.sh needs python3 to resolve links"
hw_oracle_tmp
python3 -I - "$old" > "$hw_oracle_scratch/stale" << 'PY'
import os
import re
import sys

old = os.path.normpath(sys.argv[1])
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
        for match in link.finditer(text):
            target = match.group(1).split("#", 1)[0]
            if not target or re.match(r"^[a-z][a-z0-9+.-]*:", target):
                continue
            if target.startswith("/"):
                resolved = os.path.normpath(target.lstrip("/"))
            else:
                resolved = os.path.normpath(os.path.join(os.path.dirname(path), target))
            if resolved == old:
                print(path)
                break
PY
stale=$(sort -u "$hw_oracle_scratch/stale" | wc -l | tr -d ' ')
[ "$stale" -eq 0 ] || hw_oracle_fail "$old moved to $new, and $stale file(s) still link to $old, the first $(sort -u "$hw_oracle_scratch/stale" | sed -n '1p')"
hw_oracle_pass_clean "$old moved to $new, and no link resolves to $old"
