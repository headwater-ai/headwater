#!/bin/sh
# What holds the composition demonstration of the `diataxis` entry: the
# `reader_mode` facet read over another library entry's instance corpus.
#
# Run it from anywhere:
#     sh tools/repo/diataxis-facet-fixtures.sh
#
# # THE DEFECT THIS SUITE EXISTS FOR
#
# `docs/taxonomies/**` is excluded from this repository's corpus root
# (`.headwater/taxonomy.yml`), so a number recorded in a fixtures README
# reaches no gate at all. The sibling entry measured what that costs: a planted
# defect sat inside the `diataxis-site` worked corpus for two days with every
# gate green, because the only thing that stated the expected result was prose.
# `tools/repo/diataxis-fixtures.sh` is the program that now holds that entry.
# This is the same program for the composition section of
# `docs/taxonomies/diataxis/fixtures/README.md`.
#
# # WHAT THE DEMONSTRATION CLAIMS, AND WHAT IT DOES NOT
#
# The claim is a delta of exactly one finding between two arms over one corpus,
# and the message of that finding. It is never an exit status. Both arms report
# errors that predate this entry — the `design-spec` fixture corpus is not
# clean and was never assembled to be — so a strict run exits 1 in both arms
# and discriminates nothing.
#
# The demonstration also does not need, and cannot perform, a `require` on a
# kind another entry declares. That is the operation with no add-only form, and
# HW-OBL-0040 holds it. Nothing here reaches it.
#
# # WHAT EACH JUDGE READS
#
# No facet name, no facet value, no bundle list and no finding count is written
# in this file. Every population is enumerated out of a declaration:
#
#   THE FACET and ITS ADMITTED VALUES come from the `add:` block of
#   `docs/taxonomies/diataxis/bundle.yml`, which is what the resolver reads.
#
#   THE BUNDLE SELECTION is the sibling entry, the transitive closure of the
#   `requires:` list of each bundle in it, and the entry under demonstration. A
#   selection written down here would go stale the day an entry gained a
#   dependency.
#
#   THE TWO LABELED PAGES are the first two documents of the sibling corpus in
#   sorted path order. Which two they are is printed and not asserted: any two
#   typed pages of that corpus carry the same claim, because the facet is
#   attached to the base's abstract kind and reaches all of them.
#
# The one literal is the planted value, and case group 1 holds it: a value the
# entry has since declared would make the plant meaningless, so the suite
# reddens rather than passing for the wrong reason.
#
# # WHY THE DELTA IS TAKEN OVER (PAGE, RULE) AND NOT OVER REPORT LINES
#
# Injecting a front-matter key moves every line below it, so a finding at
# `docs/x.md:6:11` in one arm is the same finding at `docs/x.md:7:11` in the
# other. A delta over raw report lines would report that shift as a difference
# and the suite would redden on a page whose findings never changed. The pair
# of (page, rule) is what survives the injection, and case group 3 holds the
# parser that reads it against the report's own findings tally, so a parser
# that read nothing cannot report a green delta of zero.
#
# # THE POPULATION GUARDS, WHICH ARE NOT COUNTS
#
# Each group opens with a floor of zero. A judge whose population came back
# empty reports green for the wrong reason. The finding denominators are
# printed and never asserted: the absolute finding count over the sibling
# corpus is a property of somebody else's prose, and asserting it would redden
# this suite every time that corpus changed for an unrelated reason. What is
# asserted is the difference between two arms of one run.
#
# # WHAT IT NEEDS, AND WHAT IT WRITES
#
# `awk`, `sed`, `comm` and a built engine. Every scratch tree is made under
# `mktemp -d`, the directory goes on an interrupt, and nothing inside this
# checkout is written. The labels are injected into the scratch copy and never
# into either corpus in the tree.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
entry=docs/taxonomies/diataxis
sibling=docs/taxonomies/design-spec
bundle="$root/$entry/bundle.yml"
fixtures="$root/$entry/fixtures"
corpus="$root/$sibling/fixtures/corpus/docs"

# The one value this file writes down. Case group 1 holds it outside the set.
planted_value=cookbook

