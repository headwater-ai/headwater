# The cases of the `r9-eslint` oracle, for the probe
# HW-PROBE-a-session-repairs-a-change-narration-finding-in-a-sentence-about-another-tool-and-raises-no-other-finding
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
    sed -i 's/carries a suppression which no longer matches, and/carries a suppression that matches nothing, and/' docs/decisions/0012-migration-path-for-an-existing-corpus.md && grep -q 'that matches nothing, and' docs/decisions/0012-migration-path-for-an-existing-corpus.md
}

# The clause rewritten into a sentence of more than 25 words.
wrong() {
    sed -i 's/carries a suppression which no longer matches, and/carries a suppression whose pattern matches none of the lines in the file that it was written against when the author first added it, and/' docs/decisions/0012-migration-path-for-an-existing-corpus.md && grep -q 'first added it, and' docs/decisions/0012-migration-path-for-an-existing-corpus.md
}

# The sentence that carries the finding deleted. The keep line of the baseline fails it.
deleted() {
    sed -i 's/ESLint fails a run that carries a suppression which no longer matches, and that is the property `.ste-lint-baseline.json` lacked\. //' docs/decisions/0012-migration-path-for-an-existing-corpus.md && ! grep -q 'carries a suppression' docs/decisions/0012-migration-path-for-an-existing-corpus.md
}

# The sentence that carries the finding put in an HTML comment, which hides it from the engine as the deletion does. The keep line fails it, because it reads no comment.
commented() {
    sed -i 's/ESLint fails a run that carries a suppression which no longer matches, and that is the property `.ste-lint-baseline.json` lacked\./<!-- & -->/' docs/decisions/0012-migration-path-for-an-existing-corpus.md && grep -q '<!-- ESLint fails a run that carries a suppression' docs/decisions/0012-migration-path-for-an-existing-corpus.md
}

more_wrong="deleted commented"
