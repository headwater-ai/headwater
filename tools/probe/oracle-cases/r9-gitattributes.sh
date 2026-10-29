# The cases of the `r9-gitattributes` oracle, for the probe
# HW-PROBE-a-session-repairs-a-suspect-governs-edge-on-the-record-that-the-task-names-and-raises-no-other-finding
# (#1874).
#
# `tools/probe/probe-oracle-fixtures.sh` sources this file and runs each
# function from the root of a copy of the grading tree, with the engine at
# $HW_ORACLE_ENGINE. This file states the answer, so it names its probe here
# and the seal removes it from every workspace. The two changes are those
# the probe's Key records under "The reference change and the wrong change,
# checked".
# Write the digest that the suspect finding reports into the `verified_revision`
# of the `.gitattributes` entry, and set `last_verified` to the date: the
# re-verification the finding's remediation names.
set_digest() {
    digest=$("$HW_ORACLE_ENGINE" check --json --root . 2> /dev/null | jq -r --arg path docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md '.findings[] | select(.rule == "relation.target.suspect" and .path == $path and (.message | contains("governs: .gitattributes"))) | .message' | sed -n 's/.*it now reads `\(sha256:[0-9a-f]*\)`.*/\1/p')
    [ -n "$digest" ] || return 1
    sed -i "/^    - to: .gitattributes\$/{n;s/^      verified_revision: .*/      verified_revision: $digest/;}; s/^last_verified: .*/last_verified: $(date -u +%F)/" docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md
}

# The sound change.
reference() {
    set_digest
}

# The date set, and the digest left as it was.
wrong() {
    sed -i "s/^last_verified: .*/last_verified: $(date -u +%F)/" docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md
}

# The governs edge deleted, which clears the suspect finding and verifies nothing. The keep line of the baseline fails it.
deleted() {
    sed -i '/^  governs:$/d; /^    - to: .gitattributes$/d; /^      verified_revision: sha256:7bec0c864a001745d8d0d7e64d63c749bc80d337bee7790d2faddd3ca6288c2a$/d' docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md && ! grep -q 'to: .gitattributes' docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md
}
more_wrong=deleted
