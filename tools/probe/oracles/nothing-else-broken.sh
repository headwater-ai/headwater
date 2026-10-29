#!/bin/sh
# Generic oracle: the tree carries no finding that the pin did not (#1874).
# Row 9 of the grid in HW-DR-0112, "repair a failing check".
#
#     sh tools/probe/oracles/nothing-else-broken.sh [--check-clean] <baseline>
#     sh tools/probe/oracles/nothing-else-broken.sh --record <baseline>
#
# A finding is the pair of its rule and its path, from `headwater check
# --json`, with every severity counted. The line and the message are not
# read, because a session's edit moves the line, and the message of some
# rules holds a count. It passes when no pair occurs more often in the tree
# than in <baseline>. So a repair that clears one finding and raises another,
# of a rule the strict check only advises on, fails here and passes
# check-clean.sh. A probe of row 9 passes `--check-clean` as well.
#
# <baseline> holds one finding a line, `<rule><TAB><path>`, sorted, and a
# line that opens with `#` is a comment. `--record` writes it from the tree it
# runs in, which is the pin, and prints the count. The grading copy holds no
# history, so the baseline of a probe is a file, and the probe's oracle
# `holds` it. Its first line is a comment that names the probe, so the seal
# removes it from every workspace.

set -u
. "${0%/*}/lib.sh"
record=no
if [ "${1:-}" = --record ]; then
    record=yes
    shift
fi
[ $# -eq 1 ] || hw_oracle_usage "nothing-else-broken.sh [--check-clean] <baseline> | --record <baseline>"
baseline=$1
command -v python3 > /dev/null 2>&1 || hw_oracle_usage "nothing-else-broken.sh needs python3 to read the findings"
hw_oracle_engine
hw_oracle_tmp
status=0
"$hw_engine" check --json --root . > "$hw_oracle_scratch/check.json" 2> "$hw_oracle_scratch/check.err" || status=$?
python3 -I -c '
import json, sys
findings = json.load(open(sys.argv[1], encoding="utf-8")).get("findings") or []
for f in findings:
    print("%s\t%s" % (f.get("rule", "-"), f.get("path", "-")))
' "$hw_oracle_scratch/check.json" 2> "$hw_oracle_scratch/parse.err" | LC_ALL=C sort > "$hw_oracle_scratch/now" || true
[ -s "$hw_oracle_scratch/parse.err" ] && hw_oracle_fail "\`headwater check --json\` exits $status and writes no findings that parse"

if [ "$record" = yes ]; then
    cp "$hw_oracle_scratch/now" "$baseline" || hw_oracle_usage "$baseline could not be written"
    hw_oracle_pass "recorded $(wc -l < "$baseline" | tr -d ' ') finding(s) in $baseline"
fi

[ -f "$baseline" ] || hw_oracle_fail "no baseline at $baseline"
grep -v '^#' "$baseline" | LC_ALL=C sort > "$hw_oracle_scratch/pin"
LC_ALL=C comm -13 "$hw_oracle_scratch/pin" "$hw_oracle_scratch/now" > "$hw_oracle_scratch/new"
raised=$(wc -l < "$hw_oracle_scratch/new" | tr -d ' ')
[ "$raised" -eq 0 ] || hw_oracle_fail "$raised finding(s) the pin did not carry, the first $(sed -n '1s/\t/ on /p' "$hw_oracle_scratch/new")"
hw_oracle_pass_clean "no finding the pin did not carry, $(wc -l < "$hw_oracle_scratch/now" | tr -d ' ') against $(wc -l < "$hw_oracle_scratch/pin" | tr -d ' ') at the pin"
