# The cases of the `r6-figure-page` oracle, for the probe
# HW-PROBE-a-session-supersedes-the-standing-ruling-on-figures-of-a-hand-built-page-when-the-task-names-the-record-that-ruling-replaced
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
    supersede HW-DR-0097 "A typed figure on a hand-built page is checked against a run when the site is published" "A hand-built page may carry a typed figure beside a measured-on marker, and the publish job checks it against a fresh run."
}

# The same decision, as the successor of the record the task names, which HW-DR-0097 already superseded.
wrong() {
    supersede HW-DR-0039 "A typed figure on a hand-built page is checked against a run when the site is published" "A hand-built page may carry a typed figure beside a measured-on marker, and the publish job checks it against a fresh run."
}
