#!/bin/sh
# What holds `tools/site/check-site-links.py`.
#
# Spec 12 refuses a check that ships with no failing fixture. Every refusal
# below is provoked on purpose, and so is every pass, because a gate that
# refuses everything is as useless as one that refuses nothing.
# `tools/site/site-noindex-fixtures.sh` is the shape.
#
# Run it from anywhere:
#     sh tools/site/site-links-fixtures.sh
#
# # THE CASES THIS SUITE EXISTS FOR
#
# Cases 3 to 6 are the four ways 44 pages of headwater.tools reached Search
# Console as "Not found (404)" while `mkdocs build --strict` exited 0: a
# relative link that leaves `docs/` (`../../LICENSE`), a root-absolute link to
# a repository file, a directory link one level too deep (`../obligations/`
# written on a page served at `/spec/page/`), and a link to a page that was
# renamed and left no redirect behind.
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
tool="$root/tools/site/check-site-links.py"
deploy="$root/.headwater/site-deploy"
origin="https://headwater.tools"

if [ ! -f "$tool" ]; then
    echo "no checker at \`tools/site/check-site-links.py\`." >&2
    exit 1
fi
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

# run DIR... -> writes $scratch/out and $scratch/err, prints the status
run() {
    python3 "$tool" "$@" >"$scratch/out" 2>"$scratch/err"
    echo $?
}

# page DIR RELATIVE-DIR [BODY] : a served page at RELATIVE-DIR/index.html
page() {
    mkdir -p "$1/$2"
    printf '<!DOCTYPE html>\n<html><head><title>%s</title></head><body>\n%s\n</body></html>\n' \
        "$2" "${3:-}" >"$1/$2/index.html"
}

# sitemap DIR PATH... : a sitemap that lists each served path
sitemap() {
    dir=$1
    shift
    {
        printf '<?xml version="1.0" encoding="UTF-8"?>\n<urlset>\n'
        for p in "$@"; do printf '  <url>\n    <loc>%s%s</loc>\n  </url>\n' "$origin" "$p"; done
        printf '</urlset>\n'
    } >"$dir/sitemap.xml"
}

# tree DIR : a clean tree, three pages that link to one another
tree() {
    rm -rf "$1"
    page "$1" . '<a href="spec/">spec</a> <a href="/decisions/">d</a>'
    page "$1" spec '<a href="../">home</a> <a href="a/">a</a> <a href="https://example.com/x">out</a> <a href="mailto:a@b.c">m</a> <a href="#top">t</a>'
    page "$1" spec/a '<a href="../">spec</a> <a href="../../decisions/">d</a>'
    page "$1" decisions '<a href="/spec/a/">a</a>'
    sitemap "$1" / /spec/ /spec/a/ /decisions/
}

echo "over the real assembled site"

# 1. The site as it stands, with a non-zero denominator.
status=$(run "$deploy")
report "the assembled site has no dead link and no http link" 0 "$status"
if grep -qE '^[1-9][0-9]* internal links on [1-9][0-9]* pages checked' "$scratch/out"; then
    passed=$((passed + 1)); echo "  ok    it states a non-zero denominator"
else
    failed=$((failed + 1)); echo "  FAIL  it states a non-zero denominator"
    sed 's/^/          /' "$scratch/out"
fi

echo "over synthetic trees"
t="$scratch/site"

# 2. A clean tree passes, with an external link, a mailto and a bare fragment
#    left alone.
tree "$t"
status=$(run "$t")
report "a clean tree passes" 0 "$status"

# 3. A relative link that leaves the served root, as `../../LICENSE` does from
#    a page served two levels deep.
tree "$t"
page "$t" decisions/0011-license '<a href="../../LICENSE">LICENSE</a>'
status=$(run "$t")
report "a relative link to a repository file is refused" 1 "$status" \
    "dead link /LICENSE" "$scratch/err"

# 4. A root-absolute link to a repository file, which MkDocs never resolves.
tree "$t"
page "$t" interfaces/help '<a href="/engine/crates/cli/src/lib.rs">lib.rs</a>'
status=$(run "$t")
report "a root-absolute link to a repository file is refused" 1 "$status" \
    "dead link /engine/crates/cli/src/lib.rs" "$scratch/err"

