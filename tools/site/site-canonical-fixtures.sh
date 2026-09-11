#!/bin/sh
# What holds `tools/site/check-site-canonical.py`.
#
# Spec 12 refuses a check that ships with no failing fixture, and the reason
# carries to a gate: a gate that has refused nothing is a gate nobody has seen
# work. Every refusal below is provoked on purpose, and so is every pass,
# because a gate that refuses everything is as useless as one that refuses
# nothing. `tools/site/site-fragments-fixtures.sh` is the shape.
#
# Run it from anywhere:
#     sh tools/site/site-canonical-fixtures.sh
#
# # THE ASSUMPTION IT STATES, AND FAILS ON
#
# The cases over the real corpus need a built generated half. This states that
# and stops when there is none, rather than reporting a row of passes over an
# empty directory. That is the same posture the tool itself takes when it is
# pointed at a directory with no page in it.
#
# # THE TWO CASES THIS SUITE EXISTS FOR
#
# Case 4 deletes the canonical element from one page of the real build, and
# case 5 points one page's canonical at another URL. Those are the two ways the
# gate can be needed, and a presence-only grep would pass case 5. Both run over
# a scratch copy, so neither touches the build in this checkout.
#
# # WHAT IT WRITES
#
# Nothing inside this checkout. Every case runs under `mktemp -d`, and the
# directory goes on an interrupt.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
tool="$root/tools/site/check-site-canonical.py"
build="$root/.headwater/site-build"
origin="https://headwater.tools"

if [ ! -f "$tool" ]; then
    echo "no checker at \`tools/site/check-site-canonical.py\`." >&2
    exit 1
fi

if [ ! -d "$build" ]; then
    echo "no generated half at \`.headwater/site-build\`, so the cases over the" >&2
    echo "  real corpus cannot run. Build it first:" >&2
    echo "        python3 -m mkdocs build --strict" >&2
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
    # run DIR [extra args...] -> writes $scratch/out and $scratch/err
    dir=$1
    shift
    ( cd "$root" && python3 "$tool" "$dir" "$@" >"$scratch/out" 2>"$scratch/err" )
    echo $?
}

# page DIR RELATIVE-PATH [CANONICAL-HREF ...]
# Writes a minimal page carrying one `<link rel="canonical">` per href given,
# and none at all when no href is given.
page() {
    dir=$1
    rel=$2
    shift 2
    mkdir -p "$dir/$(dirname "$rel")"
    {
        printf '<!DOCTYPE html>\n<html><head>\n'
        for href in "$@"; do
            printf '<link rel="canonical" href="%s">\n' "$href"
        done
        printf '<title>a page</title>\n</head><body><p>%s</p></body></html>\n' "$rel"
    } >"$dir/$rel"
}

echo "over the generated half of the real site"

# 1. The corpus as it stands. A failure here is a real defect rather than a
#    broken fixture.
status=$(run "$build")
report "every generated page carries its own canonical URL" 0 "$status"

# 2. The denominator is stated and is not zero. A run that checked nothing
#    would report zero findings too, and that is the pass this suite must not
#    accept on the corpus's behalf.
if grep -qE '[1-9][0-9]* of [1-9][0-9]* generated pages carry one canonical URL' "$scratch/out"; then
    passed=$((passed + 1)); echo "  ok    it states a non-zero denominator"
else
    failed=$((failed + 1)); echo "  FAIL  it states a non-zero denominator"
    echo "          it reported: $(tail -1 "$scratch/out")"
fi

# 3. The one exemption is named on every run rather than being silent. An
#    exemption nobody can see is indistinguishable from a page the walk missed.
report "it names \`404.html\` as the exempt page" 0 "$status" \
    "Exempt: 404.html." "$scratch/out"

echo
echo "over a scratch copy of the real build, with one page broken"

copy="$scratch/build"
cp -r "$build" "$copy"
victim=$(cd "$copy" && find . -name index.html | sort | sed -n '1p' | sed 's|^\./||')

# 4. THE FIRST CASE THIS SUITE EXISTS FOR. Delete the canonical from one real
#    page. This is what a dropped `site_url` does to all 302 at once.
grep -v 'rel="canonical"' "$copy/$victim" >"$scratch/stripped"
mv "$scratch/stripped" "$copy/$victim"
status=$(run "$copy")
report "a real page with its canonical deleted is refused" 1 "$status" \
    "pages carrying no canonical URL:" "$scratch/err"
report "  and the refusal names that page" 1 "$status" "$victim" "$scratch/err"

# 5. THE SECOND CASE THIS SUITE EXISTS FOR. Point one real page's canonical at
#    another URL. A gate that greps for the string `rel="canonical"` passes
#    this, and a search engine believes the wrong answer.
#
#    The page is restored from the build first. Without that line this case
#    edits the page case 4 already stripped, the `sed` matches nothing, and the
#    run exits 1 for case 4's reason while reading as a pass for this one. That
#    is the shape this whole suite is against, met inside the suite itself.
cp "$build/$victim" "$copy/$victim"
sed 's|rel="canonical" href="[^"]*"|rel="canonical" href="'"$origin"'/somewhere-else/"|' \
    "$copy/$victim" >"$scratch/pointed"
