#!/bin/sh
# What holds `tools/check-site-console.py`.
#
# Spec 12 refuses a check that ships with no failing fixture, and the reason
# carries double weight here. This checker's whole verdict is an *absence* — no
# console message — and an absence-of-errors check is the one assertion that
# goes quiet the moment its own mechanism breaks. A wrong URL, a server that
# never came up and a browser that failed to launch all produce the same empty
# console a healthy page produces. #532 stood as long as it did for exactly
# that reason one layer up: every instrument over this site reported green
# while every specification part threw twice on load.
#
# So the first case below injects a page that throws on purpose and asserts the
# checker fails on it. A green run of this suite over no provoked failure would
# prove nothing at all.
#
# Run it from anywhere:
#     sh tools/site-console-fixtures.sh
#
# # WHAT IT DOES NOT DO
#
# It does not load all 311 served pages. That run takes three minutes and it is
# the CI step's job, not this suite's. The cases here run over pages this file
# writes and over a four-page copy of the real assembled site, which is what
# makes case 3 a regression over real bytes rather than over a mock.
#
# One arm is asserted without being provoked in a browser: a console record
# whose source is a `chrome://` page. That record comes from the browser's own
# omnibox announcing a slow network, it appears on whichever page happened to
# be loading when it did, and nothing here can make it appear on demand. Case
# 12 calls the classifier directly on two literal records instead, and says so.
#
# # WHAT IT WRITES
#
# Nothing inside this checkout. Every case runs under `mktemp -d`, and the
# directory goes on an interrupt.

set -u

root=$(cd "$(dirname "$0")/.." && pwd)
tool="$root/tools/check-site-console.py"
deploy="$root/.headwater/site-deploy"

if [ ! -f "$tool" ]; then
    echo "no checker at \`tools/check-site-console.py\`." >&2
    exit 1
fi

if [ ! -d "$deploy" ]; then
    echo "no assembled site at \`.headwater/site-deploy\`, so the cases over the" >&2
    echo "  real corpus cannot run. Build and compose it first:" >&2
    echo "        python3 -m mkdocs build --strict" >&2
    echo "        sh tools/assemble-site.sh" >&2
    exit 1
fi

browser=${HEADWATER_CHROME:-}
if [ -z "$browser" ]; then
    for candidate in google-chrome google-chrome-stable chromium chromium-browser; do
        if command -v "$candidate" >/dev/null 2>&1; then
            browser=$candidate
            break
        fi
    done
fi
if [ -z "$browser" ]; then
    echo "SKIPPED: no browser on this host, so none of these cases ran." >&2
    echo "  This suite drives a real browser and there is nothing to drive." >&2
    exit 3
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
    # run DIR [extra args...] -> writes $scratch/out and $scratch/err, echoes status
    dir=$1
    shift
    ( cd "$root" && python3 "$tool" "$dir" --jobs 4 "$@" >"$scratch/out" 2>"$scratch/err" )
    echo $?
}

page() {
    # page ROOT RELATIVE-PATH BODY
    mkdir -p "$(dirname "$1/$2")"
    printf '<html><head><title>t</title></head><body>%s</body></html>\n' "$3" >"$1/$2"
}

echo "the arm that must fail before anything is allowed to pass"

# 1. THE INJECTED THROWING PAGE. A page whose script throws at top level is the
#    whole subject of this checker, and a suite that never saw this fail has
#    measured nothing. It was built and watched to fail before the fix in
#    `mkdocs-overrides/` was written.
thrower="$scratch/thrower"
rm -rf "$thrower"
page "$thrower" clean.html '<p>nothing here throws</p>'
page "$thrower" broken.html '<script>window.absent.field = 1;</script>'
status=$(run "$thrower")
report "a page that throws fails the run" 1 "$status"
report "  and it names the page" 1 "$status" "broken.html:" "$scratch/out"
report "  and it quotes what the console said" 1 "$status" \
    "Cannot set properties of undefined" "$scratch/out"
report "  and the clean page beside it is not blamed" 1 "$status" \
    "1 page with a finding, out of 2 served pages" "$scratch/out"

