#!/bin/sh
# What holds `tools/check-site-fragments.py`.
#
# Spec 12 refuses a check that ships with no failing fixture, and the reason
# carries to a gate: a gate that has refused nothing is a gate nobody has seen
# work. Every refusal below is provoked on purpose, and so is every pass,
# because a gate that refuses everything is as useless as one that refuses
# nothing. `.githooks/fixtures.sh` is the shape.
#
# Run it from anywhere:
#     sh tools/site-fragments-fixtures.sh
#
# # THE ASSUMPTION IT STATES, AND FAILS ON
#
# Half of these cases run over a scratch copy of the real assembled site, so
# they need one. This states that and stops when there is none, rather than
# reporting a row of passes over an empty directory. That is the same posture
# the tool itself takes when it is pointed at a root with no page in it.
#
# # WHAT IT WRITES
#
# Nothing inside this checkout. Every case runs under `mktemp -d`, and the
# directory goes on an interrupt.

set -u

root=$(cd "$(dirname "$0")/.." && pwd)
tool="$root/tools/check-site-fragments.py"
deploy="$root/.headwater/site-deploy"

if [ ! -f "$tool" ]; then
    echo "no checker at \`tools/check-site-fragments.py\`." >&2
    exit 1
fi

if [ ! -d "$deploy" ]; then
    echo "no assembled site at \`.headwater/site-deploy\`, so the cases over the" >&2
    echo "  real corpus cannot run. Build and compose it first:" >&2
    echo "        python3 -m mkdocs build --strict" >&2
    echo "        sh tools/assemble-site.sh" >&2
    exit 1
fi

scratch=$(mktemp -d) || exit 1
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

passed=0
failed=0

