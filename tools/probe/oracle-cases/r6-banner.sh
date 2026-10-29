# The cases of the `r6-banner` oracle, for the probe
# HW-PROBE-a-session-records-a-ruling-that-drops-the-help-screen-banner-as-the-settled-successor-of-the-record-the-task-names
# (#1874).
#
# `tools/probe/probe-oracle-fixtures.sh` sources this file and runs each
# function from the root of a copy of the grading tree, with the engine at
# $HW_ORACLE_ENGINE. This file states the answer, so it names its probe here
# and the seal removes it from every workspace. The two changes are those
# the probe's Key records under "The reference change and the wrong change,
# checked".
# Write a decision that supersedes <old> with the verb, settle it, and let
# `headwater check --fix` write the far half and the state on <old>, as the
# verb's own report says it does once the new document leaves `draft`.
supersede() {
    "$HW_ORACLE_ENGINE" new decision --root . --title "$2" --summary "$3" --relates "supersedes=$1" > "${TMPDIR:-/tmp}/hw-case-new.$$" || return 1
    successor=$(sed -n 's/^wrote //p' "${TMPDIR:-/tmp}/hw-case-new.$$")
    rm -f "${TMPDIR:-/tmp}/hw-case-new.$$"
    [ -f "$successor" ] || return 1
    [ "${4:-current}" = draft ] && return 0
    sed -i "s/^status: draft\$/status: current/; s/^TODO write this section\\.\$/$3/" "$successor"
    "$HW_ORACLE_ENGINE" check --fix --root . > /dev/null 2>&1 || true
}

# The sound change.
reference() {
    supersede HW-DR-0045 "No screen prints the masthead banner and color stays sensed per stream" "No screen prints the masthead banner, and color stays sensed per stream and off by --no-color or NO_COLOR."
}

# The same decision, left at `draft`, the state `headwater new` opens it at.
wrong() {
    supersede HW-DR-0045 "No screen prints the masthead banner and color stays sensed per stream" "No screen prints the masthead banner, and color stays sensed per stream and off by --no-color or NO_COLOR." draft
}

r6_old_id=HW-DR-0045

# The mutants of round 4 of #1874.3. Each one changes the text and leaves the
# structure wrong, so an oracle that reads lines of a file passes it and the
# engine does not (parent-slice-3-stop.md, mutants 2 and 3 applied to row 6).
r6_old() {
    "$HW_ORACLE_ENGINE" explain --root . "$r6_old_id" 2> /dev/null | sed -n '1p'
}

# r6_edit <file> <key-to-drop> <key-to-add> [flow]: move the items of the
# relation <key-to-drop> of <file> to the relation <key-to-add>, which is
# made when the front matter has none. With `flow`, the items go into a
# comment after an empty flow list of <key-to-add>.
r6_edit() {
    python3 -I - "$@" << 'PY'
import sys

path, drop, add = sys.argv[1:4]
flow = sys.argv[4:] == ["flow"]
lines = open(path, encoding="utf-8").read().split("\n")
end = lines.index("---", 1)
front, rest = lines[1:end], lines[end:]
items, kept, inside = [], [], False
for line in front:
    if line == "  " + drop + ":":
        inside = True
        continue
    if inside and line.startswith("    - "):
        items.append(line[6:])
        continue
    inside = False
    kept.append(line)
if not items:
    sys.exit("no item under " + drop + " in " + path)
if flow:
    at = kept.index("relations:") + 1
    kept[at:at] = ["  " + add + ": [] # " + " ".join(items)]
elif "  " + add + ":" in kept:
    at = kept.index("  " + add + ":") + 1
    kept[at:at] = ["    - " + item for item in items]
else:
    at = kept.index("relations:") + 1
    kept[at:at] = ["  " + add + ":"] + ["    - " + item for item in items]
open(path, "w", encoding="utf-8").write("\n".join(["---"] + kept + rest))
PY
}

# The successor's supersedes list rewritten as an empty flow list whose
# comment names the old record.
flow_commented() {
    reference && r6_edit "$successor" supersedes supersedes flow
}

# The old record names its successor under traces_to and not superseded_by.
traces_named() {
    reference && r6_edit "$(r6_old)" superseded_by traces_to
}

# Both at once, so neither end writes the succession and the strict check
# reads no half-written edge.
both_text() {
    reference && r6_edit "$successor" supersedes supersedes flow && r6_edit "$(r6_old)" superseded_by traces_to
}

# The two keys swapped: the successor says it is superseded by the old
# record, and the old record says it supersedes the successor.
reversed() {
    reference && r6_edit "$successor" supersedes superseded_by && r6_edit "$(r6_old)" superseded_by supersedes
}
more_wrong="flow_commented traces_named both_text reversed"
