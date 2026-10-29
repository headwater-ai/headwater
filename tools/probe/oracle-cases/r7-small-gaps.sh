# The cases of the `r7-small-gaps` oracle, for the probe
# HW-PROBE-a-session-moves-an-obligation-record-to-the-process-shelf-and-repairs-the-links-inside-it-as-well-as-the-links-to-it
# (#1874).
#
# `tools/probe/probe-oracle-fixtures.sh` sources this file and runs each
# function from the root of a copy of the grading tree, with the engine at
# $HW_ORACLE_ENGINE. This file states the answer, so it names its probe here
# and the seal removes it from every workspace. The two changes are those
# the probe's Key records under "The reference change and the wrong change,
# checked".
# Move <old> to <new>, and rewrite each Markdown link so that it resolves as
# it did: every link that resolved to <old>, and every relative link inside
# the moved file. A file named after <new> keeps its links as they were, and
# `--inside no` leaves the moved file's own links as they were.
relocate() {
    python3 -I - "$@" <<'PY'
import os
import re
import sys

args = sys.argv[1:]
inside = True
if args[:2] == ["--inside", "no"]:
    inside = False
    args = args[2:]
old, new = os.path.normpath(args[0]), os.path.normpath(args[1])
skip = {os.path.normpath(p) for p in args[2:]}
link = re.compile(r"(\]\(\s*<?)([^)\s>]+)(>?(?:\s+\"[^\"]*\")?\s*\))")
os.makedirs(os.path.dirname(new), exist_ok=True)
os.rename(old, new)


def rewrite(path, before, after, only_old):
    try:
        text = open(path, encoding="utf-8").read()
    except (OSError, UnicodeDecodeError):
        return

    def one(match):
        bare, hash_, frag = match.group(2).partition("#")
        if not bare or re.match(r"^[a-z][a-z0-9+.-]*:", bare) or bare.startswith("/"):
            return match.group(0)
        resolved = os.path.normpath(os.path.join(before, bare))
        if only_old and resolved != old:
            return match.group(0)
        if resolved == old:
            resolved = new
        return match.group(1) + os.path.relpath(resolved, after) + hash_ + frag + match.group(3)

    changed = link.sub(one, text)
    if changed != text:
        open(path, "w", encoding="utf-8").write(changed)


if inside:
    rewrite(new, os.path.dirname(old), os.path.dirname(new), False)
for top, dirs, files in os.walk("."):
    dirs[:] = [d for d in dirs if d not in (".git", "target", "node_modules")]
    for name in files:
        path = os.path.normpath(os.path.join(top, name))
        if name.endswith(".md") and path != new and path not in skip:
            rewrite(path, os.path.dirname(path), os.path.dirname(path), True)
PY
}

# The sound change.
reference() {
    relocate docs/obligations/0208-four-small-test-and-wording-gaps-from-run-20260923-0733-filed-together.md docs/process/obligations/0208-four-small-test-and-wording-gaps-from-run-20260923-0733-filed-together.md && "$HW_ORACLE_ENGINE" generate --root . > /dev/null
}

# The move with each link to the file repaired and the links inside it left as they were.
wrong() {
    relocate --inside no docs/obligations/0208-four-small-test-and-wording-gaps-from-run-20260923-0733-filed-together.md docs/process/obligations/0208-four-small-test-and-wording-gaps-from-run-20260923-0733-filed-together.md && "$HW_ORACLE_ENGINE" generate --root . > /dev/null
}
