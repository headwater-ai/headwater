#!/bin/sh
# What holds `tools/site/check-site-redirects.py`.
#
# Spec 12 refuses a check that ships with no failing fixture. Every refusal
# below is provoked on purpose, and so is every pass, because a gate that
# refuses everything is as useless as one that refuses nothing.
# `tools/site/site-noindex-fixtures.sh` is the shape.
#
# Run it from anywhere:
#     sh tools/site/site-redirects-fixtures.sh
#
# # THE CASE THIS SUITE EXISTS FOR
#
# Case 12: a document moves to a process shelf and nothing answers its old URL.
# That is how 44 URLs of headwater.tools became "Not found (404)" in Search
# Console, and the check must exit 1 on it and name the line to add.
#
# # THE ASSUMPTION IT STATES, AND FAILS ON
#
# Case 1 runs over the real assembled directory, so it needs one. It stops
# when there is none rather than passing over nothing. Build it first:
#     python3 -m mkdocs build --strict && sh tools/site/assemble-site.sh
#
# # WHAT IT WRITES
#
# Nothing inside this checkout. Every synthetic case, and the scratch git
# repository the rename cases need, runs under `mktemp -d`.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
tool="$root/tools/site/check-site-redirects.py"
deploy="$root/.headwater/site-deploy"

if [ ! -f "$tool" ]; then
    echo "no checker at \`tools/site/check-site-redirects.py\`." >&2
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

# run ARG... -> writes $scratch/out and $scratch/err, prints the status
run() {
    python3 "$tool" "$@" >"$scratch/out" 2>"$scratch/err"
    echo $?
}

# page DIR RELATIVE-DIR : a served page at RELATIVE-DIR/index.html
page() {
    mkdir -p "$1/$2"
    printf '<!DOCTYPE html>\n<html><head><title>%s</title></head><body></body></html>\n' "$2" \
        >"$1/$2/index.html"
}

# tree DIR [RULE...] : a served tree of three pages and the given redirect rules
tree() {
    dir=$1
    shift
    rm -rf "$dir"
    page "$dir" .
    page "$dir" spec
    page "$dir" process/obligations/0203-new
    : >"$dir/_redirects"
    for rule in "$@"; do printf '%s\n' "$rule" >>"$dir/_redirects"; done
}

echo "over the real assembled site"

# 1. The site as it stands, with a non-zero denominator.
status=$(run "$deploy")
report "the assembled site's redirect file holds" 0 "$status"
if grep -qE '^[0-9]+ static and [0-9]+ dynamic redirect rules? read' "$scratch/out" \
    && ! grep -q '^0 static and 0 dynamic' "$scratch/out"; then
    passed=$((passed + 1)); echo "  ok    it states a non-zero rule count"
else
    failed=$((failed + 1)); echo "  FAIL  it states a non-zero rule count"
    sed 's/^/          /' "$scratch/out"
fi

echo "over synthetic trees"
t="$scratch/site"
good='/obligations/0203-old/ /process/obligations/0203-new/ 301'

# 2. A clean tree passes.
tree "$t" "# a comment" "$good"
status=$(run "$t")
report "a clean redirect file passes" 0 "$status"

# 3. A rule with no status is a 302.
tree "$t" '/obligations/0203-old/ /process/obligations/0203-new/'
status=$(run "$t")
report "a rule with no status is refused" 1 "$status" "which is a 302" "$scratch/err"

# 4. A temporary redirect for a page that moved for good.
tree "$t" '/obligations/0203-old/ /process/obligations/0203-new/ 302'
status=$(run "$t")
report "a 302 for a moved page is refused" 1 "$status" "needs 301 or 308" "$scratch/err"

# 5. A destination that no file answers.
tree "$t" '/obligations/0203-old/ /process/obligations/0203-gone/ 301'
status=$(run "$t")
report "a redirect to a page that is not served is refused" 1 "$status" \
    "which no served file answers" "$scratch/err"

# 6. A source that is also a served page.
tree "$t" '/spec/ /process/obligations/0203-new/ 301'
status=$(run "$t")
report "a source that is a served page is refused" 1 "$status" \
    "a redirect source and a served page" "$scratch/err"

# 7. A chain of two rules.
tree "$t" "$good" '/obligations/0203-older/ /obligations/0203-old/ 301'
status=$(run "$t")
report "a redirect to a redirected URL is refused" 1 "$status" \
    "name the last page in the first rule" "$scratch/err"

