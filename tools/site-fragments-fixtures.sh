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
#    failure here is a real defect rather than a broken fixture. The tool
#    makes two passes over one walk and fails on either, so this one status
#    covers both and the two cases under it say which pass produced it.
status=$(run "$deploy")
report "the served site passes every pass this tool makes" 0 "$status"
report "  no fragment on it is dead" 0 "$status" "0 dead fragments" "$scratch/out"
report "  no title on it is carried twice" 0 "$status" \
    "0 repeated titles and 0 pages with no title" "$scratch/out"
report "  no shelf index is served under a label the lock does not declare" 0 "$status" \
    "0 shelf labels not what the lock declares" "$scratch/out"

# 2. The denominator is stated and is not zero, on both passes. A run that
#    checked nothing would print `0 dead fragments` too, and that is the pass
#    this suite must not accept on the corpus's behalf.
if grep -qE 'out of [1-9][0-9]* in-site fragment links across [1-9][0-9]* served pages' "$scratch/out"; then
    passed=$((passed + 1)); echo "  ok    the fragment pass states a non-zero denominator"
else
    failed=$((failed + 1)); echo "  FAIL  the fragment pass states a non-zero denominator"
    echo "          it reported: $(grep -c '' "$scratch/out") lines, last: $(tail -1 "$scratch/out")"
fi
if grep -qE 'out of [1-9][0-9]* distinct titles across [1-9][0-9]* served pages' "$scratch/out"; then
    passed=$((passed + 1)); echo "  ok    the title pass states a non-zero denominator"
else
    failed=$((failed + 1)); echo "  FAIL  the title pass states a non-zero denominator"
    echo "          it reported: $(tail -1 "$scratch/out")"
fi
if grep -qE 'out of [1-9][0-9]* shelf index pages checked' "$scratch/out"; then
    passed=$((passed + 1)); echo "  ok    the shelf pass states a non-zero denominator"
else
    failed=$((failed + 1)); echo "  FAIL  the shelf pass states a non-zero denominator"
    echo "          it reported: $(tail -1 "$scratch/out")"
fi
if grep -qE '\([1-9][0-9]* shel(f|ves) skipped as holding no document' "$scratch/out"; then
    passed=$((passed + 1)); echo "  ok    a shelf with no document is skipped and counted"
else
    failed=$((failed + 1)); echo "  FAIL  a shelf with no document is skipped and counted"
    echo "          this corpus declares three such shelves, and a run that"
    echo "          dropped them silently would say so here. It reported:"
    echo "          $(tail -1 "$scratch/out")"
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

echo "the title half, which #567 added"

# The pages below are written rather than doctored from the real site. A dead
# fragment is a property of one page and can be provoked inside a copy of one;
# a repeated title is a property of a *set* of pages, and the assembled site
# carries one shape of that set at a time. So the set is built here on purpose,
# which is the same posture every case above takes toward its own defect.
titles="$scratch/titles"

page() {
    # page RELATIVE-PATH HEAD-CONTENT
    mkdir -p "$(dirname "$titles/$1")"
    printf '<html><head>%s</head><body><h1>x</h1></body></html>\n' "$2" >"$titles/$1"
}

# 14. Two pages carrying one title. This is #567 in miniature: ten shelf index
#     pages served `Index - Headwater` between them, so a browser tab, a
#     bookmark and a search result could not tell any of them apart.
rm -rf "$titles"
page a/index.html '<title>Same - Headwater</title>'
page b/index.html '<title>Same - Headwater</title>'
page c/index.html '<title>Other - Headwater</title>'
status=$(run "$titles" --no-lock)
report "a title carried by two pages fails the run" 1 "$status" "of 2 served pages" "$scratch/out"
report "  and it names the first of them" 1 "$status" "a/index.html" "$scratch/out"
report "  and it names the second of them" 1 "$status" "b/index.html" "$scratch/out"
report "  and it names the string they share" 1 "$status" "Same - Headwater" "$scratch/out"
report "  and it counts one repetition, not two" 1 "$status" "1 repeated title and" "$scratch/out"

# 15. A page with no title element at all.
rm -rf "$titles"
page a/index.html '<meta charset="utf-8">'
page b/index.html '<title>B - Headwater</title>'
status=$(run "$titles" --no-lock)
report "a page with no title fails the run" 1 "$status" "a/index.html: carries no" "$scratch/out"

# 16. An empty title element is untitled. A reader gets the same nothing from
#     it that a missing element gives, so counting it as a distinct title
#     would let the defect through under a value no page displays.
rm -rf "$titles"
page a/index.html '<title></title>'
page b/index.html '<title>B - Headwater</title>'
status=$(run "$titles" --no-lock)
report "an empty title element is untitled" 1 "$status" "a/index.html: carries no" "$scratch/out"

