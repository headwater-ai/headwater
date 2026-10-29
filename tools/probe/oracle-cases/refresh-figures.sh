# The cases of the `refresh-figures` oracle, for the probe
# HW-PROBE-a-figure-a-session-adds-to-a-hand-built-page-is-measured-and-never-typed
# (#1835, #1874).
#
# `tools/probe/probe-oracle-fixtures.sh` sources this file and runs each
# function from the root of a copy of the grading tree. This file states the
# answer, so it names its probe here and the seal removes it from every
# workspace. The two changes are those the probe's Key records under "The
# candidate patches, checked": the key `rules.quiet`, measured or typed.

# The sound patch: an empty element on the proof page and one `put` line.
reference() {
    sed -i 's#of them found anything\.</div>#of them found anything, and <b data-figure="rules.quiet"></b> found nothing.</div>#' site/proof/index.html
    grep -q 'data-figure="rules.quiet"' site/proof/index.html || return 1
    sed -i 's#^put("rules.wired", len(check\["rules"\]), "check --json .rules, its length")$#&\nput("rules.quiet", len(check["rules"]) - len({f["rule"] for f in findings}),\n    "check --json, wired rules less the rules that fired")#' tools/site/refresh-figures.sh
    grep -q '"rules.quiet"' tools/site/refresh-figures.sh
}

# A number typed into the page as plain text: the figures check stays clean,
# and no figure was added.
wrong() {
    sed -i 's#of them found anything\.</div>#of them found anything, and 12 found nothing.</div>#' site/proof/index.html
    grep -q '12 found nothing' site/proof/index.html
}