echo "the arm that must pass, over a stated denominator"

# 2. Two clean pages. A gate that refuses everything is as useless as one that
#    refuses nothing, and neither page here loads `js/base.js`, so neither is
#    held to the `keyCodes` marker.
clean="$scratch/clean"
rm -rf "$clean"
page "$clean" a.html '<p>a</p>'
page "$clean" b.html '<script>var ok = 1;</script>'
status=$(run "$clean")
report "clean pages pass" 0 "$status" "0 pages with a finding" "$scratch/out"
report "  over a stated, non-zero denominator" 0 "$status" \
    "out of 2 served pages loaded in" "$scratch/out"
report "  and a page that does not load \`js/base.js\` is not held to it" 0 "$status" \
    "0 of those pages load the theme's" "$scratch/out"

echo "#532 itself, over the real served bytes"

# A four-page copy of the assembled site: one of the 13 pages that carried the
# defect, one generated page that never did, one hand-built page, and the
# shared `js/` and `css/` both halves link. All three real pages sit two
# directories down, which is what their `../../js/base.js` resolves against.
copy="$scratch/site"
rm -rf "$copy"
mkdir -p "$copy/spec/09-decisions" "$copy/tutorials/t"
# Every directory the real pages *load* from, not merely link to. The checker
# reports a subresource that fails to arrive, so a copy missing `search/` fails
# these cases for a reason that is about this suite and not about the corpus.
# That is how the omission was found.
for asset in js css search img; do
    [ -d "$deploy/$asset" ] && cp -R "$deploy/$asset" "$copy/$asset"
done
victim="$deploy/spec/09-decisions/index.html"
clean_page="$deploy/tutorials/your-first-governed-corpus/index.html"
if [ ! -f "$victim" ] || [ ! -f "$clean_page" ]; then
    echo "  FAIL  the two fixture pages are not in the assembled site, so the" >&2
    echo "        cases over real bytes cannot run. This suite names them on" >&2
    echo "        purpose: if the specification moves, this line is the notice." >&2
    exit 1
fi
cp "$victim" "$copy/spec/09-decisions/index.html"
cp "$clean_page" "$copy/tutorials/t/index.html"

# 3. The corpus as it stands, on the page that carried the defect. This is the
#    number the gate protects, and a failure here is a real defect rather than
#    a broken fixture.
status=$(run "$copy")
report "the page #532 named now loads clean" 0 "$status" "0 pages with a finding" "$scratch/out"
report "  and both real pages were held to the \`keyCodes\` marker" 0 "$status" \
    "2 of those pages load the theme's" "$scratch/out"

# 4. THE REGRESSION. Take the shim out of that page's `<head>` and nothing else,
#    and the two exceptions #532 reported come back verbatim. This is what says
#    the fix in `mkdocs-overrides/` is what is holding the page up, rather than
#    something else on the tree that happens to coincide with it.
sed -i 's|<script src="../../js/leading-digit-anchor-shim.js"></script>||' \
    "$copy/spec/09-decisions/index.html"
status=$(run "$copy")
report "removing the shim brings #532 back" 1 "$status" \
    "is not a valid selector" "$scratch/out"
report "  and it names the anchor that starts with a digit" 1 "$status" \
    "#9--the-decision-register" "$scratch/out"
report "  and the second exception follows the first" 1 "$status" \
    "Cannot read properties of undefined (reading '191')" "$scratch/out"
report "  and the positive marker fires on its own" 1 "$status" \
    "\`keyCodes\` was still undefined" "$scratch/out"
report "  and the page beside it still passes" 1 "$status" \
    "1 page with a finding, out of 2 served pages" "$scratch/out"

echo "the positive marker, on its own"

# 5. A page that names `js/base.js` and never loads it. Its console is silent,
#    so the console arm says nothing, and the marker arm alone fails it. This
#    is the case that shows the two are independent: a check that only read the
#    console would report this page clean.
marker="$scratch/marker"
rm -rf "$marker"
page "$marker" quiet.html '<!-- names js/base.js and loads nothing -->'
status=$(run "$marker")
report "a silent page whose \`js/base.js\` never ran still fails" 1 "$status" \
    "did not run to the end" "$scratch/out"

