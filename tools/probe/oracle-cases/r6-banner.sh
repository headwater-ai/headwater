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
