#!/bin/sh
# What holds `tools/site/render-tutorial.py` to "Before you start".
#
# The renderer writes the tutorial page's "Before you start" section from the
# tutorial document, and it refuses a document whose prose names a version
# other than the tag in its own download link (#1138). Each case below plants
# one drift in a scratch copy of the page or the document and runs the
# renderer there. The script prints how many cases ran, because an empty run
# is not a pass.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
tool="$root/tools/site/render-tutorial.py"
doc_rel=docs/tutorials/your-first-governed-corpus.md
page_rel=site/tutorial/index.html

for f in "$tool" "$root/$doc_rel" "$root/$page_rel"; do
    if [ ! -f "$f" ]; then
        echo "render-tutorial-fixtures: no file at \`$f\`." >&2
        exit 1
    fi
done

scratch=$(mktemp -d) || exit 1
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

passed=0
failed=0

# copy NAME: a scratch root the renderer resolves as its own, since it reads
# `parents[2]` of its own path.
copy() {
    d="$scratch/$1"
    mkdir -p "$d/tools/site" "$d/site/tutorial" "$d/docs/tutorials"
    cp "$tool" "$d/tools/site/"
    cp "$root/$page_rel" "$d/$page_rel"
    cp "$root/$doc_rel" "$d/$doc_rel"
    echo "$d"
}

# plant FILE OLD NEW: replace the first OLD with NEW, and fail the case when
# OLD is not there, so that a fixture that plants nothing never passes.
plant() {
    python3 -c '
import sys, pathlib
p, old, new = pathlib.Path(sys.argv[1]), sys.argv[2], sys.argv[3]
t = p.read_text()
if old not in t:
    sys.exit("plant: %r is not in %s" % (old, p))
p.write_text(t.replace(old, new, 1))
' "$1" "$2" "$3"
}

# run ROOT ARGS...: run the renderer in ROOT and print its exit status.
run() {
    r=$1
    shift
    python3 "$r/tools/site/render-tutorial.py" "$@" >"$scratch/out" 2>"$scratch/err"
    echo $?
}

# report NAME EXPECTED-STATUS ACTUAL-STATUS REQUIRED-TEXT
report() {
    if [ "$2" = "$3" ] && cat "$scratch/out" "$scratch/err" | grep -qF -- "$4"; then
        passed=$((passed + 1))
        echo "  ok    $1"
    else
        failed=$((failed + 1))
        echo "  FAIL  $1 (want exit $2 and \"$4\", got exit $3)"
        sed 's/^/        /' "$scratch/out" "$scratch/err"
    fi
}

echo "the control"
d=$(copy control)
report "an unchanged copy matches" 0 "$(run "$d" --check)" "matches the document"

echo "page drift"
d=$(copy page-prose)
if plant "$d/$page_rel" "This installs version 0.4.0" "This installs version 0.2.1"; then
    report "a changed version on the page is stale" 1 "$(run "$d" --check)" "stale"
else
    report "a changed version on the page is stale" planted unplanted ""
fi

d=$(copy page-block)
if plant "$d/$page_rel" '<pre class="cmd verbatim">mkdir -p ~/.local/bin' '<pre class="cmd verbatim">mkdir -p ~/.local/bin
echo planted'; then
    report "a line added to the install block is stale" 1 "$(run "$d" --check)" "stale"
else
    report "a line added to the install block is stale" planted unplanted ""
fi

echo "document drift"
d=$(copy doc-prereq)
if plant "$d/$doc_rel" "- About twenty minutes." "- About twenty minutes.
- A planted prerequisite."; then
    report "a prerequisite added to the document makes the page stale" 1 "$(run "$d" --check)" "stale"
else
    report "a prerequisite added to the document makes the page stale" planted unplanted ""
fi

d=$(copy doc-version)
if plant "$d/$doc_rel" "This installs version 0.4.0" "This installs version 0.2.1"; then
    report "prose naming another version stops --check" 1 "$(run "$d" --check)" "0.2.1"
    report "prose naming another version stops a write, naming the pin" 1 "$(run "$d")" "0.4.0"
else
    report "prose naming another version stops the renderer" planted unplanted ""
fi

echo "$((passed + failed)) cases ran: $passed passed; $failed failed"
[ "$failed" -eq 0 ] && [ "$passed" -gt 0 ]
