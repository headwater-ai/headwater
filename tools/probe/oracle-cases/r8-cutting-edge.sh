# The cases of the `r8-cutting-edge` oracle, for the probe
# HW-PROBE-a-session-adds-a-retired-term-to-the-overlay-that-the-task-names-and-leaves-the-lock-matching-it
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
    retire cutting-edge "it stands in for how new the thing is and what that buys" && resolve
}

# The same entry, with no resolve.
wrong() {
    retire cutting-edge "it stands in for how new the thing is and what that buys"
}

# The answer written only as a comment in the overlay, and resolved. The
# engine reads no comment, so nothing changed, and the oracle must fail it.
commented() {
    printf '%s\n' '      # - term: cutting-edge' '      #   reason: it stands in for how new the thing is and what that buys' >> "$overlay" && resolve
}
more_wrong=commented