# report NAME EXPECTED-STATUS ACTUAL-STATUS [SUBSTRING-THAT-MUST-APPEAR FILE]
report() {
    name=$1
    want=$2
    got=$3
    ok=yes
    if [ "$want" != "$got" ]; then
        ok=no
        why="expected exit $want, got $got"
    fi
    if [ "$ok" = yes ] && [ $# -ge 5 ] && [ -n "$4" ]; then
        if ! grep -qF -- "$4" "$5"; then
            ok=no
            why="exit $got as expected, but the report never said: $4"
        fi
    fi
    if [ "$ok" = yes ]; then
        passed=$((passed + 1))
        echo "  ok    $name"
    else
        failed=$((failed + 1))
        echo "  FAIL  $name"
        echo "          $why"
    fi
}

run() {
    # run DIR [extra args...] -> writes $scratch/out, echoes the status
    dir=$1
    shift
    ( cd "$root" && python3 "$tool" "$dir" "$@" >"$scratch/out" 2>"$scratch/err" )
    echo $?
}

echo "over the assembled site"

# 1. The corpus as it stands. This is the number the gate protects, and a
#    failure here is a real dead fragment rather than a broken fixture.
status=$(run "$deploy")
report "the served site has no dead fragment" 0 "$status" "0 dead fragments" "$scratch/out"

# 2. The denominator is stated and is not zero. A run that checked nothing
#    would print `0 dead fragments` too, and that is the pass this suite must
#    not accept on the corpus's behalf.
if grep -qE 'out of [1-9][0-9]* in-site fragment links across [1-9][0-9]* served pages' "$scratch/out"; then
    passed=$((passed + 1)); echo "  ok    the run states a non-zero denominator"
else
    failed=$((failed + 1)); echo "  FAIL  the run states a non-zero denominator"
    echo "          it reported: $(tail -1 "$scratch/out")"
fi

echo "provoked refusals"

# A scratch copy of two real pages, small enough to doctor by hand. Any page
# with a heading anchor will do, so the pair is found rather than named.
copy="$scratch/site"
mkdir -p "$copy"
cp -R "$deploy"/. "$copy"/ 2>/dev/null

victim="$copy/spec/09-decisions/index.html"
if [ ! -f "$victim" ]; then
    echo "  FAIL  the fixture page \`spec/09-decisions/index.html\` is not in the" >&2
    echo "        assembled site, so the doctored cases below cannot run. This" >&2
    echo "        suite names one page on purpose: if the specification moves," >&2
    echo "        this line is the notice." >&2
    exit 1
fi

# 3. Break an anchor the corpus links to from elsewhere: rename the `id` that
#    `#q10--naming` resolves against. The href is untouched, so this is the
#    drift a renamed heading produces.
python3 - "$victim" <<'PY'
import sys
p = sys.argv[1]
s = open(p, encoding="utf-8").read()
assert 'id="q10--naming"' in s, "the fixture anchor is gone from the built page"
open(p, "w", encoding="utf-8").write(s.replace('id="q10--naming"', 'id="q10-renamed"', 1))
PY
status=$(run "$copy")
report "a renamed anchor fails the run" 1 "$status" "q10--naming" "$scratch/out"
report "  and it names the page that carries the dead link" 1 "$status" ".html:" "$scratch/out"
report "  and it names the page that lost the anchor" 1 "$status" "carries no id" "$scratch/out"
report "  and it names the source line a person edits" 1 "$status" "docs/" "$scratch/out"

# 4. Break the other side: an href that names an anchor nothing carries. Same
#    defect from the citing document's end.
cp -R "$deploy"/. "$copy"/
python3 - "$copy/spec/09-decisions/index.html" <<'PY'
import sys
p = sys.argv[1]
s = open(p, encoding="utf-8").read()
assert 'href="#q10--naming"' in s
open(p, "w", encoding="utf-8").write(
    s.replace('href="#q10--naming"', 'href="#q10--no-such-heading"', 1))
PY
status=$(run "$copy")
report "a same-page href to no anchor fails the run" 1 "$status" "q10--no-such-heading" "$scratch/out"

# 5. A cross-page href to no anchor, which is the exact shape of all 335
#    findings #431 recorded.
cp -R "$deploy"/. "$copy"/
python3 - "$copy/spec/09-decisions/index.html" <<'PY'
import sys
p = sys.argv[1]
s = open(p, encoding="utf-8").read()
s = s.replace('</body>', '<a href="../06-engine-architecture/#no-such-anchor">x</a></body>', 1)
open(p, "w", encoding="utf-8").write(s)
PY
status=$(run "$copy")
report "a cross-page href to no anchor fails the run" 1 "$status" "no-such-anchor" "$scratch/out"

echo "what must not fire"

# 6. An external URL carrying a fragment is somebody else's page.
cp -R "$deploy"/. "$copy"/
python3 - "$copy/spec/09-decisions/index.html" <<'PY'
import sys
p = sys.argv[1]
s = open(p, encoding="utf-8").read()
s = s.replace('</body>', '<a href="https://example.invalid/x#nope">x</a></body>', 1)
open(p, "w", encoding="utf-8").write(s)
PY
status=$(run "$copy")
report "an external fragment is not checked" 0 "$status" "0 dead fragments" "$scratch/out"

# 7. The pre-HTML5 `<a name>` anchor is a target a browser honours, so a
#    checker that read only `id` would report a link that in fact resolves.
cp -R "$deploy"/. "$copy"/
python3 - "$copy/spec/09-decisions/index.html" <<'PY'
import sys
p = sys.argv[1]
s = open(p, encoding="utf-8").read()
s = s.replace('</body>',
              '<a name="legacy-anchor"></a><a href="#legacy-anchor">x</a></body>', 1)
open(p, "w", encoding="utf-8").write(s)
PY
status=$(run "$copy")
report "an \`<a name>\` anchor resolves" 0 "$status" "0 dead fragments" "$scratch/out"

# 8. A percent-encoded fragment names the same anchor a browser resolves it to.
cp -R "$deploy"/. "$copy"/
python3 - "$copy/spec/09-decisions/index.html" <<'PY'
import sys
p = sys.argv[1]
s = open(p, encoding="utf-8").read()
s = s.replace('</body>', '<a href="#q10%2d%2dnaming">x</a></body>', 1)
open(p, "w", encoding="utf-8").write(s)
PY
status=$(run "$copy")
report "a percent-encoded fragment resolves" 0 "$status" "0 dead fragments" "$scratch/out"

# 9. A fragment is compared exactly. #208 records that this corpus's own rule
#    folds case where a browser does not, and this checker must not repeat it.
cp -R "$deploy"/. "$copy"/
python3 - "$copy/spec/09-decisions/index.html" <<'PY'
import sys
p = sys.argv[1]
s = open(p, encoding="utf-8").read()
s = s.replace('</body>', '<a href="#Q10--Naming">x</a></body>', 1)
open(p, "w", encoding="utf-8").write(s)
PY
status=$(run "$copy")
report "a fragment differing only in case fails, as a browser reads it" 1 "$status" "Q10--Naming" "$scratch/out"

echo "the path half, which reports and does not fail"

# 10. A dead path is a different defect and #406 owns it. It reports.
cp -R "$deploy"/. "$copy"/
python3 - "$copy/spec/09-decisions/index.html" <<'PY'
import sys
p = sys.argv[1]
s = open(p, encoding="utf-8").read()
s = s.replace('</body>', '<a href="../no-such-page/#x">x</a></body>', 1)
open(p, "w", encoding="utf-8").write(s)
PY
status=$(run "$copy")
report "a dead path reports without failing" 0 "$status" "resolves to no file" "$scratch/out"
status=$(run "$copy" --strict-paths)
report "  and \`--strict-paths\` fails on it" 1 "$status" "resolves to no file" "$scratch/out"

echo "the assumption this suite and the tool both state"

# 11. An empty directory is not a pass. This is the case that separates
#     "nothing is wrong" from "nothing ran", and it is the reason the tool
#     exits 2 rather than 0 over one.
empty="$scratch/empty"
mkdir -p "$empty"
status=$(run "$empty")
report "an empty root exits 2, not 0" 2 "$status" "checked nothing" "$scratch/err"

# 12. A missing directory names what writes one.
status=$(run "$scratch/absent")
report "a missing root exits 2 and names \`assemble-site.sh\`" 2 "$status" "assemble-site.sh" "$scratch/err"

# 13. An unknown flag is refused rather than ignored.
status=$(run "$deploy" --not-a-flag)
report "an unknown argument exits 2" 2 "$status" "unknown argument" "$scratch/err"

echo
echo "$passed passed, $failed failed"
[ "$failed" -eq 0 ]
