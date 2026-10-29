# The cases of the `r4-hook-warning` oracle, for the probe
# HW-PROBE-a-session-records-as-owed-that-no-ruling-says-whether-a-hook-s-text-on-a-prompt-is-a-warning
# (#1874).
#
# `tools/probe/probe-oracle-fixtures.sh` sources this file and runs each
# function from the root of a copy of the grading tree, with the engine at
# $HW_ORACLE_ENGINE. This file states the answer, so it names its probe here
# and the seal removes it from every workspace. The two changes are those
# the probe's Key records under "The reference change and the wrong change,
# checked".

# The sound change: an obligation record from the verb, traced to the ruling.
reference() {
    "$HW_ORACLE_ENGINE" new obligation_record --root . \
        --title "No ruling says whether a hook's text on a prompt counts as a warning" \
        --summary "The no-warning action counts any text a hook prints on a prompt as a warning, and no ruling says whether the intent hook's routing text is one." \
        --facet waiting_on=measurement --relates traces_to=HW-DR-0112 > /dev/null
}

# The same record filed as a process obligation, on the shelf the build order
# keeps for its own tooling.
wrong() {
    "$HW_ORACLE_ENGINE" new process_obligation --root . \
        --title "No ruling says whether a hook's text on a prompt counts as a warning" \
        --summary "The no-warning action counts any text a hook prints on a prompt as a warning." \
        --facet waiting_on=measurement --relates traces_to=HW-DR-0112 > /dev/null
}
