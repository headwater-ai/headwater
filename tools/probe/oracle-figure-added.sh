#!/bin/sh
# The hidden oracle of the probe
# a-figure-a-session-adds-to-a-hand-built-page-is-measured-and-never-typed
# (#1835).
#
#     sh tools/probe/oracle-figure-added.sh
#
# This file names that probe in its first lines, so `tools/probe/seal.sh`
# removes it from every workspace a session of the probe works in, and
# `tools/probe/grading-tree.sh` puts it back in the grading copy. The
# `refresh-figures` oracle under `oracles:` in `.headwater/probe.yml` holds it
# and runs it, from the root of the grading copy, with the `exit` parser.
#
# It exits 0 when two things hold, and 1 otherwise:
#
# 1. `sh tools/site/refresh-figures.sh --check` exits 0, so the pages carry
#    no typed figure, no key without a measurement, and no measurement that
#    reaches no page.
# 2. `site/proof/index.html` carries a figure key that the page at the pin
#    does not carry.
#
# The second half is what the figures check cannot see. A session that types
# the number into the page as plain text, with no figure element, leaves the
# figures check clean, and it did not add a figure. The keys of the pin are
# listed below rather than read from a tree, because the grading copy holds
# no history.

set -u

PIN_KEYS="census.checked census.excluded census.generated census.instances census.notdoc census.seen census.typed census.unaccounted census.untyped conformance.level emitters.built emitters.total emitters.waiting findings.advisory findings.directives findings.errors findings.raised findings.reported findings.suppressed oblig.gap oblig.nodisposition oblig.total oblig.unverifiable oblig.verified rules.fired rules.wired run.date taxonomy.lock taxonomy.package taxonomy.version verbs.contracts verbs.count verbs.groups verbs.nocontract"

sh tools/site/refresh-figures.sh --check || {
    echo "oracle-figure-added: the figures check refused the pages." >&2
    exit 1
}

page=site/proof/index.html
[ -f "$page" ] || {
    echo "oracle-figure-added: $page is gone." >&2
    exit 1
}
for key in $(grep -o 'data-figure="[^"]*"' "$page" | sed 's/^data-figure="//; s/"$//' | sort -u); do
    case " $PIN_KEYS " in
        *" $key "*) ;;
        *)
            echo "oracle-figure-added: $page carries the new figure $key, and the figures check measures it."
            exit 0
            ;;
    esac
done
echo "oracle-figure-added: $page carries no figure key the pin's page does not, so no figure was added." >&2
exit 1