echo "the mechanism that could break and take the verdict with it"

# 6. A browser that is not a browser. `/bin/true` exits 0 and prints nothing,
#    which is byte for byte what a clean run looks like on stderr. The probe is
#    the only thing that can tell the two apart, and this is the case that
#    proves it does. Without it, every future run of this checker could be
#    reporting on a browser that never started.
status=$(run "$clean" --chrome /bin/true)
report "a browser that loads nothing fails every page" 1 "$status" \
    "the probe this tool injects never ran" "$scratch/out"

# 7. And one that exits non-zero is named as such as well.
status=$(run "$clean" --chrome /bin/false)
report "a browser that exits non-zero is reported" 1 "$status" \
    "the browser exited 1 on it" "$scratch/out"

echo "the allowance, and that it is a decision rather than a blind spot"

# 8. The CDN highlighter's warning is absorbed, counted, and its reason is
#    printed on every run.
allowed="$scratch/allowed"
rm -rf "$allowed"
page "$allowed" hl.html '<script>console.warn("WARN: Could not find the language '"'"'zz'"'"', did you forget to load/include a language module?");</script>'
status=$(run "$allowed")
report "the highlighter warning is allowed" 0 "$status" \
    "1 console record on 1 page allowed by name" "$scratch/out"
report "  and the run prints why" 0 "$status" "allowed because highlight.js" "$scratch/out"

# 9. THE CASE THAT SHOWS THE LIST IS NOT A FILTER NOBODY READS. The same bytes,
#    with the list off, fail. An allowance that could not be lifted would be
#    indistinguishable from a checker that never saw the record.
status=$(run "$allowed" --no-allowances --strict-console)
report "\`--no-allowances --strict-console\` fails on the same page" 1 "$status" \
    "Could not find the language" "$scratch/out"
report "  and says no allowance was applied" 1 "$status" "no allowance was applied" "$scratch/out"

# 10. An unrelated warning is not absorbed by that entry.
rm -rf "$allowed"
page "$allowed" other.html '<script>console.warn("WARN: something else entirely");</script>'
status=$(run "$allowed" --strict-console)
report "an unrelated warning is not allowed" 1 "$status" "something else entirely" "$scratch/out"
status=$(run "$allowed")
report "  and without \`--strict-console\` it is reported, not fatal" 0 "$status" \
    "reported, not fatal" "$scratch/out"

echo "the flake this gate had, and the arm that replaced it"

# 11. THE RECORD THAT REDDENED THREE OTHER PULL REQUESTS. The MkDocs search
#     worker logs `All search scripts loaded, building Lunr index...` once per
#     session, and whichever of 311 concurrent loads was capturing wore it.
#     A page that logs the same string must not fail the gate, because nothing
#     about it says the *page* is broken.
worker="$scratch/worker"
rm -rf "$worker"
page "$worker" quiet.html '<script>console.log("All search scripts loaded, building Lunr index...");</script>'
status=$(run "$worker")
report "a worker-shaped log does not fail the gate" 0 "$status" \
    "0 pages with a finding" "$scratch/out"
report "  and it is printed rather than swallowed" 0 "$status" \
    "reported, not fatal" "$scratch/out"

# 12. AND THE GATE STILL FAILS ON WHAT IT IS FOR, ON THE SAME PAGE. A quieter
#     gate is where a real defect hides, so the throw and the log are put in
#     one page together: the log is reported and the throw is fatal.
rm -rf "$worker"
page "$worker" both.html '<script>console.log("All search scripts loaded, building Lunr index...");</script><script>window.absent.field = 1;</script>'
status=$(run "$worker")
report "a page that logs and also throws still fails" 1 "$status" \
    "the page threw and nothing caught it" "$scratch/out"

