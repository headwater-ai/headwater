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
