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
#
# A comment line `# scope: <path>` scopes the oracle to the files it names
# (#1874.3). Then only the findings on those files are compared, and
# `--check-clean` counts only the strict errors on them. A baseline of the
# whole tree moves with every finding that any change adds anywhere, so a
# probe's committed baseline would go stale with each document that lands;
# a probe scopes its baseline to the file the task repairs. `--record` keeps
# the comment lines of a baseline that is already there, its scope included,
# and records the findings of the scoped files alone.
#
# A comment line `# keep: <path> <literal>` fails the oracle unless <path>
# holds <literal> as a whole name, as `hw_oracle_names_whole` in `lib.sh`
# reads one (#1874.3). A session that deletes the sentence or the edge that
# carries the finding clears it and raises nothing, so a probe names the text
# that the repair keeps. A literal that only a comment holds is not kept, as
# `hw_oracle_uncommented` in `lib.sh` reads a comment, because a session can
# comment out an edge as well as delete it.

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

# The scope a baseline declares, one path a line, and the findings on it.
: > "$hw_oracle_scratch/comments"
[ -f "$baseline" ] && grep '^#' "$baseline" > "$hw_oracle_scratch/comments"
sed -n 's/^# scope: *//p' "$hw_oracle_scratch/comments" > "$hw_oracle_scratch/scope"
if [ -s "$hw_oracle_scratch/scope" ]; then
    awk -F '\t' 'NR == FNR { scope[$0] = 1; next } ($2 in scope)' "$hw_oracle_scratch/scope" "$hw_oracle_scratch/now" > "$hw_oracle_scratch/scoped"
    mv "$hw_oracle_scratch/scoped" "$hw_oracle_scratch/now"
    hw_oracle_subjects=$(tr '\n' ' ' < "$hw_oracle_scratch/scope")
fi

if [ "$record" = no ]; then
    # Each `# keep: <path> <literal>` line names text the repair keeps. A
    # session that deletes the sentence or the edge a finding is on clears
    # the finding and raises nothing, so without this it passes (#1874.3).
    sed -n 's/^# keep: *//p' "$hw_oracle_scratch/comments" > "$hw_oracle_scratch/keep"
    while IFS= read -r hw_keep; do
        hw_keep_path=${hw_keep%% *}
        hw_keep_literal=${hw_keep#* }
        [ -n "$hw_keep_path" ] && [ "$hw_keep_literal" != "$hw_keep" ] || hw_oracle_usage "a keep line is \`# keep: <path> <literal>\`, and $baseline has \`# keep: $hw_keep\`"
        [ -f "$hw_keep_path" ] || hw_oracle_fail "nothing is at $hw_keep_path, which the repair keeps"
        hw_oracle_uncommented "$hw_keep_path" "$hw_oracle_scratch/keep-source"
        hw_oracle_names_whole "$hw_keep_literal" "$hw_oracle_scratch/keep-source" || hw_oracle_fail "$hw_keep_path does not hold \`$hw_keep_literal\` outside a comment, so the session removed what the task asked it to repair"
    done < "$hw_oracle_scratch/keep"
fi

if [ "$record" = yes ]; then
    cat "$hw_oracle_scratch/comments" "$hw_oracle_scratch/now" > "$hw_oracle_scratch/record" && cp "$hw_oracle_scratch/record" "$baseline" || hw_oracle_usage "$baseline could not be written"
    hw_oracle_pass "recorded $(wc -l < "$hw_oracle_scratch/now" | tr -d ' ') finding(s) in $baseline"
fi

[ -f "$baseline" ] || hw_oracle_fail "no baseline at $baseline"
grep -v '^#' "$baseline" | LC_ALL=C sort > "$hw_oracle_scratch/pin"
LC_ALL=C comm -13 "$hw_oracle_scratch/pin" "$hw_oracle_scratch/now" > "$hw_oracle_scratch/new"
raised=$(wc -l < "$hw_oracle_scratch/new" | tr -d ' ')
[ "$raised" -eq 0 ] || hw_oracle_fail "$raised finding(s) the pin did not carry, the first $(sed -n '1s/\t/ on /p' "$hw_oracle_scratch/new")"
hw_oracle_pass_clean "no finding the pin did not carry, $(wc -l < "$hw_oracle_scratch/now" | tr -d ' ') against $(wc -l < "$hw_oracle_scratch/pin" | tr -d ' ') at the pin"