# 13. An error thrown from a `DOMContentLoaded` handler is after the marker
#     probe has already run. That is #532's *second* exception, and a listener
#     that serialized once at the end would miss it.
rm -rf "$worker"
page "$worker" late.html '<script>document.addEventListener("DOMContentLoaded", function () { window.absent.later = 1; });</script>'
status=$(run "$worker")
report "an error from a DOMContentLoaded handler is caught" 1 "$status" \
    "the page threw and nothing caught it" "$scratch/out"

# 14. An unhandled promise rejection, which the stderr arm never separated out.
rm -rf "$worker"
page "$worker" reject.html '<script>Promise.reject(new Error("nobody caught this"));</script>'
status=$(run "$worker")
report "an unhandled rejection is caught" 1 "$status" "nobody caught this" "$scratch/out"

# 15. A SUBRESOURCE THAT NEVER ARRIVES. A `<script>` that fails to load throws
#     nothing and logs nothing this tool can attribute, and it leaves the page
#     exactly as `js/base.js` failing to load would: the marker undefined and
#     the console empty. One full-corpus run in six failed that way before the
#     listener ran in the capture phase and the server's listen backlog rose
#     off Python's default of five.
missing="$scratch/missing"
rm -rf "$missing"
page "$missing" gone.html '<script src="no-such-script.js"></script>'
status=$(run "$missing")
report "a subresource that fails to load is named" 1 "$status" \
    "a subresource failed to load" "$scratch/out"
report "  and the report says which one" 1 "$status" "no-such-script.js" "$scratch/out"

echo "the assumptions this tool states rather than reporting zero over"

# 11. An empty directory is not a pass. This separates "nothing is wrong" from
#     "nothing ran", and it is why the tool exits 2 rather than 0 over one.
empty="$scratch/empty"
mkdir -p "$empty"
status=$(run "$empty")
report "an empty root exits 2, not 0" 2 "$status" "checked nothing" "$scratch/err"

# 12. A missing root names what writes one.
status=$(run "$scratch/absent")
report "a missing root exits 2 and names \`assemble-site.sh\`" 2 "$status" \
    "assemble-site.sh" "$scratch/err"

# 13. An unknown flag is refused rather than ignored.
status=$(run "$clean" --not-a-flag)
report "an unknown argument exits 2" 2 "$status" "unknown argument" "$scratch/err"

# 14. No browser is a skip with its own status, never a pass. A caller that
#     read this as 0 would report a green step over nothing loaded at all,
#     which is the defect this whole suite is about.
status=$(run "$clean" --chrome /no/such/browser)
report "a named browser that is not there exits 3" 3 "$status" "no browser at" "$scratch/err"

echo "the classifier, called directly"

# 15. The `chrome://` scoping rule. Nothing here can make the browser's own
#     omnibox log on demand, so this case calls the classifier on two literal
#     records instead and says which is which. It is the one arm of this suite
#     that does not go through a browser.
cat >"$scratch/classify.py" <<'PY'
import importlib.util
import sys

spec = importlib.util.spec_from_file_location("checker", sys.argv[1])
checker = importlib.util.module_from_spec(spec)
spec.loader.exec_module(checker)

ui = (
    '[1:1:0907/010536.384167:INFO:CONSOLE:1158] "Slow network is detected.", '
    'source: chrome://omnibox-popup.top-chrome/omnibox_popup.js (1158)'
)
page = (
    '[1:1:0907/010536.384167:INFO:CONSOLE:48] "Uncaught TypeError: x", '
    'source: http://127.0.0.1:8000/js/base.js (48)'
)

assert checker.classify(ui, True)[0] == "noise", "a chrome:// record must not count"
assert checker.classify(ui, False)[0] == "noise", "--no-allowances must not lift it"
assert checker.classify(page, True)[0] == "finding", "a page record must count"
print("ok")
PY
if python3 "$scratch/classify.py" "$tool" >"$scratch/out" 2>"$scratch/err"; then
    passed=$((passed + 1)); echo "  ok    a \`chrome://\` record is scoped out and a page record is not"
else
    failed=$((failed + 1)); echo "  FAIL  a \`chrome://\` record is scoped out and a page record is not"
    echo "          $(tail -2 "$scratch/err")"
fi

echo
echo "$passed passed, $failed failed"
[ "$failed" -eq 0 ]
