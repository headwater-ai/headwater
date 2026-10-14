#!/bin/sh
# What holds `tools/site/review-site.py`.
#
# Spec 12 refuses a check that ships with no failing fixture. The review tool
# reads what `tools/site/check-site-console.py` does not: the layout at phone
# width, the dark theme, keyboard focus and the navigation a reader sees. Each
# of those readings is an absence until it finds something, so each one is
# provoked here on a page that must be flagged, beside a page that must not.
#
# Case 1 is the reason the tool exists: a page that is broken on a phone and
# that every other gate over this site passes, because its console is empty.
# It was written and watched to fail before the tool measured overflow.
#
# Run it from anywhere:
#     sh tools/site/site-review-fixtures.sh
#
# # WHAT IT WRITES
#
# Nothing inside this checkout. Every case runs under `mktemp -d`, and the
# directory goes on an interrupt.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
tool="$root/tools/site/review-site.py"

if [ ! -f "$tool" ]; then
    echo "no review tool at \`tools/site/review-site.py\`." >&2
    exit 1
fi

# The same search the tool makes, so that this suite skips exactly when the
# tool would. The last candidate is the headless shell that Playwright
# installs, which is the only browser on some hosts.
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
    for candidate in "$HOME"/.cache/ms-playwright/chromium_headless_shell-*/chrome-headless-shell-linux64/chrome-headless-shell; do
        if [ -x "$candidate" ]; then
            browser=$candidate
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
    tally "$name" "$ok" "${why:-}"
}

# absent NAME SUBSTRING-THAT-MUST-NOT-APPEAR FILE
absent() {
    if grep -qF -- "$2" "$3"; then
        tally "$1" no "the report said what it must not: $2"
    else
        tally "$1" yes ""
    fi
}

tally() {
    if [ "$2" = yes ]; then
        passed=$((passed + 1))
        echo "  ok    $1"
    else
        failed=$((failed + 1))
        echo "  FAIL  $1"
        echo "          $3"
    fi
}

run() {
    # run DIR [extra args...] -> writes $scratch/out and $scratch/err, echoes status
    dir=$1
    shift
    ( cd "$root" && HEADWATER_CHROME="$browser" python3 "$tool" "$dir" --jobs 4 "$@" >"$scratch/out" 2>"$scratch/err" )
    echo $?
}

page() {
    # page ROOT RELATIVE-PATH BODY [HEAD]
    # The body margin is zeroed so that a 1000-pixel element measures as 1000.
    mkdir -p "$(dirname "$1/$2")"
    printf '<html><head><title>t</title><style>body{margin:0}</style>%s</head><body>%s</body></html>\n' \
        "${4:-}" "$3" >"$1/$2"
}

echo "the reading that must flag before anything is allowed to pass"

# 1. THE PAGE THAT IS BROKEN ON A PHONE. A fixed 1000-pixel element is wider
#    than a 390-pixel screen and narrower than a 1280-pixel one. Its console is
#    empty, so `check-site-console.py` passes it. This tool must flag it at
#    phone width, must not flag it at desktop width, and must not blame the
#    clean page beside it.
wide="$scratch/wide"
page "$wide" clean.html '<p>nothing here is wide</p>'
page "$wide" wide.html '<div style="width:1000px">x</div>'
status=$(run "$wide")
report "a page wider than a phone is flagged" 1 "$status"
report "  and it names the page and quotes the reading" 1 "$status" \
    "wide.html: horizontal overflow at 390x844, light: scrollWidth 1000 > 390" "$scratch/out"
report "  and it names the element that reaches past the edge" 1 "$status" \
    "scrollWidth 1000 > 390 (div to 1000)" "$scratch/out"
absent "  and it is not flagged at desktop width" "horizontal overflow at 1280x800" "$scratch/out"
report "  and the clean page beside it is not blamed" 1 "$status" \
    "1 page flagged, out of 2 served pages" "$scratch/out"

# 1b. The width a page overflows is the width it has for content, not the
#     window. A page tall enough to show a vertical scrollbar has 15 pixels
#     less than the window on this browser, and an element 380 pixels wide
#     overflows it at 390. A reading against `innerWidth` passes this page.
tall="$scratch/tall"
page "$tall" tall.html '<div style="width:380px;height:3000px">x</div>'
status=$(run "$tall")
report "an overflow narrower than the scrollbar is still flagged" 1 "$status" \
    "tall.html: horizontal overflow at 390x844, light: scrollWidth 380 >" "$scratch/out"