# 17. And so is a whitespace-only one.
rm -rf "$titles"
page a/index.html '<title>   </title>'
page b/index.html '<title>B - Headwater</title>'
status=$(run "$titles" --no-lock)
report "a whitespace-only title element is untitled" 1 "$status" "a/index.html: carries no" "$scratch/out"

# 18. An inline icon may carry a `<title>` as its accessible name. A reader
#     that took the first `<title>` in document order would read that icon's
#     label as the page's title, so an untitled page would report as titled
#     under a string it never displays, and two pages sharing one icon would
#     report as a collision on it. The tool skips a `<title>` inside `<svg>`,
#     and this case fails without that skip.
rm -rf "$titles"
mkdir -p "$titles/a" "$titles/b"
printf '<html><head><meta charset="utf-8"></head><body><svg><title>menu</title></svg></body></html>\n' >"$titles/a/index.html"
printf '<html><head><title>B - Headwater</title></head><body><svg><title>menu</title></svg></body></html>\n' >"$titles/b/index.html"
status=$(run "$titles" --no-lock)
report "an svg icon's title is not the page's title" 1 "$status" "a/index.html: carries no" "$scratch/out"

# 19. Distinct titles throughout, over a stated denominator that is not zero.
#     A suite of refusals alone cannot tell a working gate from one that
#     refuses everything.
rm -rf "$titles"
page a/index.html '<title>A - Headwater</title>'
page b/index.html '<title>B - Headwater</title>'
status=$(run "$titles" --no-lock)
report "distinct titles throughout pass" 0 "$status" \
    "0 repeated titles and 0 pages with no title" "$scratch/out"
report "  over a stated, non-zero denominator" 0 "$status" \
    "out of 2 distinct titles across 2 served pages" "$scratch/out"

# 20. The empty-root refusal covers this pass too. The tool returns before
#     either pass runs, so no title summary is printed at all. A title pass
#     that printed `0 repeated titles` over zero pages would be case 11's
#     defect arriving through the door #567 opened.
status=$(run "$empty")
if [ "$status" = 2 ] && ! grep -q "repeated title" "$scratch/out"; then
    passed=$((passed + 1)); echo "  ok    an empty root states no title verdict"
else
    failed=$((failed + 1)); echo "  FAIL  an empty root states no title verdict"
    echo "          exit $status, and the report said: $(cat "$scratch/out")"
fi

echo "the shelf half, which #538 added"

# Every case below is written rather than doctored, for the reason case 14
# gives: a shelf label is a property of a lock and a served tree together, and
# the assembled site carries one pairing of those at a time. Writing both sides
# is also what lets this suite show the property the pass rests on — that the
# expectation comes from the lock and not from the page — which no doctoring of
# one side alone can demonstrate.
shelf="$scratch/shelf"
lock="$scratch/lock.yml"

shelfpage() {
    # shelfpage RELATIVE-PATH TITLE-TEXT
    mkdir -p "$(dirname "$shelf/$1")"
    printf '<html><head><title>%s - Headwater</title></head><body><h1>x</h1></body></html>\n' \
        "$2" >"$shelf/$1"
}

writelock() {
    # writelock BODY-OF-resolved.shelves
    printf 'lock:\n  format: 3\nresolved:\n  shelves:\n%s' "$1" >"$lock"
}

# 21. The green case. One shelf, declaring a display name, served under it.
rm -rf "$shelf"
shelfpage decisions/index.html 'Decision records'
writelock '    decisions:
      path: "docs/decisions/**"
      title: Decision records
'
status=$(run "$shelf" --lock "$lock")
report "a shelf served under its declared display name passes" 0 "$status" \
    "0 shelf labels not what the lock declares" "$scratch/out"
report "  over a stated, non-zero denominator" 0 "$status" \
    "out of 1 shelf index page checked" "$scratch/out"

# 22. The defect #538 records: the emitter prints the key and the declaration
#     says otherwise. This is the state of `main` before the emitter changed.
rm -rf "$shelf"
shelfpage decisions/index.html 'decisions'
writelock '    decisions:
      path: "docs/decisions/**"
      title: Decision records
'
status=$(run "$shelf" --lock "$lock")
report "a shelf served under its key against a declared name fails" 1 "$status" \
    "1 shelf label not what the lock declares" "$scratch/out"
report "  and it names the page a visitor lands on" 1 "$status" \
    "decisions/index.html: served as \`decisions\`" "$scratch/out"
report "  and it names the declaration the page disagrees with" 1 "$status" \
    "\`shelves.decisions.title\` declares \`Decision records\`" "$scratch/out"