# 8. A loop is a chain, so it is refused too.
tree "$t" '/a/ /b/ 301' '/b/ /a/ 301'
status=$(run "$t")
report "a redirect loop is refused" 1 "$status" "itself redirected" "$scratch/err"

# 9. A source listed twice.
tree "$t" "$good" "$good"
status=$(run "$t")
report "a source listed twice is refused" 1 "$status" "is the source of two rules" "$scratch/err"

# 10. A rule that does not start with a slash.
tree "$t" 'obligations/0203-old/ /process/obligations/0203-new/ 301'
status=$(run "$t")
report "a source with no leading slash is refused" 1 "$status" \
    "the source must start with" "$scratch/err"

# 11. More static rules than the host allows.
tree "$t"
i=0
while [ $i -le 2000 ]; do
    printf '/old-%s/ /spec/ 301\n' "$i" >>"$t/_redirects"
    i=$((i + 1))
done
status=$(run "$t")
report "more than 2000 static rules is refused" 1 "$status" "static rules; the host allows" \
    "$scratch/err"

echo "over a renamed document"

# A scratch repository with one document, then the same document under a
# process shelf. `--repo` points the checker at it.
repo="$scratch/repo"
rm -rf "$repo"
mkdir -p "$repo/docs/obligations" "$repo/docs/taxonomies/x/fixtures"
printf 'exclude_docs: |\n  taxonomies/*/fixtures/\n  taxonomies/*/templates/\n' >"$repo/mkdocs.yml"
printf '# old\n' >"$repo/docs/obligations/0203-old.md"
printf '# fixture\n' >"$repo/docs/taxonomies/x/fixtures/README.md"
printf '# gone\n' >"$repo/docs/obligations/0300-gone.md"
g() { git -C "$repo" -c user.name=t -c user.email=t@example.com "$@"; }
g init -q . 2>/dev/null || git init -q "$repo"
g add -A
g commit -q -m base
base=$(g rev-parse HEAD)
mkdir -p "$repo/docs/process/obligations" "$repo/docs/taxonomies/y/fixtures"
g mv docs/obligations/0203-old.md docs/process/obligations/0203-old.md
g mv docs/taxonomies/x/fixtures/README.md docs/taxonomies/y/fixtures/README.md
g commit -q -am "move"

# 12. The document moved and nothing answers the old URL.
tree "$t"
page "$t" obligations/0203-other
status=$(run "$t" --base "$base" --repo "$repo")
report "a renamed document whose old URL answers nothing is refused" 1 "$status" \
    "add \`/obligations/0203-old/ /process/obligations/0203-old/ 301\`" "$scratch/err"
cp "$scratch/err" "$scratch/err12"

# 13. The same move with a redirect for it.
tree "$t" '/obligations/0203-old/ /process/obligations/0203-new/ 301'
status=$(run "$t" --base "$base" --repo "$repo")
report "a renamed document with a redirect passes" 0 "$status"

# 14. The same move where the old URL is still a served page.
tree "$t"
page "$t" obligations/0203-old
status=$(run "$t" --base "$base" --repo "$repo")
report "a renamed document whose old URL is still served passes" 0 "$status"

# 15. A document that exclude_docs removes was never served, so moving it costs
#     nothing. Case 12 moved a fixture as well as the obligation and left both
#     URLs unanswered, so its report must name the obligation and never the
#     fixture.
if grep -q 'fixtures' "$scratch/err12" || ! grep -q '0203-old' "$scratch/err12"; then
    failed=$((failed + 1)); echo "  FAIL  a renamed excluded document is not read"
else
    passed=$((passed + 1)); echo "  ok    a renamed excluded document is not read"
fi

# 16. A deleted document is reported and does not fail.
g rm -q docs/obligations/0300-gone.md
g commit -q -m delete
tree "$t" '/obligations/0203-old/ /process/obligations/0203-new/ 301'
status=$(run "$t" --base "$base" --repo "$repo")
report "a deleted document is reported and does not fail" 0 "$status" \
    "docs/obligations/0300-gone.md\` was deleted" "$scratch/err"

# 17. A base the repository cannot see is exit 2, not a pass over nothing.
status=$(run "$t" --base 0000000000000000000000000000000000000000 --repo "$repo")
report "an unreadable base exits 2" 2 "$status"

# 18. No pages at all is exit 2.
rm -rf "$t"
mkdir -p "$t"
status=$(run "$t")
report "an empty root exits 2" 2 "$status"

echo
echo "$passed passed, $failed failed"
[ "$failed" -eq 0 ]
