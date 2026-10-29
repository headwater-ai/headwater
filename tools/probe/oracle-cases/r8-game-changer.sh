# The cases of the `r8-game-changer` oracle, for the probe
# HW-PROBE-a-session-retires-a-stock-phrase-in-the-list-that-the-check-reads-and-resolves-the-lock
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
    retire game-changer "it stands in for the change the thing makes" && resolve
}

# The phrase added to the prose list in CLAUDE.md, which no check reads, and not to the overlay.
wrong() {
    sed -i 's/"robust" as filler\./"robust" as filler, "game-changer"./' CLAUDE.md && grep -q game-changer CLAUDE.md
}

# The answer written only as a comment in the overlay, and resolved. The
# engine reads no comment, so nothing changed, and the oracle must fail it.
commented() {
    printf '%s\n' '      # - term: game-changer' '      #   reason: it stands in for the change the thing makes' >> "$overlay" && resolve
}

# The mutants of round 4 of #1874.3. Each one writes the answer as text
# that is no entry of the list, and resolves, so a reader of lines passes it
# and the engine's own retired-term rule does not (parent-slice-3-stop.md,
# mutant 5).
#
# No entry for the term, and `term: game-changer` inside the quoted reason of
# another entry.
quoted() {
    sed -i 's/^        reason: it stands in for the failure the thing survives$/        reason: "it stands in for the failure the thing survives, term: game-changer"/' "$overlay" && grep -q 'term: game-changer' "$overlay" && resolve
}
more_wrong="commented quoted"
