# The cases of the `r9-stale-comments` oracle, for the probe
# HW-PROBE-a-session-rewrites-the-change-narration-in-the-summary-of-the-obligation-on-eight-stale-comments-and-raises-no-other-finding
# (#1874).
#
# `tools/probe/probe-oracle-fixtures.sh` sources this file and runs each
# function from the root of a copy of the grading tree, with the engine at
# $HW_ORACLE_ENGINE. This file states the answer, so it names its probe here
# and the seal removes it from every workspace. The two changes are those
# the probe's Key records under "The reference change and the wrong change,
# checked".
target=docs/obligations/0230-comments-in-the-command-surface-crates-and-their-subsystem-spec-state-eight-stale-facts.md
pin='^summary: "Eight comments in cli, paint, generate, probe, sweep and the command-surface subsystem spec state facts the code no longer holds, found by the builds and verifies of run 20261001-1107."$'

# The sound change.
reference() {
    sed -i "s/$pin/summary: \"Eight comments in cli, paint, generate, probe, sweep and the command-surface subsystem spec state facts the code does not hold, found by the builds and verifies of run 20261001-1107.\"/" "$target" && grep -q 'the code does not hold' "$target"
}

# The narration rewritten with a contraction, which the language rule
# reports as an error.
wrong() {
    sed -i "s/$pin/summary: \"Eight comments in cli, paint, generate, probe, sweep and the command-surface subsystem spec state facts the code doesn't hold, found by the builds and verifies of run 20261001-1107.\"/" "$target" && grep -q "the code doesn't hold" "$target"
}

# The summary with its subject cut, which drops the eight comments it
# lists. The summary line of the baseline fails it.
deleted() {
    sed -i "s/$pin/summary: \"The command-surface subsystem spec states facts the code does not hold, found by the builds and verifies of run 20261001-1107.\"/" "$target" && ! grep -q 'Eight comments in cli' "$target"
}

# The subject cut, and kept as a comment in the front matter. The summary
# line fails it, because it reads no comment.
commented() {
    deleted && sed -i 's/^summary: "The command-surface subsystem spec states facts.*$/&\n# Eight comments in cli, paint, generate, probe, sweep/' "$target" && grep -q '^# Eight comments in cli, paint, generate, probe, sweep$' "$target"
}

# The mutants of round 4 of #1874.3 (parent-slice-3-stop.md, mutants 1 to 3
# applied to this probe). Each one keeps the text of the subject and takes
# it out of the summary, so a reader of lines passes it and the summary as
# the engine parses it does not.
#
# The subject cut, and appended as a line of the body.
body_line() {
    deleted && printf '\nEight comments in cli, paint, generate, probe, sweep and the subsystem spec state stale facts.\n' >> "$target"
}

# The whole summary commented out, and a summary without the subject written
# below it.
summary_commented() {
    sed -i "s/$pin/# &\nsummary: \"The command-surface subsystem spec states facts the code does not hold.\"/" "$target" && grep -q '^# summary: "Eight comments in cli' "$target"
}

# The subject cut, and moved into the title.
other_key() {
    deleted && sed -i 's/^title: "Comments in the command-surface crates and their subsystem spec state eight stale facts"$/title: "Eight comments in cli, paint, generate, probe, sweep and their subsystem spec state stale facts"/' "$target" && grep -q '^title: "Eight comments in cli, paint, generate, probe, sweep' "$target"
}

# The file carries a second change-narration finding, in a body sentence of
# its Context. A session that repairs that sentence and leaves the summary
# clears one finding of the same rule and path, so the count matches the
# baseline. The `# summary-not:` line of the baseline fails it, because the
# summary the engine parses still holds the narrated words (#1874.3b).
body_repaired() {
    sed -i 's/found eight comments that state facts the code no longer holds\./found eight comments that state facts the code does not hold./' "$target" && grep -q 'that state facts the code does not hold\.' "$target" && grep -q 'the code no longer holds, found by' "$target"
}

# The summary rewritten with the crate list dropped, as session
# b2/L2-additive-scenario-6-p9-r4 of phase B wrote it. It loses the five
# crates that the summary names, so it fails (#1978).
crates_dropped() {
    sed -i "s/$pin/summary: \"Eight comments in the command-surface crates and their subsystem spec state facts that mismatch the code, found by the builds and verifies of run 20261001-1107.\"/" "$target" && grep -q 'command-surface crates and their subsystem spec state facts that mismatch' "$target"
}
more_wrong="deleted commented body_line summary_commented other_key body_repaired crates_dropped"

# A paraphrase that keeps every fact of the summary in other words, as
# session b3/L2-additive-scenario-6-p9-r1 of phase B wrote it (#1978). The
# task asks the session to keep the meaning and not the words, so it must
# pass.
paraphrase() {
    sed -i "s/$pin/summary: \"Eight comments in the cli, paint, generate, probe and sweep crates, and the command-surface subsystem spec, state facts the code does not hold. The builds and verifies of run 20261001-1107 surfaced them.\"/" "$target" && grep -q 'The builds and verifies of run 20261001-1107 surfaced them' "$target"
}
more_sound="paraphrase"

# The reference change less one crate name, or less the count, so that only
# the `# summary-any:` line of that fact fails each one (#1978).
sweep_dropped() {
    sed -i "s/$pin/summary: \"Eight comments in cli, paint, generate, probe and the command-surface subsystem spec state facts the code does not hold, found by the builds and verifies of run 20261001-1107.\"/" "$target" && grep -q 'Eight comments in cli, paint, generate, probe and the' "$target"
}
count_dropped() {
    sed -i "s/$pin/summary: \"Comments in cli, paint, generate, probe, sweep and the command-surface subsystem spec state facts the code does not hold, found by the builds and verifies of run 20261001-1107.\"/" "$target" && grep -q 'summary: "Comments in cli, paint' "$target"
}
crates_less() {
    sed -i "s/$pin/summary: \"Eight comments in $1 and the command-surface subsystem spec state facts the code does not hold, found by the builds and verifies of run 20261001-1107.\"/" "$target" && grep -q -F "summary: \"Eight comments in $1 and the" "$target"
}
cli_dropped() {
    crates_less "paint, generate, probe, sweep"
}
paint_dropped() {
    crates_less "cli, generate, probe, sweep"
}
generate_dropped() {
    crates_less "cli, paint, probe, sweep"
}
probe_dropped() {
    crates_less "cli, paint, generate, sweep"
}
more_wrong="$more_wrong sweep_dropped count_dropped cli_dropped paint_dropped generate_dropped probe_dropped"
