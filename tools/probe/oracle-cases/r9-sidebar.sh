# The cases of the `r9-sidebar` oracle, for the probe
# HW-PROBE-a-session-repairs-a-change-narration-finding-in-the-sidebar-decision-and-raises-no-other-finding
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
    sed -i 's/^The generated index of a shelf stays the first entry of its group, as before\.$/The generated index of a shelf is the first entry of its group./' docs/decisions/0108-the-site-sidebar-lists-a-shelf-in-path-order-and-the-shelf-index-keeps-the-reading-order.md && grep -q '^The generated index of a shelf is the first' docs/decisions/0108-the-site-sidebar-lists-a-shelf-in-path-order-and-the-shelf-index-keeps-the-reading-order.md
}

# The sentence rewritten with a contraction.
wrong() {
    sed -i "s/^The generated index of a shelf stays the first entry of its group, as before\.\$/The generated index of a shelf can't leave the first entry of its group./" docs/decisions/0108-the-site-sidebar-lists-a-shelf-in-path-order-and-the-shelf-index-keeps-the-reading-order.md && grep -q "can't leave" docs/decisions/0108-the-site-sidebar-lists-a-shelf-in-path-order-and-the-shelf-index-keeps-the-reading-order.md
}

# The sentence that carries the finding deleted. The keep line of the baseline fails it.
deleted() {
    sed -i '/^The generated index of a shelf stays the first entry of its group, as before\.$/d' docs/decisions/0108-the-site-sidebar-lists-a-shelf-in-path-order-and-the-shelf-index-keeps-the-reading-order.md && ! grep -q '^The generated index of a shelf' docs/decisions/0108-the-site-sidebar-lists-a-shelf-in-path-order-and-the-shelf-index-keeps-the-reading-order.md
}
more_wrong=deleted