# 5. A directory link written one level too deep for a page served at /spec/a/.
tree "$t"
page "$t" spec/b '<a href="../obligations/">obligations</a>'
status=$(run "$t")
report "a directory link one level too deep is refused" 1 "$status" \
    "dead link /spec/obligations/" "$scratch/err"

# 6. A link to a page that was renamed and left no redirect.
tree "$t"
page "$t" evaluations/x '<a href="../../obligations/0203-old-slug/">old</a>'
status=$(run "$t")
report "a link to a page that no longer exists is refused" 1 "$status" \
    "dead link /obligations/0203-old-slug/" "$scratch/err"

# 7. An absolute link on this site's own origin that is dead.
tree "$t"
page "$t" evaluations/y "<a href=\"$origin/process/gone/\">gone</a>"
status=$(run "$t")
report "a dead link written with the site's origin is refused" 1 "$status" \
    "dead link /process/gone/" "$scratch/err"

# 8. An internal link written with http://.
tree "$t"
page "$t" evaluations/z '<a href="http://headwater.tools/spec/">spec</a>'
status=$(run "$t")
report "an internal link written with http:// is refused" 1 "$status" \
    "with \`http://\`" "$scratch/err"

# 9. A link to a redirect source passes while the destination is served, and is
#    refused when the destination is not.
tree "$t"
printf '/obligations/0203-old-slug/ /spec/a/ 301\n' >"$t/_redirects"
page "$t" evaluations/x '<a href="../../obligations/0203-old-slug/">old</a>'
status=$(run "$t")
report "a link answered by a redirect to a served page passes" 0 "$status"
if grep -q '1 answered by a redirect' "$scratch/out"; then
    passed=$((passed + 1)); echo "  ok    it counts the link a redirect answered"
else
    failed=$((failed + 1)); echo "  FAIL  it counts the link a redirect answered"
    sed 's/^/          /' "$scratch/out"
fi
printf '/obligations/0203-old-slug/ /spec/nowhere/ 301\n' >"$t/_redirects"
status=$(run "$t")
report "a link answered by a redirect to a dead page is refused" 1 "$status" \
    "redirected to /spec/nowhere/" "$scratch/err"

# 10. A page that is served with no trailing slash on the link passes, because
#     the host redirects it.
tree "$t"
page "$t" evaluations/w '<a href="/spec/a">a</a>'
status=$(run "$t")
report "a directory link without a trailing slash passes" 0 "$status"

# 11. The sitemap lists a page that is not served.
tree "$t"
sitemap "$t" / /spec/ /spec/a/ /decisions/ /spec/renamed/
status=$(run "$t")
report "a sitemap that lists an unserved page is refused" 1 "$status" \
    "lists $origin/spec/renamed/, which is not served" "$scratch/err"

# 12. The sitemap lists a URL that only a redirect answers.
tree "$t"
printf '/spec/old/ /spec/a/ 301\n' >"$t/_redirects"
sitemap "$t" / /spec/ /spec/a/ /decisions/ /spec/old/
status=$(run "$t")
report "a sitemap that lists a redirected URL is refused" 1 "$status" \
    "lists $origin/spec/old/, which is redirected" "$scratch/err"

# 13. A sitemap that lists http://.
tree "$t"
printf '<urlset><url><loc>http://headwater.tools/spec/</loc></url></urlset>\n' >"$t/sitemap.xml"
status=$(run "$t")
report "a sitemap that lists an http:// URL is refused" 1 "$status" \
    "with \`http://\`" "$scratch/err"

# 14. No pages at all is exit 2, not a pass over nothing.
rm -rf "$t"
mkdir -p "$t"
status=$(run "$t")
report "an empty root exits 2 rather than reporting zero dead links" 2 "$status"
status=$(run "$scratch/missing")
report "a missing root exits 2" 2 "$status"

echo
echo "$passed passed, $failed failed"
[ "$failed" -eq 0 ]