echo "the reading that must pass, over a stated denominator"

# 2. Two clean pages. A review that flags everything is as useless as one
#    that flags nothing.
clean="$scratch/clean"
page "$clean" a.html '<p>a</p>'
page "$clean" b.html '<p><a href="a.html">b</a></p>'
status=$(run "$clean")
report "clean pages are not flagged" 0 "$status" \
    "0 pages flagged, out of 2 served pages" "$scratch/out"
report "  and the run states how many readings it took" 0 "$status" \
    "4 readings" "$scratch/out"

echo "the dark reading, and that it is dark"

# 3. A page that overflows only under `prefers-color-scheme: dark`. A flag at
#    the dark reading and none at the light one is what shows the browser
#    applied the scheme this tool asked for. A tool that read light twice
#    would pass this page.
dark="$scratch/dark"
page "$dark" night.html '<div class="w">x</div>' \
    '<style>@media (prefers-color-scheme: dark) { .w { width: 1000px } }</style>'
status=$(run "$dark" --dark-all)
report "an overflow under the dark scheme alone is flagged dark" 1 "$status" \
    "night.html: horizontal overflow at 390x844, dark: scrollWidth 1000 > 390" "$scratch/out"
absent "  and not light" "horizontal overflow at 390x844, light" "$scratch/out"
status=$(run "$dark")
report "  and without a dark reading it is not seen at all" 0 "$status" \
    "0 pages flagged" "$scratch/out"

echo "keyboard focus"

# 4. The first focusable element with its outline taken away and nothing put
#    in its place. The page beside it keeps the browser's own ring.
focus="$scratch/focus"
page "$focus" ring.html '<a href="ring.html">a link</a>'
page "$focus" bare.html '<a href="ring.html">a link</a>' \
    '<style>a:focus, a:focus-visible { outline: none; box-shadow: none }</style>'
page "$focus" clear.html '<a href="ring.html">a link</a>' \
    '<style>a:focus, a:focus-visible { outline: 2px solid transparent }</style>'
status=$(run "$focus")
report "a link with no visible focus is flagged" 1 "$status" \
    "bare.html: no visible focus indicator on <a> at 1280x800, light" "$scratch/out"
report "  and so is a ring of full width drawn in a transparent color" 1 "$status" \
    "clear.html: no visible focus indicator on <a> at 1280x800, light: outline solid 2px rgba(0, 0, 0, 0)" "$scratch/out"
absent "  and a link with the browser's ring is not" "ring.html: no visible focus" "$scratch/out"

echo "the navigation a reader sees, against \`.headwater/nav.yml\`"

# 5. A page that carries the shelves navigation the generated site renders,
#    with one section of the nav file missing from it and one link that
#    resolves to nothing served. Each of the two is its own finding.
nav="$scratch/nav"
printf 'nav:\n  - "Reference":\n      - "Guides":\n          - "G": "g.md"\n      - "Specification":\n          - "S": "s.md"\n' >"$scratch/nav.yml"
page "$nav" guides/index.html \
    '<nav aria-label="Shelves"><span class="hw-grp">Reference</span><a href="../guides/">Guides</a><a href="../gone/">Specification</a></nav>'
page "$nav" spec/index.html \
    '<nav aria-label="Shelves"><span class="hw-grp">Reference</span><a href="../guides/">Guides</a><a href="../spec/">Specification</a></nav>'
page "$nav" short/index.html \
    '<nav aria-label="Shelves"><span class="hw-grp">Reference</span><a href="../guides/">Guides</a></nav>'
page "$nav" nogroup/index.html \
    '<nav aria-label="Shelves"><a href="../guides/">Guides</a><a href="../spec/">Specification</a></nav>'
page "$nav" plain/index.html '<p>a hand-built page carries no shelves navigation</p>'
status=$(run "$nav" --nav "$scratch/nav.yml")
report "a navigation link to no served page is flagged" 1 "$status" \
    "guides/: a navigation link resolves to no served page: gone/" "$scratch/out"
report "  and a section of the nav file the page does not show is flagged" 1 "$status" \
    "short/: the navigation does not show the section \`Specification\` of nav.yml" "$scratch/out"
report "  and a top-level section the page does not show is flagged" 1 "$status" \
    "nogroup/: the navigation does not show the section \`Reference\` of nav.yml" "$scratch/out"
report "  and the page that shows every section and links only served pages is not" 1 "$status" \
    "3 pages flagged, out of 5 served pages" "$scratch/out"
absent "  and a page with no shelves navigation is not held to it" "plain/: the navigation" "$scratch/out"

