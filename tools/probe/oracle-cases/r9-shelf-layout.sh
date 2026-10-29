# The cases of the `r9-shelf-layout` oracle, for the probe
# HW-PROBE-a-session-splits-the-long-summary-of-the-obligation-that-the-task-names-and-raises-no-other-finding
# (#1874).
#
# `tools/probe/probe-oracle-fixtures.sh` sources this file and runs each
# function from the root of a copy of the grading tree, with the engine at
# $HW_ORACLE_ENGINE. This file states the answer, so it names its probe here
# and the seal removes it from every workspace. The two changes are those
# the probe's Key records under "The reference change and the wrong change,
# checked".
# The sound change.
reference() {
    sed -i 's/^summary: "A shelf layout is read by the scaffolder and by nothing else, and a rule that read it would report every document that a person renamed or typed by hand."$/summary: "A shelf layout is read by the scaffolder and by nothing else. A rule that read it would report every document that a person renamed or typed by hand."/' docs/obligations/0106-a-shelf-layout-names-a-file-at-birth-and-no-rule-reads-it.md && grep -q 'nothing else. A rule' docs/obligations/0106-a-shelf-layout-names-a-file-at-birth-and-no-rule-reads-it.md
}

# The summary split, with a second sentence that writes "no longer".
wrong() {
    sed -i 's/^summary: "A shelf layout is read by the scaffolder and by nothing else, and a rule that read it would report every document that a person renamed or typed by hand."$/summary: "A shelf layout is read by the scaffolder and by nothing else. A rule that read it would report every document that no longer has the name the scaffolder gave it."/' docs/obligations/0106-a-shelf-layout-names-a-file-at-birth-and-no-rule-reads-it.md && grep -q 'nothing else. A rule' docs/obligations/0106-a-shelf-layout-names-a-file-at-birth-and-no-rule-reads-it.md
}

# The summary cut to its first clause, which drops what the second one says. The keep line of the baseline fails it.
deleted() {
    sed -i 's/^summary: "A shelf layout is read by the scaffolder and by nothing else, and a rule that read it would report every document that a person renamed or typed by hand."$/summary: "A shelf layout is read by the scaffolder and by nothing else."/' docs/obligations/0106-a-shelf-layout-names-a-file-at-birth-and-no-rule-reads-it.md && ! grep -q 'would report every document' docs/obligations/0106-a-shelf-layout-names-a-file-at-birth-and-no-rule-reads-it.md
}

# The summary cut to its first clause, and the clause it drops kept as a comment in the front matter. The keep line fails it, because it reads no comment.
commented() {
    deleted && sed -i 's/^summary: "A shelf layout is read by the scaffolder and by nothing else."$/&\n# and a rule that read it would report every document that a person renamed or typed by hand/' docs/obligations/0106-a-shelf-layout-names-a-file-at-birth-and-no-rule-reads-it.md && grep -q '^# and a rule that read it would report every document' docs/obligations/0106-a-shelf-layout-names-a-file-at-birth-and-no-rule-reads-it.md
}


# The mutants of round 4 of #1874.3 (parent-slice-3-stop.md, mutants 1 to 3
# applied to this probe). Each one keeps the text of the clause and takes it
# out of the summary, so a reader of lines passes it and the summary as the
# engine parses it does not.
#
# The summary cut to its first clause, and the clause appended as a line of
# the body.
body_line() {
    deleted && printf '\nA rule that read it would report every document that a person renamed or typed by hand.\n' >> docs/obligations/0106-a-shelf-layout-names-a-file-at-birth-and-no-rule-reads-it.md
}

# The whole summary commented out, and a shorter summary without the clause
# written below it.
summary_commented() {
    sed -i 's/^summary: "A shelf layout is read by the scaffolder and by nothing else, and a rule that read it would report every document that a person renamed or typed by hand."$/# &\nsummary: "A shelf layout is read by the scaffolder and by nothing else."/' docs/obligations/0106-a-shelf-layout-names-a-file-at-birth-and-no-rule-reads-it.md && grep -q '^# summary: ' docs/obligations/0106-a-shelf-layout-names-a-file-at-birth-and-no-rule-reads-it.md
}

# The summary cut to its first clause, and the clause moved into the title.
other_key() {
    deleted && sed -i 's/^title: "A shelf layout names a file at birth and no rule reads it"$/title: "A shelf layout names a file at birth, and a rule that read it would report every document that a person renamed"/' docs/obligations/0106-a-shelf-layout-names-a-file-at-birth-and-no-rule-reads-it.md && grep -q '^title: .*would report every document' docs/obligations/0106-a-shelf-layout-names-a-file-at-birth-and-no-rule-reads-it.md
}
more_wrong="deleted commented body_line summary_commented other_key"
