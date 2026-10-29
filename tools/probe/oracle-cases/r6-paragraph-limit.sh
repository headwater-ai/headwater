# The cases of the `r6-paragraph-limit` oracle, for the probe
# HW-PROBE-a-session-supersedes-the-standing-ruling-on-a-paragraph-limit-when-the-task-describes-the-ruling-that-it-replaced
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
    supersede HW-DR-0087 "An eight-sentence paragraph limit joins the language rule as an advisory defect" "The house language rule limits a paragraph to eight sentences, as an advisory defect."
}

# The same decision, as the successor of HW-DR-0069, the ruling the task describes, which HW-DR-0087 superseded.
wrong() {
    supersede HW-DR-0069 "An eight-sentence paragraph limit joins the language rule as an advisory defect" "The house language rule limits a paragraph to eight sentences, as an advisory defect."
}