echo "a host read through its sitemap, as the live site is"

# 6. `--base` reads a host's `sitemap.xml` and fetches each page from it. The
#    host here is a loopback server this suite starts. Its sitemap names one
#    page, whose navigation links one page the sitemap leaves out and one that
#    is not served at all. The first is read in a second round and named once.
#    The second is the page's own defect.
host="$scratch/host"
page "$host" a/index.html \
    '<nav aria-label="Shelves"><span class="hw-grp">Reference</span><a href="../a/">Guides</a><a href="../p/">Specification</a><a href="../gone/">Specification</a></nav>'
page "$host" p/index.html \
    '<nav aria-label="Shelves"><span class="hw-grp">Reference</span><a href="../a/">Guides</a><a href="../p/">Specification</a></nav>'
printf '<urlset><url><loc>http://example.invalid/a/</loc></url></urlset>\n' >"$host/sitemap.xml"
cat >"$scratch/serve.py" <<'PY'
import functools, http.server, sys
handler = functools.partial(http.server.SimpleHTTPRequestHandler, directory=sys.argv[1])
handler.func.log_message = lambda *args: None
server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), handler)
with open(sys.argv[2], "w") as out:
    out.write(str(server.server_address[1]))
server.serve_forever()
PY
python3 "$scratch/serve.py" "$host" "$scratch/port" &
server_pid=$!
trap 'kill "$server_pid" 2>/dev/null; rm -rf "$scratch"' EXIT HUP INT TERM
tries=0
while [ ! -s "$scratch/port" ] && [ "$tries" -lt 50 ]; do
    sleep 0.1
    tries=$((tries + 1))
done
( cd "$root" && HEADWATER_CHROME="$browser" python3 "$tool" --base "http://127.0.0.1:$(cat "$scratch/port")/" \
    --nav "$scratch/nav.yml" --jobs 4 >"$scratch/out" 2>"$scratch/err" )
status=$?
report "a page the sitemap leaves out is read in a second round" 1 "$status" \
    "1 page the navigation links answer 200 and the page list does not name them, so they were read in a second round: p/" \
    "$scratch/out"
report "  and a link to a page the host does not serve is the page's defect" 1 "$status" \
    "a/: a navigation link resolves to no served page: gone/" "$scratch/out"
report "  over both pages, and only the one is flagged" 1 "$status" \
    "1 page flagged, out of 2 served pages" "$scratch/out"
kill "$server_pid" 2>/dev/null

echo "screenshots, and only where asked"

# 6. `--shoot` takes every reading of the pages it matches, and a screenshot of
#    each, and none of the others.
shots="$scratch/shots"
status=$(run "$clean" --shots "$shots" --shoot '^a\.html$')
report "a matched page is read four ways" 0 "$status" "6 readings" "$scratch/out"
for name in a--1280x800-light a--1280x800-dark a--390x844-light a--390x844-dark; do
    if [ -s "$shots/$name.png" ]; then
        tally "  and \`$name.png\` was written" yes ""
    else
        tally "  and \`$name.png\` was written" no "no file at $shots/$name.png"
    fi
done
if ls "$shots" | grep -q '^b--'; then
    tally "  and the page it did not match has no screenshot" no "a \`b--\` file is in $shots"
else
    tally "  and the page it did not match has no screenshot" yes ""
fi

echo "the mechanism that could break and take the verdict with it"

# 7. A browser that is not a browser. `/bin/true` exits 0 and prints nothing,
#    which is what a page with nothing to flag also prints. The probe this tool
#    injects is the only thing that tells the two apart.
status=$(run "$clean" --chrome /bin/true)
report "a browser that loads nothing flags every page" 1 "$status" \
    "the probe this tool injects never ran" "$scratch/out"
report "  over every page, not some" 1 "$status" \
    "2 pages flagged, out of 2 served pages" "$scratch/out"

# 7b. A browser that writes a clean reading and then crashes. The reading alone
#     passes the page, so the exit status is the only thing that says the
#     browser did not finish. The fake reads the width it was asked for, so
#     that the reading is otherwise one this tool accepts.
cat >"$scratch/crash.sh" <<'SH'
#!/bin/sh
width=0
for arg in "$@"; do
    case $arg in --window-size=*) width=${arg#--window-size=}; width=${width%%,*} ;; esac
done
case "$*" in
    *--dump-dom*)
        reading=$(printf '{"probe":"ran","dark":false,"scrollWidth":10,"clientWidth":10,"innerWidth":%s,"framed":true,"focus":null,"nav":null}' "$width" | base64 | tr -d '\n')
        printf '<html data-headwater-review="%s"></html>' "$reading" ;;
