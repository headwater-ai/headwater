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
# Case groups 2-7 are the optional half: the entry attaches the facet to the
# base's abstract kind and needs no operation on another entry. Case group 8 is
# the required half, which HW-DR-0095 (Q67) made legal: a bundle that names the
# sibling entry in `requires` writes `add_to` into the `facets.require` list of
# a kind that entry declares. The `diataxis` entry itself does not do this, and
# its bundle comments say why. So group 8 builds a scratch-only bundle, adds it
# to a scratch copy of the package source and publishes that copy, because a
# bundle written into the vendored copy would fail `taxonomy.pin.diverged`.
# Nothing in the tree gains the bundle.
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
    ' "$bundle" | LC_ALL=C sort -u
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
    LC_ALL=C sort -u "$scratch/sel"
}

# A scratch root that selects the bundles named in $2. $1 = destination. $3
# and $4, when given, are a published package directory and the digest its
# publish printed. Without them the root takes the vendored copy and the pin
# of this repository.
make_root() {
    at=$1
    bundles=$2
    pkg=${3:-$root/.headwater/packages/headwater-standard}
    mkdir -p "$at/.headwater/packages" "$at/docs"
    cp -r "$pkg" "$at/.headwater/packages/headwater-standard"
    {
        echo 'taxonomy:'
        echo '  package: headwater/standard'
        sed -n 's/^version: /  version: /p' "$pkg/package.yml" | head -n 1
        if [ -n "${4:-}" ]; then
            echo "  digest: $4"
        else
            sed -n 's/^  digest: /  digest: /p' "$root/.headwater/taxonomy.yml" | head -n 1
        fi
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
        "$scratch/ns.out" "$scratch/ns.err" | LC_ALL=C sort -u > "$scratch/ns.list"
    # A selection that is refused before validation names no scheme, and an
    # empty `add:` is itself a refusal that would hide the one under test.
    if [ -s "$scratch/ns.list" ]; then
        {
            echo 'add:'
            while read -r scheme; do
                [ -n "$scheme" ] || continue
                echo "  identifier_schemes.${scheme}.namespace: DX"
            done < "$scratch/ns.list"
        } > "$at/.headwater/overlay.yml"
    fi
    rm -rf "$at/docs"
    cp -r "$corpus" "$at/docs"
}

# The label, written into the scratch copy of one page and never into the
# tree. It goes at the top of the front-matter block, which moves every line
# below it; nothing downstream reads a line number, for the reason the header
# of this file gives. $1 = root, $2 = page, $3 = value.
label_page() {
    f="$1/$2"
    if [ ! -f "$f" ]; then
        fail "the page to label is there" "no $2 under the assembled root"
        return 1
    fi
    if [ "$(sed -n '1p' "$f")" != "---" ]; then
        fail "$2 opens with a front-matter block" \
            "the label would land in the body and the page would carry no value"
        return 1
    fi
    sed -i "1a $facet: $3" "$f"
    pass "$2 carries $facet: $3 in the assembled root and nowhere else"
}

# One arm: resolve, then check with an injected clock, plain and strict. The
# strict arm is run because the demonstration's claim is that no exit status
# discriminates the two arms, and a claim of that shape has to be measured.
# $1 = root, $2 = tag.
run_arm() {
    (cd "$1" && "$engine" taxonomy resolve) > "$scratch/$2-resolve.out" 2>&1
    echo $? > "$scratch/$2-resolve.code"
    (cd "$1" && "$engine" check --no-cache --now "$today") \
        > "$scratch/$2.out" 2> "$scratch/$2.err"
    echo $? > "$scratch/$2.code"
    (cd "$1" && "$engine" check --strict --no-cache --now "$today") \
        > "$scratch/$2-strict.out" 2> "$scratch/$2-strict.err"
    echo $? > "$scratch/$2-strict.code"
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
    ' "$1" | LC_ALL=C sort
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

# Every `facet.required.missing` finding of a report whose message names the
# facet $2, as `<page> TAB <kind>`. The message wraps over continuation lines,
# so each finding is joined into one line before it is read. $1 = report.
required_facet_findings() {
    awk -v want="$2" '
        function flush() {
            if (inb && buf ~ /^ facet\.required\.missing /) {
                re = "`[a-z_]+` requires the facet `" want "`"
                if (match(buf, re)) {
                    k = substr(buf, RSTART + 1)
                    sub(/`.*/, "", k)
                    print page "\t" k
                }
            }
            inb = 0
            buf = ""
        }
        /^  [^ ].* (✗|▲|●) (error|warn|info)$/ {
            flush()
            hdr = $0
            sub(/^  /, "", hdr)
            sub(/ [^ ]+ [a-z]+$/, "", hdr)
            page = hdr
            sub(/:[0-9]+:[0-9]+$/, "", page)
            inb = 1
            next
        }
        inb && /^    / { l = $0; sub(/^ +/, "", l); buf = buf " " l; next }
        inb { flush() }
        END { flush() }
    ' "$1" | LC_ALL=C sort
}

today=$(date -u +%Y-%m-%d)

# From here on, standard error is captured rather than printed: a `sort` or a
# `comm` this suite runs that writes a diagnostic (for instance `comm`'s "not
# in sorted order" warning, the exact shape a collation mismatch produces and
# that nothing used to read, #828) fails the case at the bottom of this file
# instead of leaving a line on the console nobody reads. fd 3 holds the real
# standard error so it can be restored before the summary.
exec 3>&2 2>"$scratch/stderr"

facets=$(entry_facets)
facet_floor=$(printf '%s\n' "$facets" | grep -c . || true)
facet=$(printf '%s\n' "$facets" | head -n 1)
values=""
[ "$facet_floor" -ge 1 ] && values=$(facet_values "$facet")
value_floor=$(printf '%s\n' "$values" | grep -c . || true)
# Feeds a positional `sed -n` pick below, never a `comm`, but this file also
# runs `comm` elsewhere, and the coarser rule `tools/repo/collation-fixtures.sh`
# holds is that every `sort` in a file that runs `comm` at all is pinned.
pages=$(cd "$corpus/.." && find docs -name '*.md' | LC_ALL=C sort)
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

echo
echo "case group 3 — the labels, injected into the scratch root"
labeled="$scratch/labeled"
make_root "$labeled" "$selection"
label_page "$labeled" "$labeled_page" "$good_value"
label_page "$labeled" "$planted_page" "$planted_value"
judge "the sibling corpus in the tree carries no $facet line of its own" "" \
    "$(grep -rl "^$facet:" "$root/$sibling" 2>/dev/null |
        sed "s|^$root/||" | tr '\n' ' ' | sed 's/ *$//')"

echo
echo "case group 4 — the two arms, and the parser that reads them"
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
# The claim this holds is negative: the corpus is not clean in either arm, so
# neither exit status separates them and the discriminator has to be the
# finding. A suite that read an exit status here would report a pass on a day
# the plant stopped working.
judge "no plain run separates the arms" \
    "$(cat "$scratch/unlabeled.code")" "$(cat "$scratch/labeled.code")"
judge "no strict run separates the arms either" \
    "$(cat "$scratch/unlabeled-strict.code")" "$(cat "$scratch/labeled-strict.code")"

echo
echo "case group 5 — the one finding the labels add"
judge "the labeled arm carries exactly one more finding" \
    "$((${u_find:-0} + 1))" "${l_find:-0}"
LC_ALL=C comm -13 "$scratch/unlabeled.pairs" "$scratch/labeled.pairs" > "$scratch/gained"
LC_ALL=C comm -23 "$scratch/unlabeled.pairs" "$scratch/labeled.pairs" > "$scratch/lost"
judge "the labels lose no finding" "" \
    "$(tr '\t' ' ' < "$scratch/lost" | tr '\n' ';' | sed 's/;*$//')"
judge "the labels gain exactly one" 1 "$(grep -c . "$scratch/gained" || true)"
judge "what they gain is a facet value refusal against the planted page" \
    "$planted_page facet.value.not_permitted" \
    "$(tr '\t' ' ' < "$scratch/gained" | head -n 1 | sed 's/ *$//')"

echo
echo "case group 6 — what that finding says"
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
echo "case group 7 — the correctly labeled page stays silent"
grep "^$labeled_page	" "$scratch/unlabeled.pairs" > "$scratch/labeled-page.before" || true
grep "^$labeled_page	" "$scratch/labeled.pairs" > "$scratch/labeled-page.after" || true
judge "an admitted value adds no finding to the page that carries it" "" \
    "$(LC_ALL=C comm -13 "$scratch/labeled-page.before" "$scratch/labeled-page.after" |
        tr '\t' ' ' | tr '\n' ';' | sed 's/;*$//')"
said=""
while IFS="$(printf '\t')" read -r p r; do
    [ -n "${r:-}" ] || continue
    finding_block "$scratch/labeled.out" "$p" "$r" > "$scratch/page.block"
    grep -q "$facet" "$scratch/page.block" && said="$said $r"
done < "$scratch/labeled-page.after"
# The count is in the name because this case passes over an empty population
# too, and a page that reported nothing at all would read the same as a page
# that reported and never named the facet.
judge "no finding against that page names $facet ($(grep -c . "$scratch/labeled-page.after" || true) read on it)" \
    "" "$(echo "$said" | sed 's/^ *//')"

echo
echo "case group 8 — a mode made mandatory over another entry's kind"
# The required half (HW-DR-0095, Q67). A scratch-only bundle names the sibling
# entry and this entry in `requires`, and appends the facet to the
# `facets.require` list of one kind the sibling entry declares. It needs the
# sibling for the list and this entry for the facet, and it is refused if the
# second is left out, because the facet it names is then declared by nothing.
#
# The bundle cannot go into the vendored copy: `make_root` pins that copy's
# digest, and one added file fails `taxonomy.pin.diverged`. So the package
# source and the library are copied into scratch, the bundle is added beside
# the others, and the copy is published. Every arm below takes that published
# package and the digest its publish printed, so the only thing that separates
# the arms is the selection.
required_kind=design_spec
req_bundle=diataxis-required-fixture
judge "the sibling entry declares the kind the fixture requires the facet on" 1 \
    "$(grep -c "^  kinds\.$required_kind:" "$root/$sibling/bundle.yml" || true)"
src8="$scratch/src8"
mkdir -p "$src8/taxonomy-source" "$src8/docs"
cp -r "$root/taxonomy-source/headwater-standard" "$src8/taxonomy-source/"
cp -r "$root/docs/taxonomies" "$src8/docs/"
mkdir -p "$src8/docs/taxonomies/$req_bundle"
{
    echo "# Written by tools/repo/diataxis-facet-fixtures.sh into scratch, and never"
    echo "# into the tree: the required half of the composition demonstration."
    echo "bundle: $req_bundle"
    echo 'extends: headwater/standard@1.0.0'
    echo "requires: [$(basename "$sibling"), $(basename "$entry")]"
    echo
    echo 'add_to:'
    echo "  kinds.$required_kind.facets.require: [$facet]"
} > "$src8/docs/taxonomies/$req_bundle/bundle.yml"
(cd "$src8" && "$engine" taxonomy publish --from taxonomy-source/headwater-standard \
    --out "$scratch/pkg8") > "$scratch/publish8.out" 2> "$scratch/publish8.err"
judge "the scratch package with the fixture bundle publishes" 0 "$?"
digest8=$(sed -n 's/^  digest //p' "$scratch/publish8.out" | head -n 1)
if [ -n "$digest8" ]; then
    pass "the publish printed a digest to pin"
else
    fail "the publish printed a digest to pin" "none in its output"
fi

# The four selections: the base selection over the republished package, the
# same with the fixture bundle, that one in reverse order, and that one less
# the sibling entry.
sel8_plain=$selection
sel8_req=$( { selection_list; echo "$req_bundle"; } | LC_ALL=C sort -u |
    tr '\n' ',' | sed -e 's/,/, /g' -e 's/, *$//')
sel8_rev=$( { selection_list; echo "$req_bundle"; } | LC_ALL=C sort -u -r |
    tr '\n' ',' | sed -e 's/,/, /g' -e 's/, *$//')
sel8_nodep=$( { selection_list; echo "$req_bundle"; } | LC_ALL=C sort -u |
    grep -vx "$(basename "$sibling")" | tr '\n' ',' | sed -e 's/,/, /g' -e 's/, *$//')
echo "  with fixture:  $sel8_req"
for arm in plain req rev nodep; do
    eval "sel=\$sel8_$arm"
    make_root "$scratch/r8-$arm" "$sel" "$scratch/pkg8" "$digest8"
done
for arm in plain req rev; do
    run_arm "$scratch/r8-$arm" "r8-$arm"
    judge "the $arm selection resolves" 0 "$(cat "$scratch/r8-$arm-resolve.code")"
    finding_pairs "$scratch/r8-$arm.out" > "$scratch/r8-$arm.pairs"
    required_facet_findings "$scratch/r8-$arm.out" "$facet" > "$scratch/r8-$arm.req"
done

# (a) The facet is now required on the sibling's kind.
judge "republishing with the fixture bundle and leaving it unselected changes no finding" \
    "" "$(LC_ALL=C comm -3 "$scratch/unlabeled.pairs" "$scratch/r8-plain.pairs" |
        tr '\t' ' ' | tr '\n' ';' | sed 's/;*$//')"
judge "without the fixture bundle no finding requires $facet" "" \
    "$(tr '\t' ' ' < "$scratch/r8-plain.req" | tr '\n' ';' | sed 's/;*$//')"
req8_n=$(grep -c . "$scratch/r8-req.req" || true)
if [ "$req8_n" -ge 1 ]; then
    pass "with it, unlabeled pages report $facet as required ($req8_n read)"
else
    fail "with it, unlabeled pages report $facet as required" "none parsed out of the report"
fi
judge "every such finding is against a $required_kind page" "" \
    "$(cut -f 2 "$scratch/r8-req.req" | grep -vx "$required_kind" | LC_ALL=C sort -u | tr '\n' ' ' | sed 's/ *$//')"
r8p_find=$(report_number "$scratch/r8-plain.out" "findings")
r8r_find=$(report_number "$scratch/r8-req.out" "findings")
judge "the fixture bundle adds exactly those findings and no other" \
    "$((${r8p_find:-0} + req8_n))" "${r8r_find:-0}"
judge "the fixture bundle loses no finding" "" \
    "$(LC_ALL=C comm -23 "$scratch/r8-plain.pairs" "$scratch/r8-req.pairs" |
        tr '\t' ' ' | tr '\n' ';' | sed 's/;*$//')"

# (b) Bundle order. The `sources` list of a lock records the order the
# resolver applied the bundles in, and bundles that do not depend on each other
# apply in the order the selection names them. So the list moves with the
# selection and is not asserted. What is asserted is that the resolved taxonomy
# and its digest do not move.
lock_req="$scratch/r8-req/.headwater/taxonomy.lock"
lock_rev="$scratch/r8-rev/.headwater/taxonomy.lock"
judge "both bundle orders resolve to one lock digest" \
    "$(sed -n 's/^  digest: //p' "$lock_req" | head -n 1)" \
    "$(sed -n 's/^  digest: //p' "$lock_rev" | head -n 1)"
sed '/^    - path: /,/^      digest: /d' "$lock_req" > "$scratch/lock-req.body"
sed '/^    - path: /,/^      digest: /d' "$lock_rev" > "$scratch/lock-rev.body"
# Two empty bodies compare equal, so each body must still hold the resolved
# taxonomy after the strip.
judge "each lock keeps its resolved taxonomy after the sources list is removed" "1 1" \
    "$(grep -c '^resolved:' "$scratch/lock-req.body") $(grep -c '^resolved:' "$scratch/lock-rev.body")"
if cmp -s "$scratch/lock-req.body" "$scratch/lock-rev.body"; then
    pass "both bundle orders write the same lock outside its sources list"
else
    fail "both bundle orders write the same lock outside its sources list" \
        "$(diff "$scratch/lock-req.body" "$scratch/lock-rev.body" | head -n 4 | tr '\n' ' ')"
fi
judge "both bundle orders report the same findings" "" \
    "$(LC_ALL=C comm -3 "$scratch/r8-req.pairs" "$scratch/r8-rev.pairs" |
        tr '\t' ' ' | tr '\n' ';' | sed 's/;*$//')"

# (c) The dependency is enforced, not assumed.
(cd "$scratch/r8-nodep" && "$engine" taxonomy resolve) \
    > "$scratch/r8-nodep-resolve.out" 2> "$scratch/r8-nodep-resolve.err"
judge "a selection without $(basename "$sibling") is refused" 1 "$?"
tr '\n' ' ' < "$scratch/r8-nodep-resolve.out" > "$scratch/r8-nodep.msg"
tr '\n' ' ' < "$scratch/r8-nodep-resolve.err" >> "$scratch/r8-nodep.msg"
for needle in "\`$req_bundle\`" "\`$(basename "$sibling")\` it names in \`requires\`" \
    "kinds.$required_kind.facets.require"; do
    if grep -qF "$needle" "$scratch/r8-nodep.msg"; then
        pass "the refusal names $needle"
    else
        fail "the refusal names $needle" "$(head -c 300 "$scratch/r8-nodep.msg")"
    fi
done

echo
echo "the run record, recorded and not asserted"
echo "  engine:        $("$engine" --version)"
echo "  now:           $today"
echo "  selection:     $selection"
echo "  labeled page:  $labeled_page   <- $facet: $good_value"
echo "  planted page:  $planted_page   <- $facet: $planted_value"
echo "  census:        ${u_files:-0} files, ${u_typed:-0} typed, ${u_inst:-0} check instances"
echo "  findings:      ${u_find:-0} unlabeled, ${l_find:-0} labeled"
echo "  unlabeled:     $(report_number "$scratch/unlabeled.out" "✗ error") error, $(report_number "$scratch/unlabeled.out" "▲ warn") warn"
echo "  labeled:       $(report_number "$scratch/labeled.out" "✗ error") error, $(report_number "$scratch/labeled.out" "▲ warn") warn"
echo "  check exit:    $(cat "$scratch/unlabeled.code") and $(cat "$scratch/labeled.code") plain, $(cat "$scratch/unlabeled-strict.code") and $(cat "$scratch/labeled-strict.code") strict"
echo "  the one finding:"
sed 's/^/  /' "$scratch/gained.block"
echo "  group 8:       $req_bundle requires $(basename "$sibling") and $(basename "$entry"), add_to kinds.$required_kind.facets.require: [$facet]"
echo "  group 8 pkg:   $digest8"
echo "  group 8:       ${r8p_find:-0} findings without it, ${r8r_find:-0} with it, $req8_n of them requiring $facet on:"
cut -f 1 "$scratch/r8-req.req" | sed 's/^/                   /'
echo "  group 8 lock:  $(sed -n 's/^  digest: //p' "$lock_req" | head -n 1) in both orders"

exec 2>&3 3>&-
judge "no sort or comm in this run wrote to standard error" \
    "" "$(tr '\n' '|' <"$scratch/stderr")"

echo
echo "$passed passed, $failed failed"
[ "$failed" -eq 0 ] || exit 1
