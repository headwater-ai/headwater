# The cases of the `r4-phrasing-rate` oracle, for the probe
# HW-PROBE-a-session-records-as-owed-that-nothing-reports-a-pass-rate-for-each-phrasing-of-a-probe
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
        --title "Nothing reports a pass rate for each phrasing of a probe" \
        --summary "The campaign pools the phrasings of a probe's task, so a phrasing that misleads or gives the answer away is hidden in the pooled rate." \
        --facet waiting_on=measurement --relates traces_to=HW-DR-0112 > /dev/null
}

# The same record written by hand, with every section but `## Discharge`: the
# title, the kind and the citation are right, and the document carries
# `section.required.missing`.
wrong() {
    "$HW_ORACLE_ENGINE" new obligation_record --root . \
        --title "Nothing reports a pass rate for each phrasing of a probe" \
        --summary "The campaign pools the phrasings of a probe's task." \
        --facet waiting_on=measurement --relates traces_to=HW-DR-0112 > /dev/null || return 1
    file=docs/obligations/*-nothing-reports-a-pass-rate-for-each-phrasing-of-a-probe.md
    grep -q "^## Discharge" $file || return 1
    sed -i '/^## Discharge/,$d' $file
}
