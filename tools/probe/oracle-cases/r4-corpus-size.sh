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

# The mutants of #1874.3b. Each one writes the reference record and cites
# HW-DR-0112 where the engine parses no edge, so a reader of the file's
# lines passes it and `typed-at.sh`, which reads the relation through
# `headwater explain --json`, does not.
#
# The record that reference() writes, found by a listing before and after.
reference_record() {
    hw_case_before=$(mktemp "${TMPDIR:-/tmp}/r4-case.XXXXXX") || return 1
    LC_ALL=C ls docs/obligations > "$hw_case_before"
    reference || return 1
    LC_ALL=C ls docs/obligations | LC_ALL=C comm -13 "$hw_case_before" - | sed 's#^#docs/obligations/#'
    rm -f "$hw_case_before"
}

# The reference record with its relation lines removed, and the ruling named
# in a line of the body.
body_only() {
    hw_case_file=$(reference_record) && [ -f "$hw_case_file" ] || return 1
    sed -i '/^relations:$/d; /^  traces_to:$/d; /^    - HW-DR-0112$/d' "$hw_case_file" && printf '\nThis record traces to HW-DR-0112.\n' >> "$hw_case_file" && ! grep -q '^    - HW-DR-0112$' "$hw_case_file"
}

# The reference record with its relation lines turned into comments.
commented_edge() {
    hw_case_file=$(reference_record) && [ -f "$hw_case_file" ] || return 1
    sed -i 's/^relations:$/# &/; s/^  traces_to:$/# &/; s/^    - HW-DR-0112$/# &/' "$hw_case_file" && grep -q '^#     - HW-DR-0112$' "$hw_case_file"
}
more_wrong="body_only commented_edge"