engine=""
if [ -x "$root/engine/target/release/headwater" ]; then
    engine="$root/engine/target/release/headwater"
fi
if [ -x "$root/engine/target/dev-release/headwater" ] &&
    { [ -z "$engine" ] || [ "$root/engine/target/dev-release/headwater" -nt "$engine" ]; }; then
    engine="$root/engine/target/dev-release/headwater"
fi
if [ -z "$engine" ]; then
    echo "no engine at engine/target/{release,dev-release}/headwater, and every" >&2
    echo "  case below runs one. This suite states that and stops rather than" >&2
    echo "  reporting a row of passes over a binary that is not there." >&2
    echo "  Build one:" >&2
    echo "  cargo build --profile dev-release -p headwater-cli --manifest-path engine/Cargo.toml --locked" >&2
    exit 1
fi

for needed in "$bundle" "$fixtures/README.md" "$corpus"; do
    if [ ! -e "$needed" ]; then
        echo "missing $needed, so the composition cases cannot run." >&2
        exit 1
    fi
done

scratch=$(mktemp -d) || exit 1
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

passed=0
failed=0

pass() {
    passed=$((passed + 1))
    echo "  ok    $1"
}

fail() {
    failed=$((failed + 1))
    echo "  FAIL  $1"
    [ $# -gt 1 ] && echo "          $2"
}

judge() { # name, expected, actual
    if [ "$2" = "$3" ]; then
        pass "$1"
    else
        fail "$1" "expected [$2], read [$3]"
    fi
}

# --- the populations, read out of the declarations -------------------------

# The facets the entry declares: every key under `add:` of the form
# `facets.<name>:`.
entry_facets() {
    awk '
        /^add:/ { in_add = 1; next }
        /^[a-z_]+:/ { in_add = 0 }
        in_add && /^  facets\.[a-z_]+:/ {
            f = $0
            sub(/^  facets\./, "", f)
            sub(/:.*/, "", f)
            print f
        }
    ' "$bundle" | sort -u
}

# The `values:` list of one facet, one per line.
facet_values() {
    awk -v want="$1" '
        /^add:/ { in_add = 1; next }
        /^[a-z_]+:/ { in_add = 0 }
        in_add && $0 ~ "^  facets\\." want ":" { hit = 1; next }
        in_add && /^  [a-z_]/ { hit = 0 }
        hit && /^    values:/ {
            v = $0
            sub(/^[^[]*\[/, "", v)
            sub(/\].*/, "", v)
            gsub(/ /, "", v)
            n = split(v, a, ",")
            for (i = 1; i <= n; i++) if (a[i] != "") print a[i]
            exit
        }
    ' "$bundle"
}

# The `requires:` list of one bundle file, one per line.
bundle_requires() {
    awk '
        /^requires:/ {
            v = $0
            sub(/^[^[]*\[/, "", v)
            sub(/\].*/, "", v)
            gsub(/ /, "", v)
            n = split(v, a, ",")
            for (i = 1; i <= n; i++) if (a[i] != "") print a[i]
            exit
        }
    ' "$1"
}

# The selection: the sibling entry, the entry under demonstration, and the
# transitive closure of every `requires:` below them.
selection_list() {
    : > "$scratch/sel"
    pending="$(basename "$sibling") $(basename "$entry")"
    while [ -n "$pending" ]; do
        next=""
        for b in $pending; do
            grep -qx "$b" "$scratch/sel" 2>/dev/null && continue
            echo "$b" >> "$scratch/sel"
            if [ -f "$root/docs/taxonomies/$b/bundle.yml" ]; then
                for r in $(bundle_requires "$root/docs/taxonomies/$b/bundle.yml"); do
                    next="$next $r"
                done
            fi
        done
        pending="$next"
    done
    sort -u "$scratch/sel"
}

# A scratch root that selects the bundles named in $2. $1 = destination.
make_root() {
    at=$1
    bundles=$2
    mkdir -p "$at/packages" "$at/.headwater" "$at/docs"
    cp -r "$root/packages/headwater-standard" "$at/packages/"
    {
        echo 'taxonomy:'
        echo '  package: headwater/standard'
        sed -n 's/^  version: /  version: /p' "$root/.headwater/taxonomy.yml" | head -n 1
        sed -n 's/^  digest: /  digest: /p' "$root/.headwater/taxonomy.yml" | head -n 1
        echo "  bundles: [$bundles]"
        echo '  overlay: .headwater/overlay.yml'
        echo
        echo 'corpus:'
        echo '  root: docs'
    } > "$at/.headwater/taxonomy.yml"
    # The overlay supplies exactly the namespaces the selection is missing, and
    # it is derived rather than listed: an overlay that declares a namespace on
    # a scheme the selection does not carry is itself a refusal, so a written
    # list would redden every time the bundle set changed.
    echo 'add: {}' > "$at/.headwater/overlay.yml"
    (cd "$at" && "$engine" taxonomy validate) > "$scratch/ns.out" 2> "$scratch/ns.err"
    sed -n 's/.*`identifier_schemes\.\([a-z_]*\)`: identifier integrity: carries no namespace.*/\1/p' \
        "$scratch/ns.out" "$scratch/ns.err" | sort -u > "$scratch/ns.list"
    {
        echo 'add:'
        while read -r scheme; do
            [ -n "$scheme" ] || continue
            echo "  identifier_schemes.${scheme}.namespace: DX"
        done < "$scratch/ns.list"
    } > "$at/.headwater/overlay.yml"
    rm -rf "$at/docs"
    cp -r "$corpus" "$at/docs"
}

# One arm: resolve, then check with an injected clock. $1 = root, $2 = tag.
run_arm() {
    (cd "$1" && "$engine" taxonomy resolve) > "$scratch/$2-resolve.out" 2>&1
    (cd "$1" && "$engine" check --no-cache --now "$today") \
        > "$scratch/$2.out" 2> "$scratch/$2.err"
    echo $? > "$scratch/$2.code"
}

# A scalar of the census or the checks block: `<n> <label>` at the end of a
# line, so `115 check instances` reads out of the compound line that carries it.
report_number() { # file, label
    sed -n "s/.*[ ,]\([0-9][0-9]*\) $2\$/\1/p" "$1" | head -n 1
}

# Every finding of a report as `<page> TAB <rule>`. A finding opens with a
# header line that carries a severity glyph and closes the line after, which
# names the rule and its obligation. The location is dropped on purpose: see
# the header of this file.
finding_pairs() {
    awk '
        /^  [^ ].* (✗|▲|●) (error|warn|info)$/ {
            hdr = $0
            sub(/^  /, "", hdr)
            sub(/ [^ ]+ [a-z]+$/, "", hdr)
            page = hdr
            sub(/:[0-9]+:[0-9]+$/, "", page)
            pending = 1
            next
        }
        pending && /^    [a-z]/ {
            r = $0
            sub(/^    /, "", r)
            sub(/[ (].*/, "", r)
            print page "\t" r
            pending = 0
        }
    ' "$1" | sort
}

# The whole reported block of one finding. $1 = file, $2 = page, $3 = rule.
finding_block() {
    awk -v want_page="$2" -v want_rule="$3" '
        function flush() { if (inb && ok) printf "%s", buf; inb = 0; ok = 0; buf = "" }
        /^  [^ ].* (✗|▲|●) (error|warn|info)$/ {
            flush()
            hdr = $0
            sub(/^  /, "", hdr)
            sub(/ [^ ]+ [a-z]+$/, "", hdr)
            p = hdr
            sub(/:[0-9]+:[0-9]+$/, "", p)
            inb = (p == want_page)
            matched = 0
            ok = 0
            buf = $0 "\n"
            next
        }
        inb && matched == 0 && /^    [a-z]/ {
            r = $0
            sub(/^    /, "", r)
            sub(/[ (].*/, "", r)
            matched = 1
            ok = (r == want_rule)
            buf = buf $0 "\n"
            next
        }
        inb && /^    / { buf = buf $0 "\n"; next }
        inb { flush() }
        END { flush() }
    ' "$1"
}

today=$(date -u +%Y-%m-%d)

facets=$(entry_facets)
facet_floor=$(printf '%s\n' "$facets" | grep -c . || true)
facet=$(printf '%s\n' "$facets" | head -n 1)
values=""
[ "$facet_floor" -ge 1 ] && values=$(facet_values "$facet")
value_floor=$(printf '%s\n' "$values" | grep -c . || true)
pages=$(cd "$corpus/.." && find docs -name '*.md' | sort)
page_floor=$(printf '%s\n' "$pages" | grep -c . || true)

echo "diataxis composition over $sibling, against $engine"
echo

echo "population guards"
judge "the entry declares exactly one facet" 1 "$facet_floor"
if [ "$value_floor" -lt 2 ]; then
    fail "the facet declares at least two values" "$value_floor parsed out of $entry/bundle.yml"
else
    pass "the facet declares at least two values ($value_floor read: $(printf '%s' "$values" | tr '\n' ' '))"
fi
if [ "$page_floor" -lt 2 ]; then
    fail "the sibling corpus holds at least two documents" "$page_floor under $sibling/fixtures/corpus"
else
    pass "the sibling corpus holds at least two documents ($page_floor read)"
fi
[ "$facet_floor" -eq 1 ] || exit 1
[ "$value_floor" -ge 2 ] || exit 1
[ "$page_floor" -ge 2 ] || exit 1

labeled_page=$(printf '%s\n' "$pages" | sed -n '1p')
planted_page=$(printf '%s\n' "$pages" | sed -n '2p')
good_value=$(printf '%s\n' "$values" | tail -n 1)

echo
echo "case group 1 — the plant is outside the set the entry declares"
if printf '%s\n' "$values" | grep -qx "$planted_value"; then
    fail "the planted value is not one the entry declares" \
        "\`$facet\` now admits $planted_value, so the plant demonstrates nothing"
else
    pass "the planted value is not one the entry declares ($planted_value)"
fi

echo
echo "case group 2 — the composed selection resolves and validates"
selection=$(selection_list | tr '\n' ',' | sed -e 's/,/, /g' -e 's/, *$//')
echo "  selection:     $selection"
unlabeled="$scratch/unlabeled"
make_root "$unlabeled" "$selection"
(cd "$unlabeled" && "$engine" taxonomy validate) > "$scratch/validate.out" 2> "$scratch/validate.err"
judge "the composed selection validates" 0 "$?"
judge "validate's last line names the package as valid" \
    "headwater/standard is valid" "$(tail -n 1 "$scratch/validate.out")"
(cd "$unlabeled" && "$engine" taxonomy resolve) > "$scratch/resolve.out" 2> "$scratch/resolve.err"
judge "the composed selection resolves" 0 "$?"

labeled="$scratch/labeled"
make_root "$labeled" "$selection"

echo
echo "case group 3 — the two arms, and the parser that reads them"
run_arm "$unlabeled" unlabeled
run_arm "$labeled" labeled

u_files=$(report_number "$scratch/unlabeled.out" "files under the corpus root")
u_typed=$(report_number "$scratch/unlabeled.out" "typed")
u_inst=$(report_number "$scratch/unlabeled.out" "check instances")
u_find=$(report_number "$scratch/unlabeled.out" "findings")
l_files=$(report_number "$scratch/labeled.out" "files under the corpus root")
l_typed=$(report_number "$scratch/labeled.out" "typed")
l_inst=$(report_number "$scratch/labeled.out" "check instances")
l_find=$(report_number "$scratch/labeled.out" "findings")

finding_pairs "$scratch/unlabeled.out" > "$scratch/unlabeled.pairs"
finding_pairs "$scratch/labeled.out" > "$scratch/labeled.pairs"
u_pairs=$(grep -c . "$scratch/unlabeled.pairs" || true)
l_pairs=$(grep -c . "$scratch/labeled.pairs" || true)

if [ "${u_inst:-0}" -eq 0 ]; then
    fail "the checks block reports a non-zero instance count" "none parsed"
else
    pass "the checks block reports a non-zero instance count (${u_inst} read)"
fi
judge "the parser reads every finding the unlabeled arm tallies" "${u_find:-0}" "$u_pairs"
judge "the parser reads every finding the labeled arm tallies" "${l_find:-0}" "$l_pairs"
judge "both arms walk the same census" "${u_files:-0}/${u_typed:-0}" "${l_files:-0}/${l_typed:-0}"
judge "both arms instantiate the same number of checks" "${u_inst:-0}" "${l_inst:-0}"

echo
echo "case group 4 — the one finding the labels add"
judge "the labeled arm carries exactly one more finding" \
    "$((${u_find:-0} + 1))" "${l_find:-0}"
comm -13 "$scratch/unlabeled.pairs" "$scratch/labeled.pairs" > "$scratch/gained"
comm -23 "$scratch/unlabeled.pairs" "$scratch/labeled.pairs" > "$scratch/lost"
judge "the labels lose no finding" "" \
    "$(tr '\t' ' ' < "$scratch/lost" | tr '\n' ';' | sed 's/;*$//')"
judge "the labels gain exactly one" 1 "$(grep -c . "$scratch/gained" || true)"
judge "what they gain is a facet value refusal against the planted page" \
    "$planted_page facet.value.not_permitted" \
    "$(tr '\t' ' ' < "$scratch/gained" | head -n 1 | sed 's/ *$//')"

echo
echo "case group 5 — what that finding says"
finding_block "$scratch/labeled.out" "$planted_page" facet.value.not_permitted \
    > "$scratch/gained.block"
if [ ! -s "$scratch/gained.block" ]; then
    fail "the gained finding has a reported block" "nothing parsed out of the report"
else
    pass "the gained finding has a reported block"
    if grep -q "\`$facet\`" "$scratch/gained.block"; then
        pass "it names the facet the entry declares ($facet)"
    else
        fail "it names the facet the entry declares ($facet)" "not in the block"
    fi
    if grep -q "\`$planted_value\`" "$scratch/gained.block"; then
        pass "it names the value the page declares ($planted_value)"
    else
        fail "it names the value the page declares ($planted_value)" "not in the block"
    fi
    missing=""
    for v in $values; do
        grep -q "$v" "$scratch/gained.block" || missing="$missing $v"
    done
    judge "it enumerates every value the entry admits" "" "$(echo "$missing" | sed 's/^ *//')"
fi

echo
echo "case group 6 — the correctly labeled page stays silent"
grep "^$labeled_page	" "$scratch/unlabeled.pairs" > "$scratch/labeled-page.before" || true
grep "^$labeled_page	" "$scratch/labeled.pairs" > "$scratch/labeled-page.after" || true
judge "an admitted value adds no finding to the page that carries it" "" \
    "$(comm -13 "$scratch/labeled-page.before" "$scratch/labeled-page.after" |
        tr '\t' ' ' | tr '\n' ';' | sed 's/;*$//')"
said=""
while IFS="$(printf '\t')" read -r p r; do
    [ -n "${r:-}" ] || continue
    finding_block "$scratch/labeled.out" "$p" "$r" > "$scratch/page.block"
    grep -q "$facet" "$scratch/page.block" && said="$said $r"
done < "$scratch/labeled-page.after"
judge "no finding against that page names $facet" "" "$(echo "$said" | sed 's/^ *//')"

echo
echo "the run record, recorded and not asserted"
echo "  engine:        $("$engine" --version)"
echo "  now:           $today"
echo "  selection:     $selection"
echo "  labeled page:  $labeled_page   <- $facet: $good_value"
echo "  planted page:  $planted_page   <- $facet: $planted_value"
echo "  census:        ${u_files:-0} files, ${u_typed:-0} typed, ${u_inst:-0} check instances"
echo "  findings:      ${u_find:-0} unlabeled, ${l_find:-0} labeled"
echo "  check exit:    $(cat "$scratch/unlabeled.code") unlabeled, $(cat "$scratch/labeled.code") labeled"
echo "  the one finding:"
sed 's/^/  /' "$scratch/gained.block"

echo
echo "$passed passed, $failed failed"
[ "$failed" -eq 0 ] || exit 1
