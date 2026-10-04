#!/bin/sh
# What holds `tools/site/check-site-noindex.py`, and the `noindex` rule of
# `tools/site/sitemap.py` (#1681).
#
# Spec 12 refuses a check that ships with no failing fixture. Every refusal
# below is provoked on purpose, and so is every pass, because a gate that
# refuses everything is as useless as one that refuses nothing.
# `tools/site/site-canonical-fixtures.sh` is the shape.
#
# Run it from anywhere:
#     sh tools/site/site-noindex-fixtures.sh
#
# # THE CASE THIS SUITE EXISTS FOR
#
# Case 3: an assembled tree whose `sitemap.xml` lists a page that carries the
# robots `noindex` meta. That is the defect the served sitemap had before
# #1681, when it listed all 56 process pages, and the check must exit 1 on it.
#
# # THE ASSUMPTION IT STATES, AND FAILS ON
#
# Case 1 runs over the real assembled directory, so it needs one. It stops
# when there is none rather than passing over nothing. Build it first:
#     python3 -m mkdocs build --strict && sh tools/site/assemble-site.sh
#
# # WHAT IT WRITES
#
# Nothing inside this checkout. Every synthetic case runs under `mktemp -d`.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
tool="$root/tools/site/check-site-noindex.py"
sitemap="$root/tools/site/sitemap.py"
deploy="$root/.headwater/site-deploy"
origin="https://headwater.tools"

if [ ! -d "$deploy" ]; then
    echo "no assembled site at \`.headwater/site-deploy\`, so case 1 cannot run." >&2
    echo "  Build it first: python3 -m mkdocs build --strict && sh tools/site/assemble-site.sh" >&2
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

# run DIR [NAV] -> writes $scratch/out and $scratch/err, prints the status
run() {
    python3 "$tool" "$@" >"$scratch/out" 2>"$scratch/err"
    echo $?
}

# page DIR RELATIVE-DIR noindex|plain [reversed]
page() {
    mkdir -p "$1/$2"
    {
        printf '<!DOCTYPE html>\n<html><head>\n'
        case "$3:${4:-}" in
            noindex:reversed) printf '<meta content="noindex, follow" name="robots">\n' ;;
            noindex:*) printf '<meta name="robots" content="noindex">\n' ;;
        esac
        printf '<title>%s</title>\n</head><body></body></html>\n' "$2"
    } >"$1/$2/index.html"
}

# nav FILE PREFIX... : a generated nav with the given noindex prefixes
nav() {
    file=$1
    shift
    {
        printf '# headwater:generated site_nav.\n\nnav:\n  - "A": "a/README.md"\n'
        if [ $# -gt 0 ]; then
            printf '\nextra:\n  headwater_noindex:\n'
            for prefix in "$@"; do printf '    - "%s"\n' "$prefix"; done
        fi
    } >"$file"
}

# tree DIR : a clean assembled tree, one public page and two process pages,
# with the sitemap that `sitemap.py` derives from it
tree() {
    rm -rf "$1"
    page "$1" . plain
    page "$1" spec plain
    page "$1" process/decisions noindex
    page "$1" process/decisions/0001-a noindex reversed
    python3 "$sitemap" "$1" >"$1/sitemap.xml"
    printf '# t\n\n- [Spec](%s/spec/): the spec\n' "$origin" >"$1/llms.txt"
}

echo "over the real assembled site"

# 1. The site as it stands, with a non-zero denominator.
status=$(run "$deploy")
report "the assembled site lists no noindex page in its crawler files" 0 "$status"
if grep -qE '^[1-9][0-9]* of [1-9][0-9]* pages carry `noindex`; [1-9]' "$scratch/out"; then
    passed=$((passed + 1)); echo "  ok    it states a non-zero denominator"
else
    failed=$((failed + 1)); echo "  FAIL  it states a non-zero denominator"
    sed 's/^/          /' "$scratch/out"
fi

echo "over synthetic trees"
t="$scratch/site"
n="$scratch/nav.yml"
nav "$n" "process/decisions/"

# 2. A clean tree passes, and `sitemap.py` left both noindex pages out,
#    whichever order the meta's attributes are in.
tree "$t"
status=$(run "$t" "$n")
report "a clean tree passes" 0 "$status"
if grep -q 'process/' "$t/sitemap.xml"; then
    failed=$((failed + 1)); echo "  FAIL  sitemap.py leaves out every noindex page"
else
    passed=$((passed + 1)); echo "  ok    sitemap.py leaves out every noindex page"
fi
if [ "$(grep -c '<loc>' "$t/sitemap.xml")" = 2 ]; then
    passed=$((passed + 1)); echo "  ok    sitemap.py keeps every other page"
else
    failed=$((failed + 1)); echo "  FAIL  sitemap.py keeps every other page"
fi

# 3. The sitemap lists a noindex page.
tree "$t"
sed -i "s|</urlset>|  <url>\n    <loc>$origin/process/decisions/0001-a/</loc>\n  </url>\n</urlset>|" "$t/sitemap.xml"
status=$(run "$t" "$n")
report "a sitemap that lists a noindex page is refused" 1 "$status" \
    "\`sitemap.xml\` lists $origin/process/decisions/0001-a/" "$scratch/err"

# 4. llms.txt lists a noindex page.
tree "$t"
printf -- '- [D](%s/process/decisions/): process\n' "$origin" >>"$t/llms.txt"
status=$(run "$t" "$n")
report "an llms.txt that lists a noindex page is refused" 1 "$status" \
    "\`llms.txt\` lists $origin/process/decisions/" "$scratch/err"

# 5. A page under a declared prefix lost the meta.
tree "$t"
page "$t" process/decisions/0002-b plain
status=$(run "$t" "$n")
report "a page under a noindex prefix with no meta is refused" 1 "$status" \
    "\`process/decisions/0002-b/index.html\` is under a \`noindex\` section" "$scratch/err"

# 6. The nav declares a prefix that no page sits under.
tree "$t"
nav "$scratch/stray.yml" "process/nowhere/"
status=$(run "$t" "$scratch/stray.yml")
report "a prefix that covers no page is refused, not passed" 1 "$status" \
    "no served page sits under any of them" "$scratch/err"

# 7. A file prefix is served by MkDocs's rule: `a/README.md` at `a/`.
tree "$t"
nav "$scratch/file.yml" "process/decisions/README.md"
status=$(run "$t" "$scratch/file.yml")
report "a file prefix maps to the directory MkDocs serves it at" 0 "$status"
page "$t" process/decisions plain
python3 "$sitemap" "$t" >"$t/sitemap.xml"
status=$(run "$t" "$scratch/file.yml")
report "a file-prefix page that lost the meta is refused" 1 "$status" \
    "\`process/decisions/index.html\` is under" "$scratch/err"

# 8. Input that cannot be read exits 2 rather than passing.
mkdir -p "$scratch/empty"
status=$(run "$scratch/empty" "$n")
report "a directory with no page exits 2" 2 "$status"
tree "$t"
rm "$t/sitemap.xml"
status=$(run "$t" "$n")
report "a tree with no sitemap.xml exits 2" 2 "$status"

echo "$passed passed, $failed failed"
[ "$failed" -eq 0 ]