cp "$scratch/pointed" "$copy/$victim"
status=$(run "$copy")
report "a real page whose canonical names another URL is refused" 1 "$status" \
    "pages whose canonical URL is not the URL they are served at:" "$scratch/err"
report "  and the refusal names that page" 1 "$status" "$victim" "$scratch/err"
report "  and prints both the URL it says and the URL it is served at" 1 "$status" \
    "$origin/somewhere-else/" "$scratch/err"

rm -rf "$copy"

echo
echo "over scratch pages"

# 6. The passing shape, so the suite is not one that refuses everything.
good="$scratch/good"
page "$good" "spec/12-check-layer/index.html" "$origin/spec/12-check-layer/"
page "$good" "index.html" "$origin/"
status=$(run "$good")
report "a directory URL and a root page both pass" 0 "$status" \
    "2 of 2 generated pages" "$scratch/out"

# 7. A missing trailing slash is a different URL, and a presence grep passes it.
slash="$scratch/slash"
page "$slash" "spec/12-check-layer/index.html" "$origin/spec/12-check-layer"
status=$(run "$slash")
report "a canonical missing the trailing slash is refused" 1 "$status" \
    "1 naming another URL" "$scratch/out"

# 8. A typo in `site_url` reaches every page as a wrong origin.
typo="$scratch/typo"
page "$typo" "spec/12-check-layer/index.html" "https://headwater.tool/spec/12-check-layer/"
status=$(run "$typo")
report "a canonical on the wrong origin is refused" 1 "$status" \
    "https://headwater.tool/spec/12-check-layer/" "$scratch/err"

# 9. Two canonicals on one page name two answers, which is no answer.
two="$scratch/two"
page "$two" "a/index.html" "$origin/a/" "$origin/b/"
status=$(run "$two")
report "a page carrying two canonical URLs is refused" 1 "$status" \
    "pages carrying more than one canonical URL:" "$scratch/err"

# 10. A page that is not an `index.html` is served at its own path.
flat="$scratch/flat"
page "$flat" "assets/note.html" "$origin/assets/note.html"
status=$(run "$flat")
report "a page that is not an index is served at its own path" 0 "$status"

# 11. The exemption is honored: `404.html` needs no canonical.
notfound="$scratch/notfound"
page "$notfound" "a/index.html" "$origin/a/"
page "$notfound" "404.html"
status=$(run "$notfound")
report "\`404.html\` with no canonical passes" 0 "$status" \
    "Exempt: 404.html." "$scratch/out"

# 12. And the exemption is total rather than presence-only. A wrong canonical
#     on `404.html` is not read either, which is what "exempt" has to mean.
wrong404="$scratch/wrong404"
page "$wrong404" "a/index.html" "$origin/a/"
page "$wrong404" "404.html" "$origin/somewhere/"
status=$(run "$wrong404")
report "\`404.html\` is exempt from the correctness pass too" 0 "$status"

# 13. The exemption is by exact path at the root and not by file name, so a
#     shelf that publishes its own `404.html` is checked like any other page.
deep404="$scratch/deep404"
page "$deep404" "spec/404.html"
status=$(run "$deep404")
report "a \`404.html\` inside a shelf is not exempt" 1 "$status" \
    "spec/404.html" "$scratch/err"

# 14. An empty directory satisfies both assertions vacuously. That is exit 2.
empty="$scratch/empty"
mkdir -p "$empty"
status=$(run "$empty")
report "an empty directory is refused rather than passed" 2 "$status" \
    "would report success without looking" "$scratch/err"

# 15. So is a directory whose only page is the exempt one. The population is
#     empty after the exemption, and the exemption must not create a vacuous
#     pass.
only404="$scratch/only404"
page "$only404" "404.html"
status=$(run "$only404")
report "a directory holding only the exempt page is refused" 2 "$status" \
    "1 of them exempt" "$scratch/err"

# 16. A directory that is not there at all is an environment failure and not a
#     finding, and it names the command that produces one.
status=$(run "$scratch/no-such-directory")
report "a missing directory exits 2 and names \`mkdocs build\`" 2 "$status" \
    "mkdocs build --strict" "$scratch/err"

# 17. `--site-url` is what makes this tool independent of `mkdocs.yml`, and it
#     accepts the origin with or without a trailing slash.
status=$(run "$good" --site-url "$origin")
report "\`--site-url\` without a trailing slash gives the same verdict" 0 "$status"

# 18. And a `--site-url` that disagrees with the pages refuses them, which is
#     the whole reason the origin is stated twice.
status=$(run "$good" --site-url "https://example.invalid/")
report "a \`--site-url\` the pages disagree with is refused" 1 "$status" \
    "2 naming another URL" "$scratch/out"

# 19. The `rel` attribute is matched by token and case-insensitively, which is
#     what HTML says it means. A page carrying `rel="canonical alternate"` is
#     read, and so is `rel="Canonical"`.
tokens="$scratch/tokens"
mkdir -p "$tokens/a"
printf '<!DOCTYPE html><html><head><link rel="Canonical alternate" href="%s/a/"><title>t</title></head><body></body></html>\n' \
    "$origin" >"$tokens/a/index.html"
status=$(run "$tokens")
report "a multi-token, mixed-case \`rel\` is read as a canonical" 0 "$status" \
    "1 of 1 generated pages" "$scratch/out"

echo
echo "$passed passed, $failed failed"
[ "$failed" -eq 0 ]
