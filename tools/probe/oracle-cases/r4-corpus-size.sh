# The cases of the `r4-corpus-size` oracle, for the probe
# HW-PROBE-a-session-records-as-owed-the-corpus-size-axis-that-the-additive-campaign-defers
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
        --title "The additive campaign defers the corpus-size axis and nothing records it as owed" \
        --summary "The campaign measures one corpus of one size, so no result says whether a layer that helps here helps in a corpus ten times larger." \
        --facet waiting_on=measurement --relates traces_to=HW-DR-0112 > /dev/null
}

# The same record traced to HW-DR-0076, the decision that prices a probe run.
wrong() {
    "$HW_ORACLE_ENGINE" new obligation_record --root . \
        --title "The additive campaign defers the corpus-size axis and nothing records it as owed" \
        --summary "The campaign measures one corpus of one size." \
        --facet waiting_on=measurement --relates traces_to=HW-DR-0076 > /dev/null
}