# 23. THE CASE THAT SHOWS THE EXPECTATION IS NOT THE PAGE'S OWN. The served
#     bytes of case 21 are unchanged and only the lock moves. A pass that
#     took its expectation from the emitter's own output — from
#     `.headwater/nav.yml`, which is what the title pass's header refuses to
#     read — could not tell these two runs apart.
rm -rf "$shelf"
shelfpage decisions/index.html 'Decision records'
writelock '    decisions:
      path: "docs/decisions/**"
      title: Rulings
'
status=$(run "$shelf" --lock "$lock")
report "moving the lock alone moves the verdict on unchanged bytes" 1 "$status" \
    "declares \`Rulings\`" "$scratch/out"

# 24. The fall-through. A shelf that declares no display name is printed under
#     its key, and that is the defect in its other form: the reader still
#     meets the key. `taxonomy validate` reports such a shelf and refuses
#     nothing, and this is where it becomes a finding — on a served page.
rm -rf "$shelf"
shelfpage decisions/index.html 'decisions'
writelock '    decisions:
      path: "docs/decisions/**"
'
status=$(run "$shelf" --lock "$lock")
report "a shelf with no display name served under its key fails" 1 "$status" \
    "declares no display name" "$scratch/out"

# 25. And the same shelf served under anything else passes. Nothing here
#     requires a declaration; what it requires is that no reader meets the key.
rm -rf "$shelf"
shelfpage decisions/index.html 'Rulings of this project'
writelock '    decisions:
      path: "docs/decisions/**"
'
status=$(run "$shelf" --lock "$lock")
report "a shelf with no display name served under some other label passes" 0 "$status" \
    "0 shelf labels not what the lock declares" "$scratch/out"

# 26. An empty shelf is skipped and counted, not dropped. Three of this
#     repository's thirteen hold no document, so a guard of the form "every
#     declared shelf has a page" would fire on a corpus that is correct.
rm -rf "$shelf"
shelfpage decisions/index.html 'Decision records'
writelock '    decisions:
      path: "docs/decisions/**"
      title: Decision records
    probe_runs:
      path: "docs/probe-runs/**"
      title: Probe runs
'
status=$(run "$shelf" --lock "$lock")
report "a shelf with no served page is skipped, not failed" 0 "$status" \
    "0 shelf labels not what the lock declares" "$scratch/out"
report "  and the skip is counted against the declared total" 0 "$status" \
    "(1 shelf skipped as holding no document, out of 2 declared" "$scratch/out"

# 27. THE DENOMINATOR GUARD. Every declared shelf is empty, so the pass has an
#     empty set to assert over, and an assertion over an empty set passes.
#     This repository has shipped that shape before, which is why case 11
#     exists for the walk; this is the same refusal one pass further in.
rm -rf "$shelf"
shelfpage somewhere/else/index.html 'Unrelated'
writelock '    decisions:
      path: "docs/decisions/**"
      title: Decision records
'
status=$(run "$shelf" --lock "$lock")
report "no shelf reaching a page exits 2, not 0" 2 "$status" \
    "checked no shelf label" "$scratch/err"

# 28. A missing lock is the assumption of this pass, so it refuses and names
#     the verb that writes one. It is never a quiet skip: a pass that skipped
#     itself when its expectation was absent would report a green run over an
#     expectation nobody supplied.
rm -rf "$shelf"
shelfpage decisions/index.html 'Decision records'
status=$(run "$shelf" --lock "$scratch/no-such-lock.yml")
report "a missing lock exits 2 and names \`taxonomy resolve\`" 2 "$status" \
    "taxonomy resolve" "$scratch/err"

# 29. And so is a lock that carries no shelves at all.
printf 'lock:\n  format: 3\nresolved:\n  kinds: {}\n' >"$lock"
status=$(run "$shelf" --lock "$lock")
report "a lock with no shelves exits 2" 2 "$status" \
    "declares no \`resolved.shelves\`" "$scratch/err"

# 30. `--no-lock` is the one way to leave the pass out, and a run that leaves
#     it out says nothing about a shelf rather than saying zero.
status=$(run "$shelf" --no-lock)
if [ "$status" = 0 ] && ! grep -q "shelf label" "$scratch/out"; then
    passed=$((passed + 1)); echo "  ok    \`--no-lock\` states no shelf verdict at all"
else
    failed=$((failed + 1)); echo "  FAIL  \`--no-lock\` states no shelf verdict at all"
    echo "          exit $status, and the report said: $(tail -1 "$scratch/out")"
fi

echo
echo "$passed passed, $failed failed"
[ "$failed" -eq 0 ]
