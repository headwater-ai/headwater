# The cases of the `r4-adopter-axis` oracle, for the probe
# HW-PROBE-a-session-records-as-owed-the-axis-of-an-adopter-s-own-repository-that-the-named-decision-defers
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
        --title "The additive campaign measures no adopter's own repository" \
        --summary "Every scenario of the campaign runs on this repository, so no result says whether a layer helps on a repository an adopter wrote." \
        --facet waiting_on=measurement --relates traces_to=HW-DR-0112 > /dev/null
}

# The same record traced to the register that lists it, and to nothing that
# rules the axis.
wrong() {
    "$HW_ORACLE_ENGINE" new obligation_record --root . \
        --title "The additive campaign measures no adopter's own repository" \
        --summary "Every scenario of the campaign runs on this repository." \
        --facet waiting_on=measurement --relates traces_to=HW-REG-open-obligations > /dev/null
}
