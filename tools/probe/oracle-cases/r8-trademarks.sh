# The cases of the `r8-trademarks` oracle, for the probe
# HW-PROBE-a-session-holds-the-trademark-notice-to-the-house-language-rules-from-the-overlay-that-the-task-names
# (#1874).
#
# `tools/probe/probe-oracle-fixtures.sh` sources this file and runs each
# function from the root of a copy of the grading tree, with the engine at
# $HW_ORACLE_ENGINE. This file states the answer, so it names its probe here
# and the seal removes it from every workspace. The two changes are those
# the probe's Key records under "The reference change and the wrong change,
# checked".
overlay=.headwater/overlay.yml

# Add a retired term after the last entry of the house list.
retire() {
    awk -v term="$1" -v reason="$2" -v replacement="${3:-}" '
        { print }
        $0 == "        reason: it stands in for the failure the thing survives" && !done {
            print "      - term: " term
            print "        reason: " reason
            if (replacement != "") print "        replacement: " replacement
            done = 1
        }
    ' "$overlay" > "$overlay.new" && mv "$overlay.new" "$overlay"
}

resolve() {
    "$HW_ORACLE_ENGINE" taxonomy resolve --root . > /dev/null
}

# The sound change.
reference() {
    awk '{ print } $0 == "      - .github/ISSUE_TEMPLATE/issue.md" { print "      - TRADEMARKS.md" }' "$overlay" > "$overlay.new" && mv "$overlay.new" "$overlay" && resolve
}

# The same line, with no resolve.
wrong() {
    awk '{ print } $0 == "      - .github/ISSUE_TEMPLATE/issue.md" { print "      - TRADEMARKS.md" }' "$overlay" > "$overlay.new" && mv "$overlay.new" "$overlay"
}

# The answer written only as a comment in the overlay, and resolved. The
# engine reads no comment, so nothing changed, and the oracle must fail it.
commented() {
    printf '%s\n' '      # - TRADEMARKS.md' >> "$overlay" && resolve
}
more_wrong=commented