esac
exit 139
SH
chmod +x "$scratch/crash.sh"
status=$(run "$clean" --chrome "$scratch/crash.sh")
report "a browser that crashes after a clean reading flags the page" 1 "$status" \
    "a.html: the browser exited 139 at 1280x800, light" "$scratch/out"

# 7c. A browser that never returns. Without the flag, a page whose every
#     reading timed out has no flag at all and reads as clean.
printf '#!/bin/sh\nexec sleep 30\n' >"$scratch/hang.sh"
chmod +x "$scratch/hang.sh"
status=$(run "$clean" --chrome "$scratch/hang.sh" --timeout 2)
report "a browser that does not finish flags the page" 1 "$status" \
    "a.html: the browser did not finish 1280x800, light in 2s" "$scratch/out"

echo "the assumptions this tool states rather than reporting zero over"

# 8. An empty directory is not a pass. HW-OBL-0165 is the reason.
empty="$scratch/empty"
mkdir -p "$empty"
status=$(run "$empty")
report "an empty root exits 2, not 0" 2 "$status" "reviewed nothing" "$scratch/err"

# 9. No browser is a skip with its own status, never a pass.
status=$(run "$clean" --chrome /no/such/browser)
report "a named browser that is not there exits 3" 3 "$status" "no browser at" "$scratch/err"

# 10. An unknown flag is refused rather than ignored.
status=$(run "$clean" --not-a-flag)
report "an unknown argument exits 2" 2 "$status" "unknown argument" "$scratch/err"

echo "the judge, called directly"

# 11. A dark reading that the page saw as light is a reading that did not
#     take, and it is never judged as a light one. Nothing here can make the
#     browser ignore the scheme on demand, so this case calls the judge on a
#     literal reading instead, and says so.
cat >"$scratch/judge.py" <<'PY'
import importlib.util
import sys

spec = importlib.util.spec_from_file_location("review", sys.argv[1])
review = importlib.util.module_from_spec(spec)
spec.loader.exec_module(review)

case = sys.argv[2]
if case == "scheme":
    reading = {"dark": False, "scrollWidth": 1000, "clientWidth": 390, "innerWidth": 390, "framed": True, "focus": None}
    flags = review.judge(reading, (390, 844), "dark")
    assert any("did not take" in f for f in flags), flags
elif case == "unframed":
    # A page read as the window and not in the frame took whatever width the
    # browser gave the window, even when that width matches by chance.
    reading = {"dark": False, "scrollWidth": 1000, "clientWidth": 390, "innerWidth": 390, "framed": False, "focus": None}
    flags = review.judge(reading, (390, 844), "light")
    assert any("not read inside the frame" in f for f in flags), flags
else:
    # Full headless Chrome makes no window narrower than 500 pixels. A page
    # 450 pixels wide passes in that window, so a reading taken at 500 and
    # labeled 390 must not be judged at all.
    reading = {"dark": False, "scrollWidth": 450, "clientWidth": 500, "innerWidth": 500, "framed": True, "focus": None}
    flags = review.judge(reading, (390, 844), "light")
    assert any("did not take: the page saw a window 500 pixels wide" in f for f in flags), flags
assert not any("overflow" in f for f in flags), flags
print("ok")
PY
if python3 "$scratch/judge.py" "$tool" scheme >"$scratch/out" 2>"$scratch/err"; then
    tally "a dark reading the page saw as light is flagged as not taken, and not judged" yes ""
else
    tally "a dark reading the page saw as light is flagged as not taken, and not judged" no \
        "$(tail -1 "$scratch/err")"
fi
if python3 "$scratch/judge.py" "$tool" unframed >"$scratch/out" 2>"$scratch/err"; then
    tally "a reading not taken inside the frame is flagged as not taken, and not judged" yes ""
else
    tally "a reading not taken inside the frame is flagged as not taken, and not judged" no \
        "$(tail -1 "$scratch/err")"
fi
if python3 "$scratch/judge.py" "$tool" width >"$scratch/out" 2>"$scratch/err"; then
    tally "a reading at a width other than its label is flagged as not taken, and not judged" yes ""
else
    tally "a reading at a width other than its label is flagged as not taken, and not judged" no \
        "$(tail -1 "$scratch/err")"
fi

echo
echo "$passed passed, $failed failed"
[ "$failed" -eq 0 ]
