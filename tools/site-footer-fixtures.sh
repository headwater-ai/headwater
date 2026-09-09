#!/bin/sh
# What holds `tools/check-site-footer.sh`.
#
# The footer assertion is an absence check. It must state how many served pages
# it read: an empty walk has no forbidden strings, but it has not checked a site.

set -u

root=$(cd "$(dirname "$0")/.." && pwd)
tool="$root/tools/check-site-footer.sh"
minimum_pages=2

if [ ! -f "$tool" ]; then
    echo "no checker at \`tools/check-site-footer.sh\`." >&2
    exit 1
fi

scratch=$(mktemp -d) || exit 1
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

passed=0
failed=0

# report NAME EXPECTED-STATUS ACTUAL-STATUS REQUIRED-TEXT OUTPUT-FILE
report() {
    name=$1
    want=$2
    got=$3
    text=$4
    output=$5
    if [ "$want" = "$got" ] && grep -qF -- "$text" "$output"; then
        passed=$((passed + 1))
        echo "  ok    $name"
    else
        failed=$((failed + 1))
        echo "  FAIL  $name"
    fi
}

run() {
    HEADWATER_SITE_FOOTER_MINIMUM_PAGES=$minimum_pages "$tool" "$1" >"$scratch/out" 2>"$scratch/err"
    echo $?
}

page() {
    mkdir -p "$(dirname "$1/$2")"
    printf '<html><body>%s</body></html>\n' "$3" >"$1/$2"
}

echo "the passing population"
good="$scratch/good"
page "$good" index.html 'Headwater'
page "$good" docs/index.html 'A governed corpus'
status=$(run "$good")
report "a clean nonempty population passes" 0 "$status" "0 forbidden footer strings out of 2 served HTML pages" "$scratch/out"

echo "the forbidden footer"
footer="$scratch/footer"
page "$footer" index.html 'Documentation built with MkDocs'
page "$footer" docs/index.html 'A governed corpus'
status=$(run "$footer")
report "a planted footer fails and names its page" 1 "$status" "index.html" "$scratch/err"

echo "the empty population"
empty="$scratch/empty"
mkdir -p "$empty"
status=$(run "$empty")
report "an empty directory fails for the denominator" 2 "$status" "denominator is 0" "$scratch/err"

echo "$passed passed; $failed failed"
[ "$failed" -eq 0 ]
